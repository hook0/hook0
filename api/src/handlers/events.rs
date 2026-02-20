use actix_web::rt::time::timeout;
use actix_web::web::ReqData;
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as Base64;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::{PgTransaction, query_as, query_scalar};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use strum::{IntoStaticStr, VariantNames};
use uuid::Uuid;
use validator::Validate;

use crate::PulsarConfig;
use crate::extractor_user_ip::UserIp;
use crate::iam::{Action, authorize_for_application};
use crate::mailer::Mail;
use crate::openapi::OaBiscuit;
use crate::opentelemetry::{
    report_event_payloads_stored_in_object_storage, report_ingested_events, report_replayed_events,
    report_request_attempts_sent_to_pulsar,
};
use crate::problems::Hook0Problem;
use crate::quotas::{Quota, QuotaNotificationType};
use hook0_protobuf::RequestAttempt;
use hook0_sentry_integration::log_object_storage_error_with_context;

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
    description = "Returns the list of valid content types for event payloads: 'text/plain' for plain text, 'application/json' for JSON data, 'application/octet-stream+base64' for binary data encoded as base64.",
    operation_id = "payload_content_types.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp")
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
    labels: Value,
}

#[api_v2_operation(
    summary = "List latest events",
    description = "Retrieves the 100 most recently ingested events for an application. Each event includes its type, payload content type, metadata, labels, and timestamps. Use application_id query parameter to filter by application.",
    operation_id = "events.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp")
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let raw_events = query_as!(
            EventRaw,
            "
                SELECT event__id, event_type__name, payload_content_type, ip, metadata, occurred_at, received_at, labels
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
    payload: Option<Vec<u8>>,
    payload_content_type: String,
    ip: IpNetwork,
    metadata: Option<Value>,
    occurred_at: DateTime<Utc>,
    received_at: DateTime<Utc>,
    labels: Value,
}

impl EventWithPayloadRaw {
    pub fn to_event(&self, payload: &[u8]) -> EventWithPayload {
        EventWithPayload {
            event_id: self.event__id,
            event_type_name: self.event_type__name.clone(),
            payload: Base64.encode(payload),
            payload_content_type: self.payload_content_type.clone(),
            ip: self.ip.ip().to_string(),
            metadata: self.metadata.clone(),
            occurred_at: self.occurred_at,
            received_at: self.received_at,
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
    labels: Value,
}

#[api_v2_operation(
    summary = "Get an event by its ID",
    description = "Retrieves full details of a specific event including its payload (base64-encoded), event type, content type, metadata, labels, and timestamps. The event must belong to the specified application.",
    operation_id = "events.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp")
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let event_id = event_id.into_inner();

    let raw_event = query_as!(
            EventWithPayloadRaw,
            "
                SELECT event__id, event_type__name, payload, payload_content_type, ip, metadata, occurred_at, received_at, labels
                FROM event.event
                WHERE application__id = $1 AND event__id = $2
            ",
            &qs.application_id,
            &event_id,
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    match raw_event {
        Some(re) => {
            if let Some(p) = &re.payload {
                Ok(Json(re.to_event(p)))
            } else if let Some(object_storage) = &state.object_storage {
                let key = format!(
                    "{}/event/{}/{event_id}",
                    qs.application_id,
                    re.received_at.naive_utc().date()
                );
                let payload_object = object_storage
                    .client
                    .get_object()
                    .bucket(&object_storage.bucket)
                    .key(&key)
                    .send()
                    .await
                    .map_err(|e| {
                        log_object_storage_error_with_context!(
                            "S3 GET OBJECT failed",
                            error_chain = DisplayErrorContext(&e).to_string(),
                            object_key = &key,
                        );
                        Hook0Problem::InternalServerError
                    })?;
                let payload = payload_object
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        log_object_storage_error_with_context!(
                            "S3 GET OBJECT body collect failed",
                            error_chain = format!("{e}"),
                            object_key = &key,
                        );
                        Hook0Problem::InternalServerError
                    })?
                    .to_vec();
                Ok(Json(re.to_event(&payload)))
            } else {
                error!(
                    "Payload of event {event_id} is not in database but object storage is disabled"
                );
                Ok(Json(re.to_event(&[])))
            }
        }
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
    #[validate(custom(function = "crate::validators::metadata"))]
    metadata: Option<HashMap<String, String>>,
    occurred_at: DateTime<Utc>,
    #[validate(custom(function = "crate::validators::labels"))]
    labels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct IngestedEvent {
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
}

#[api_v2_operation(
    summary = "Ingest an event",
    description = "Sends an event to Hook0 for processing. The event will be matched against active subscriptions based on event type and labels, triggering webhook deliveries to matching endpoints. Requires event_type, payload, payload_content_type, labels, and occurred_at.",
    operation_id = "events.ingest",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp")
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
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

