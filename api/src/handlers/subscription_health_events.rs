use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query_as, query_scalar};
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
// Always PAGE_SIZE + 1 to detect next page
const FETCH_LIMIT: i64 = 31;

// --- Models ---

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
    // Only used for pagination URL generation, not for SQL filtering or auth.
    organization_id: Uuid,
    pagination_cursor: Option<EncodedDescCursor>,
    pagination_before_cursor: Option<EncodedAscCursor>,
}

// --- Data access (separated from HTTP handler) ---

impl HealthEvent {
    /// Resolves the application owning a subscription.
    async fn fetch_app_id(
        db: &PgPool,
        subscription_id: &Uuid,
    ) -> Result<Option<Uuid>, Hook0Problem> {
        query_scalar!(
            r#"
            select application__id
            from webhook.subscription
            where subscription__id = $1 and deleted_at is null
            limit 1
            "#,
            subscription_id
        )
        .fetch_optional(db)
        .await
        .map_err(|e| {
            error!("DB error fetching application for subscription: {e}");
            Hook0Problem::InternalServerError
        })
    }

    /// Fetches events going back in time (newest first). Default direction.
    async fn fetch_forward(
        db: &PgPool,
        subscription_id: Uuid,
        cursor: Cursor,
    ) -> Result<Vec<Self>, Hook0Problem> {
        query_as!(
            HealthEvent,
            r#"
            select
                health_event__id as health_event_id,
                subscription__id as subscription_id,
                status as "status: HealthStatus",
                cause as "cause: HealthEventCause",
                user__id as user_id,
                created_at
            from webhook.subscription_health_event
            where subscription__id = $1 and (created_at, health_event__id) < ($2, $3)
            order by created_at desc, health_event__id desc
            limit $4
            "#,
            subscription_id,
            cursor.date,
            cursor.id,
            FETCH_LIMIT,
        )
        .fetch_all(db)
        .await
        .map_err(Hook0Problem::from)
    }

    /// Fetches events going forward in time (oldest first).
    /// Used for "Previous page". Results are reversed by the caller.
    async fn fetch_backward(
        db: &PgPool,
        subscription_id: Uuid,
        cursor: Cursor,
    ) -> Result<Vec<Self>, Hook0Problem> {
        query_as!(
            HealthEvent,
            r#"
            select
                health_event__id as health_event_id,
                subscription__id as subscription_id,
                status as "status: HealthStatus",
                cause as "cause: HealthEventCause",
                user__id as user_id,
                created_at
            from webhook.subscription_health_event
            where subscription__id = $1 and (created_at, health_event__id) > ($2, $3)
            order by created_at asc, health_event__id asc
            limit $4
            "#,
            subscription_id,
            cursor.date,
            cursor.id,
            FETCH_LIMIT,
        )
        .fetch_all(db)
        .await
        .map_err(Hook0Problem::from)
    }
}

// --- HTTP Handler ---

#[api_v2_operation(
    summary = "List health events for a subscription",
    description = "Retrieves health events for a subscription with bidirectional pagination.",
    operation_id = "subscriptionHealthEvents.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id_path: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<Paginated<Json<Vec<HealthEvent>>>, Hook0Problem> {
    let subscription_id = subscription_id_path.into_inner();

    // 1. Resolve application
    let application_id = HealthEvent::fetch_app_id(&state.db, &subscription_id)
        .await?
        .ok_or(Hook0Problem::NotFound)?;

    // 2. Authorize
    let is_authorized = authorize_for_application(
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
    .is_ok();

    if !is_authorized {
        return Err(Hook0Problem::Forbidden);
    }

    // 3. Fetch events
    let is_backward = qs.pagination_before_cursor.is_some();

    let mut events = if let Some(before) = &qs.pagination_before_cursor {
        // Backward: fetch ASC, then reverse to keep newest-first display
        let mut items = HealthEvent::fetch_backward(&state.db, subscription_id, before.0).await?;
        items.reverse();
        items
    } else {
        let cursor = qs.pagination_cursor.unwrap_or_default().0;
        HealthEvent::fetch_forward(&state.db, subscription_id, cursor).await?
    };

    // 4. Trim extra row used for has_more detection
    let has_more = events.len() > PAGE_SIZE;
    if has_more {
        events.truncate(PAGE_SIZE);
    }

    // 5. Build pagination links
    let endpoint_url = state
        .app_url
        .join(&format!(
            "/api/v1/subscriptions/{subscription_id}/health_events"
        ))
        .inspect_err(|e| error!("Failed to build pagination URL: {e}"))
        .ok();

    let (next_page_parts, prev_page_parts) = match endpoint_url {
        Some(url) => BidirectionalPageConfig {
            endpoint_url: url,
            query_params: vec![("organization_id", Some(qs.organization_id.to_string()))],
            prev_cursor: events.first().map(|ev| Cursor {
                date: ev.created_at,
                id: ev.health_event_id,
            }),
            next_cursor: events.last().map(|ev| Cursor {
                date: ev.created_at,
                id: ev.health_event_id,
            }),
            is_backward,
            has_more,
            is_past_first_page: qs.pagination_cursor.is_some(),
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
