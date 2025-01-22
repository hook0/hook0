use actix::clock::sleep;
use chrono::TimeDelta;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{query, Acquire, PgPool, Postgres};
use std::time::{Duration, Instant};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(30);

pub async fn periodically_clean_up_deleted_applications(
    db: &PgPool,
    period: Duration,
    grace_period_in_day: u32,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(TimeDelta::days(grace_period_in_day.into())) {
        Ok(grace_period) => loop {
            if let Err(e) = clean_up_deleted_applications(db, &grace_period).await {
                error!("Could not clean up deleted applications: {e}");
            }

            sleep(period).await;
        },
        Err(e) => {
            error!("Could not convert grace period ({grace_period_in_day:?}) to a PG interval: {e}")
        }
    }
}

async fn clean_up_deleted_applications(
    db: &PgPool,
    grace_period: &PgInterval,
) -> anyhow::Result<()> {
    trace!("Cleaning up deleted applications...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    let total_deleted_deleted_applications = delete_deleted_applications(&mut *tx, grace_period).await?;

    tx.commit().await?;

    if total_deleted_deleted_applications > 0 {
        debug!("Running vacuum analyze and reindexing...");
        vacuum_analyze_and_reindex(db).await?;
    }

    info!(
        "Cleaned up {total_deleted_deleted_applications} deleted applications in {:?}",
        start.elapsed()
    );
    Ok(())
}

async fn delete_deleted_applications<'a, A: Acquire<'a, Database = Postgres>>(
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

async fn vacuum_analyze_and_reindex<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    query!("VACUUM ANALYZE event.application")
        .execute(&mut *db)
        .await?;

    query!("REINDEX TABLE CONCURRENTLY event.application")
        .execute(&mut *db)
        .await?;

    Ok(())
}
