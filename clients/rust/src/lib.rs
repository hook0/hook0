// Force exposed items to be documented
#![deny(missing_docs)]

//! This is the Rust client for Hook0.
//! It makes it easier to send events from a Rust application to a Hook0 instance.
//!
//! # Sending an event is idempotent, and retried
//!
//! [`Hook0Client::send_event`] sends every event under an ID it knows: the one set on the
//! [`Event`], or a UUIDv7 it generates when the event carries none. Passing no ID no longer means
//! the ID comes from Hook0 — the interface is unchanged, but the value now comes from the client,
//! is sent with the request and is what `send_event` returns.
//!
//! That is what makes retrying safe. Hook0 keys events on their ID, so a request that is repeated
//! after a network failure or a server error ingests the event once rather than twice; without a
//! client-chosen ID, a repeated request would create a second event and deliver it to every
//! subscriber.
//!
//! Every send is bounded, and every bound is configurable:
//! [`Hook0Client::with_max_payload_bytes`] rules an oversized payload out before anything is sent,
//! [`Hook0Client::with_request_timeout`] bounds one attempt, and [`RetryPolicy`] bounds how many
//! attempts are made and how long they may spend waiting between them. Pass
//! [`RetryPolicy::disabled`] to send each event exactly once.

#[cfg(all(not(feature = "producer"), not(feature = "consumer")))]
compile_error!("at least one of feature \"producer\" and feature \"consumer\" must be enabled");

use chrono::{DateTime, Utc};

#[cfg(feature = "producer")]
use lazy_regex::regex_captures;
#[cfg(feature = "producer")]
use reqwest::StatusCode;
#[cfg(feature = "producer")]
use reqwest::header::{AUTHORIZATION, InvalidHeaderValue};
#[cfg(feature = "producer")]
use reqwest::{Client, Url};
#[cfg(feature = "producer")]
use serde::ser::Error as SerializationError;
#[cfg(feature = "producer")]
use serde::{Deserialize, Serialize, Serializer};
#[cfg(feature = "producer")]
use std::borrow::Cow;
#[cfg(feature = "producer")]
use std::collections::hash_map::RandomState;
#[cfg(feature = "producer")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "producer")]
use std::fmt::Display;
#[cfg(feature = "producer")]
use std::hash::{BuildHasher, Hasher};
#[cfg(feature = "producer")]
use std::str::FromStr;
#[cfg(feature = "producer")]
use tracing::{debug, error, trace};
#[cfg(feature = "producer")]
use url::ParseError;
#[cfg(feature = "producer")]
use uuid::Uuid;

#[cfg(feature = "consumer")]
use chrono::{Duration, OutOfRangeError};
#[cfg(any(feature = "consumer", feature = "producer"))]
use std::time::Duration as StdDuration;
#[cfg(feature = "consumer")]
mod signature;

/// Everything the API document describes, written by the SDK generator and never by hand.
///
/// It is reached as a module rather than flattened into this one on purpose. The document declares
/// schemas called `Event` and `EventType`, which are the API's own resources and not the [`Event`]
/// an emitter fills in here; re-exporting them side by side would either fail to compile or, worse,
/// let a glob quietly drop whichever one lost. Under a module of its own, every name the document
/// declares is reachable, unambiguous, and safe for the API to add to.
///
/// It follows the `producer` feature because everything it declares is the control plane of the
/// API: a consumer that only verifies webhook signatures pulls in none of it.
#[cfg(feature = "producer")]
pub mod generated;

#[cfg(feature = "producer")]
/// Longest one attempt at reaching Hook0 is given before it is abandoned, unless the client is
/// told otherwise with [`Hook0Client::with_request_timeout`].
///
/// Ten seconds is far above what ingesting an event takes when the API is healthy, and short
/// enough that a stuck connection does not hold an emitter's task for a noticeable time.
pub const DEFAULT_REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(10);

