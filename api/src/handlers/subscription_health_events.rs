use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar};
use tracing::error;
use uuid::Uuid;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::pagination::{
    BidirectionalPageConfig, Cursor, EncodedAscCursor, EncodedDescCursor, Paginated,
};
use crate::problems::Hook0Problem;
use crate::subscription_health_monitor::{HealthEventCause, HealthStatus};

const PAGE_SIZE: usize = 30;

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct HealthEvent {
    pub health_event_id: Uuid,
    pub subscription_id: Uuid,
    pub status: HealthStatus,
    pub cause: HealthEventCause,
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
        "select application__id from webhook.subscription where subscription__id = $1 and deleted_at is null limit 1",
        &subscription_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("{e}");
        Hook0Problem::InternalServerError
    })?;

    let Some(application_id) = application_id else {
        return Err(Hook0Problem::NotFound);
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
        query_as!(
            HealthEvent,
            "select
                health_event__id as health_event_id,
                subscription__id as subscription_id,
                status as \"status: HealthStatus\",
                cause as \"cause: HealthEventCause\",
                user__id as user_id,
                created_at
            from webhook.subscription_health_event
            where subscription__id = $1
              and (created_at, health_event__id) > ($2, $3)
            order by created_at asc, health_event__id asc
            limit $4",
            subscription_id,
            cursor.date,
            cursor.id,
            fetch_limit,
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    } else {
        let cursor = qs.pagination_cursor.unwrap_or_default().0;
        query_as!(
            HealthEvent,
            "select
                health_event__id as health_event_id,
                subscription__id as subscription_id,
                status as \"status: HealthStatus\",
                cause as \"cause: HealthEventCause\",
                user__id as user_id,
                created_at
            from webhook.subscription_health_event
            where subscription__id = $1
              and (created_at, health_event__id) < ($2, $3)
            order by created_at desc, health_event__id desc
            limit $4",
            subscription_id,
            cursor.date,
            cursor.id,
            fetch_limit,
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    };

    let has_extra = events.len() > PAGE_SIZE;
    if has_extra {
        events.truncate(PAGE_SIZE);
    }

    // Backward fetched ASC to get nearest rows; reverse to newest-first
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

    let (next_page_parts, prev_page_parts) = match endpoint_url {
        Some(url) => BidirectionalPageConfig {
            endpoint_url: url,
            base_qs: vec![("organization_id", Some(qs.organization_id.to_string()))],
            first_cursor: events.first().map(|ev| Cursor {
                date: ev.created_at,
                id: ev.health_event_id,
            }),
            last_cursor: events.last().map(|ev| Cursor {
                date: ev.created_at,
                id: ev.health_event_id,
            }),
            is_backward,
            has_more: has_extra,
            has_previous_page: qs.pagination_cursor.is_some(),
        }
        .into_page_parts(),
        None => (None, None),
    };

    Ok(Paginated {
        data: Json(events),
        next_page_parts,
        prev_page_parts,
    })
}
