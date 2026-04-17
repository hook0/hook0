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
    Cursor, EncodedCursor, PageLimit, Paginated, Pagination, build_endpoint_url,
};
use crate::problems::Hook0Problem;
use crate::subscription_health_monitor::{HealthEventCause, HealthStatus};

const PAGE: PageLimit = PageLimit::new(30);

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
    pagination_cursor: Option<EncodedCursor>,
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
            PAGE.fetch_limit(),
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
            PAGE.fetch_limit(),
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

    // 3. Fetch events based on cursor direction
    let pagination = Pagination::new(PAGE, qs.pagination_cursor);

    let mut events = if pagination.is_backward() {
        HealthEvent::fetch_backward(&state.db, subscription_id, pagination.resolved_cursor())
            .await?
    } else {
        HealthEvent::fetch_forward(&state.db, subscription_id, pagination.resolved_cursor()).await?
    };

    // 4. Trim the over-fetched row and reverse if backward.
    let has_more = pagination.trim_and_orient(&mut events);

    // 5. Build pagination links
    let endpoint_url = build_endpoint_url(
        &state.app_url,
        &format!("/api/v1/subscriptions/{subscription_id}/health_events"),
    );

    let (next_page_parts, prev_page_parts) = pagination.build_page_parts(
        &events,
        endpoint_url,
        vec![("organization_id", qs.organization_id.to_string())],
        has_more,
        |ev| (ev.created_at, ev.health_event_id),
    );

    Ok(Paginated {
        data: Json(events),
        next_page_parts,
        prev_page_parts,
    })
}
