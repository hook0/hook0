use actix_web::HttpRequest;
use base64::encode;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query},
    Apiv2Schema, CreatedJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::query_as;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;
use validator::Validate;

use crate::iam::{AuthProof, Role};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
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
            event_id: self.event__id,
            event_type_name: self.event_type__name.clone(),
            payload_content_type_name: self.payload_content_type__name.clone(),
            ip: self.ip.ip().to_string(),
            metadata: self.metadata.clone(),
            occurred_at: self.occurred_at,
            received_at: self.received_at,
            application_secret_token: self.application_secret__token,
            labels: self.labels.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Event {
    event_id: Uuid,
    event_type_name: String,
    payload_content_type_name: String,
    ip: String,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret_token: Uuid,
    labels: Value,
}

#[api_v2_operation(
    summary = "List latest events",
    description = "",
    operation_id = "events.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn list(
    state: Data<crate::State>,
    auth: AuthProof,
    qs: Query<Qs>,
) -> Result<Json<Vec<Event>>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &qs.application_id, &Role::Viewer)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
        .map_err(Hook0Problem::from)?;

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
            event_id: self.event__id,
            event_type_name: self.event_type__name.clone(),
            payload: encode(self.payload.as_slice()),
            payload_content_type_name: self.payload_content_type__name.clone(),
            ip: self.ip.ip().to_string(),
            metadata: self.metadata.clone(),
            occurred_at: self.occurred_at,
            received_at: self.received_at,
            application_secret_token: self.application_secret__token,
            labels: self.labels.clone(),
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct EventWithPayload {
    event_id: Uuid,
    event_type_name: String,
    payload: String,
    payload_content_type_name: String,
    ip: String,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    application_secret_token: Uuid,
    labels: Value,
}

#[api_v2_operation(
    summary = "Get an event",
    description = "",
    operation_id = "events.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn get(
    state: Data<crate::State>,
    auth: AuthProof,
    event_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<Json<EventWithPayload>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &qs.application_id, &Role::Viewer)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
        .map_err(Hook0Problem::from)?;

    match raw_event {
        Some(re) => Ok(Json(re.to_event())),
        None => Err(Hook0Problem::NotFound),
    }
}

#[derive(Debug, Deserialize, Apiv2Schema, Validate)]
pub struct EventPost {
    application_id: Uuid,
    event_id: Uuid,
    #[validate(non_control_character, length(min = 1, max = 200))]
    event_type: String,
    #[validate(length(max = 699_050))] // 512 kio of payload * 4/3 (base64) in bytes
    payload: String,
    #[validate(non_control_character, length(min = 1, max = 100))]
    payload_content_type: String,
    #[validate(custom = "crate::validators::metadata")]
    metadata: Option<HashMap<String, Value>>,
    occurred_at: DateTime<Utc>,
    #[validate(custom = "crate::validators::labels")]
    labels: HashMap<String, Value>,
}

#[derive(Debug)]
struct ContentTypeLookup {
    nb: Option<i64>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct IngestedEvent {
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
}

#[api_v2_operation(
    summary = "Ingest an event",
    description = "",
    operation_id = "events.ingest",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn ingest(
    state: Data<crate::State>,
    auth: AuthProof,
    req: HttpRequest,
    body: Json<EventPost>,
) -> Result<CreatedJson<IngestedEvent>, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let metadata = match body.metadata.as_ref() {
        Some(m) => serde_json::to_value(m.clone())
            .expect("could not serialize subscription metadata into JSON"),
        None => json!({}),
    };
    let labels = serde_json::to_value(body.labels.clone())
        .expect("could not serialize event labels into JSON");

    if let Some(AuthProof::ApplicationSecret {
        application_id: _,
        name: _,
        secret,
    }) = auth
        .can_access_application(&state.db, &body.application_id, &Role::Editor)
        .await
    {
        let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

        let content_type_lookup = query_as!(
            ContentTypeLookup,
            "
                SELECT COUNT(*) AS nb
                FROM event.payload_content_type
                WHERE payload_content_type__name = $1
            ",
            &body.payload_content_type,
        )
        .fetch_one(&mut tx)
        .await
        .map_err(Hook0Problem::from)?;

        let content_type_ok = matches!(content_type_lookup, ContentTypeLookup { nb: Some(1) });

        let payload = base64::decode(body.payload.as_str());

        match (content_type_ok, payload) {
            (true, Ok(p)) => {
                let ip = req
                    .connection_info()
                    .realip_remote_addr()
                    .and_then(|str| str.split(':').next())
                    .ok_or(Hook0Problem::InternalServerError)
                    .and_then(|str| {
                        IpNetwork::from_str(str).map_err(|e| {
                            error!("{}", &e);
                            Hook0Problem::InternalServerError
                        })
                    })?;

                let event = query_as!(
                    IngestedEvent,
                    "
                        INSERT INTO event.event (application__id, event__id, event_type__name, payload, payload_content_type__name, ip, metadata, occurred_at, received_at, application_secret__token, labels)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, statement_timestamp(), $9, $10)
                        RETURNING application__id AS application_id, event__id AS event_id, received_at
                    ",
                    &body.application_id,
                    &body.event_id,
                    &body.event_type,
                    &p,
                    &body.payload_content_type,
                    &ip,
                    metadata,
                    &body.occurred_at,
                    secret,
                    labels,
                )
                    .fetch_one(&state.db)
                    .await
                    .map_err(Hook0Problem::from)?;

                tx.commit().await.map_err(Hook0Problem::from)?;

                Ok(CreatedJson(event))
            }
            (false, _) => Err(Hook0Problem::EventInvalidPayloadContentType),
            (_, Err(_)) => Err(Hook0Problem::EventInvalidBase64Payload),
        }
    } else {
        Err(Hook0Problem::Forbidden)
    }
}
