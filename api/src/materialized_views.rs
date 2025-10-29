use actix_web::rt::time::sleep;
use log::{debug, error, trace};
use sqlx::{PgPool, query};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(10);

pub async fn periodically_refresh_materialized_views(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
) {
    let timeout = period / 2;
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = refresh_materialized_views(db, timeout).await {
            error!("Could not refresh materialized views: {e}")
        }

        drop(permit);

        sleep(period).await;
    }
}

async fn refresh_materialized_views(db: &PgPool, timeout: Duration) -> Result<(), sqlx::Error> {
    trace!("Refreshing materialized view event.events_per_day...");
    let start = Instant::now();

    let mut tx = db.begin().await?;
    query(&format!(
        "SET LOCAL statement_timeout = '{}s'",
        timeout.as_secs()
    ))
    .execute(&mut *tx)
    .await?;
    query!("REFRESH MATERIALIZED VIEW CONCURRENTLY event.events_per_day")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    query!("VACUUM ANALYZE event.events_per_day")
        .execute(db)
        .await?;

    debug!(
        "Materialized view event.events_per_day was refreshed in {:?}",
        start.elapsed()
    );
    Ok(())
}