#[cfg(feature = "producer")]
/// Largest event payload the client agrees to send, unless it is told otherwise with
/// [`Hook0Client::with_max_payload_bytes`].
///
/// Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
/// being refused once the JSON envelope around it (metadata, labels, identifiers) is counted. The
/// client rules such an event out rather than spending a round trip, and every retry after it, on
/// a request that cannot be accepted.
pub const DEFAULT_MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

#[cfg(feature = "producer")]
/// Most attempts a [`RetryPolicy`] can ever make, whatever `max_attempts` says.
///
/// A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
/// `max_attempts` from turning one send into an unbounded series of requests.
pub const MAX_ATTEMPTS_CAP: u32 = 16;

#[cfg(feature = "producer")]
/// How a client spaces out the attempts of a single send.
///
/// The delay before a retry doubles from [`RetryPolicy::initial_backoff`] and is capped by
/// [`RetryPolicy::max_backoff`]; the actual delay is then drawn anywhere between zero and that
/// ceiling, so that emitters which failed at the same moment do not come back at the same moment.
/// Retrying stops as soon as the delays of the send would add up to more than
/// [`RetryPolicy::max_total_delay`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Attempts a single send makes at most, the first one included. `1` disables retrying.
    ///
    /// Never more than [`MAX_ATTEMPTS_CAP`], and never less than one.
    pub max_attempts: u32,

    /// Ceiling of the delay before the first retry.
    pub initial_backoff: StdDuration,

    /// Ceiling no single delay ever exceeds, however many retries were made.
    pub max_backoff: StdDuration,

    /// Budget all the delays of one send share.
    pub max_total_delay: StdDuration,
}

#[cfg(feature = "producer")]
impl Default for RetryPolicy {
    /// Four attempts spread over at most five seconds.
    ///
    /// Three retries absorb the blips a webhook emitter meets in production (a connection reset, a
    /// rolling deployment answering 503) without holding the caller's task for long, and the
    /// five-second budget bounds what the worst send costs whatever the individual delays turn out
    /// to be.
    fn default() -> Self {
        Self {
            max_attempts: 4,
            initial_backoff: StdDuration::from_millis(100),
            max_backoff: StdDuration::from_secs(2),
            max_total_delay: StdDuration::from_secs(5),
        }
    }
}

#[cfg(feature = "producer")]
impl RetryPolicy {
    /// A policy that never retries: one attempt, and the caller hears about whatever it returned.
    pub const fn disabled() -> Self {
        Self {
            max_attempts: 1,
            initial_backoff: StdDuration::ZERO,
            max_backoff: StdDuration::ZERO,
            max_total_delay: StdDuration::ZERO,
        }
    }

    /// Attempts this policy actually makes: [`RetryPolicy::max_attempts`], brought back inside
    /// `1..=`[`MAX_ATTEMPTS_CAP`].
    pub fn attempts(&self) -> u32 {
        self.max_attempts.clamp(1, MAX_ATTEMPTS_CAP)
    }

    /// Ceiling of the delay before retry number `retry`, where `1` is the first retry.
    ///
    /// It doubles from [`RetryPolicy::initial_backoff`] and never exceeds
    /// [`RetryPolicy::max_backoff`], so the ceilings of successive retries never decrease.
    pub fn backoff_ceiling(&self, retry: u32) -> StdDuration {
        // 2^31 doublings of any non-zero duration already saturate `max_backoff`, so the exponent
        // is capped there rather than left to overflow.
        let doublings = retry.saturating_sub(1).min(u32::BITS - 1);
        self.initial_backoff
            .saturating_mul(2u32.saturating_pow(doublings))
            .min(self.max_backoff)
    }

