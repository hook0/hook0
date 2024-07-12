use actix::clock::sleep;
use anyhow::anyhow;
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

    loop {
        if let Err(e) = clean_up_unverified_users(db, grace_period_in_day, delete).await {
            error!("Could not clean up unverified users: {e}");
        }

        sleep(period).await;
    }
}

async fn clean_up_unverified_users(
    db: &PgPool,
    grace_period_in_day: u32,
    delete: bool,
) -> anyhow::Result<()> {
    trace!("Cleaning up unverified users...");
    let start = Instant::now();

    let days = TimeDelta::days(grace_period_in_day.into());
    let interval = PgInterval::try_from(days).map_err(|e| anyhow!("{e}"))?;

    let mut tx = db.begin().await?;

    let result = query!(
        "
            DELETE FROM iam.user
            WHERE email_verified_at IS NULL
                AND created_at < NOW() - $1::interval
        ",
        interval,
    )
    .execute(&mut *tx)
    .await?;

    if delete {
        tx.commit().await?;
        info!(
            "Cleaned up {} unverified users in {:?}",
            result.rows_affected(),
            start.elapsed()
        );
    } else {
        tx.rollback().await?;
        info!(
            "Could clean up {} unverified users in {:?} (but transaction was rolled back)",
            result.rows_affected(),
            start.elapsed()
        );
    }
    Ok(())
}
