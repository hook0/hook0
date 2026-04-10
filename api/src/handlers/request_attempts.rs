use actix_web::web::ReqData;
use actix_web::HttpResponse;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use paperclip::v2::models::{DataType, DataTypeFormat, DefaultSchemaRaw};
use paperclip::v2::schema::Apiv2Schema as Apiv2SchemaTrait;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar};
use std::cmp::max;
use std::collections::BTreeMap;
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

use crate::iam::{Action, AuthorizedToken, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::pagination::{Cursor, EncodedDescCursor, NextPageParts, Paginated};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RequestAttempt {
    pub request_attempt_id: Uuid,
    pub event_id: Uuid, // Kept to avoid breaking compatibility
    pub event: EventSummary,
    pub subscription: SubscriptionSummary,
    pub created_at: DateTime<Utc>,
    pub picked_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub succeeded_at: Option<DateTime<Utc>>,
    pub delay_until: Option<DateTime<Utc>>,
    pub response_id: Option<Uuid>,
    pub retry_count: i16,
    pub http_response_status: Option<i16>,
    pub status: RequestAttemptStatus,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct EventSummary {
    pub event_id: Uuid,
    pub event_type_name: String,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct SubscriptionSummary {
    pub subscription_id: Uuid,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestAttemptStatus {
    Waiting {
        since: DateTime<Utc>,
        until: DateTime<Utc>,
    },
    Pending {
        since: DateTime<Utc>,
    },
    InProgress {
        since: DateTime<Utc>,
    },
    Successful {
        at: DateTime<Utc>,
        full_processing_ms: i64,
    },
    Failed {
        at: DateTime<Utc>,
        full_processing_ms: i64,
    },
}

impl Apiv2SchemaTrait for RequestAttemptStatus {
    fn name() -> Option<String> {
        Some("RequestAttemptStatus".to_owned())
    }

    fn raw_schema() -> DefaultSchemaRaw {
        // OpenAPI v2 doesn't support oneOf/discriminator, so we document the status
        // as an object with all possible fields. The "type" field determines which
        // variant is present.
        //
        // Variants:
        // - waiting: {type: "waiting", since: DateTime, until: DateTime}
        // - pending: {type: "pending", since: DateTime}
        // - in_progress: {type: "in_progress", since: DateTime}
        // - successful: {type: "successful", at: DateTime, full_processing_ms: i64}
        // - failed: {type: "failed", at: DateTime, full_processing_ms: i64}

        let mut properties = BTreeMap::new();

        // type field (discriminator)
        properties.insert(
            "type".to_owned(),
            Box::new(DefaultSchemaRaw {
                data_type: Some(DataType::String),
                description: Some(
                    "Status type discriminator. One of: waiting, pending, in_progress, successful, failed"
                        .to_owned(),
                ),
                enum_: vec![
                    serde_json::Value::String("waiting".to_owned()),
                    serde_json::Value::String("pending".to_owned()),
                    serde_json::Value::String("in_progress".to_owned()),
                    serde_json::Value::String("successful".to_owned()),
                    serde_json::Value::String("failed".to_owned()),
                ],
                ..Default::default()
            }),
        );

        // since field (present in waiting, pending, in_progress)
        properties.insert(
            "since".to_owned(),
            Box::new(DefaultSchemaRaw {
                data_type: Some(DataType::String),
                format: Some(DataTypeFormat::DateTime),
                description: Some(
                    "Timestamp when the status started (present in waiting, pending, in_progress)"
                        .to_owned(),
                ),
                ..Default::default()
            }),
        );

        // until field (only in waiting)
        properties.insert(
            "until".to_owned(),
            Box::new(DefaultSchemaRaw {
                data_type: Some(DataType::String),
                format: Some(DataTypeFormat::DateTime),
                description: Some(
                    "Timestamp until which waiting (only present in waiting status)".to_owned(),
                ),
                ..Default::default()
            }),
        );

        // at field (present in successful, failed)
        properties.insert(
            "at".to_owned(),
            Box::new(DefaultSchemaRaw {
                data_type: Some(DataType::String),
                format: Some(DataTypeFormat::DateTime),
                description: Some(
                    "Timestamp when completed (present in successful, failed)".to_owned(),
                ),
                ..Default::default()
            }),
        );

        // full_processing_ms field (present in successful, failed)
        properties.insert(
            "full_processing_ms".to_owned(),
            Box::new(DefaultSchemaRaw {
                data_type: Some(DataType::Integer),
                format: Some(DataTypeFormat::Int64),
                description: Some(
                    "Total processing time in milliseconds (present in successful, failed)"
                        .to_owned(),
                ),
                ..Default::default()
            }),
        );

        // Only type is always required
        let mut required = std::collections::BTreeSet::new();
        required.insert("type".to_owned());

        DefaultSchemaRaw {
            data_type: Some(DataType::Object),
            description: Some(
                "Status of a request attempt. The 'type' field indicates the status variant. \
                 - waiting: {type, since, until} - Scheduled for future delivery \
                 - pending: {type, since} - Ready to be processed \
                 - in_progress: {type, since} - Currently being delivered \
                 - successful: {type, at, full_processing_ms} - Delivered successfully \
                 - failed: {type, at, full_processing_ms} - Delivery failed"
                    .to_owned(),
            ),
            properties,
            required,
            ..Default::default()
        }
    }
}

impl RequestAttemptStatus {
    pub fn compute(
        current_time: &DateTime<Utc>,
        created_at: &DateTime<Utc>,
        picked_at: &Option<DateTime<Utc>>,
        failed_at: &Option<DateTime<Utc>>,
        succeeded_at: &Option<DateTime<Utc>>,
        delay_until: &Option<DateTime<Utc>>,
    ) -> Self {
        let start = match delay_until {
            Some(d) => max(created_at, d),
            None => created_at,
        };

        match (delay_until, picked_at, succeeded_at, failed_at) {
            (_, _, _, Some(at)) => Self::Failed {
                at: *at,
                full_processing_ms: (*at - *start).num_milliseconds(),
            },
            (_, _, Some(at), None) => Self::Successful {
                at: *at,
                full_processing_ms: (*at - *start).num_milliseconds(),
            },
            (_, Some(since), None, None) => Self::InProgress { since: *since },
            (Some(until), None, None, None) if until > current_time => Self::Waiting {
                since: *created_at,
                until: *until,
            },
            (_, None, None, None) => Self::Pending { since: *created_at },
        }
    }
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct GetQs {
    application_id: Uuid,
}

#[api_v2_operation(
    summary = "Get a request attempt by its ID",
    description = "Retrieves a single webhook delivery attempt by ID, including delivery status, retry count, timestamps, and HTTP response status.",
    operation_id = "requestAttempts.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<GetQs>,
    request_attempt_id: Path<Uuid>,
) -> Result<Json<RequestAttempt>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptGet {
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

    #[allow(non_snake_case)]
    struct RawRequestAttempt {
        request_attempt__id: Uuid,
        event__id: Uuid,
        subscription__id: Uuid,
        subscription__description: Option<String>,
        created_at: DateTime<Utc>,
        picked_at: Option<DateTime<Utc>>,
        failed_at: Option<DateTime<Utc>>,
        succeeded_at: Option<DateTime<Utc>>,
        delay_until: Option<DateTime<Utc>>,
        response__id: Option<Uuid>,
        retry_count: i16,
        event_type__name: String,
        http_response_status: Option<i16>,
    }

    let raw = query_as!(
        RawRequestAttempt,
        "
            SELECT
                ra.request_attempt__id,
                ra.event__id,
                ra.subscription__id,
                ra.created_at,
                ra.picked_at,
                ra.failed_at,
                ra.succeeded_at,
                ra.delay_until,
                ra.response__id,
                ra.retry_count,
                s.description AS subscription__description,
                e.event_type__name,
                r.http_code AS http_response_status
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN event.event AS e ON e.event__id = ra.event__id
            LEFT JOIN webhook.response AS r ON r.response__id = ra.response__id
            WHERE ra.application__id = $1
                AND ra.request_attempt__id = $2
        ",
        &qs.application_id,
        &request_attempt_id.into_inner(),
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match raw {
        Some(ra) => Ok(Json(RequestAttempt {
            request_attempt_id: ra.request_attempt__id,
            event_id: ra.event__id,
            event: EventSummary {
                event_id: ra.event__id,
                event_type_name: ra.event_type__name,
            },
            subscription: SubscriptionSummary {
                subscription_id: ra.subscription__id,
                description: ra.subscription__description,
            },
            created_at: ra.created_at,
            picked_at: ra.picked_at,
            failed_at: ra.failed_at,
            succeeded_at: ra.succeeded_at,
            delay_until: ra.delay_until,
            response_id: ra.response__id,
            retry_count: ra.retry_count,
            http_response_status: ra.http_response_status,
            status: RequestAttemptStatus::compute(
                &Utc::now(),
                &ra.created_at,
                &ra.picked_at,
                &ra.failed_at,
                &ra.succeeded_at,
                &ra.delay_until,
            ),
        })),
        None => Err(Hook0Problem::NotFound),
    }
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
    event_id: Option<Uuid>,
    subscription_id: Option<Uuid>,
    pagination_cursor: Option<EncodedDescCursor>,
    min_created_at: Option<DateTime<Utc>>,
    max_created_at: Option<DateTime<Utc>>,
    /// Comma-separated event types
    #[serde(rename = "event.event_type_names")]
    event_type_names: Option<String>,
}

#[api_v2_operation(
    summary = "List request attempts",
    description = "Retrieves webhook delivery attempts for an application. Each attempt shows the delivery status (pending, in_progress, successful, failed, waiting), retry count, and timestamps. Filter by event_id, subscription_id, date range, or event types. Paginated via Link header.",
    operation_id = "requestAttempts.read",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management", "mcp")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Paginated<Json<Vec<RequestAttempt>>>, Hook0Problem> {
    let min_created_at = qs.min_created_at.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let max_created_at = qs.max_created_at.unwrap_or_else(Utc::now);
    let event_type_names = qs
        .event_type_names
        .as_ref()
        .map(|s| {
            s.split(",")
                .map(|p| p.trim().to_owned())
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptList {
            application_id: &qs.application_id,
            event_type_names: &event_type_names,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let pagination = qs.pagination_cursor.unwrap_or_default().0;

    #[allow(non_snake_case)]
    struct RawRequestAttempt {
        request_attempt__id: Uuid,
        event__id: Uuid,
        subscription__id: Uuid,
        subscription__description: Option<String>,
        created_at: DateTime<Utc>,
        picked_at: Option<DateTime<Utc>>,
        failed_at: Option<DateTime<Utc>>,
        succeeded_at: Option<DateTime<Utc>>,
        delay_until: Option<DateTime<Utc>>,
        response__id: Option<Uuid>,
        retry_count: i16,
        event_type__name: String,
        http_response_status: Option<i16>,
    }
    let raw_request_attempts = query_as!(
        RawRequestAttempt,
        "
            SELECT
                ra.request_attempt__id,
                ra.event__id,
                ra.subscription__id,
                ra.created_at,
                ra.picked_at,
                ra.failed_at,
                ra.succeeded_at,
                ra.delay_until,
                ra.response__id,
                ra.retry_count,
                s.description AS subscription__description,
                e.event_type__name,
                r.http_code AS http_response_status
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN event.event AS e ON e.event__id = ra.event__id
            LEFT JOIN webhook.response AS r ON r.response__id = ra.response__id
            WHERE ra.application__id = $1
                AND (ra.event__id = $2 OR $2 IS NULL)
                AND (s.subscription__id = $3 OR $3 IS NULL)
                AND ra.created_at BETWEEN $4 AND $5
                AND (ra.created_at, ra.request_attempt__id) < ($6, $7)
                AND (e.event_type__name = any($8) OR $8 = '{}')
            ORDER BY
                ra.created_at DESC,
                ra.request_attempt__id ASC
            LIMIT 50
        ",
        &qs.application_id,
        qs.event_id,
        qs.subscription_id,
        min_created_at,
        max_created_at,
        pagination.date,
        pagination.id,
        &event_type_names,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    let request_attempts = raw_request_attempts
        .iter()
        .map(|ra| RequestAttempt {
            request_attempt_id: ra.request_attempt__id,
            event_id: ra.event__id,
            event: EventSummary {
                event_id: ra.event__id,
                event_type_name: ra.event_type__name.to_owned(),
            },
            subscription: SubscriptionSummary {
                subscription_id: ra.subscription__id,
                description: ra.subscription__description.clone(),
            },
            created_at: ra.created_at,
            picked_at: ra.picked_at,
            failed_at: ra.failed_at,
            succeeded_at: ra.succeeded_at,
            delay_until: ra.delay_until,
            response_id: ra.response__id,
            retry_count: ra.retry_count,
            http_response_status: ra.http_response_status,
            status: RequestAttemptStatus::compute(
                &Utc::now(),
                &ra.created_at,
                &ra.picked_at,
                &ra.failed_at,
                &ra.succeeded_at,
                &ra.delay_until,
            ),
        })
        .collect::<Vec<_>>();

    let next_page_parts = request_attempts.last().and_then(|ra| {
        if state.app_url.as_str().ends_with('/') {
            Ok(state.app_url.clone())
        } else {
            Url::parse(&format!("{}/", &state.app_url))
        }
        .inspect_err(|e| {
            error!("Error that should never happen while building app URL for pagination: {e}");
        })
        .ok()
        .and_then(|app_url| {
            app_url
                .join("/api/v1/request_attempts")
                .inspect_err(|e| {
                    error!(
                        "Error that should never happen while building app URL for pagination: {e}"
                    );
                })
                .ok()
        })
        .map(|endpoint_url| NextPageParts {
            endpoint_url,
            qs: vec![
                ("application_id", Some(qs.application_id.to_string())),
                ("event_id", qs.event_id.map(|v| v.to_string())),
                ("subscription_id", qs.subscription_id.map(|v| v.to_string())),
                ("min_created_at", qs.min_created_at.map(|v| v.to_string())),
                ("max_created_at", qs.max_created_at.map(|v| v.to_string())),
                ("event.event_type_names", qs.event_type_names.to_owned()),
            ],
            cursor: Cursor {
                date: ra.created_at,
                id: ra.request_attempt_id,
            },
        })
    });

    Ok(Paginated {
        data: Json(request_attempts),
        next_page_parts,
    })
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RetryResponse {
    pub request_attempt_id: Uuid,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryQs {
    application_id: Uuid,
}

#[api_v2_operation(
    summary = "Retry a delivery attempt",
    description = "Creates a new one-shot delivery attempt for the same event. The new attempt is independent: it will not trigger automatic retries on failure. Subject to a configurable per-event cooldown.",
    operation_id = "requestAttempts.retry",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn retry(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<RetryQs>,
    request_attempt_id: Path<Uuid>,
) -> Result<HttpResponse, Hook0Problem> {
    let request_attempt_id = request_attempt_id.into_inner();

    // 1. Fetch source attempt metadata (without payload — avoid loading ~100KB for unauthorized requests)
    #[allow(non_snake_case)]
    struct SourceAttempt {
        event__id: Uuid,
        subscription__id: Uuid,
        application__id: Uuid,
        received_at: DateTime<Utc>,
        payload: Option<Vec<u8>>,
        payload_content_type: String,
        event_type__name: String,
        subscription_deleted_at: Option<DateTime<Utc>>,
    }

    let source = query_as!(
        SourceAttempt,
        "
            SELECT
                ra.event__id,
                ra.subscription__id,
                ra.application__id,
                e.received_at,
                e.payload,
                e.payload_content_type,
                e.event_type__name,
                s.deleted_at AS subscription_deleted_at
            FROM webhook.request_attempt AS ra
            INNER JOIN event.event AS e ON e.event__id = ra.event__id
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            WHERE ra.request_attempt__id = $1
        ",
        &request_attempt_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // 2. Authorize — return 404 uniformly for both not-found and forbidden (anti-enumeration)
    let authorized = authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptRetry {
            application_id: &qs.application_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await;

    let authorized = match authorized {
        Ok(token) => token,
        Err(_) => return Err(Hook0Problem::NotFound),
    };

    // Verify the attempt belongs to the requested application
    if source.application__id != qs.application_id {
        return Err(Hook0Problem::NotFound);
    }

    // 3. Check subscription not deleted
    if source.subscription_deleted_at.is_some() {
        return Err(Hook0Problem::NotFound);
    }

    // 4. Check cooldown per event
    if state.manual_retry_cooldown_seconds > 0 {
        #[allow(non_snake_case)]
        struct CooldownCheck {
            exists: Option<bool>,
        }
        let check = query_as!(
            CooldownCheck,
            "
                SELECT EXISTS(
                    SELECT 1 FROM webhook.request_attempt
                    WHERE event__id = $1
                        AND attempt_trigger = 'manual_retry'
                        AND created_at > now() - make_interval(secs => $2::float8)
                ) AS exists
            ",
            &source.event__id,
            state.manual_retry_cooldown_seconds as f64,
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;
        let cooldown_active = check.exists.unwrap_or(false);

        if cooldown_active {
            return Err(Hook0Problem::RetryCooldown {
                seconds: state.manual_retry_cooldown_seconds,
            });
        }
    }

    // 5. Fetch event payload (DB or S3 fallback)
    let payload = crate::event_payload::fetch_event_payload(
        source.payload,
        state.object_storage.as_ref(),
        source.application__id,
        source.event__id,
        source.received_at,
    )
    .await
    .ok_or(Hook0Problem::EventPayloadUnavailable)?;

    // 6. Extract user_id from token (None for service/master tokens)
    let user_id = match &authorized {
        AuthorizedToken::User(aut) => Some(aut.user_id),
        _ => None,
    };

    // 7. INSERT new attempt
    #[allow(non_snake_case)]
    struct InsertedAttempt {
        request_attempt__id: Uuid,
    }
    let inserted = query_as!(
        InsertedAttempt,
        "
            INSERT INTO webhook.request_attempt
                (application__id, event__id, subscription__id, retry_count, attempt_trigger, triggered_by)
            VALUES ($1, $2, $3, 0, 'manual_retry', $4)
            RETURNING request_attempt__id
        ",
        &source.application__id,
        &source.event__id,
        &source.subscription__id,
        user_id.as_ref(),
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;
    let new_id = inserted.request_attempt__id;

    info!(
        request_attempt_id = %new_id,
        source_attempt_id = %request_attempt_id,
        event_id = %source.event__id,
        "Manual retry attempt created"
    );

    // 8. Dispatch to Pulsar (optional — PG poller is the safety net)
    if let Some(pulsar) = &state.pulsar {
        // Reuse the existing send_request_attempts_to_pulsar by replaying the event
        // The new attempt will be picked up since it has no succeeded_at/failed_at
        use actix_web::rt::time::timeout;
        use std::time::Duration;

        #[allow(non_snake_case)]
        struct WorkerInfo {
            http_method: String,
            http_url: String,
            http_headers: serde_json::Value,
            secret: Uuid,
            worker_id: Option<Uuid>,
            worker_queue_type: Option<String>,
        }

        let worker = query_as!(
            WorkerInfo,
            "
                SELECT
                    t_http.method AS http_method,
                    t_http.url AS http_url,
                    t_http.headers AS http_headers,
                    s.secret,
                    COALESCE(sw.worker__id, ow.worker__id) AS worker_id,
                    COALESCE(w1.queue_type, w2.queue_type) AS worker_queue_type
                FROM webhook.subscription AS s
                INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                LEFT JOIN infrastructure.worker AS w1 ON w1.worker__id = sw.worker__id
                INNER JOIN event.application AS a ON a.application__id = s.application__id
                LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = a.organization__id AND ow.default = true
                LEFT JOIN infrastructure.worker AS w2 ON w2.worker__id = ow.worker__id
                WHERE s.subscription__id = $1
            ",
            &source.subscription__id,
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        if let Some(w) = worker
            && let Some(worker_id) = w.worker_id
            && w.worker_queue_type.as_deref() == Some("pulsar")
        {
            let request_attempt = hook0_protobuf::RequestAttempt {
                application_id: source.application__id,
                request_attempt_id: new_id,
                event_id: source.event__id,
                event_received_at: source.received_at,
                subscription_id: source.subscription__id,
                created_at: Utc::now(),
                retry_count: 0,
                http_method: w.http_method,
                http_url: w.http_url,
                http_headers: w.http_headers,
                event_type_name: source.event_type__name,
                payload,
                payload_content_type: source.payload_content_type,
                secret: w.secret,
                attempt_trigger: hook0_protobuf::AttemptTrigger::ManualRetry,
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

            if let Err(e) = producer
                .send_non_blocking(
                    format!(
                        "persistent://{}/{}/{}.request_attempt",
                        &pulsar.tenant, &pulsar.namespace, worker_id,
                    ),
                    request_attempt,
                )
                .await
            {
                // Log but don't fail — PG poller will pick it up
                error!("Failed to send manual retry to Pulsar: {e}");
            }
        }
    }

    // 9. Return 202 Accepted
    Ok(HttpResponse::Accepted().json(RetryResponse {
        request_attempt_id: new_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_attempt_status_schema_contract() {
        let schema = RequestAttemptStatus::raw_schema();

        // Verify it's an object type
        assert_eq!(schema.data_type, Some(DataType::Object));

        // Verify description is set
        assert!(
            schema.description.is_some(),
            "RequestAttemptStatus should have description"
        );

        // Verify the type field (discriminator) has enum values
        let type_field = schema
            .properties
            .get("type")
            .expect("Should have 'type' field");
        assert!(
            !type_field.enum_.is_empty(),
            "Type field should have enum values"
        );
        assert_eq!(
            type_field.enum_.len(),
            5,
            "Should have 5 status type values"
        );

        let type_values: Vec<&str> = type_field.enum_.iter().filter_map(|v| v.as_str()).collect();

        assert!(type_values.contains(&"waiting"), "Missing 'waiting' type");
        assert!(type_values.contains(&"pending"), "Missing 'pending' type");
        assert!(
            type_values.contains(&"in_progress"),
            "Missing 'in_progress' type"
        );
        assert!(
            type_values.contains(&"successful"),
            "Missing 'successful' type"
        );
        assert!(type_values.contains(&"failed"), "Missing 'failed' type");
    }

    #[test]
    fn request_attempt_status_schema_snapshot() {
        let schema = RequestAttemptStatus::raw_schema();
        insta::assert_json_snapshot!(
            "request_attempt_status_schema",
            serde_json::to_value(&schema).unwrap()
        );
    }

    #[test]
    fn request_attempt_status_has_all_variant_fields() {
        let schema = RequestAttemptStatus::raw_schema();
        let props = &schema.properties;

        // Check for all fields from all variants
        assert!(props.contains_key("type"), "Should have 'type' field");
        assert!(props.contains_key("since"), "Should have 'since' field");
        assert!(props.contains_key("until"), "Should have 'until' field");
        assert!(props.contains_key("at"), "Should have 'at' field");
        assert!(
            props.contains_key("full_processing_ms"),
            "Should have 'full_processing_ms' field"
        );
    }

    #[test]
    fn request_attempt_status_since_field_is_datetime() {
        let schema = RequestAttemptStatus::raw_schema();
        let since = schema
            .properties
            .get("since")
            .expect("Should have 'since' field");

        assert_eq!(since.data_type, Some(DataType::String));
        assert_eq!(since.format, Some(DataTypeFormat::DateTime));
    }

    #[test]
    fn request_attempt_status_full_processing_ms_is_int64() {
        let schema = RequestAttemptStatus::raw_schema();
        let full_processing_ms = schema
            .properties
            .get("full_processing_ms")
            .expect("Should have 'full_processing_ms' field");

        assert_eq!(full_processing_ms.data_type, Some(DataType::Integer));
        assert_eq!(full_processing_ms.format, Some(DataTypeFormat::Int64));
    }

    #[test]
    fn request_attempt_status_type_is_required() {
        let schema = RequestAttemptStatus::raw_schema();

        assert!(
            schema.required.contains("type"),
            "'type' should be in required fields"
        );
    }
}
