use chrono::{DateTime, Utc};
use clap::{crate_name, crate_version};
use hex::ToHex;
use hmac::{Hmac, Mac};
use log::{debug, error, trace, warn};
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use reqwest::{Certificate, Client, Method, Url};
use sha2::Sha256;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use strum::EnumVariantNames;

use crate::RequestAttempt;

const USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, strum::Display, EnumVariantNames)]
pub enum ResponseError {
    #[strum(serialize = "E_UNKNOWN")]
    Unknown,
    #[strum(serialize = "E_INVALID_TARGET")]
    InvalidTarget,
    #[strum(serialize = "E_CONNECTION")]
    Connection,
    #[strum(serialize = "E_TIMEOUT")]
    Timeout,
    #[strum(serialize = "E_HTTP")]
    Http,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub response_error: Option<ResponseError>,
    pub http_code: Option<u16>,
    pub headers: Option<HeaderMap>,
    pub body: Option<String>,
    pub elapsed_time: Duration,
}

impl Response {
    pub fn is_success(&self) -> bool {
        self.response_error.is_none()
    }

    #[allow(non_snake_case)]
    pub fn response_error__name(&self) -> Option<String> {
        self.response_error.map(|re| re.to_string())
    }

    pub fn http_code(&self) -> Option<i16> {
        self.http_code.and_then(|c| c.try_into().ok())
    }

    pub fn headers(&self) -> Option<serde_json::Value> {
        self.headers.as_ref().and_then(|hm| {
            let iter = hm
                .iter()
                .map(|(k, v)| {
                    let key = k.to_string();
                    let value = v
                        .to_str()
                        .expect("Failed to extract a HTTP header value (there might be invisible characters)")
                        .to_owned();
                    (key, value)
                });
            let hashmap: HashMap<String, String> = iter.collect();
            serde_json::to_value(hashmap).ok()
        })
    }

    pub fn elapsed_time_ms(&self) -> i32 {
        self.elapsed_time.as_millis().try_into().unwrap_or(0)
    }
}

