use actix::clock::sleep;
use chrono::TimeDelta;
use log::{error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{query, PgPool};
use std::time::{Duration, Instant};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(30);

pub async fn periodically_clean_up_unverified_users(
    db: &PgPool,
    period: Duration,
    grace_period_in_day: u32,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(TimeDelta::days(grace_period_in_day.into())) {
        Ok(grace_period) => loop {
            if let Err(e) = clean_up_unverified_users(db, &grace_period, delete).await {
                error!("Could not clean up unverified users: {e}");
            }

            sleep(period).await;
        },
        Err(e) => {
            error!("Could not convert grace period ({grace_period_in_day:?}) to a PG interval: {e}")
        }
    }
}

async fn clean_up_unverified_users(
    db: &PgPool,
    grace_period: &PgInterval,
    delete: bool,
) -> anyhow::Result<()> {
    trace!("Cleaning up unverified users...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    let deleted_users = query!(
        "
            DELETE FROM iam.user
            WHERE email_verified_at IS NULL
                AND created_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .execute(&mut *tx)
    .await?;

    let deleted_unreachable_organizations = query!(
        "
            DELETE FROM iam.organization
            WHERE created_at + $1 < statement_timestamp()
                AND organization__id IN (
                    SELECT o.organization__id
                    FROM iam.organization AS o
                    LEFT JOIN iam.user__organization AS uo ON uo.organization__id = o.organization__id
                    LEFT JOIN event.application AS a ON a.organization__id = o.organization__id
                    WHERE uo.user__id IS NULL
                         AND a.application__id IS NULL
                )
        ",
        grace_period,
    )
    .execute(&mut *tx)
    .await?;

    if delete {
        tx.commit().await?;
        info!(
            "Cleaned up {} unverified users and {} unreachable organizations in {:?}",
            deleted_users.rows_affected(),
            deleted_unreachable_organizations.rows_affected(),
            start.elapsed()
        );
    } else {
        tx.rollback().await?;
        info!(
            "Could clean up {} unverified users and {} unreachable organizations in {:?} (but transaction was rolled back)",
            deleted_users.rows_affected(),
            deleted_unreachable_organizations.rows_affected(),
            start.elapsed()
        );
    }
    Ok(())
}
