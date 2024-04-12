use actix_web::web::ReqData;
use base64::engine::general_purpose::STANDARD as Base64;
use base64::Engine;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{query_as, query_scalar};
use std::collections::HashMap;
use std::str::FromStr;
use strum::{IntoStaticStr, VariantNames};
use uuid::Uuid;
use validator::Validate;

use crate::extractor_user_ip::UserIp;
use crate::iam::{authorize_for_application, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::quotas::Quota;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantNames)]
pub enum PayloadContentType {
    #[strum(serialize = "text/plain")]
    Text,
    #[strum(serialize = "application/json")]
    Json,
    #[strum(serialize = "application/octet-stream+base64")]
    Binary,
}

impl FromStr for PayloadContentType {
    type Err = Hook0Problem;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let text: &str = Self::Text.into();
        let json: &str = Self::Json.into();
        let binary: &str = Self::Binary.into();

        match s {
            s if s == text => Ok(Self::Text),
            s if s == json => Ok(Self::Json),
            s if s == binary => Ok(Self::Binary),
            _ => Err(Hook0Problem::EventInvalidPayloadContentType),
        }
    }
}

impl PayloadContentType {
    pub fn validate_and_decode(&self, payload: &str) -> Result<Vec<u8>, Hook0Problem> {
        match self {
            Self::Text => Ok(payload.as_bytes().to_vec()),
            Self::Json => {
                serde_json::from_str::<Value>(payload)
                    .map_err(|e| Hook0Problem::EventInvalidJsonPayload(e.to_string()))?;
                Ok(payload.as_bytes().to_vec())
            }
            Self::Binary => Ok(Base64
                .decode(payload)
                .map_err(|e| Hook0Problem::EventInvalidBase64Payload(e.to_string()))?),
        }
    }
}

#[api_v2_operation(
    summary = "List supported event payload content types",
    description = "List of every possible content types that can be used in event payloads.",
    operation_id = "payload_content_types.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn payload_content_types() -> Result<Json<Vec<&'static str>>, Hook0Problem> {
    Ok(Json(PayloadContentType::VARIANTS.to_vec()))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
}

#[derive(Debug)]
#[allow(non_snake_case)]
struct EventRaw {
    event__id: Uuid,
    event_type__name: String,
    payload_content_type: String,
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
            payload_content_type: self.payload_content_type.clone(),
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
    payload_content_type: String,
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
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Event>>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventList {
            application_id: &qs.application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let raw_events = query_as!(
            EventRaw,
            "
                SELECT event__id, event_type__name, payload_content_type, ip, metadata, occurred_at, received_at, application_secret__token, labels
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
    payload_content_type: String,
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
            payload: Base64.encode(self.payload.as_slice()),
            payload_content_type: self.payload_content_type.clone(),
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
    payload_content_type: String,
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
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<Json<EventWithPayload>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventGet {
            application_id: &qs.application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let raw_event = query_as!(
            EventWithPayloadRaw,
            "
                SELECT event__id, event_type__name, payload, payload_content_type, ip, metadata, occurred_at, received_at, application_secret__token, labels
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
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    ip: UserIp,
    body: Json<EventPost>,
) -> Result<CreatedJson<IngestedEvent>, Hook0Problem> {
    let application_id = body.application_id;

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventIngest {
            application_id: &application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }
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

    let can_exceed_events_per_day_quota = query_scalar!(
        "
            SELECT o.price__id
            FROM event.application AS a
            INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
            WHERE a.application__id = $1
        ",
        &application_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .is_some();
    let events_per_days_limit = state
        .quotas
        .get_limit_for_application(&state.db, Quota::EventsPerDay, &application_id)
        .await?;

    let can_ingest = if can_exceed_events_per_day_quota {
        true
    } else {
        let current_events_per_day = query_scalar!(
            r#"
                SELECT COALESCE(amount, 0) AS "amount!"
                FROM event.events_per_day
                WHERE application__id = $1 AND date = current_date
            "#,
            application_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?
        .unwrap_or(0);

        current_events_per_day < events_per_days_limit
    };

    if can_ingest {
        let content_type = PayloadContentType::from_str(&body.payload_content_type)?;
        let payload = content_type.validate_and_decode(&body.payload)?;

        let event = query_as!(
                IngestedEvent,
                "
                    INSERT INTO event.event (application__id, event__id, event_type__name, payload, payload_content_type, ip, metadata, occurred_at, received_at, labels)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, statement_timestamp(), $9)
                    RETURNING application__id AS application_id, event__id AS event_id, received_at
                ",
                application_id,
                &body.event_id,
                &body.event_type,
                &payload,
                &body.payload_content_type,
                ip.into_inner(),
                metadata,
                &body.occurred_at,
                labels,
            )
            .fetch_one(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

        Ok(CreatedJson(event))
    } else {
        Err(Hook0Problem::TooManyEventsToday(events_per_days_limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use strum::VariantNames;

    #[test]
    fn payload_content_type_parsing() {
        for v in PayloadContentType::VARIANTS {
            let parsed_and_serialized: &str = PayloadContentType::from_str(v).unwrap().into();
            assert_eq!(*v, parsed_and_serialized);
        }
    }

    #[test]
    fn validate_json_payload() {
        let valid_payload = r#"{"test": true}"#;
        let invalid_payload = r#"{"test": true"#;

        assert_eq!(
            valid_payload.as_bytes().to_vec(),
            PayloadContentType::Json
                .validate_and_decode(valid_payload)
                .unwrap()
        );
        assert!(matches!(
            PayloadContentType::Json.validate_and_decode(invalid_payload),
            Err(Hook0Problem::EventInvalidJsonPayload(_))
        ));
    }

    #[test]
    fn validate_binary_payload() {
        let empty: Vec<u8> = vec![];
        let valid_payload = b"test";
        let valid_encoded_payload = Base64.encode(valid_payload);
        let invalid_payload = "   ";

        assert_eq!(
            empty,
            PayloadContentType::Binary.validate_and_decode("").unwrap()
        );
        assert_eq!(
            valid_payload,
            PayloadContentType::Binary
                .validate_and_decode(&valid_encoded_payload)
                .unwrap()
                .as_slice()
        );
        assert!(matches!(
            PayloadContentType::Binary.validate_and_decode(invalid_payload),
            Err(Hook0Problem::EventInvalidBase64Payload(_))
        ));
    }
}
