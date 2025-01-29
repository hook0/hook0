// Force exposed items to be documented
#![deny(missing_docs)]

//! This is the Rust client for Hook0.
//! It makes it easier to send events from a Rust application to a Hook0 instance.

use chrono::{DateTime, Utc};

#[cfg(feature = "hook0-webhook-producer")]
use lazy_regex::regex_captures;
#[cfg(feature = "hook0-webhook-producer")]
use log::{debug, error, trace};
#[cfg(feature = "hook0-webhook-producer")]
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION};
#[cfg(feature = "hook0-webhook-producer")]
use reqwest::{Client, Url};
#[cfg(feature = "hook0-webhook-producer")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "hook0-webhook-producer")]
use serde_json::{Map, Value};
#[cfg(feature = "hook0-webhook-producer")]
use std::borrow::Cow;
#[cfg(feature = "hook0-webhook-producer")]
use std::collections::HashSet;
#[cfg(feature = "hook0-webhook-producer")]
use std::fmt::Display;
#[cfg(feature = "hook0-webhook-producer")]
use std::str::FromStr;
#[cfg(feature = "hook0-webhook-producer")]
use url::ParseError;
#[cfg(feature = "hook0-webhook-producer")]
use uuid::Uuid;

#[cfg(feature = "hook0-webhook-consumer")]
use chrono::{Duration, OutOfRangeError};
#[cfg(feature = "hook0-webhook-consumer")]
mod signature;

#[cfg(feature = "hook0-webhook-producer")]
/// The Hook0 client
///
/// This struct is supposed to be initialized once and shared/reused wherever you need to send events in your app.
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    api_url: Url,
    application_id: Uuid,
}

#[cfg(feature = "hook0-webhook-producer")]
impl Hook0Client {
    /// Initialize a client
    ///
    /// - `api_url` - Base API URL of a Hook0 instance (example: `https://app.hook0.com/api/v1`).
    /// - `application_id` - UUID of your Hook0 application.
    /// - `token` - Authentication token valid for your Hook0 application.
    pub fn new(api_url: Url, application_id: Uuid, token: &str) -> Result<Self, Hook0ClientError> {
        let authenticated_client = HeaderValue::from_str(&format!("Bearer {token}"))
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

    /// Get the API URL of this client
    pub fn api_url(&self) -> &Url {
        &self.api_url
    }

    /// Get the application ID of this client
    pub fn application_id(&self) -> &Uuid {
        &self.application_id
    }

    fn mk_url(&self, segments: &[&str]) -> Result<Url, Hook0ClientError> {
        append_url_segments(&self.api_url, segments)
            .map_err(|e| Hook0ClientError::Url(e).log_and_return())
    }

    /// Send an event to Hook0
    pub async fn send_event(&self, event: &Event<'_>) -> Result<Uuid, Hook0ClientError> {
        let event_ingestion_url = self.mk_url(&["event"])?;
        let full_event = FullEvent::from_event(event, &self.application_id);

        let res = self
            .client
            .post(event_ingestion_url)
            .json(&full_event)
            .send()
            .await
            .map_err(|e| {
                Hook0ClientError::EventSending {
                    event_id: full_event.event_id.to_owned(),
                    error: e,
                    body: None,
                }
                .log_and_return()
            })?;

        match res.error_for_status_ref() {
            Ok(_) => Ok(full_event.event_id),
            Err(e) => {
                let body = res.text().await.ok();
                Err(Hook0ClientError::EventSending {
                    event_id: full_event.event_id.to_owned(),
                    error: e,
                    body,
                }
                .log_and_return())
            }
        }
    }

    /// Ensure the configured app has the right event types or create them
    ///
    /// Returns the list of event types that were created, if any.
    pub async fn upsert_event_types(
        &self,
        event_types: &[&str],
    ) -> Result<Vec<String>, Hook0ClientError> {
        let structured_event_types = event_types
            .iter()
            .map(|str| {
                EventType::from_str(str)
                    .map_err(|_| Hook0ClientError::InvalidEventType(str.to_string()))
            })
            .collect::<Result<Vec<EventType>, Hook0ClientError>>()?;

        let event_types_url = self.mk_url(&["event_types"])?;
        #[derive(Debug, Deserialize)]
        struct ApiEventType {
            event_type_name: String,
        }

        trace!("Getting the list of available event types");
        let available_event_types_vec = self
            .client
            .get(event_types_url.as_str())
            .query(&[("application_id", self.application_id())])
            .send()
            .await
            .map_err(Hook0ClientError::GetAvailableEventTypes)?
            .error_for_status()
            .map_err(Hook0ClientError::GetAvailableEventTypes)?
            .json::<Vec<ApiEventType>>()
            .await
            .map_err(Hook0ClientError::GetAvailableEventTypes)?;
        let available_event_types = available_event_types_vec
            .iter()
            .map(|et| et.event_type_name.to_owned())
            .collect::<HashSet<String>>();
        debug!(
            "There are currently {} event types",
            available_event_types.len(),
        );

        #[derive(Debug, Serialize)]
        struct ApiEventTypePost {
            application_id: Uuid,
            service: String,
            resource_type: String,
            verb: String,
        }
        impl Display for ApiEventTypePost {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}.{}.{}", self.service, self.resource_type, self.verb)
            }
        }

