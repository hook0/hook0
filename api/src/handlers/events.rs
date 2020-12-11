use base64::encode;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query},
    Apiv2Schema, CreatedJson,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::query_as;
use uuid::Uuid;

use crate::errors::*;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct QS {
    application_id: Uuid,
}

#[derive(Debug)]
#[allow(non_snake_case)]
struct EventRaw {
    event__id: Uuid,
    event_type__name: String,
    payload_content_type__name: String,
    ip: IpNetwork,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret__token: Uuid,
    labels: Value,
}

impl EventRaw {
    pub fn to_event(&self) -> Event {
        Event {
            event__id: self.event__id,
            event_type__name: self.event_type__name.clone(),
            payload_content_type__name: self.payload_content_type__name.clone(),
            ip: self.ip.ip().to_string(),
            metadata: self.metadata.clone(),
            occurred_at: self.occurred_at,
            received_at: self.received_at,
            application_secret__token: self.application_secret__token,
            labels: self.labels.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct Event {
    event__id: Uuid,
    event_type__name: String,
    payload_content_type__name: String,
    ip: String,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret__token: Uuid,
    labels: Value,
}

/// List latest events
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    qs: Query<QS>,
) -> Result<Json<Vec<Event>>, UnexpectedError> {
    let raw_events = query_as!(
        EventRaw,
        "
            SELECT event__id, event_type__name, payload_content_type__name, ip, metadata, occurred_at, received_at, application_secret__token, labels
            FROM event.event
            WHERE application__id = $1
            ORDER BY received_at DESC
            LIMIT 100
        ",
        &qs.application_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| UnexpectedError::InternalServerError)?;

    let events = raw_events.iter().map(|re| re.to_event()).collect();
    Ok(Json(events))
}

#[derive(Debug)]
#[allow(non_snake_case)]
struct EventWithPayloadRaw {
    event__id: Uuid,
    event_type__name: String,
    payload: Vec<u8>,
    payload_content_type__name: String,
    ip: IpNetwork,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret__token: Uuid,
    labels: Value,
}

impl EventWithPayloadRaw {
    pub fn to_event(&self) -> EventWithPayload {
        EventWithPayload {
            event__id: self.event__id,
            event_type__name: self.event_type__name.clone(),
            payload: encode(self.payload.as_slice()),
            payload_content_type__name: self.payload_content_type__name.clone(),
            ip: self.ip.ip().to_string(),
            metadata: self.metadata.clone(),
            occurred_at: self.occurred_at,
            received_at: self.received_at,
            application_secret__token: self.application_secret__token,
            labels: self.labels.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct EventWithPayload {
    event__id: Uuid,
    event_type__name: String,
    payload: String,
    payload_content_type__name: String,
    ip: String,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret__token: Uuid,
    labels: Value,
}

/// Show an event
#[api_v2_operation]
pub async fn show(
    state: Data<crate::State>,
    event_id: Path<Uuid>,
    qs: Query<QS>,
) -> Result<Json<EventWithPayload>, ShowError> {
    let raw_event = query_as!(
        EventWithPayloadRaw,
        "
            SELECT event__id, event_type__name, payload, payload_content_type__name, ip, metadata, occurred_at, received_at, application_secret__token, labels
            FROM event.event
            WHERE application__id = $1 AND event__id = $2
        ",
        &qs.application_id,
        &event_id.into_inner(),
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match raw_event {
        Some(re) => Ok(Json(re.to_event())),
        None => Err(ShowError::NotFound),
    }
}
