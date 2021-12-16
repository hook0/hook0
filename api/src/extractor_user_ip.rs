use actix_web::{FromRequest, ResponseError};
use futures_util::future::{ready, Ready};
use ipnetwork::IpNetwork;
use paperclip::actix::OperationModifier;
use paperclip::v2::schema::Apiv2Schema;
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, Clone)]
/// Extractor for user's IP address
pub struct UserIp(IpNetwork);

impl UserIp {
    pub fn into_inner(self) -> IpNetwork {
        self.0
    }
}

impl Deref for UserIp {
    type Target = IpNetwork;

    fn deref(&self) -> &IpNetwork {
        &self.0
    }
}

impl FromRequest for UserIp {
    type Error = UserIpExtractorError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.req_data()
                .get::<IpNetwork>()
                .ok_or(UserIpExtractorError)
                .map(|ip| Self(*ip)),
        )
    }
}

impl Apiv2Schema for UserIp {}
impl OperationModifier for UserIp {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserIpExtractorError;

impl Display for UserIpExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not extract user IP")
    }
}

impl ResponseError for UserIpExtractorError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
