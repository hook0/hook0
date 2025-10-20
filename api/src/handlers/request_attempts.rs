use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::error;
use paperclip::actix::web::{Data, Json, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::cmp::max;
use url::Url;
use uuid::Uuid;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::pagination::{Cursor, EncodedDescCursor, NextPageParts, Paginated};
use crate::problems::Hook0Problem;
use crate::query_builder::QueryBuilder;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RequestAttempt {
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    pub subscription: SubscriptionSummary,
    pub created_at: DateTime<Utc>,
    pub picked_at: Option<DateTime<Utc>>,
    pub failed_at: Option<DateTime<Utc>>,
    pub succeeded_at: Option<DateTime<Utc>>,
    pub delay_until: Option<DateTime<Utc>>,
    pub response_id: Option<Uuid>,
    pub retry_count: i16,
    pub status: RequestAttemptStatus,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct SubscriptionSummary {
    pub subscription_id: Uuid,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Apiv2Schema)]
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
    #[serde(default, rename = "event.event_type")]
    event_event_type: Option<String>,
    pagination_cursor: Option<EncodedDescCursor>,
    min_created_at: Option<DateTime<Utc>>,
    max_created_at: Option<DateTime<Utc>>,
}

#[api_v2_operation(
    summary = "List request attempts",
    description = "Retrieves the most recent attempts to deliver events to subscriptions for a given application. Request attempts track the status and history of event deliveries, including retries and failures. This endpoint is paginated: the next URL is given in the `Link` header of the response, following HATEOAS conventions.",
    operation_id = "requestAttempts.read",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
#[allow(non_snake_case)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Paginated<Json<Vec<RequestAttempt>>>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptList {
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

    let pagination = qs.pagination_cursor.unwrap_or_default().0;
    let min_created_at = qs.min_created_at.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let max_created_at = qs.max_created_at.unwrap_or_else(Utc::now);

    #[derive(sqlx::FromRow)]
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
    }

    // Build dynamic WHERE conditions using QueryBuilder
    let mut query_builder = QueryBuilder::new(
        "s.application__id = $1".to_string(),
        2, // Next parameter index after application_id
    );

    // Add optional filters
    query_builder.add_uuid_filter("ra.event__id", qs.event_id);
    query_builder.add_uuid_filter("s.subscription__id", qs.subscription_id);
    query_builder.add_string_filter("e.event_type__name", qs.event_event_type.clone());

    // Build the initial WHERE clause from QueryBuilder
    let mut where_parts = vec![query_builder.build_where_clause()];

    // Calculate next parameter index for pagination filters
    let next_param_idx = query_builder.param_index;

    // Add pagination and date range filters (these are always present, not optional)
    where_parts.push(format!("(ra.created_at, ra.request_attempt__id) < (${}, ${})", next_param_idx, next_param_idx + 1));
    where_parts.push(format!("ra.created_at >= ${}", next_param_idx + 2));
    where_parts.push(format!("ra.created_at <= ${}", next_param_idx + 3));

    let where_clause = where_parts.join(" AND ");
    let sql = format!(
        "
            SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.picked_at, ra.failed_at, ra.succeeded_at, ra.delay_until, ra.response__id, ra.retry_count, s.description AS subscription__description
            FROM webhook.request_attempt AS ra
            INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
            INNER JOIN event.event AS e ON e.event__id = ra.event__id
            WHERE {}
            ORDER BY ra.created_at DESC, ra.request_attempt__id ASC
            LIMIT 50
        ",
        where_clause
    );

    // Build query with dynamic parameters
    let query = query_as::<_, RawRequestAttempt>(&sql).bind(qs.application_id);

    // Bind QueryBuilder params
    let query = query_builder.bind_params(query);

    // Bind pagination and date range params
    let query = query
        .bind(pagination.date)
        .bind(pagination.id)
        .bind(min_created_at)
        .bind(max_created_at);

    let raw_request_attempts = query
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    let request_attempts = raw_request_attempts
        .iter()
        .map(|ra| RequestAttempt {
            request_attempt_id: ra.request_attempt__id,
            event_id: ra.event__id,
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
