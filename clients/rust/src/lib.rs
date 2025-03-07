// Force exposed items to be documented
#![deny(missing_docs)]

//! This is the Rust client for Hook0.
//! It makes it easier to send events from a Rust application to a Hook0 instance.

#[cfg(all(
    not(feature = "reqwest-rustls-tls-webpki-roots"),
    not(feature = "reqwest-rustls-tls-native-roots")
))]
compile_error!(
    "at least one of feature \"reqwest-rustls-tls-webpki-roots\" and feature \"reqwest-rustls-tls-native-roots\" must be enabled"
);

#[cfg(all(not(feature = "producer"), not(feature = "consumer")))]
compile_error!("at least one of feature \"producer\" and feature \"consumer\" must be enabled");

use chrono::{DateTime, Utc};

#[cfg(feature = "producer")]
use lazy_regex::regex_captures;
#[cfg(feature = "producer")]
use log::{debug, error, trace};
#[cfg(feature = "producer")]
use reqwest::header::{AUTHORIZATION, InvalidHeaderValue};
#[cfg(feature = "producer")]
use reqwest::{Client, Url};
#[cfg(feature = "producer")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "producer")]
use serde_json::{Map, Value};
#[cfg(feature = "producer")]
use std::borrow::Cow;
#[cfg(feature = "producer")]
use std::collections::HashSet;
#[cfg(feature = "producer")]
use std::fmt::Display;
#[cfg(feature = "producer")]
use std::str::FromStr;
#[cfg(feature = "producer")]
use url::ParseError;
#[cfg(feature = "producer")]
use uuid::Uuid;

#[cfg(feature = "consumer")]
use chrono::{Duration, OutOfRangeError};
#[cfg(feature = "consumer")]
use std::time::Duration as StdDuration;
#[cfg(feature = "consumer")]
mod signature;

#[cfg(feature = "producer")]
/// The Hook0 client
///
/// This struct is supposed to be initialized once and shared/reused wherever you need to send events in your app.
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    api_url: Url,
    application_id: Uuid,
}

