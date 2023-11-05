use actix::clock::sleep;
use log::{debug, error, trace};
use sqlx::{query, PgPool};
use std::time::{Duration, Instant};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(10);

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
