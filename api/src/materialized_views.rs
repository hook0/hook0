use actix::clock::sleep;
use log::{debug, error, trace};
use sqlx::{query, PgPool};
use std::time::{Duration, Instant};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(10);
// Cleaning task each hour
const STARTUP_CLEAN_UNVERIFIED_USERS_PERIOD: Duration = Duration::from_secs(60 * 60);

pub async fn periodically_clean_unverified_users(db: &PgPool, period: Duration) {
    sleep(STARTUP_CLEAN_UNVERIFIED_USERS_PERIOD).await;

    loop {
        if let Err(e) = clean_unverified_users(db).await {
            error!("Could not clean unverified users: {e}");
        }

        sleep(period).await;
    }
}

pub async fn periodically_refresh_materialized_views(db: &PgPool, period: Duration) {
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
        if let Err(e) = refresh_materialized_views(db).await {
            error!("Could not refresh materialized views: {e}");
        }

        sleep(period).await;
    }
}

async fn refresh_materialized_views(db: &PgPool) -> Result<(), sqlx::Error> {
    trace!("Refreshing materialized view event.events_per_day...");
    let start = Instant::now();
    query!("REFRESH MATERIALIZED VIEW CONCURRENTLY event.events_per_day")
        .execute(db)
        .await?;
    debug!(
        "Materialized view event.events_per_day was refreshed in {:?}",
        start.elapsed()
    );
    Ok(())
}

async fn clean_unverified_users(db: &PgPool) -> Result<(), sqlx::Error> {
    trace!("Cleaning unverified users...");
    let start = Instant::now();
    query!("DELETE FROM iam.user where email_verified_at is null and created_at < now() - interval '7 day'")
        .execute(db)
        .await?;
    debug!("Unverified users cleaned in {:?}", start.elapsed());
    Ok(())
}