        let mut added_event_types = vec![];
        for event_type in structured_event_types {
            let event_type_str = event_type.to_string();
            if !available_event_types.contains(&event_type_str) {
                debug!("Creating the '{event_type}' event type");

                let body = ApiEventTypePost {
                    application_id: self.application_id,
                    service: event_type.service,
                    resource_type: event_type.resource_type,
                    verb: event_type.verb,
                };

                self.client
                    .post(event_types_url.as_str())
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| Hook0ClientError::CreatingEventType {
                        event_type_name: body.to_string(),
                        error: e,
                    })?
                    .error_for_status()
                    .map_err(|e| Hook0ClientError::CreatingEventType {
                        event_type_name: body.to_string(),
                        error: e,
                    })?;

                added_event_types.push(body.to_string());
            }
        }
        debug!("{} new event types were created", added_event_types.len());

        Ok(added_event_types)
    }
}

#[cfg(feature = "hook0-webhook-consumer")]
/// Verifies the signature of a webhook
///
/// - `signature` - The value of the `X-Hook0-Signature` header.
/// - `payload` - The raw body of the webhook request.
/// - `subscription_secret` - The signing secret used to validate the signature.
/// - `tolerance` - The maximum allowed time difference for the timestamp (5 minutes is a good trade-off between flexibility and protecting against replay attacks).
pub fn verify_webhook_signature(
    signature: &str,
    payload: &[u8],
    subscription_secret: &str,
    tolerance: std::time::Duration,
) -> Result<(), Hook0ClientError> {
    let parsed_sig =
        signature::Signature::parse(signature).map_err(|_| Hook0ClientError::InvalidSignature)?;

    if !parsed_sig.verify(payload, subscription_secret) {
        Err(Hook0ClientError::InvalidSignature)
    } else {
        let now = Utc::now();

        let signed_at = DateTime::from_timestamp(parsed_sig.timestamp, 0);

        match signed_at {
            Some(signed_at) => {
                let tolerance = Duration::from_std(tolerance);
                match tolerance {
                    Ok(tolerance) => {
                        if (now - signed_at) > tolerance {
                            Err(Hook0ClientError::ExpiredWebhook {
                                signed_at,
                                tolerance,
                                current_time: now,
                            })
                        } else {
                            Ok(())
                        }
                    }
                    Err(e) => Err(Hook0ClientError::InvalidTolerance(e)),
                }
            }
            None => Err(Hook0ClientError::InvalidSignature),
        }
    }
}

#[cfg(feature = "hook0-webhook-producer")]
/// A structured event type
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct EventType {
    service: String,
    resource_type: String,
    verb: String,
}

#[cfg(feature = "hook0-webhook-producer")]
impl FromStr for EventType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = regex_captures!("^([A-Z0-9_]+)[.]([A-Z0-9_]+)[.]([A-Z0-9_]+)$"i, s);
        if let Some((_, service, resource_type, verb)) = captures {
            Ok(Self {
                resource_type: resource_type.to_owned(),
                service: service.to_owned(),
                verb: verb.to_owned(),
            })
        } else {
            Err(())
        }
    }
}

#[cfg(feature = "hook0-webhook-producer")]
impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.service, self.resource_type, self.verb)
    }
}

#[cfg(feature = "hook0-webhook-producer")]
/// An event that can be sent to Hook0
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<'a> {
    /// Unique ID of the event (a UUIDv4 will be generated if nothing is provided)
    pub event_id: &'a Option<&'a Uuid>,
    /// Type of the event (as configured in your Hook0 application)
    pub event_type: &'a str,
    /// Payload
    pub payload: Cow<'a, str>,
    /// Content type of the payload
    pub payload_content_type: &'a str,
    /// Optional key-value metadata
    pub metadata: Option<Vec<(String, Value)>>,
    /// Datetime of when the event occurred (current time will be used if nothing is provided)
    pub occurred_at: Option<DateTime<Utc>>,
    /// Labels that Hook0 will use to route the event
    pub labels: Vec<(String, Value)>,
}

#[cfg(feature = "hook0-webhook-producer")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FullEvent<'a> {
    pub application_id: Uuid,
    pub event_id: Uuid,
    pub event_type: &'a str,
    pub payload: &'a str,
    pub payload_content_type: &'a str,
    pub metadata: Option<Map<String, Value>>,
    pub occurred_at: DateTime<Utc>,
    pub labels: Map<String, Value>,
}