#[cfg(feature = "producer")]
impl Hook0Client {
    /// Initialize a client
    ///
    /// - `api_url` - Base API URL of a Hook0 instance (example: `https://app.hook0.com/api/v1`).
    /// - `application_id` - UUID of your Hook0 application.
    /// - `token` - Authentication token valid for your Hook0 application.
    pub fn new(api_url: Url, application_id: Uuid, token: &str) -> Result<Self, Hook0ClientError> {
        let authenticated_client =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| Hook0ClientError::AuthHeader(e).log_and_return())
                .map(|hv| reqwest::header::HeaderMap::from_iter([(AUTHORIZATION, hv)]))
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

#[cfg(feature = "consumer")]
/// Verifies the signature of a webhook
///
/// - `signature` - The value of the `X-Hook0-Signature` header.
/// - `payload` - The raw body of the webhook request.
/// - `headers` - Headers of the webhook request.
/// - `subscription_secret` - The signing secret used to validate the signature.
/// - `tolerance` - The maximum allowed time difference for the timestamp (5 minutes is a good trade-off between flexibility and protecting against replay attacks).
/// - `current_time` - The current time (used to check the timestamp).
pub fn verify_webhook_signature_with_current_time<
    HeaderKey: AsRef<[u8]>,
    HeaderValue: AsRef<[u8]>,
>(
    signature: &str,
    payload: &[u8],
    headers: &[(HeaderKey, HeaderValue)],
    subscription_secret: &str,
    tolerance: StdDuration,
    current_time: DateTime<Utc>,
) -> Result<(), Hook0ClientError> {
    let parsed_sig =
        signature::Signature::parse(signature).map_err(|_| Hook0ClientError::InvalidSignature)?;

    let headers_with_parsed_name = headers
        .iter()
        .map(|(k, v)| {
            let name = http::HeaderName::from_bytes(k.as_ref()).map_err(|error| {
                Hook0ClientError::InvalidHeaderName {
                    header_name: String::from_utf8_lossy(k.as_ref()).into_owned(),
                    error,
                }
            });
            name.map(|n| (n, v))
        })
        .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
    let headers_vec = parsed_sig
        .h
        .iter()
        .map(|expected| {
            headers_with_parsed_name
                .get(expected)
                .ok_or_else(|| Hook0ClientError::MissingHeader(expected.to_owned()))
                .and_then(|v| {
                    String::from_utf8(v.as_ref().to_vec()).map_err(|error| {
                        Hook0ClientError::InvalidHeaderValue {
                            header_name: expected.to_owned(),
                            header_value: String::from_utf8_lossy(v.as_ref()).into_owned(),
                            error,
                        }
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if !parsed_sig.verify(payload, &headers_vec, subscription_secret) {
        Err(Hook0ClientError::InvalidSignature)
    } else {
        let signed_at = DateTime::from_timestamp(parsed_sig.timestamp, 0);

        match signed_at {
            Some(signed_at) => {
                let tolerance = Duration::from_std(tolerance);
                match tolerance {
                    Ok(tolerance) => {
                        if (current_time - signed_at) > tolerance {
                            Err(Hook0ClientError::ExpiredWebhook {
                                signed_at,
                                tolerance,
                                current_time,
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

#[cfg(feature = "consumer")]
/// Verifies the signature of a webhook
///
/// - `signature` - The value of the `X-Hook0-Signature` header.
/// - `payload` - The raw body of the webhook request.
/// - `headers` - Headers of the webhook request.
/// - `subscription_secret` - The signing secret used to validate the signature.
/// - `tolerance` - The maximum allowed time difference for the timestamp (5 minutes is a good trade-off between flexibility and protecting against replay attacks).
pub fn verify_webhook_signature<HeaderKey: AsRef<[u8]>, HeaderValue: AsRef<[u8]>>(
    signature: &str,
    payload: &[u8],
    headers: &[(HeaderKey, HeaderValue)],
    subscription_secret: &str,
    tolerance: StdDuration,
) -> Result<(), Hook0ClientError> {
    verify_webhook_signature_with_current_time(
        signature,
        payload,
        headers,
        subscription_secret,
        tolerance,
        Utc::now(),
    )
}

#[cfg(feature = "producer")]
/// A structured event type
#[derive(Debug, Serialize, PartialEq, Eq)]
struct EventType {
    service: String,
    resource_type: String,
    verb: String,
}

#[cfg(feature = "producer")]
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

#[cfg(feature = "producer")]
impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.service, self.resource_type, self.verb)
    }
}

#[cfg(feature = "producer")]
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

#[cfg(feature = "producer")]
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

#[cfg(feature = "producer")]
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
    #[cfg(feature = "producer")]
    /// Cannot build a structurally-valid `Authorization` header
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build auth header: {0}")]
    AuthHeader(InvalidHeaderValue),

    #[cfg(feature = "producer")]
    /// Cannot build a Reqwest HTTP client
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not build reqwest HTTP client: {0}")]
    ReqwestClient(reqwest::Error),

    #[cfg(feature = "producer")]
    /// Cannot build a structurally-valid endpoint URL
    ///
    /// _This is an internal error that is unlikely to happen._
    #[error("Could not create a valid URL to request Hook0's API: {0}")]
    Url(ParseError),

    #[cfg(feature = "producer")]
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

    #[cfg(feature = "producer")]
    /// Provided event type does not have a valid syntax
    #[error("Provided event type '{0}' does not have a valid syntax (service.resource_type.verb)")]
    InvalidEventType(String),

    #[cfg(feature = "producer")]
    /// Something went wrong when trying to fetch the list of available event types
    #[error("Getting available event types failed: {0}")]
    GetAvailableEventTypes(reqwest::Error),

    #[cfg(feature = "producer")]
    /// Something went wrong when creating an event type
    #[error("Creating event type '{event_type_name}' failed: {error}")]
    CreatingEventType {
        /// Name of the event type
        event_type_name: String,

        /// Error as reported by Reqwest
        error: reqwest::Error,
    },

    #[cfg(feature = "consumer")]
    /// The webhook signature is invalid
    #[error("Invalid signature")]
    InvalidSignature,

    #[cfg(feature = "consumer")]
    /// The webhook has expired because it was sent too long ago
    #[error(
        "The webhook has expired because it was sent too long ago (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})"
    )]
    ExpiredWebhook {
        /// Timestamp of the moment the webhook was signed
        signed_at: DateTime<Utc>,

        /// Maximum difference between the signature timestamp and the current time for the webhook to be considered valid
        tolerance: Duration,

        /// Current time
        current_time: DateTime<Utc>,
    },

    #[cfg(feature = "consumer")]
    /// Could not parse signature header
    #[error("Could not parse signature header: {0}")]
    SignatureHeaderParsing(String),

    #[cfg(feature = "consumer")]
    /// Could not parse timestamp in signature
    #[error("Could not parse timestamp `{timestamp}` in signature: {error}")]
    TimestampParsing {
        /// Invalid timestamp value
        timestamp: String,

        /// Timestamp parsing error
        error: std::num::ParseIntError,
    },

    #[cfg(feature = "consumer")]
    /// Could not parse v0 signature
    #[error("Could not parse v0 signature `{signature}`: {error}")]
    V0SignatureParsing {
        /// Invalid signature value
        signature: String,

        /// Signature parsing error
        error: hex::FromHexError,
    },

    #[cfg(feature = "consumer")]
    /// Could not parse header names (`h` field)
    #[error("Could not parse header name `{header}` in `h` field: {error}")]
    HeaderNameParsing {
        /// Invalid header name
        header: String,

        /// Header name parsing error
        error: http::header::InvalidHeaderName,
    },

    #[cfg(feature = "consumer")]
    /// Could not parse v1 signature
    #[error("Could not parse v1 signature `{signature}`: {error}")]
    V1SignatureParsing {
        /// Invalid signature value
        signature: String,

        /// Signature parsing error
        error: hex::FromHexError,
    },

    #[cfg(feature = "consumer")]
    /// A header present in the webhook's signature was not provided with a value
    #[error("The `{0}` header present in the webhook's signature was not provided with a value")]
    MissingHeader(http::HeaderName),

    #[cfg(feature = "consumer")]
    /// Provided header has an invalid name
    #[error("Provided `{header_name}` has an invalid header name: {error}")]
    InvalidHeaderName {
        /// Invalid header name
        header_name: String,

        /// Header name parsing error
        error: http::header::InvalidHeaderName,
    },

    #[cfg(feature = "consumer")]
    /// Provided header has an invalid value
    #[error("Provided `{header_name}` has an invalid header value `{header_value}`: {error}")]
    InvalidHeaderValue {
        /// Header name
        header_name: http::HeaderName,

        /// Invalid header value
        header_value: String,

        /// Header value parsing error
        error: std::string::FromUtf8Error,
    },

    #[cfg(feature = "consumer")]
    /// Invalid tolerance Duration
    #[error("Invalid tolerance Duration: {0}")]
    InvalidTolerance(OutOfRangeError),
}

#[cfg(feature = "producer")]
impl Hook0ClientError {
    /// Log the error (using the log crate) and return it as a result of this function's call
    pub fn log_and_return(self) -> Self {
        error!("{self}");
        self
    }
}

#[cfg(feature = "producer")]
fn append_url_segments(base_url: &Url, segments: &[&str]) -> Result<Url, url::ParseError> {
    const SEP: &str = "/";
    let segments_str = segments.join(SEP);

    let url = Url::parse(&format!("{base_url}/{segments_str}").replace("//", "/"))?;

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "producer")]
    #[test]
    fn displaying_event_type() {
        let et = EventType {
            service: "service".to_owned(),
            resource_type: "resource".to_owned(),
            verb: "verb".to_owned(),
        };

        assert_eq!(et.to_string(), "service.resource.verb")
    }

    #[cfg(feature = "producer")]
    #[test]
    fn parsing_valid_event_type() {
        let et = EventType {
            service: "service".to_owned(),
            resource_type: "resource".to_owned(),
            verb: "verb".to_owned(),
        };

        assert_eq!(EventType::from_str(&et.to_string()), Ok(et))
    }

    #[cfg(feature = "producer")]
    #[test]
    fn parsing_invalid_event_type() {
        assert_eq!(EventType::from_str("test.test"), Err(()))
    }

    #[cfg(feature = "consumer")]
    #[test]
    fn verifying_valid_signature_v0() {
        let signature =
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
        let payload = "hello !".as_bytes();
        let subscription_secret = "secret";
        let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

        assert!(
            verify_webhook_signature::<&str, &str>(
                signature,
                payload,
                &[],
                subscription_secret,
                tolerance
            )
            .is_ok()
        );
    }

    #[cfg(feature = "consumer")]
    #[test]
    fn verifying_valid_signature_v0_with_current_time() {
        let signature =
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
        let payload = "hello !".as_bytes();
        let subscription_secret = "secret";
        let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

        assert!(
            verify_webhook_signature::<&str, &str>(
                signature,
                payload,
                &[],
                subscription_secret,
                tolerance
            )
            .is_ok()
        );
    }

    #[cfg(feature = "consumer")]
    #[test]
    fn verifying_expired_signature_v0() {
        let signature =
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
        let payload = "hello !".as_bytes();
        let subscription_secret = "secret";
        let tolerance = StdDuration::from_secs(300);

        assert!(
            verify_webhook_signature::<&str, &str>(
                signature,
                payload,
                &[],
                subscription_secret,
                tolerance
            )
            .is_err()
        );
    }

    #[cfg(feature = "consumer")]
    #[test]
    fn verifying_valid_signature_v1() {
        let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
        let payload = "hello !".as_bytes();
        let header_values = [("x-test", "val1"), ("x-test2", "val2")];
        let subscription_secret = "secret";
        let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

        assert!(
            verify_webhook_signature::<&str, &str>(
                signature,
                payload,
                &header_values,
                subscription_secret,
                tolerance
            )
            .is_ok()
        );
    }

    #[cfg(feature = "consumer")]
    #[test]
    fn verifying_valid_signature_v1_with_current_time() {
        let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
        let payload = "hello !".as_bytes();
        let header_values = [("x-test", "val1"), ("x-test2", "val2")];
        let subscription_secret = "secret";
        let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

        assert!(
            verify_webhook_signature::<&str, &str>(
                signature,
                payload,
                &header_values,
                subscription_secret,
                tolerance
            )
            .is_ok()
        );
    }
}
