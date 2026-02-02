use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures_util::future::{Ready, ok};
use log::trace;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::rate_limiting::RateLimiterOrganizationKey;

/// Middleware that extracts organization ID from URL path parameters
/// and inserts it into request extensions for rate limiting.
///
/// This middleware should be applied to scopes that have `{organization_id}` in the path.
/// It will overwrite any existing `RateLimiterOrganizationKey` in the extensions,
/// allowing user tokens (which default to `NoOrganization`) to be properly rate-limited
/// when accessing organization-specific routes.
#[derive(Debug, Clone, Copy)]
pub struct GetOrganizationId;

impl<S> Transform<S, ServiceRequest> for GetOrganizationId
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = GetOrganizationIdMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        trace!("Initialize GetOrganizationIdMiddleware");
        ok(GetOrganizationIdMiddleware {
            service: Rc::new(service),
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetOrganizationIdMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for GetOrganizationIdMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Try to extract organization ID from path parameters
        if let Some(org_id_str) = req.match_info().get("organization_id")
            && let Ok(org_id) = Uuid::parse_str(org_id_str)
        {
            trace!("Extracted organization ID from path: {org_id}");
            let mut extensions = req.extensions_mut();
            extensions.insert(RateLimiterOrganizationKey::Organization(org_id));
        }

        let srv = Rc::clone(&self.service);
        Box::pin(async move { srv.call(req).await })
    }
}
