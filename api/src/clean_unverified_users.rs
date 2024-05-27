use actix::clock::sleep;
use log::{debug, error, trace};
use sqlx::error::BoxDynError;
use sqlx::postgres::types::PgInterval;
use sqlx::{query, PgPool};
use std::time::{Duration, Instant};

// Cleaning task each hour
const STARTUP_CLEAN_UNVERIFIED_USERS_PERIOD: Duration = Duration::from_secs(60 * 60);

pub async fn periodically_clean_unverified_users(db: &PgPool, period: Duration, interval_day: u64) {
    sleep(STARTUP_CLEAN_UNVERIFIED_USERS_PERIOD).await;

    loop {
        if let Err(e) = clean_unverified_users(db, &interval_day).await {
            error!("Could not clean unverified users: {e}");
        }

        sleep(period).await;
    }
}

async fn clean_unverified_users(db: &PgPool, interval_day: &u64) -> Result<(), BoxDynError> {
    trace!("Cleaning unverified users...");
    let start = Instant::now();
    let days = Duration::from_secs(*interval_day * 24 * 60 * 60);
    let interval = match PgInterval::try_from(days) {
        Ok(interval) => interval,
        Err(e) => {
            error!("Could not create interval: {e}");
            return Err(e);
        }
    };
    let result = query!(
        r#"
        DELETE FROM iam.user
        WHERE email_verified_at IS NULL
        AND created_at < NOW() - $1::INTERVAL
        "#,
        interval
    )
    .execute(db)
    .await?;
    let affected_rows = result.rows_affected();
    debug!(
        "Deleted {} unverified users in {:?}",
        affected_rows,
        start.elapsed()
    );
    Ok(())
}
