use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::{Deserialize, Serialize};
use sqlx::query_scalar;
use tracing::error;
use uuid::Uuid;

use crate::iam::{authorize_for_application, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct SubscriptionHealthEvent {
    pub health_event_id: Uuid,
    pub subscription_id: Uuid,
    pub status: String,
    pub source: String,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct HealthEventsQs {
    pub organization_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

/// List the most recent health events for a subscription.
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
    qs: Query<HealthEventsQs>,
) -> Result<Json<Vec<SubscriptionHealthEvent>>, Hook0Problem> {
    let subscription_id = subscription_id.into_inner();
    let organization_id = qs.organization_id;
    let limit = qs.limit.clamp(1, MAX_LIMIT);

    // Look up application_id from subscription (same pattern as subscriptions::get)
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

    if application_id.is_none() {
        return Err(Hook0Problem::NotFound);
    }

    let application_id = application_id.expect("Could not unwrap application_id");

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

    let events = sqlx::query_as::<_, SubscriptionHealthEvent>(
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
            ORDER BY she.created_at DESC
            LIMIT $3
        ",
    )
    .bind(&subscription_id)
    .bind(&organization_id)
    .bind(&limit)
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(events))
}