    /// The delays this policy waits between the attempts of one send, one per retry, given one
    /// random draw in `[0, 1)` per retry.
    ///
    /// Each delay lands between zero and the ceiling of its retry, and the schedule is cut short as
    /// soon as the next delay would spend more than [`RetryPolicy::max_total_delay`]. There are
    /// therefore at most [`RetryPolicy::attempts`]` - 1` delays, and they add up to at most
    /// `max_total_delay`.
    ///
    /// A draw that is missing or is not a finite number is read as `1`, which asks for the whole
    /// ceiling: an unusable source of randomness makes the client wait longer, never less.
    pub fn delays(&self, draws: &[f64]) -> Vec<StdDuration> {
        let retries = self.attempts().saturating_sub(1);
        let mut delays = Vec::with_capacity(retries as usize);
        let mut spent = StdDuration::ZERO;

        for retry in 1..=retries {
            let draw = match draws.get((retry - 1) as usize) {
                Some(draw) if draw.is_finite() => draw.clamp(0.0, 1.0),
                _ => 1.0,
            };
            let delay = self.backoff_ceiling(retry).mul_f64(draw);

            if spent.saturating_add(delay) > self.max_total_delay {
                break;
            }
            spent = spent.saturating_add(delay);
            delays.push(delay);
        }

        delays
    }
}

#[cfg(feature = "producer")]
/// Draws used to jitter the delays of one send.
///
/// Jitter only has to keep emitters that failed together from coming back together; it does not
/// have to be unpredictable. The randomness the standard library seeds its hashers with is enough
/// for that, and it keeps this client free of a random-number-generator dependency.
fn jitter_draws(count: usize) -> Vec<f64> {
    // An `f64` carries 53 bits exactly, so keeping the 53 high bits and dividing by 2^53 lands in
    // `[0, 1)` without rounding to 1.
    const KEPT_BITS: u32 = 53;

    (0..count)
        .map(|_| {
            let drawn = RandomState::new().build_hasher().finish();
            (drawn >> (u64::BITS - KEPT_BITS)) as f64 / (1u64 << KEPT_BITS) as f64
        })
        .collect()
}

#[cfg(feature = "producer")]
/// The Hook0 client
///
/// This struct is supposed to be initialized once and shared/reused wherever you need to send events in your app.
#[derive(Debug, Clone)]
pub struct Hook0Client {
    client: Client,
    api_url: Url,
    application_id: Uuid,
    retry_policy: RetryPolicy,
    request_timeout: StdDuration,
    max_payload_bytes: usize,
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
            retry_policy: RetryPolicy::default(),
            request_timeout: DEFAULT_REQUEST_TIMEOUT,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
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

    /// Change how this client retries a send that failed in a way a repetition could fix
    ///
    /// Pass [`RetryPolicy::disabled`] to send each event exactly once.
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Get the retry policy of this client
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Change how long one attempt at reaching Hook0 is given before it is abandoned
    ///
    /// The timeout applies to each attempt, not to the send as a whole.
    pub fn with_request_timeout(mut self, request_timeout: StdDuration) -> Self {
        self.request_timeout = request_timeout;
        self
    }

    /// Get the timeout this client gives one attempt at reaching Hook0
    pub fn request_timeout(&self) -> StdDuration {
        self.request_timeout
    }

    /// Change the largest event payload this client agrees to send
    ///
    /// An event whose payload is larger is refused before any request is issued.
    pub fn with_max_payload_bytes(mut self, max_payload_bytes: usize) -> Self {
        self.max_payload_bytes = max_payload_bytes;
        self
    }

    /// Get the largest event payload this client agrees to send
    pub fn max_payload_bytes(&self) -> usize {
        self.max_payload_bytes
    }

    fn mk_url(&self, segments: &[&str]) -> Result<Url, Hook0ClientError> {
        append_url_segments(&self.api_url, segments)
            .map_err(|e| Hook0ClientError::Url(e).log_and_return())
    }