#[cfg(feature = "hook0-webhook-producer")]
impl<'a> FullEvent<'a> {
    pub fn from_event(event: &'a Event, application_id: &Uuid) -> Self {
        let event_id = event
            .event_id
            .map(|uuid| uuid.to_owned())
            .unwrap_or_else(Uuid::new_v4);
        let occurred_at = event.occurred_at.unwrap_or_else(Utc::now);

        Self {
            application_id: application_id.to_owned(),
            event_id,
            event_type: event.event_type,
            payload: event.payload.as_ref(),
            payload_content_type: event.payload_content_type,
            metadata: event
                .metadata
                .as_ref()
                .map(|items| Map::from_iter(items.iter().cloned())),
            occurred_at,
            labels: Map::from_iter(event.labels.iter().cloned()),
        }
    }
}

/// Every error Hook0 client can encounter
#[derive(Debug, thiserror::Error)]
pub enum Hook0ClientError {
    #[cfg(feature = "hook0-webhook-producer")]
    /// Cannot build a structurally-valid `Authorization` header
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build auth header: {0}")]
    AuthHeader(InvalidHeaderValue),

    #[cfg(feature = "hook0-webhook-producer")]
    /// Cannot build a Reqwest HTTP client
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build reqwest HTTP client: {0}")]
    ReqwestClient(reqwest::Error),

    #[cfg(feature = "hook0-webhook-producer")]
    /// Cannot build a structurally-valid endpoint URL
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not create a valid URL to request Hook0's API: {0}")]
    Url(ParseError),

    #[cfg(feature = "hook0-webhook-producer")]
    /// Something went wrong when sending an event to Hook0
    #[error("Sending event {event_id} failed: {error} [body={}]", body.as_deref().unwrap_or(""))]
    EventSending {
        /// ID of the event
        event_id: Uuid,

        /// Error as reported by Reqwest
        error: reqwest::Error,

        /// Body of the HTTP response
        body: Option<String>,
    },

    #[cfg(feature = "hook0-webhook-producer")]
    /// Provided event type does not have a valid syntax
    #[error("Provided event type '{0}' does not have a valid syntax (service.resource_type.verb)")]
    InvalidEventType(String),

    #[cfg(feature = "hook0-webhook-producer")]
    /// Something went wrong when trying to fetch the list of available event types
    #[error("Getting available event types failed: {0}")]
    GetAvailableEventTypes(reqwest::Error),

    #[cfg(feature = "hook0-webhook-producer")]
    /// Something went wrong when creating an event type
    #[error("Creating event type '{event_type_name}' failed: {error}")]
    CreatingEventType {
        /// Name of the event type
        event_type_name: String,

        /// Error as reported by Reqwest
        error: reqwest::Error,
    },

    #[cfg(feature = "hook0-webhook-consumer")]
    /// The webhook signature is invalid
    #[error("Invalid signature")]
    InvalidSignature,

    #[cfg(feature = "hook0-webhook-consumer")]
    /// The webhook has expired because it was sent too long ago
    #[error("The webhook has expired because it was sent too long ago (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})")]
    ExpiredWebhook {
        /// Timestamp when the webhook was signed
        signed_at: DateTime<Utc>,

        /// Maximum allowed time difference for the timestamp (5 minutes is a good trade-off between flexibility and protecting against replay attacks)
        tolerance: Duration,

        /// Current time
        current_time: DateTime<Utc>,
    },

    #[cfg(feature = "hook0-webhook-consumer")]
    /// Could not parse signature
    #[error("Could not parse signature: {0}")]
    SignatureParsing(String),

    #[cfg(feature = "hook0-webhook-consumer")]
    /// Could not parse timestamp in signature
    #[error("Could not parse timestamp in signature: {0}")]
    TimestampParsingInSignature(String),

    #[cfg(feature = "hook0-webhook-consumer")]
    /// Invalid tolerance Duration
    #[error("Invalid tolerance Duration: {0}")]
    InvalidTolerance(OutOfRangeError),
}

#[cfg(feature = "hook0-webhook-producer")]
impl Hook0ClientError {
    /// Log the error (using the log crate) and return it as a result of this function's call
    pub fn log_and_return(self) -> Self {
        error!("{self}");
        self
    }
}

#[cfg(feature = "hook0-webhook-producer")]
fn append_url_segments(base_url: &Url, segments: &[&str]) -> Result<Url, url::ParseError> {
    const SEP: &str = "/";
    let segments_str = segments.join(SEP);

    let url = Url::parse(&format!("{base_url}/{segments_str}").replace("//", "/"))?;

    Ok(url)
}

#[cfg(feature = "hook0-webhook-producer")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displaying_event_type() {
        let et = EventType {
            service: "service".to_owned(),
            resource_type: "resource".to_owned(),
            verb: "verb".to_owned(),
        };

        assert_eq!(et.to_string(), "service.resource.verb")
    }

    #[test]
    fn parsing_valid_event_type() {
        let et = EventType {
            service: "service".to_owned(),
            resource_type: "resource".to_owned(),
            verb: "verb".to_owned(),
        };

        assert_eq!(EventType::from_str(&et.to_string()), Ok(et))
    }

    #[test]
    fn parsing_invalid_event_type() {
        assert_eq!(EventType::from_str("test.test"), Err(()))
    }
}