    let current_events_per_day = if can_exceed_events_per_day_quota {
        0
    } else {
        query_scalar!(
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
        .unwrap_or(0)
    };

    let can_ingest = if can_exceed_events_per_day_quota {
        true
    } else {
        current_events_per_day < events_per_days_limit
    };

    if can_ingest {
        if state.enable_quota_based_email_notifications {
            let actual_consumption_percent = 100 * current_events_per_day / events_per_days_limit;

            if actual_consumption_percent
                > i32::from(state.quota_notification_events_per_day_threshold)
            {
                let mail = Mail::QuotaEventsPerDayWarning {
                    pricing_url_hash: "#pricing".to_owned(),
                    actual_consumption_percent,
                    current_events_per_day,
                    events_per_days_limit,
                    extra_variables: Vec::new(),
                };
                state
                    .quotas
                    .send_application_email_notification(
                        &state,
                        Quota::EventsPerDay,
                        QuotaNotificationType::Warning,
                        application_id,
                        mail,
                    )
                    .await?;
            }
        }

        let content_type = PayloadContentType::from_str(&body.payload_content_type)?;
        let payload = content_type.validate_and_decode(&body.payload)?;

        let mut tx = state.db.begin().await?;

        let payload_to_insert = if let Some(true) =
            state.object_storage.as_ref().map(|object_storage| {
                object_storage.store_event_payloads
                    && (object_storage.store_event_only_for.is_empty()
                        || object_storage
                            .store_event_only_for
                            .contains(&application_id))
            }) {
            None
        } else {
            Some(&payload)
        };
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
                payload_to_insert,
                &body.payload_content_type,
                IpNetwork::from(ip.into_inner()),
                metadata,
                &body.occurred_at,
                labels,
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

        if let Some(object_storage) = &state.object_storage
            && object_storage.store_event_payloads
            && (object_storage.store_event_only_for.is_empty()
                || object_storage
                    .store_event_only_for
                    .contains(&application_id))
        {
            let key = format!(
                "{application_id}/event/{}/{}",
                event.received_at.naive_utc().date(),
                &body.event_id
            );
            object_storage
                .client
                .put_object()
                .bucket(&object_storage.bucket)
                .key(&key)
                .content_type(&body.payload_content_type)
                .body(ByteStream::from(payload.clone()))
                .send()
                .await
                .map_err(|e| {
                    log_object_storage_error_with_context!(
                        "S3 PUT OBJECT failed",
                        error_chain = DisplayErrorContext(&e).to_string(),
                        object_key = &key,
                    );
                    Hook0Problem::InternalServerError
                })?;
            report_event_payloads_stored_in_object_storage(1);
        }

        if let Some(pulsar) = &state.pulsar {
            send_request_attempts_to_pulsar(
                &mut tx,
                pulsar,
                application_id,
                event.event_id,
                event.received_at,
                &body.event_type,
                &payload,
                &body.payload_content_type,
            )
            .await?;
        }

        tx.commit().await?;
        report_ingested_events(1);
        Ok(CreatedJson(event))
    } else {
        if state.enable_quota_based_email_notifications {
            let mail = Mail::QuotaEventsPerDayReached {
                pricing_url_hash: "#pricing".to_owned(),
                current_events_per_day,
                events_per_days_limit,
                extra_variables: Vec::new(),
            };
            state
                .quotas
                .send_application_email_notification(
                    &state,
                    Quota::EventsPerDay,
                    QuotaNotificationType::Reached,
                    application_id,
                    mail,
                )
                .await?;
        }
        Err(Hook0Problem::TooManyEventsToday(events_per_days_limit))
    }
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct ReplayEvent {
    application_id: Uuid,
}

#[api_v2_operation(
    summary = "Replay an event",
    description = "Re-triggers webhook deliveries for an existing event. All active subscriptions matching the event type and labels will receive the event again. Useful for retrying failed deliveries or testing webhooks.",
    operation_id = "events.replay",
    consumes = "application/json",
    tags("Events Management", "mcp")
)]
pub async fn replay(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_id: Path<Uuid>,
    body: Json<ReplayEvent>,
) -> Result<NoContent, Hook0Problem> {
    let event_id = event_id.into_inner();

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventReplay {
            application_id: &body.application_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let mut tx = state.db.begin().await?;

    struct ReplayedEvent {
        received_at: DateTime<Utc>,
        event_type: String,
        payload: Option<Vec<u8>>,
        payload_content_type: String,
    }
    let replayed = query_as!(
        ReplayedEvent,
        "
            UPDATE event.event
            SET dispatched_at = NULL
            WHERE event__id = $1
                AND application__id = $2
            RETURNING received_at, event_type__name AS event_type, payload, payload_content_type
        ",
        event_id,
        body.application_id,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    match replayed {
        Some(event) => {
            if let Some(pulsar) = &state.pulsar {
                let payload = if let Some(p) = event.payload {
                    Some(p)
                } else if let Some(os) = &state.object_storage {
                    let key = format!(
                        "{}/event/{}/{event_id}",
                        body.application_id,
                        event.received_at.naive_utc().date(),
                    );
                    match os
                        .client
                        .get_object()
                        .bucket(&os.bucket)
                        .key(&key)
                        .send()
                        .await
                    {
                        Ok(obj) => match obj.body.collect().await {
                            Ok(ab) => Some(ab.to_vec()),
                            Err(e) => {
                                log_object_storage_error_with_context!(
                                    "S3 GET OBJECT body collect failed",
                                    error_chain = format!("{e}"),
                                    object_key = &key,
                                );
                                None
                            }
                        },
                        Err(e) => {
                            log_object_storage_error_with_context!(
                                "S3 GET OBJECT failed",
                                error_chain = DisplayErrorContext(&e).to_string(),
                                object_key = &key,
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(p) = payload {
                    send_request_attempts_to_pulsar(
                        &mut tx,
                        pulsar,
                        body.application_id,
                        event_id,
                        event.received_at,
                        &event.event_type,
                        &p,
                        &event.payload_content_type,
                    )
                    .await?;

                    tx.commit().await?;
                    report_replayed_events(1);
                    Ok(NoContent)
                } else {
                    tx.rollback().await?;
                    Err(Hook0Problem::InternalServerError)
                }
            } else {
                tx.commit().await?;
                report_replayed_events(1);
                Ok(NoContent)
            }
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[allow(clippy::too_many_arguments)]
async fn send_request_attempts_to_pulsar<'a>(
    tx: &mut PgTransaction<'a>,
    pulsar: &Arc<PulsarConfig>,
    application_id: Uuid,
    event_id: Uuid,
    event_received_at: DateTime<Utc>,
    event_type: &str,
    payload: &[u8],
    payload_content_type: &str,
) -> Result<(), Hook0Problem> {
    #[derive(Debug, Clone)]
    #[allow(non_snake_case)]
    struct RawRequestAttempt {
        request_attempt__id: Uuid,
        subscription__id: Uuid,
        created_at: DateTime<Utc>,
        http_method: String,
        http_url: String,
        http_headers: serde_json::Value,
        secret: Uuid,
        worker_id: Option<Uuid>,
        worker_queue_type: Option<String>,
    }

    let mut request_attempts_stream = query_as!(
        RawRequestAttempt,
        "
            SELECT
                ra.request_attempt__id,
                ra.subscription__id,
                ra.created_at,
                t_http.method AS http_method,
                t_http.url AS http_url,
                t_http.headers AS http_headers,
                s.secret,
                COALESCE(sw.worker__id, ow.worker__id) AS worker_id,
                COALESCE(w1.queue_type, w2.queue_type) AS worker_queue_type
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
            LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = ra.subscription__id
            LEFT JOIN infrastructure.worker AS w1 ON w1.worker__id = sw.worker__id
            INNER JOIN event.application AS a ON a.application__id = s.application__id
            LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = a.organization__id AND ow.default = true
            LEFT JOIN infrastructure.worker AS w2 ON w2.worker__id = ow.worker__id
            WHERE ra.event__id = $1
                AND ra.succeeded_at IS NULL AND ra.failed_at IS NULL
                AND a.deleted_at IS NULL
        ",
        &event_id,
    )
    .fetch(&mut **tx);

    while let Some(ra) = request_attempts_stream.try_next().await? {
        if let Some(worker_id) = ra.worker_id
            && ra.worker_queue_type.as_deref() == Some("pulsar")
        {
            let request_attempt = RequestAttempt {
                application_id,
                request_attempt_id: ra.request_attempt__id,
                event_id,
                event_received_at,
                subscription_id: ra.subscription__id,
                created_at: ra.created_at,
                retry_count: 0,
                http_method: ra.http_method,
                http_url: ra.http_url,
                http_headers: ra.http_headers,
                event_type_name: event_type.to_owned(),
                payload: payload.to_owned(),
                payload_content_type: payload_content_type.to_owned(),
                secret: ra.secret,
            };

            let mut producer = timeout(
                Duration::from_secs(3),
                pulsar.request_attempts_producer.lock(),
            )
            .await
            .map_err(|_| {
                error!("Timed out while waiting access to Pulsar producer");
                Hook0Problem::InternalServerError
            })?;
            producer
                .send_non_blocking(
                    format!(
                        "persistent://{}/{}/{}.request_attempt",
                        &pulsar.tenant, &pulsar.namespace, worker_id,
                    ),
                    request_attempt,
                )
                .await
                .map_err(|e| {
                    error!("Error while sending a message to Pulsar: {e}");
                    Hook0Problem::InternalServerError
                })?;
            report_request_attempts_sent_to_pulsar(1);
        }
    }

    Ok(())
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