pub async fn work(attempt: &RequestAttempt, custom_ca: &Option<Certificate>) -> Response {
    debug!(
        "Processing request attempt {}",
        &attempt.request_attempt__id
    );
    let start = Instant::now();

    let m = Method::from_str(attempt.http_method.as_str());
    let u = Url::parse(attempt.http_url.as_str());
    let c = mk_http_client(custom_ca);
    let hs = attempt.headers();
    let event_id = HeaderValue::from_str(attempt.event__id.to_string().as_str())
        .expect("Could not create a header value from the event ID UUID");
    let et = HeaderValue::from_str(&attempt.event_type__name)
        .expect("Could not create a header value from the event type");
    let content_type = HeaderValue::from_str(attempt.payload_content_type.as_str())
        .expect("Could not create a header value from the event content type");
    let sig = Signature::new(&attempt.secret.to_string(), &attempt.payload, Utc::now())
        .to_header_value()
        .expect("Could not create a header value from the event ID UUID");

    match (m, u, c, hs) {
        (Ok(method), Ok(url), Ok(client), Ok(mut headers)) => {
            headers.insert("Content-Type", content_type);
            headers.insert("X-Event-Id", event_id);
            headers.insert("X-Event-Type", et);
            headers.insert("X-Hook0-Signature", sig);

            debug!("Calling webhook...");
            trace!(
                "HTTP {} {} {:?}",
                &method.to_string().to_uppercase(),
                &url,
                &headers
            );
            let response = client
                .request(method, url)
                .headers(headers)
                .body(attempt.payload.clone())
                .send()
                .await;

            match response {
                Ok(res) => {
                    let status = res.status();
                    let headers = res.headers().clone();
                    let body = res.text().await.ok();

                    if status.is_success() {
                        debug!("Webhook call was successful");
                        Response {
                            response_error: None,
                            http_code: Some(status.as_u16()),
                            headers: Some(headers),
                            body,
                            elapsed_time: start.elapsed(),
                        }
                    } else {
                        warn!("Webhook call failed with HTTP code {}", &status);
                        Response {
                            response_error: Some(ResponseError::Http),
                            http_code: Some(status.as_u16()),
                            headers: Some(headers),
                            body,
                            elapsed_time: start.elapsed(),
                        }
                    }
                }
                Err(e) if e.is_connect() => {
                    warn!("Webhook call failed with connection error: {}", &e);
                    Response {
                        response_error: Some(ResponseError::Connection),
                        http_code: None,
                        headers: None,
                        body: Some(e.to_string()),
                        elapsed_time: start.elapsed(),
                    }
                }
                Err(e) if e.is_timeout() => {
                    warn!("Webhook call failed with timeout error: {}", &e);
                    Response {
                        response_error: Some(ResponseError::Timeout),
                        http_code: None,
                        headers: None,
                        body: Some(e.to_string()),
                        elapsed_time: start.elapsed(),
                    }
                }
                Err(e) => {
                    warn!("Webhook call failed with unknown error: {}", &e);
                    Response {
                        response_error: Some(ResponseError::Unknown),
                        http_code: None,
                        headers: None,
                        body: Some(e.to_string()),
                        elapsed_time: start.elapsed(),
                    }
                }
            }
        }
        (Err(e), _, _, _) => {
            error!("Target has an invalid HTTP method: {}", &e);
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, Err(e), _, _) => {
            error!("Target has an invalid URL: {}", &e);
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, Err(e), _) => {
            error!("Could not create HTTP client: {}", &e);
            Response {
                response_error: Some(ResponseError::Unknown),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
        (_, _, _, Err(e)) => {
            error!("Target has invalid headers: {}", &e);
            Response {
                response_error: Some(ResponseError::InvalidTarget),
                http_code: None,
                headers: None,
                body: Some(e.to_string()),
                elapsed_time: start.elapsed(),
            }
        }
    }
}

fn mk_http_client(custom_ca: &Option<Certificate>) -> reqwest::Result<Client> {
    let mut client = Client::builder()
        .connection_verbose(true)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TIMEOUT)
        .user_agent(USER_AGENT)
        .tcp_keepalive(None);

    if let Some(cert) = custom_ca {
        client = client.add_root_certificate(cert.to_owned());
    }

    client.build()
}

struct Signature {
    pub timestamp: i64,
    pub v0: String,
}

impl Signature {
    const PAYLOAD_SEPARATOR: &'static [u8] = b".";
    const SIGNATURE_PART_ASSIGNATOR: &'static str = "=";
    const SIGNATURE_PART_SEPARATOR: &'static str = ",";

    pub fn new(secret: &str, payload: &[u8], signed_at: DateTime<Utc>) -> Self {
        let timestamp = signed_at.timestamp();
        let timestamp_str = timestamp.to_string();
        let timestamp_str_bytes = timestamp_str.as_bytes();

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // MAC can take key of any size; this should never fail
        mac.update(timestamp_str_bytes);
        mac.update(Self::PAYLOAD_SEPARATOR);
        mac.update(payload);
        let v0 = mac.finalize().into_bytes().encode_hex::<String>();

        Self { timestamp, v0 }
    }

    pub fn value(&self) -> String {
        let timestamp_str = self.timestamp.to_string();
        let parts = &[("t", timestamp_str.as_str()), ("v0", self.v0.as_str())];

        itertools::Itertools::intersperse(
            parts
                .iter()
                .map(|p| format!("{}{}{}", p.0, Self::SIGNATURE_PART_ASSIGNATOR, p.1)),
            Self::SIGNATURE_PART_SEPARATOR.to_owned(),
        )
        .collect::<String>()
    }

    pub fn to_header_value(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(&self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::prelude::*;

    #[test]
    fn create_signature() {
        let signed_at = Utc.with_ymd_and_hms(2021, 11, 15, 0, 30, 0).unwrap();
        let payload = "hello !";
        let secret = "secret";

        let sig = Signature::new(secret, payload.as_bytes(), signed_at);
        assert_eq!(
            sig.value(),
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98"
        );
    }
}
