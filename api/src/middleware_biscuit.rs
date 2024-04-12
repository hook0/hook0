use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use anyhow::anyhow;
use biscuit_auth::{Biscuit, PrivateKey};
use futures_util::future::{ok, ready, Ready};
use log::{debug, error, trace};
use sentry_integration::set_user_from_token;
use sqlx::{query_scalar, PgPool};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::iam::create_master_access_token;
use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct BiscuitAuth {
    pub db: PgPool,
    pub biscuit_private_key: PrivateKey,
    pub master_api_key: Option<Uuid>,
}

impl<S> Transform<S, ServiceRequest> for BiscuitAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = BiscuitAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        trace!("Initialize BiscuitAuthMiddleware");
        ok(BiscuitAuthMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
            biscuit_private_key: self.biscuit_private_key.clone(),
            master_api_key: self.master_api_key,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BiscuitAuthMiddleware<S> {
    service: Rc<S>,
    db: PgPool,
    biscuit_private_key: PrivateKey,
    master_api_key: Option<Uuid>,
}

impl<S> Service<ServiceRequest> for BiscuitAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        debug!("Attempting auth using Biscuit");

        let auth_header = req.headers().get("Authorization");
        match auth_header {
            Some(auth_header_value) => {
                let auth_header_str = auth_header_value
                    .to_str()
                    .map_err(|e| anyhow!(e))
                    .map(|str| str.trim_start_matches("Bearer "));

                match auth_header_str {
                    Ok(token) => {
                        debug!("Token was extracted from request headers");

                        match Biscuit::from_base64(token, self.biscuit_private_key.public())
                            .and_then(|biscuit| {
                                biscuit
                                    .revocation_identifiers()
                                    .first()
                                    .map(|rid| (biscuit, rid.to_owned()))
                                    .ok_or(biscuit_auth::error::Token::InternalError)
                            }) {
                            Ok((biscuit, revocation_id)) => {
                                let pool = Box::new(self.db.clone());
                                let pool: &'static PgPool = Box::leak(pool);
                                let srv = Rc::clone(&self.service);
                                Box::pin(async move {
                                    let biscuit_token_id = query_scalar!(
                                        "
                                            SELECT token__id AS token_id
                                            FROM iam.token
                                            WHERE revocation_id = $1
                                                AND (expired_at IS NULL OR expired_at > statement_timestamp())
                                            LIMIT 1
                                        ",
                                        &revocation_id
                                    )
                                    .fetch_optional(pool)
                                    .await;

                                    match biscuit_token_id {
                                        Ok(Some(token_id)) => {
                                            {
                                                debug!(
                                                    "Auth with Biscuit succeeded (token ID = {})",
                                                    token_id
                                                );
                                                set_user_from_token(&token_id.to_string());
                                                let mut extensions = req.extensions_mut();
                                                extensions.insert(biscuit);
                                            }
                                            srv.call(req).await
                                        }
                                        Ok(None) => {
                                            let e = Hook0Problem::AuthInvalidBiscuit;
                                            debug!("{e} (root token was not found in database or was expired)");
                                            Ok(req.error_response(e))
                                        }
                                        Err(err) => {
                                            let e = Hook0Problem::AuthBiscuitLookupError;
                                            error!("{e}: {err}");
                                            Ok(req.error_response(e))
                                        }
                                    }
                                })
                            }
                            Err(biscuit_err) => {
                                let uuid_token = Uuid::parse_str(token);
                                let is_master_key =
                                    if let Some(master_api_key) = self.master_api_key {
                                        uuid_token == Ok(master_api_key)
                                    } else {
                                        false
                                    };

                                if is_master_key {
                                    match create_master_access_token(&self.biscuit_private_key) {
                                        Ok(biscuit) => {
                                            let srv = Rc::clone(&self.service);
                                            Box::pin(async move {
                                                {
                                                    debug!("Auth with master API key succeeded");
                                                    let mut extensions = req.extensions_mut();
                                                    extensions.insert(biscuit);
                                                }
                                                srv.call(req).await
                                            })
                                        }
                                        Err(e) => {
                                            error!("Error while creating master key Biscuit: {e}");
                                            let res = Hook0Problem::InternalServerError;
                                            Box::pin(ready(Ok(req.error_response(res))))
                                        }
                                    }
                                } else {
                                    #[cfg(feature = "application-secret-compatibility")]
                                    if let Ok(application_secret_token) = uuid_token {
                                        let pool = Box::new(self.db.clone());
                                        let pool: &'static PgPool = Box::leak(pool);
                                        let biscuit_private_key = self.biscuit_private_key.clone();
                                        let srv = Rc::clone(&self.service);
                                        Box::pin(async move {
                                            #[derive(Debug)]
                                            struct ApplicationSecretLookup {
                                                organization_id: Uuid,
                                                application_id: Uuid,
                                            }
                                            let application_secret_lookup = sqlx::query_as!(
                                                ApplicationSecretLookup,
                                                "
                                                    SELECT a.organization__id AS organization_id, s.application__id AS application_id
                                                    FROM event.application_secret AS s
                                                    INNER JOIN event.application AS a ON a.application__id = s.application__id
                                                    WHERE s.token = $1
                                                ",
                                                application_secret_token,
                                            )
                                            .fetch_optional(pool)
                                            .await;

                                            match application_secret_lookup {
                                                Ok(Some(application_secret)) => {
                                                    let service_access_biscuit = crate::iam::create_service_access_token(
                                                        &biscuit_private_key,
                                                        Uuid::nil(),
                                                        application_secret.organization_id,
                                                    )
                                                    .and_then(|root_token| {
                                                        use biscuit_auth::builder_ext::BuilderExt;

                                                        let biscuit = root_token.biscuit.append({
                                                            let mut block = biscuit_auth::builder::BlockBuilder::new();
                                                            block.add_check(biscuit_auth::macros::check!(
                                                                "check if application_id({application_id})",
                                                                application_id = application_secret.application_id
                                                            ))?;
                                                            block.check_expiration_date(std::time::SystemTime::now() + std::time::Duration::from_secs(1));
                                                            block
                                                        })?;
                                                        Ok(biscuit)
                                                    });

                                                    match service_access_biscuit {
                                                        Ok(biscuit) => {
                                                            {
                                                                debug!(
                                                                    "Auth with application secret succeeded (application ID = {})",
                                                                    application_secret.application_id
                                                                );
                                                                sentry_integration::set_user_from_application_secret(
                                                                    &application_secret
                                                                        .application_id
                                                                        .to_string(),
                                                                );
                                                                let mut extensions =
                                                                    req.extensions_mut();
                                                                extensions.insert(biscuit);
                                                            }
                                                            srv.call(req).await
                                                        }
                                                        Err(e) => {
                                                            error!("Error while creating service access Biscuit from application secret: {e}");
                                                            let res =
                                                                Hook0Problem::InternalServerError;
                                                            Ok(req.error_response(res))
                                                        }
                                                    }
                                                }
                                                Ok(None) => {
                                                    let e = Hook0Problem::AuthInvalidBiscuit;
                                                    debug!("{e}: {biscuit_err}");
                                                    Ok(req.error_response(e))
                                                }
                                                Err(e) => {
                                                    error!("Error while searching for an application scret: {e}");
                                                    let res = Hook0Problem::InternalServerError;
                                                    Ok(req.error_response(res))
                                                }
                                            }
                                        })
                                    } else {
                                        let e = Hook0Problem::AuthInvalidBiscuit;
                                        debug!("{e}: {biscuit_err}");
                                        Box::pin(ready(Ok(req.error_response(e))))
                                    }

                                    #[cfg(not(feature = "application-secret-compatibility"))]
                                    {
                                        let e = Hook0Problem::AuthInvalidBiscuit;
                                        debug!("{e}: {biscuit_err}");
                                        Box::pin(ready(Ok(req.error_response(e))))
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        let e = Hook0Problem::AuthInvalidAuthorizationHeader;
                        debug!("{e}");
                        Box::pin(ready(Ok(req.error_response(e))))
                    }
                }
            }
            None => {
                let e = Hook0Problem::AuthNoAuthorizationHeader;
                debug!("{e}");
                Box::pin(ready(Ok(req.error_response(e))))
            }
        }
    }
}