    /// Send an event to Hook0
    ///
    /// The event is sent under an ID this client knows: the one set on the event, or a UUIDv7 this
    /// client generates when the event carries none. Because Hook0 keys events on that ID, a
    /// request that is repeated after a network failure or a server error ingests the event once,
    /// not twice — which is what makes retrying safe.
    ///
    /// A send is bounded on four axes, each of them configurable:
    /// [`Hook0Client::with_max_payload_bytes`] rules an oversized payload out before anything is
    /// sent, [`Hook0Client::with_request_timeout`] bounds one attempt,
    /// [`RetryPolicy::max_attempts`] bounds how many attempts are made, and
    /// [`RetryPolicy::max_total_delay`] bounds the time spent waiting between them. Only a network
    /// failure or a server error is retried; anything Hook0 refuses outright is reported as is.
    ///
    /// A retried request that Hook0 answers with `EventAlreadyIngested` reports success: an earlier
    /// attempt of this very send reached the API, and the event carries the ID returned here. That
    /// answer to a *first* attempt is a genuine conflict and is reported as an error.
    pub async fn send_event(&self, event: &Event<'_>) -> Result<Uuid, Hook0ClientError> {
        let event_ingestion_url = self.mk_url(&["event"])?;
        let event_id = match event.event_id {
            Some(event_id) => event_id.to_owned(),
            None => Uuid::now_v7(),
        };
        let full_event = FullEvent::from_event(event, &self.application_id, &event_id);
        let body = BoundedEvent {
            event: &full_event,
            max_payload_bytes: self.max_payload_bytes,
        };

        let delays = self.retry_policy.delays(&jitter_draws(
            self.retry_policy.attempts().saturating_sub(1) as usize,
        ));
        let mut waited = StdDuration::ZERO;
        let mut attempts = 0u32;

        loop {
            attempts += 1;
            let outcome = self.attempt_event_send(&event_ingestion_url, &body).await;

            let failure = match outcome {
                Attempt::Ingested(id) => return Ok(id),
                Attempt::AlreadyIngested { error, body } => {
                    if attempts > 1 {
                        debug!(
                            "Event {event_id} was already ingested by a previous attempt of this send"
                        );
                        return Ok(event_id);
                    }
                    Failure {
                        error,
                        body,
                        retryable: false,
                    }
                }
                Attempt::Failed(failure) => failure,
            };

            match delays.get((attempts - 1) as usize) {
                Some(delay) if failure.retryable => {
                    trace!("Attempt {attempts} at sending event {event_id} failed, retrying");
                    waited = waited.saturating_add(*delay);
                    tokio::time::sleep(*delay).await;
                }
                _ => {
                    return Err(Hook0ClientError::EventSending {
                        event_id: Some(event_id),
                        error: failure.error,
                        body: give_up_reason(attempts, waited, failure.body),
                    }
                    .log_and_return());
                }
            }
        }
    }

