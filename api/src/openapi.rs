use actix::fut::{ready, Ready};
use actix_web::{Error, FromRequest, HttpRequest};
use clap::{crate_description, crate_version};
use paperclip::actix::Apiv2Security;
use paperclip::v2::models::{DefaultApiRaw, Info, OperationProtocol};
use reqwest::Url;
use serde::Deserialize;

use crate::APP_TITLE;

pub fn default_spec(hook0_client_api_url: &Option<Url>) -> DefaultApiRaw {
    DefaultApiRaw {
        info: Info {
            title: APP_TITLE.to_owned(),
            description: match crate_description!() {
                "" => None,
                d => Some(d.to_owned()),
            },
            version: crate_version!().to_owned(),
            ..Default::default()
        },
        host: hook0_client_api_url
            .as_ref()
            .and_then(|url| url.host_str().map(|host| host.to_string())),
        schemes: [hook0_client_api_url
            .as_ref()
            .and_then(|x| {
                if x.scheme() == "https" {
                    Some(OperationProtocol::Https)
                } else {
                    None
                }
            })
            .unwrap_or(OperationProtocol::Http)]
        .into(),
        ..Default::default()
    }
}

#[derive(Apiv2Security, Deserialize)]
#[openapi(
    apiKey,
    alias = "biscuit",
    in = "header",
    name = "Authorization",
    description = "Authentication using a Biscuit token (use the format `Bearer TOKEN`)"
)]
pub struct OaBiscuit;

impl FromRequest for OaBiscuit {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(Self {}))
    }
}

#[derive(Apiv2Security, Deserialize)]
#[openapi(
    apiKey,
    alias = "biscuit_user_access",
    in = "header",
    name = "Authorization",
    description = "Authentication using a Biscuit token of type 'user_access' (use the format `Bearer TOKEN`)"
)]
pub struct OaBiscuitUserAccess;

impl FromRequest for OaBiscuitUserAccess {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(Self {}))
    }
}

#[derive(Apiv2Security, Deserialize)]
#[openapi(
    apiKey,
    alias = "biscuit_refresh",
    in = "header",
    name = "Authorization",
    description = "Authentication using a Biscuit token of type 'refresh' (use the format `Bearer TOKEN`)"
)]
pub struct OaBiscuitRefresh;

impl FromRequest for OaBiscuitRefresh {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(Self {}))
    }
}
