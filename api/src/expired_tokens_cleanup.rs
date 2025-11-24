use actix_web::rt::time::sleep;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{Acquire, PgPool, Postgres, query};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(35);

pub async fn periodically_clean_up_expired_tokens(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    grace_period: Duration,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;
    match PgInterval::try_from(grace_period) {
        Ok(grace_period) => {
            while let Ok(permit) = housekeeping_semaphore.acquire().await {
                if let Err(e) = clean_up_expired_tokens(db, &grace_period, delete).await {
                    error!("Could not clean up expired tokens: {e}");
                }
                drop(permit);

                sleep(period).await;
            }
        }
        Err(e) => error!("Could not convert grace period ({grace_period:?}) to a PG interval: {e}"),
    }
}

async fn clean_up_expired_tokens(
    db: &PgPool,
    grace_period: &PgInterval,
    delete: bool,
) -> Result<(), sqlx::Error> {
    trace!("Start cleaning up expired tokens...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    debug!("Removing expired tokens...");
    let total_deleted_tokens = delete_expired_tokens(&mut *tx, grace_period).await?;

    if delete {
        tx.commit().await?;

        if total_deleted_tokens > 0 {
            debug!("Running vacuum analyze...");
            vacuum_analyze(db).await?;
        }

        info!(
            "Cleaned up {total_deleted_tokens} expired tokens in {:?}",
            start.elapsed()
        );
    } else {
        tx.rollback().await?;
        info!(
            "Could clean up {total_deleted_tokens} expired tokens in {:?} (but transaction was rolled back)",
            start.elapsed()
        );
    }
    Ok(())
}

async fn delete_expired_tokens<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM iam.token
            WHERE expired_at IS NOT NULL
                AND expired_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn vacuum_analyze<'a, A: Acquire<'a, Database = Postgres>>(db: A) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    query!("VACUUM ANALYZE iam.token").execute(&mut *db).await?;

    Ok(())
}
