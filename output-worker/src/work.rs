use clap::{crate_name, crate_version};
use log::{debug, error, trace, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, Url};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::RequestAttempt;

const USER_AGENT: &str = concat!(crate_name!(), "/", crate_version!());
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy)]
pub enum ResponseError {
    Unknown,
    InvalidTarget,
    Connection,
    Timeout,
    Http,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "E_UNKNOWN"),
            Self::InvalidTarget => write!(f, "E_INVALID_TARGET"),
            Self::Connection => write!(f, "E_CONNECTION"),
            Self::Timeout => write!(f, "E_TIMEOUT"),
            Self::Http => write!(f, "E_HTTP"),
        }
    }
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
        use std::convert::TryInto;
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
            serde_json::to_value(&hashmap).ok()
        })
    }

    pub fn elapsed_time_ms(&self) -> i32 {
        use std::convert::TryInto;

        self.elapsed_time.as_millis().try_into().unwrap_or(0)
    }
}

pub async fn work(attempt: &RequestAttempt) -> Response {
    debug!(
        "Processing request attempt {}",
        &attempt.request_attempt__id
    );
    let start = Instant::now();

    let m = Method::from_str(attempt.http_method.as_str());
    let u = Url::parse(attempt.http_url.as_str());
    let c = mk_http_client();
    let hs = attempt.headers();
    let event_id = HeaderValue::from_str(attempt.event__id.to_string().as_str())
        .expect("Could not create a header value from the event ID UUID");
    let content_type = HeaderValue::from_str(attempt.payload_content_type.as_str())
        .expect("Could not create a header value from the event content type");

    match (m, u, c, hs) {
        (Ok(method), Ok(url), Ok(client), Ok(mut headers)) => {
            headers.insert("X-Event-Id", event_id);
            headers.insert("Content-Type", content_type);

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

fn mk_http_client() -> reqwest::Result<Client> {
    Client::builder()
        .connection_verbose(true)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TIMEOUT)
        .user_agent(USER_AGENT)
        .tcp_keepalive(None)
        .build()
}
