//! Periodic retention cleanup for `iam.signup_attribution` gclids.
//!
//! Data-minimisation safety net (GDPR art. 5.1.e): a gclid is a pseudonymous
//! ad-click identifier retained only until every enabled Google Ads conversion
//! has been uploaded, and never beyond `SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS`.
//!
//! The registration handler already prunes stale attribution rows lazily, but
//! that only runs when a new registration happens; if signups pause, stale
//! gclids would linger past the retention window. This periodic job enforces the
//! documented retention maximum unconditionally, on a timer, independent of
//! registration traffic. It deletes attribution rows older than the retention
//! window (the same logic the register handler runs inline), bounded per pass by
//! a `LIMIT` so a large backlog drains over successive passes rather than locking
//! a huge range at once.

use actix_web::rt::time::sleep;
use sqlx::PgPool;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info};

/// Wait before the first pass so the rest of the process has finished starting.
const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(55);

/// Upper bound on rows deleted per pass. Bounds the work (and lock footprint) of
/// a single pass; a larger backlog simply drains over successive passes.
const MAX_ROWS_PER_RUN: i64 = 500;

/// Periodically delete signup-attribution rows whose gclid is older than the
/// retention window. Acquires the housekeeping semaphore each pass, then sleeps
/// for `period`.
pub async fn periodically_prune_signup_attribution(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    retention_in_days: i32,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        match prune_stale_signup_attribution(db, retention_in_days).await {
            Ok(0) => {}
            Ok(pruned) => info!(
                target: "api::signup_attribution",
                rows = pruned,
                "pruned stale signup attribution rows past the retention window"
            ),
            Err(e) => error!("Could not prune stale signup attribution rows: {e}"),
        }
        drop(permit);

        sleep(period).await;
    }
}

/// Delete up to `MAX_ROWS_PER_RUN` signup-attribution rows whose `created_at` is
/// older than `retention_in_days` (clearing their gclids). Returns how many rows
/// were deleted. Mirrors the lazy cleanup in the registration handler; the LIMIT
/// (applied through a `ctid` subquery, since Postgres has no `DELETE ... LIMIT`)
/// bounds each pass.
async fn prune_stale_signup_attribution(
    db: &PgPool,
    retention_in_days: i32,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "
            DELETE FROM iam.signup_attribution
            WHERE ctid IN (
                SELECT ctid
                FROM iam.signup_attribution
                WHERE created_at < statement_timestamp() - MAKE_INTERVAL(days => $1)
                LIMIT $2
            )
        ",
        retention_in_days,
        MAX_ROWS_PER_RUN,
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google_ads::test_support::{seed_attribution, seed_org, seed_user};

    /// A gclid older than the retention window is cleared; a fresh one is kept.
    #[sqlx::test]
    async fn prunes_over_retention_but_keeps_fresh(pool: PgPool) {
        // Stale attribution: backdated beyond the 30-day retention window.
        let stale_user = seed_user(&pool).await;
        let stale_org = seed_org(&pool, stale_user).await;
        seed_attribution(&pool, stale_user, stale_org, "gclid-stale", true).await;
        sqlx::query(
            "UPDATE iam.signup_attribution SET created_at = statement_timestamp() - INTERVAL '60 days' WHERE organization__id = $1",
        )
        .bind(stale_org)
        .execute(&pool)
        .await
        .expect("backdate stale attribution");

        // Fresh attribution: created just now, within the window.
        let fresh_user = seed_user(&pool).await;
        let fresh_org = seed_org(&pool, fresh_user).await;
        seed_attribution(&pool, fresh_user, fresh_org, "gclid-fresh", true).await;

        let pruned = prune_stale_signup_attribution(&pool, 30)
            .await
            .expect("prune ok");
        assert_eq!(pruned, 1, "exactly the over-retention row is deleted");

        let stale_gone: bool = sqlx::query_scalar(
            "SELECT NOT EXISTS (SELECT 1 FROM iam.signup_attribution WHERE organization__id = $1)",
        )
        .bind(stale_org)
        .fetch_one(&pool)
        .await
        .expect("check stale row");
        assert!(stale_gone, "the over-retention gclid row is cleared");

        let fresh_gclid: Option<String> = sqlx::query_scalar(
            "SELECT gclid FROM iam.signup_attribution WHERE organization__id = $1",
        )
        .bind(fresh_org)
        .fetch_one(&pool)
        .await
        .expect("check fresh row");
        assert_eq!(
            fresh_gclid.as_deref(),
            Some("gclid-fresh"),
            "a gclid within the retention window is kept"
        );
    }
}
