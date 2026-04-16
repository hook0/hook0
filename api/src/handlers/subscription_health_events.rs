use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
use tracing::error;
use uuid::Uuid;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::pagination::{
    Cursor, EncodedAscCursor, EncodedDescCursor, NextPageParts, Paginated, PrevPageParts,
};
use crate::problems::Hook0Problem;
use crate::subscription_health_monitor::{HealthEventCause, HealthStatus};

const PAGE_SIZE: usize = 30;

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct HealthEvent {
    #[sqlx(rename = "health_event__id")]
    pub health_event_id: Uuid,
    #[sqlx(rename = "subscription__id")]
    pub subscription_id: Uuid,
    pub status: HealthStatus,
    pub cause: HealthEventCause,
    #[sqlx(rename = "user__id")]
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct Qs {
    organization_id: Uuid,
    pagination_cursor: Option<EncodedDescCursor>,
    pagination_before_cursor: Option<EncodedAscCursor>,
}

#[api_v2_operation(
    summary = "List health events for a subscription",
    description = "Retrieves health events (warning, disabled, resolved) for a subscription with bidirectional cursor pagination via Link headers.",
    operation_id = "subscriptionHealthEvents.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<Paginated<Json<Vec<HealthEvent>>>, Hook0Problem> {
    let subscription_id = subscription_id.into_inner();

    // Resolve application from subscription
    let application_id = query_scalar!(
        "SELECT application__id FROM webhook.subscription WHERE subscription__id = $1 AND deleted_at IS NULL LIMIT 1",
        &subscription_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("{e}");
        Hook0Problem::InternalServerError
    })?;

    let application_id = match application_id {
        Some(id) => id,
        None => return Err(Hook0Problem::NotFound),
    };

    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::SubscriptionGet {
            application_id: &application_id,
            subscription_id: &subscription_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let fetch_limit = (PAGE_SIZE + 1) as i64;
    let is_backward = qs.pagination_before_cursor.is_some();

    let mut events = if let Some(ref before) = qs.pagination_before_cursor {
        let cursor = before.0;
        sqlx::query_as::<_, HealthEvent>(
            "SELECT health_event__id, subscription__id, status, cause, user__id, created_at
             FROM webhook.subscription_health_event
             WHERE subscription__id = $1
               AND (created_at, health_event__id) > ($2, $3)
             ORDER BY created_at ASC, health_event__id ASC
             LIMIT $4",
        )
        .bind(&subscription_id)
        .bind(&cursor.date)
        .bind(&cursor.id)
        .bind(&fetch_limit)
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    } else {
        let cursor = qs.pagination_cursor.unwrap_or_default().0;
        sqlx::query_as::<_, HealthEvent>(
            "SELECT health_event__id, subscription__id, status, cause, user__id, created_at
             FROM webhook.subscription_health_event
             WHERE subscription__id = $1
               AND (created_at, health_event__id) < ($2, $3)
             ORDER BY created_at DESC, health_event__id DESC
             LIMIT $4",
        )
        .bind(&subscription_id)
        .bind(&cursor.date)
        .bind(&cursor.id)
        .bind(&fetch_limit)
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    };

    let has_extra = events.len() > PAGE_SIZE;
    if has_extra {
        events.truncate(PAGE_SIZE);
    }

    // Backward fetched ASC; reverse to newest-first
    if is_backward {
        events.reverse();
    }

    let endpoint_url = state
        .app_url
        .join(&format!(
            "/api/v1/subscriptions/{subscription_id}/health_events"
        ))
        .inspect_err(|e| {
            error!("Error building pagination URL: {e}");
        })
        .ok();

    let base_qs: Vec<(&'static str, Option<String>)> =
        vec![("organization_id", Some(qs.organization_id.to_string()))];

    let next_page_parts = if is_backward || has_extra {
        events.last().and_then(|ev| {
            endpoint_url.clone().map(|url| NextPageParts {
                endpoint_url: url,
                qs: base_qs.clone(),
                cursor: Cursor {
                    date: ev.created_at,
                    id: ev.health_event_id,
                },
            })
        })
    } else {
        None
    };

    let has_forward_cursor = qs.pagination_cursor.is_some();
    let prev_page_parts = if (is_backward && has_extra) || (!is_backward && has_forward_cursor) {
        events.first().and_then(|ev| {
            endpoint_url.map(|url| PrevPageParts {
                endpoint_url: url,
                qs: base_qs,
                cursor: Cursor {
                    date: ev.created_at,
                    id: ev.health_event_id,
                },
            })
        })
    } else {
        None
    };

    Ok(Paginated {
        data: Json(events),
        next_page_parts,
        prev_page_parts,
    })
}
