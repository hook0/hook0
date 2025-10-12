use actix_web::rt::time::{sleep, timeout};
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
    let timeout_duration = period / 2;
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        match timeout(timeout_duration, refresh_materialized_views(db)).await {
            Err(_) => {
                error!("Timed out after {timeout_duration:?} while refreshing materialized views")
            }
            Ok(Err(e)) => error!("Could not refresh materialized views: {e}"),
            Ok(_) => (),
        }

        drop(permit);

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
