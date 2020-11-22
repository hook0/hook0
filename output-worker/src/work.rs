use async_std::task::sleep;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::RequestAttempt;

#[derive(Debug, Clone, Copy)]
pub enum ResponseError {
    Dns,
    Timeout,
    Http,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns => write!(f, "E_DNS"),
            Self::Timeout => write!(f, "E_TIMEOUT"),
            Self::Http => write!(f, "E_HTTP"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub response_error: Option<ResponseError>,
    pub http_code: Option<u8>,
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
        self.http_code.map(|c| c.into())
    }

    pub fn headers(&self) -> Option<serde_json::Value> {
        use std::iter::FromIterator;

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
            let hashmap: HashMap<String, String> = HashMap::from_iter(iter);
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

    // TODO: implement actual work here

    // Actually for now we simulate working
    sleep(Duration::from_secs(5)).await;

    // Let's simulate failing for one of my test items
    let cursed_item = Uuid::parse_str("8536a6a6-e7ec-4cea-b984-d7f377f394e4").unwrap();
    if attempt.request_attempt__id == cursed_item {
        Response {
            response_error: Some(ResponseError::Dns),
            http_code: None,
            headers: None,
            body: None,
            elapsed_time: start.elapsed(),
        }
    } else {
        let mut fake_headers = HeaderMap::new();
        fake_headers.insert("X-Test", HeaderValue::from_static("Test"));

        Response {
            response_error: None,
            http_code: Some(200),
            headers: Some(fake_headers),
            body: Some("TEST".to_owned()),
            elapsed_time: start.elapsed(),
        }
    }
}
