//! Paginated listing of subscription health events — the audit trail of warning/disabled/resolved transitions.
//!
//! How it works:
//! 1. The caller provides a subscription_id in the path and an organization_id in the query string.
//! 2. We look up the subscription's application_id, then authorize via IAM (health events are subscription-scoped, but IAM checks are application-scoped).
//! 3. Results come back newest-first with cursor-based pagination.

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
use tracing::error;
use url::Url;
use uuid::Uuid;

use crate::health_monitor::types::{HealthEventSource, HealthStatus};
use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::pagination::{Cursor, EncodedDescCursor, NextPageParts, Paginated};
use crate::problems::Hook0Problem;

const DEFAULT_PAGE_SIZE: i64 = 50;

/// A single health state transition — what happened, why, who triggered it, when.
#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct SubscriptionHealthEventStatus {
    pub health_event_id: Uuid,
    pub subscription_id: Uuid,
    pub status: HealthStatus,
    pub source: HealthEventSource,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Query string for listing health events — scopes to an org and optionally provides a pagination cursor.
#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct SubscriptionHealthEventListQs {
    pub organization_id: Uuid,
    pub pagination_cursor: Option<EncodedDescCursor>,
}

/// List the most recent health events for a subscription, newest first, with cursor-based pagination.
/// Returns an empty list (not 404) when the subscription exists but has no events yet.
#[api_v2_operation(
    summary = "List subscription health events",
    description = "Returns the most recent health events (warning, disabled, resolved) for a subscription, ordered newest first.",
    operation_id = "subscription_health_events.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Subscriptions Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    subscription_id: Path<Uuid>,
    qs: Query<SubscriptionHealthEventListQs>,
) -> Result<Paginated<Json<Vec<SubscriptionHealthEventStatus>>>, Hook0Problem> {
    let subscription_id = subscription_id.into_inner();
    let organization_id = qs.organization_id;

    // We need the application_id to authorize — health events are subscription-scoped, but IAM checks are application-scoped
    let application_id = query_scalar!(
        "SELECT application__id FROM webhook.subscription WHERE subscription__id = $1 AND deleted_at IS NULL LIMIT 1",
        &subscription_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("{}", &e);
        Hook0Problem::InternalServerError
    })?;

    // Subscription doesn't exist or was deleted — no point checking auth
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

    let pagination = qs.pagination_cursor.unwrap_or_default().0;

    let events = sqlx::query_as::<_, SubscriptionHealthEventStatus>(
        "
            SELECT
                she.health_event__id AS health_event_id,
                she.subscription__id AS subscription_id,
                she.status,
                she.source,
                she.user__id AS user_id,
                she.created_at
            FROM webhook.subscription_health_event she
            INNER JOIN webhook.subscription s ON s.subscription__id = she.subscription__id
            INNER JOIN event.application a ON a.application__id = s.application__id
            WHERE she.subscription__id = $1
              AND a.organization__id = $2
              AND (she.created_at, she.health_event__id) < ($3, $4)
            ORDER BY she.created_at DESC, she.health_event__id ASC
            LIMIT $5
        ",
    )
    .bind(subscription_id)
    .bind(organization_id)
    .bind(pagination.date)
    .bind(pagination.id)
    .bind(DEFAULT_PAGE_SIZE)
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // Build the "next page" link — we reconstruct the full URL from app_url because the pagination contract requires an absolute URI
    let next_page_parts = events.last().and_then(|e| {
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
                .join(&format!(
                    "/api/v1/subscriptions/{}/health_events",
                    subscription_id
                ))
                .inspect_err(|e| {
                    error!(
                        "Error that should never happen while building app URL for pagination: {e}"
                    );
                })
                .ok()
        })
        .map(|endpoint_url| NextPageParts {
            endpoint_url,
            qs: vec![("organization_id", Some(organization_id.to_string()))],
            cursor: Cursor {
                date: e.created_at,
                id: e.health_event_id,
            },
        })
    });

    Ok(Paginated {
        data: Json(events),
        next_page_parts,
    })
}