    /// Perform one attempt at sending an already-bounded event to Hook0
    async fn attempt_event_send(&self, url: &Url, body: &BoundedEvent<'_>) -> Attempt {
        let response = self
            .client
            .post(url.as_str())
            .timeout(self.request_timeout)
            .json(body)
            .send()
            .await;

        let res = match response {
            Ok(res) => res,
            Err(error) => {
                return Attempt::Failed(Failure {
                    retryable: is_transient(&error),
                    body: underlying_cause(&error),
                    error,
                });
            }
        };
        let status = res.status();

        match res.error_for_status_ref() {
            Ok(_) => {
                #[derive(Debug, Deserialize)]
                struct Response {
                    event_id: Uuid,
                }
                match res.json::<Response>().await {
                    Ok(response) => Attempt::Ingested(response.event_id),
                    // Hook0 accepted the event but answered something this client cannot read;
                    // repeating the request would meet the same answer.
                    Err(error) => Attempt::Failed(Failure {
                        error,
                        body: None,
                        retryable: false,
                    }),
                }
            }
            Err(error) => {
                let body = res.text().await.ok();
                if status == StatusCode::CONFLICT && is_already_ingested(body.as_deref()) {
                    Attempt::AlreadyIngested { error, body }
                } else {
                    Attempt::Failed(Failure {
                        error,
                        body,
                        // Only the server side of a 5xx can change between two identical requests.
                        retryable: status.is_server_error(),
                    })
                }
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
/// - `tolerance` - The maximum allowed time difference for the timestamp, in either direction (5 minutes is a good trade-off between flexibility and protecting against replay attacks). A timestamp that is too far in the future is rejected just like one that is too far in the past, so that the acceptance window of any given webhook stays bounded.
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
                        if (current_time - signed_at).abs() > tolerance {
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
/// - `tolerance` - The maximum allowed time difference for the timestamp, in either direction (5 minutes is a good trade-off between flexibility and protecting against replay attacks). A timestamp that is too far in the future is rejected just like one that is too far in the past, so that the acceptance window of any given webhook stays bounded.
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
    /// Unique ID of the event (the client generates a UUIDv7 if nothing is provided)
    ///
    /// Providing nothing no longer means the ID comes from Hook0: [`Hook0Client::send_event`]
    /// generates it, sends it, and returns it. That is what lets it retry a request without
    /// risking a second copy of the event being ingested and delivered to every subscriber.
    pub event_id: Option<&'a Uuid>,
    /// Type of the event (as configured in your Hook0 application)
    pub event_type: &'a str,
    /// Payload
    pub payload: Cow<'a, str>,
    /// Content type of the payload
    pub payload_content_type: &'a str,
    /// Optional key-value metadata
    pub metadata: Option<Vec<(String, String)>>,
    /// Datetime of when the event occurred (current time will be used if nothing is provided)
    pub occurred_at: Option<DateTime<Utc>>,
    /// Labels that Hook0 will use to route the event
    pub labels: Vec<(String, String)>,
}

#[cfg(feature = "producer")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FullEvent<'a> {
    pub application_id: Uuid,
    pub event_id: &'a Uuid,
    pub event_type: &'a str,
    pub payload: &'a str,
    pub payload_content_type: &'a str,
    pub metadata: Option<HashMap<String, String>>,
    pub occurred_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
}

#[cfg(feature = "producer")]
impl<'a> FullEvent<'a> {
    pub fn from_event(event: &'a Event, application_id: &Uuid, event_id: &'a Uuid) -> Self {
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
                .map(|items| HashMap::from_iter(items.iter().cloned())),
            occurred_at,
            labels: HashMap::from_iter(event.labels.iter().cloned()),
        }
    }
}

#[cfg(feature = "producer")]
/// An event that refuses, while it is being serialized, a payload larger than the client accepts.
///
/// The refusal has to happen here rather than in a check of its own: `reqwest` turns a
/// serialization failure into a builder error that it returns from `send` without opening a socket,
/// and that error is the only shape a refusal can take. [`Hook0ClientError::EventSending`] is the
/// one error the send path reports, and the `reqwest::Error` it carries can only be built by
/// `reqwest` itself.
struct BoundedEvent<'a> {
    event: &'a FullEvent<'a>,
    max_payload_bytes: usize,
}

#[cfg(feature = "producer")]
impl Serialize for BoundedEvent<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let size = self.event.payload.len();
        if size > self.max_payload_bytes {
            return Err(S::Error::custom(format!(
                "event payload is {size} bytes, which is more than the {} bytes this client sends at most; nothing was sent",
                self.max_payload_bytes
            )));
        }

        self.event.serialize(serializer)
    }
}

#[cfg(feature = "producer")]
/// What one attempt at sending an event ended with.
enum Attempt {
    /// Hook0 ingested the event and answered with its ID.
    Ingested(Uuid),

    /// Hook0 refused the event because it already holds one under the same ID.
    AlreadyIngested {
        /// Error as reported by Reqwest for the conflict status
        error: reqwest::Error,

        /// Body of the HTTP response
        body: Option<String>,
    },

    /// The attempt did not ingest anything.
    Failed(Failure),
}

#[cfg(feature = "producer")]
/// A failed attempt, and whether repeating it could end differently.
struct Failure {
    error: reqwest::Error,
    body: Option<String>,
    retryable: bool,
}

