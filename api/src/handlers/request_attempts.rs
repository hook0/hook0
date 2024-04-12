use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use std::cmp::max;
use uuid::Uuid;

use crate::iam::{authorize_for_application, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

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

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
    event_id: Option<Uuid>,
    subscription_id: Option<Uuid>,
}

#[api_v2_operation(
    summary = "List request attempts",
    description = "",
    operation_id = "requestAttempts.read",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<RequestAttempt>>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::RequestAttemptList {
            application_id: &qs.application_id,
        },
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
    }
    let raw_request_attempts = match (&qs.event_id, &qs.subscription_id) {
        (None, None) => {
            query_as!(
                RawRequestAttempt,
                "
                    SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.picked_at, ra.failed_at, ra.succeeded_at, ra.delay_until, ra.response__id, ra.retry_count, s.description AS subscription__description
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    WHERE s.application__id = $1
                    ORDER BY ra.created_at DESC
                    LIMIT 50
                ",
                &qs.application_id,
            )
            .fetch_all(&state.db)
            .await
            .map_err(Hook0Problem::from)?
        }
        (Some(eid), None) => {
            query_as!(
                RawRequestAttempt,
                "
                    SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.picked_at, ra.failed_at, ra.succeeded_at, ra.delay_until, ra.response__id, ra.retry_count, s.description AS subscription__description
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    WHERE s.application__id = $1 AND ra.event__id = $2
                    ORDER BY ra.created_at DESC
                    LIMIT 50
                ",
                &qs.application_id,
                eid,
            )
            .fetch_all(&state.db)
            .await
            .map_err(Hook0Problem::from)?
        }
        (None, Some(sid)) => {
            query_as!(
                RawRequestAttempt,
                "
                    SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.picked_at, ra.failed_at, ra.succeeded_at, ra.delay_until, ra.response__id, ra.retry_count, s.description AS subscription__description
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    WHERE s.application__id = $1 AND s.subscription__id = $2
                    ORDER BY ra.created_at DESC
                    LIMIT 50
                ",
                &qs.application_id,
                sid,
            )
            .fetch_all(&state.db)
            .await
            .map_err(Hook0Problem::from)?
        }
        (Some(eid), Some(sid)) => {
            query_as!(
                RawRequestAttempt,
                "
                    SELECT ra.request_attempt__id, ra.event__id, ra.subscription__id, ra.created_at, ra.picked_at, ra.failed_at, ra.succeeded_at, ra.delay_until, ra.response__id, ra.retry_count, s.description AS subscription__description
                    FROM webhook.request_attempt AS ra
                    INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                    WHERE s.application__id = $1 AND ra.event__id = $2 AND s.subscription__id = $3
                    ORDER BY ra.created_at DESC
                    LIMIT 50
                ",
                &qs.application_id,
                eid,
                sid,
            )
            .fetch_all(&state.db)
            .await
            .map_err(Hook0Problem::from)?
        }
    };

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

    Ok(Json(request_attempts))
}
