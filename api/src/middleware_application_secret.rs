use actix_web::body::AnyBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::{Error, HttpMessage, HttpResponse, ResponseError};
use actix_web_middleware_keycloak_auth::{extract_jwt_claims, KeycloakAuthStatus};
use anyhow::anyhow;
use futures_util::future::{ok, ready, Ready};
use log::{debug, error, trace};
use sqlx::{query_as, PgPool};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::iam::AuthProof;

const DETAILED_RESPONSES: bool = true;

#[derive(Debug, Clone)]
pub struct ApplicationSecretAuth {
    pub db: PgPool,
}

impl<S> Transform<S, ServiceRequest> for ApplicationSecretAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ApplicationSecretAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        trace!("Initialize ApplicationSecretAuthMiddleware");
        ok(ApplicationSecretAuthMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    NoJwtAuthStatus,
    NoAuthorizationHeader,
    InvalidAuthorizationHeader,
    ApplicationSecretLookupError,
    InvalidApplicationSecret,
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NoJwtAuthStatus | Self::ApplicationSecretLookupError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::InvalidAuthorizationHeader => StatusCode::BAD_REQUEST,
            Self::InvalidApplicationSecret => StatusCode::FORBIDDEN,
            _ => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(self.status_code()).set_body(self.to_string().into())
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoJwtAuthStatus => f.write_str(""),
            Self::NoAuthorizationHeader => f.write_str("No bearer token was provided"),
            Self::InvalidAuthorizationHeader => {
                f.write_str("Authorization header value is invalid")
            }
            Self::ApplicationSecretLookupError => f.write_str("Application secret lookup failed"),
            Self::InvalidApplicationSecret => f.write_str("Application secret is not valid"),
        }
    }
}

impl AuthError {
    pub fn to_response(&self, detailed_responses: bool) -> HttpResponse {
        if detailed_responses {
            self.error_response()
        } else {
            HttpResponse::build(self.status_code()).body(self.status_code().to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApplicationSecretAuthMiddleware<S> {
    service: Rc<S>,
    db: PgPool,
}

impl<S> Service<ServiceRequest> for ApplicationSecretAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let extensions = req.extensions();
        let auth_status = extensions.get::<KeycloakAuthStatus>();

        match auth_status {
            Some(jwt_auth_status) => {
                let jwt_auth_status = jwt_auth_status.to_owned();
                debug!("JWT auth status is: {:?}", &jwt_auth_status);
                drop(extensions);

                match jwt_auth_status {
                    KeycloakAuthStatus::Success => {
                        let (request, payload) = req.into_parts();
                        let claims_extraction = extract_jwt_claims(&request);
                        let req = ServiceRequest::from_parts(request, payload);

                        match claims_extraction {
                            Ok(claims) => {
                                {
                                    let mut extensions = req.extensions_mut();
                                    extensions.insert(AuthProof::Jwt { claims });
                                }
                                debug!("Auth with JWT succeeded");
                                Box::pin(self.service.call(req))
                            }
                            Err(e) => {
                                debug!("{}", &e);
                                let (request, _payload) = req.into_parts();
                                Box::pin(ready(Ok(ServiceResponse::from_err(e, request))))
                            }
                        }
                    }
                    KeycloakAuthStatus::Failure(_) => {
                        debug!("Attempting auth using application secret");

                        let auth_header = req.headers().get("Authorization");
                        match auth_header {
                            Some(auth_header_value) => {
                                let auth_header_uuid = auth_header_value
                                    .to_str()
                                    .map_err(|e| anyhow!(e))
                                    .and_then(|str| {
                                        Uuid::parse_str(str.trim_start_matches("Bearer "))
                                            .map_err(|e| anyhow!(e))
                                    });
                                match auth_header_uuid {
                                    Ok(token) => {
                                        debug!(
                                            "Application secret was extracted from request headers"
                                        );

                                        let pool = Box::new(self.db.clone());
                                        let pool: &'static PgPool = Box::leak(pool);
                                        let srv = Rc::clone(&self.service);
                                        Box::pin(async move {
                                            #[derive(Debug)]
                                            #[allow(non_snake_case)]
                                            struct ApplicationSecretLookup {
                                                application__id: Uuid,
                                                name: Option<String>,
                                            }

                                            let application_secret_lookup = query_as!(
                                                ApplicationSecretLookup,
                                                "
                                                    SELECT application__id, name
                                                    FROM event.application_secret
                                                    WHERE token = $1
                                                    LIMIT 1
                                                ",
                                                &token
                                            )
                                            .fetch_optional(pool)
                                            .await;

                                            match application_secret_lookup {
                                                Ok(Some(application_secret)) => {
                                                    {
                                                        debug!("Auth with application secret succeeded");
                                                        let mut extensions = req.extensions_mut();
                                                        extensions.insert(
                                                            AuthProof::ApplicationSecret {
                                                                secret: token,
                                                                name: application_secret.name,
                                                                application_id: application_secret
                                                                    .application__id,
                                                            },
                                                        );
                                                    }
                                                    srv.call(req).await
                                                }
                                                Ok(None) => {
                                                    let e = AuthError::InvalidApplicationSecret;
                                                    debug!("{}", &e);
                                                    Ok(req.into_response::<AnyBody, HttpResponse>(
                                                        e.to_response(DETAILED_RESPONSES),
                                                    ))
                                                }
                                                Err(err) => {
                                                    let e = AuthError::ApplicationSecretLookupError;
                                                    debug!("{}: {}", &e, &err);
                                                    Ok(req.into_response::<AnyBody, HttpResponse>(
                                                        e.to_response(DETAILED_RESPONSES),
                                                    ))
                                                }
                                            }
                                        })
                                    }
                                    Err(_) => {
                                        let e = AuthError::InvalidAuthorizationHeader;
                                        debug!("{}", &e);
                                        Box::pin(ready(Ok(req
                                            .into_response::<AnyBody, HttpResponse>(
                                                e.to_response(DETAILED_RESPONSES),
                                            ))))
                                    }
                                }
                            }
                            None => {
                                let e = AuthError::NoAuthorizationHeader;
                                debug!("{}", &e);
                                Box::pin(ready(Ok(req.into_response::<AnyBody, HttpResponse>(
                                    e.to_response(DETAILED_RESPONSES),
                                ))))
                            }
                        }
                    }
                }
            }
            None => {
                error!("ApplicationSecretAuthMiddleware cannot find the KeycloakAuthStatus left in ReqData by KeycloakAuthMiddleware");
                drop(extensions);
                Box::pin(ready(Ok(req.into_response(
                    AuthError::NoJwtAuthStatus.to_response(DETAILED_RESPONSES),
                ))))
            }
        }
    }
}