#[cfg(feature = "producer")]
/// Whether a Reqwest error comes from the transport rather than from what Hook0 answered.
///
/// These are the failures an identical request can survive: a connection that was refused or
/// reset, an attempt that ran out of time, a response whose body stopped mid-way. None of them
/// says whether Hook0 ingested the event, which is precisely why the client sends an ID it chose
/// itself.
fn is_transient(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect() || error.is_request() || error.is_body()
}

#[cfg(feature = "producer")]
/// What a Reqwest error carries under its own summary.
///
/// `reqwest::Error` renders as its kind and nothing else: the refusal
/// [`BoundedEvent`] raises while the event is being serialized reads as `builder error`, and a
/// connection that was reset reads as `error sending request`. What names the actual cause is the
/// chain underneath, which is what a caller needs to see.
fn underlying_cause(error: &reqwest::Error) -> Option<String> {
    /// No error chain a client meets is anywhere near this long; the bound keeps a cyclic one from
    /// being walked forever.
    const MAX_LINKS: usize = 8;

    let mut causes = Vec::new();
    let mut cause = std::error::Error::source(error);
    while let Some(current) = cause {
        if causes.len() >= MAX_LINKS {
            break;
        }
        causes.push(current.to_string());
        cause = current.source();
    }

    if causes.is_empty() {
        None
    } else {
        Some(causes.join(": "))
    }
}

#[cfg(feature = "producer")]
/// Whether an RFC 9457 problem body is the one Hook0 answers when an event ID is already taken.
///
/// The body names the problem but does not carry the event ID, which is the other reason the
/// client has to know the ID it sent.
fn is_already_ingested(body: Option<&str>) -> bool {
    /// Public identifier Hook0 gives that problem.
    const ALREADY_INGESTED: &str = "EventAlreadyIngested";

    #[derive(Debug, Deserialize)]
    struct Problem {
        id: String,
    }

    match body {
        Some(body) => serde_json::from_str::<Problem>(body)
            .map(|problem| problem.id == ALREADY_INGESTED)
            .unwrap_or(false),
        None => false,
    }
}

#[cfg(feature = "producer")]
/// What to report as the body of a send that is being given up on.
///
/// A send that never retried reports what Hook0 answered, unchanged. A send that did retry reports
/// that it ran out of attempts, and how much of its delay budget it spent doing so — without that,
/// an exhausted send and a single refused request are indistinguishable to the caller.
fn give_up_reason(attempts: u32, waited: StdDuration, body: Option<String>) -> Option<String> {
    if attempts <= 1 {
        return body;
    }

    let answer = match body {
        Some(body) => format!("; last response body: {body}"),
        None => String::new(),
    };
    Some(format!(
        "gave up after {attempts} attempts spread over {waited:?} of retry delay{answer}"
    ))
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
    #[error("Sending event{} failed: {error} [body={}]", event_id.map(|id| format!(" {id}")).unwrap_or_else(String::new), body.as_deref().unwrap_or(""))]
    EventSending {
        /// ID of the event
        ///
        /// Always the ID the request was sent under, whether the caller chose it or the client
        /// generated it.
        event_id: Option<Uuid>,

        /// Error as reported by Reqwest
        ///
        /// For a send that was retried, this is what the last attempt ran into.
        error: reqwest::Error,

        /// Body of the HTTP response, or why the client gave up
        ///
        /// A send that was retried until it ran out of attempts or of delay budget says so here,
        /// along with the body of the last response it got.
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
    /// The webhook's signature timestamp is outside the tolerance window
    ///
    /// This covers both a webhook that was signed too long ago (a replay) and one that was signed too far in the future (a clock that is ahead, or a forged timestamp meant to widen the acceptance window).
    #[error(
        "The webhook's signature timestamp is outside the tolerance window (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})"
    )]
    ExpiredWebhook {
        /// Timestamp of the moment the webhook was signed
        signed_at: DateTime<Utc>,

        /// Maximum difference, in either direction, between the signature timestamp and the current time for the webhook to be considered valid
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
    /// Log the error (using the tracing crate) and return it as a result of this function's call
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
