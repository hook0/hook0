use chrono::{DateTime, Utc};
use log::error;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue, AUTHORIZATION};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::Value;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<'a> {
    pub event_id: &'a Option<&'a Uuid>,
    pub event_type: &'a str,
    pub payload: &'a str,
    pub payload_content_type: &'a str,
    pub metadata: &'a Option<&'a [(&'a str, &'a Value)]>,
    pub occurred_at: &'a Option<&'a DateTime<Utc>>,
    pub labels: &'a [(&'a str, &'a Value)],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FullEvent<'a> {
    pub application_id: Uuid,
    pub event_id: Uuid,
    pub event_type: &'a str,
    pub payload: &'a str,
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
            payload: event.payload,
            payload_content_type: event.payload_content_type,
            metadata: event.metadata,
            occurred_at,
            labels: event.labels,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Hook0ClientError {
    #[error("Could not build auth header: {0}")]
    AuthHeader(InvalidHeaderValue),

    #[error("Could not build reqwest HTTP client: {0}")]
    ReqwestClient(reqwest::Error),

    #[error("Could not create a valid URL to request Hook0's API: {0}")]
    Url(ParseError),

    #[error("Sending event {event_id} failed: {error}")]
    EventSending {
        event_id: Uuid,
        error: reqwest::Error,
    },
}

impl Hook0ClientError {
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
