use actix_web::rt::time::sleep;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{Acquire, PgPool, Postgres, query};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(20);

pub async fn periodically_clean_up_soft_deleted_applications(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    grace_period: Duration,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(grace_period) {
        Ok(grace_period) => {
            while let Ok(permit) = housekeeping_semaphore.acquire().await {
                if let Err(e) = clean_up_soft_deleted_applications(db, &grace_period).await {
                    error!("Could not clean up deleted applications: {e}");
                }
                drop(permit);

                sleep(period).await;
            }
        }
        Err(e) => {
            error!("Could not convert grace period ({grace_period:?}) to a PG interval: {e}")
        }
    }
}

async fn clean_up_soft_deleted_applications(
    db: &PgPool,
    grace_period: &PgInterval,
) -> anyhow::Result<()> {
    trace!("Cleaning up deleted applications...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    let total_deleted_soft_deleted_applications =
        purge_soft_soft_deleted_applications(&mut *tx, grace_period).await?;

    tx.commit().await?;

    if total_deleted_soft_deleted_applications > 0 {
        debug!("Running vacuum analyze...");
        vacuum_analyze(db).await?;
    }

    info!(
        "Cleaned up {total_deleted_soft_deleted_applications} deleted applications in {:?}",
        start.elapsed()
    );
    Ok(())
}

async fn purge_soft_soft_deleted_applications<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM event.application
            WHERE deleted_at IS NOT NULL
                AND deleted_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn vacuum_analyze<'a, A: Acquire<'a, Database = Postgres>>(db: A) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    // We do not vacuum the following tables: event.event, webhook.request_attempt, webhook.response
    // This is because it is done by the old events cleanup task anyway

    query!(
        "
            VACUUM ANALYZE
                event.application,
                event.application_secret,
                event.service,
                event.resource_type,
                event.verb,
                event.event_type,
                webhook.subscription,
                webhook.subscription__event_type,
                webhook.subscription__worker,
                webhook.target_http
        "
    )
    .execute(&mut *db)
    .await?;

    Ok(())
}
