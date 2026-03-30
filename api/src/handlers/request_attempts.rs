use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, api_v2_operation};
use paperclip::v2::models::{DataType, DataTypeFormat, DefaultSchemaRaw};
use paperclip::v2::schema::Apiv2Schema as Apiv2SchemaTrait;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::BTreeMap;
use tracing::error;
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
    pub source: String,
    pub user_id: Option<Uuid>,
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

    #[derive(sqlx::FromRow)]
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
        source: String,
        user__id: Option<Uuid>,
    }
    let raw_request_attempts = sqlx::query_as::<_, RawRequestAttempt>(
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
                ra.source,
                ra.user__id
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN event.event AS e ON e.event__id = ra.event__id
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
    )
    .bind(&qs.application_id)
    .bind(qs.event_id)
    .bind(qs.subscription_id)
    .bind(min_created_at)
    .bind(max_created_at)
    .bind(pagination.date)
    .bind(pagination.id)
    .bind(&event_type_names)
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
            source: ra.source.clone(),
            user_id: ra.user__id,
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

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct RetryResponse {
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub source: String,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
}

#[api_v2_operation(
    summary = "Retry a request attempt",
    description = "Creates a new request attempt for the same event and subscription as the original attempt. The new attempt is marked with source='user' and linked to the authenticated user.",
    operation_id = "requestAttempts.retry",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn retry(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    request_attempt_id: Path<Uuid>,
) -> Result<CreatedJson<RetryResponse>, Hook0Problem> {
    let request_attempt_id = request_attempt_id.into_inner();

    // Fetch original attempt to get event_id, subscription_id, application_id
    #[derive(sqlx::FromRow)]
    #[allow(non_snake_case)]
    struct OriginalAttempt {
        event__id: Uuid,
        subscription__id: Uuid,
        application__id: Uuid,
    }

    let original = sqlx::query_as::<_, OriginalAttempt>(
        "
            SELECT ra.event__id, ra.subscription__id, ra.application__id
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN event.application AS a ON a.application__id = ra.application__id
            WHERE ra.request_attempt__id = $1
                AND s.deleted_at IS NULL
                AND a.deleted_at IS NULL
        ",
    )
    .bind(request_attempt_id)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // Authorize
    let authorized_token = authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptRetry {
            application_id: &original.application__id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .map_err(|_| Hook0Problem::Forbidden)?;

    let auth_user_id = match authorized_token {
        AuthorizedToken::User(u) => Some(u.user_id),
        _ => None,
    };

    // Insert new attempt with source='user'
    let new_attempt = sqlx::query_as::<_, RetryResponse>(
        "
            INSERT INTO webhook.request_attempt (event__id, subscription__id, application__id, source, user__id)
            VALUES ($1, $2, $3, 'user', $4)
            RETURNING
                request_attempt__id AS request_attempt_id,
                event__id AS event_id,
                subscription__id AS subscription_id,
                source,
                user__id AS user_id,
                created_at,
                retry_count
        ",
    )
    .bind(original.event__id)
    .bind(original.subscription__id)
    .bind(original.application__id)
    .bind(auth_user_id)
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // TODO: Pulsar publish for manual retries
    // When Pulsar is configured, the new attempt should be published to the worker topic
    // so it gets picked up immediately. For now, PG-polling mode will pick it up.
    // To implement: fetch subscription target details (http_method, http_url, http_headers, secret,
    // worker_id) and follow the send_request_attempts_to_pulsar pattern in events.rs.

    Ok(CreatedJson(new_attempt))
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
