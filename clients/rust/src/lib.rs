// Force exposed items to be documented
#![deny(missing_docs)]

//! This is the Rust client for Hook0.
//! It makes it easier to send events from a Rust application to a Hook0 instance.

use base64::encode;
use chrono::{DateTime, Utc};
use log::error;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::Value;
use std::borrow::Cow;
use url::ParseError;
use uuid::Uuid;

/// The Hook0 client
///
/// This struct is supposed to be initialized once and shared/reused wherever you need to send events in your app.
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    api_url: Url,
    application_id: Uuid,
}

impl Hook0Client {
    /// Initialize a client
    ///
    /// - `api_url` - Base API URL of a Hook0 instance (example: `https://app.hook0.com/api/v1`).
    /// - `application_id` - UUID of your Hook0 application.
    /// - `application_secret` - Secret of your Hook0 application.
    pub fn new(
        api_url: Url,
        application_id: Uuid,
        application_secret: &Uuid,
    ) -> Result<Self, Hook0ClientError> {
        let authenticated_client = HeaderValue::from_str(&format!("Bearer {application_secret}"))
            .map_err(|e| Hook0ClientError::AuthHeader(e).log_and_return())
            .map(|hv| HeaderMap::from_iter([(AUTHORIZATION, hv)]))
            .and_then(|headers| {
                Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|e| Hook0ClientError::ReqwestClient(e).log_and_return())
            })?;

        Ok(Self {
            api_url,
            client: authenticated_client,
            application_id,
        })
    }

    fn mk_url(&self, segments: &[&str]) -> Result<Url, Hook0ClientError> {
        append_url_segments(&self.api_url, segments)
            .map_err(|e| Hook0ClientError::Url(e).log_and_return())
    }

    /// Send an event to Hook0
    pub async fn send_event(&self, event: &Event<'_>) -> Result<Uuid, Hook0ClientError> {
        let event_ingestion_url = self.mk_url(&["event"])?;
        let full_event = FullEvent::from_event(event, &self.application_id);

        self.client
            .post(event_ingestion_url)
            .json(&full_event)
            .send()
            .await
            .map_err(|e| {
                Hook0ClientError::EventSending {
                    event_id: full_event.event_id.to_owned(),
                    error: e,
                }
                .log_and_return()
            })?
            .error_for_status()
            .map_err(|e| {
                Hook0ClientError::EventSending {
                    event_id: full_event.event_id.to_owned(),
                    error: e,
                }
                .log_and_return()
            })?;

        Ok(full_event.event_id)
    }
}

/// A wrapper to handle event's payload
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload<'a> {
    /// Payload is already base64-encoded
    Base64(&'a str),

    /// Payload is binary
    Binary(&'a [u8]),

    /// Payload is a string
    Str(&'a str),

    /// Payload as a JSON value
    Json(&'a Value),
}

impl<'a> Payload<'a> {
    /// Get the payload as base64
    pub fn as_base64(&self) -> Cow<'a, str> {
        match self {
            Self::Base64(b64) => (*b64).into(),
            Self::Binary(bin) => encode(bin).into(),
            Self::Str(str) => encode(str).into(),
            Self::Json(json) => encode(json.to_string()).into(),
        }
    }
}

/// An event that can be sent to Hook0
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<'a> {
    /// Unique ID of the event (a UUIDv4 will be generated if nothing is provided)
    pub event_id: &'a Option<&'a Uuid>,
    /// Type of the event (as configured in your Hook0 application)
    pub event_type: &'a str,
    /// Payload
    pub payload: Payload<'a>,
    /// Content type of the payload
    pub payload_content_type: &'a str,
    /// Optional key-value metadata
    pub metadata: &'a Option<&'a [(&'a str, &'a Value)]>,
    /// Datetime of when the event occurred (current time will be used if nothing is provided)
    pub occurred_at: &'a Option<&'a DateTime<Utc>>,
    /// Labels that Hook0 will use to route the event
    pub labels: &'a [(&'a str, &'a Value)],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FullEvent<'a> {
    pub application_id: Uuid,
    pub event_id: Uuid,
    pub event_type: &'a str,
    pub payload: Cow<'a, str>,
    pub payload_content_type: &'a str,
    pub metadata: &'a Option<&'a [(&'a str, &'a Value)]>,
    pub occurred_at: DateTime<Utc>,
    pub labels: &'a [(&'a str, &'a Value)],
}

impl<'a> FullEvent<'a> {
    pub fn from_event(event: &'a Event, application_id: &Uuid) -> Self {
        let event_id = event
            .event_id
            .map(|uuid| uuid.to_owned())
            .unwrap_or_else(Uuid::new_v4);
        let occurred_at = event
            .occurred_at
            .map(|datetime| datetime.to_owned())
            .unwrap_or_else(Utc::now);

        Self {
            application_id: application_id.to_owned(),
            event_id,
            event_type: event.event_type,
            payload: event.payload.as_base64(),
            payload_content_type: event.payload_content_type,
            metadata: event.metadata,
            occurred_at,
            labels: event.labels,
        }
    }
}

/// Every error Hook0 client can encounter
#[derive(Debug, thiserror::Error)]
pub enum Hook0ClientError {
    /// Cannot build a structurally-valid `Authorization` header
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build auth header: {0}")]
    AuthHeader(InvalidHeaderValue),

    /// Cannot build a Reqwest HTTP client
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build reqwest HTTP client: {0}")]
    ReqwestClient(reqwest::Error),

    /// Cannot build a structurally-valid endpoint URL
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not create a valid URL to request Hook0's API: {0}")]
    Url(ParseError),

    /// Something went wrong when sending an event to Hook0
    #[error("Sending event {event_id} failed: {error}")]
    EventSending {
        /// ID of the event
        event_id: Uuid,

        /// Error as reported by Reqwest
        error: reqwest::Error,
    },
}

impl Hook0ClientError {
    /// Log the error (using the log crate) and return it as a result of this function's call
    pub fn log_and_return(self) -> Self {
        error!("{self}");
        self
    }
}

fn append_url_segments(base_url: &Url, segments: &[&str]) -> Result<Url, url::ParseError> {
    const SEP: &str = "/";
    let segments_str = segments.join(SEP);

    let url = Url::parse(&format!("{base_url}/{segments_str}").replace("//", "/"))?;

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::decode;
    use serde_json::json;
    use std::str::FromStr;

    const PAYLOAD: &str = "Hook0";

    #[test]
    fn payload_base64() {
        let b64 = encode(PAYLOAD);
        let payload = Payload::Base64(&b64);
        assert_eq!(
            PAYLOAD,
            String::from_utf8(decode(payload.as_base64().as_ref()).unwrap()).unwrap()
        )
    }

    #[test]
    fn payload_bin() {
        let bin = PAYLOAD.as_bytes();
        let payload = Payload::Binary(bin);
        assert_eq!(
            PAYLOAD,
            String::from_utf8(decode(payload.as_base64().as_ref()).unwrap()).unwrap()
        )
    }

    #[test]
    fn payload_str() {
        let payload = Payload::Str(PAYLOAD);
        assert_eq!(
            PAYLOAD,
            String::from_utf8(decode(payload.as_base64().as_ref()).unwrap()).unwrap()
        )
    }

    #[test]
    fn payload_json() {
        let json = json!({ "hook0": PAYLOAD });
        let payload = Payload::Json(&json);
        assert_eq!(
            json,
            Value::from_str(
                String::from_utf8(decode(payload.as_base64().as_ref()).unwrap())
                    .unwrap()
                    .as_str()
            )
            .unwrap()
        )
    }
}
