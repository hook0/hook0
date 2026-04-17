use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar};
use tracing::error;
use uuid::Uuid;

use crate::iam::{Action, authorize_for_application};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::subscription_health_monitor::{HealthEventCause, HealthStatus};

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct HealthEvent {
    pub health_event_id: Uuid,
    pub subscription_id: Uuid,
    pub status: HealthStatus,
    pub cause: HealthEventCause,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// One aggregated point on the failure-rate timeline.
#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct HealthBucket {
    pub bucket_start: DateTime<Utc>,
    pub total_count: i64,
    pub failed_count: i64,
}

// Events + buckets in one round-trip for the chart.
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct HealthTimeline {
    pub events: Vec<HealthEvent>,
    pub buckets: Vec<HealthBucket>,
}

// Window selector. Granularity adapts so point count stays under ~360.
#[derive(Debug, Clone, Copy, Deserialize, Apiv2Schema)]
#[serde(deny_unknown_fields)]
pub enum HealthWindow {
    #[serde(rename = "24h")]
    H24,
    #[serde(rename = "7d")]
    D7,
    #[serde(rename = "30d")]
    D30,
}

impl HealthWindow {
    fn lookback(self) -> ChronoDuration {
        match self {
            Self::H24 => ChronoDuration::hours(24),
            Self::D7 => ChronoDuration::days(7),
            Self::D30 => ChronoDuration::days(30),
        }
    }

    // date_bin aggregation interval. Keeps chart point count under ~360.
    fn granularity(self) -> ChronoDuration {
        match self {
            Self::H24 => ChronoDuration::minutes(5),
            Self::D7 => ChronoDuration::minutes(30),
            Self::D30 => ChronoDuration::hours(2),
        }
    }
}

#[derive(Debug, Deserialize, Apiv2Schema)]
#[serde(deny_unknown_fields)]
pub struct Qs {
    window: HealthWindow,
}

#[api_v2_operation(
    summary = "Health timeline for a subscription (events + chart buckets)",
    description = "Returns state-change events plus aggregated total/failed counters within the selected window. Granularity adapts: 5min for 24h, 30min for 7d, 2h for 30d.",
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
) -> Result<Json<HealthTimeline>, Hook0Problem> {
    let subscription_id = subscription_id_path.into_inner();
    let window = qs.window;

    // 1. Resolve application owning the subscription.
    let application_id = query_scalar!(
        r#"
        select application__id
        from webhook.subscription
        where subscription__id = $1 and deleted_at is null
        "#,
        subscription_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("DB error fetching application for subscription: {e}");
        Hook0Problem::InternalServerError
    })?
    .ok_or(Hook0Problem::NotFound)?;

    // 2. Authorize.
    let authorized = authorize_for_application(
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

    if !authorized {
        return Err(Hook0Problem::Forbidden);
    }

    let window_start = Utc::now() - window.lookback();
    let granularity = window.granularity();

    let (events, buckets) = tokio::try_join!(
        async {
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
                    where subscription__id = $1 and created_at >= $2
                    order by created_at desc
                "#,
                subscription_id,
                window_start,
            )
            .fetch_all(&state.db)
            .await
            .map_err(|e| {
                error!(
                    "health timeline: events query failed for subscription {subscription_id}: {e}"
                );
                Hook0Problem::from(e)
            })
        },
        async {
            query_as!(
            HealthBucket,
            r#"
                select
                    date_bin($1::interval, bucket_start, '1970-01-01T00:00:00Z'::timestamptz) as "bucket_start!",
                    sum(total_count)::bigint  as "total_count!",
                    sum(failed_count)::bigint as "failed_count!"
                from webhook.subscription_health_bucket
                where subscription__id = $2 and bucket_start >= $3
                group by 1
                order by 1 asc
            "#,
            granularity as _,
            subscription_id,
            window_start,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            error!(
                "health timeline: bucket query failed for subscription {subscription_id} window {window:?}: {e}"
            );
            Hook0Problem::from(e)
        })
        }
    )?;

    Ok(Json(HealthTimeline { events, buckets }))
}
