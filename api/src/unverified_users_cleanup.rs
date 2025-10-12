use actix_web::rt::time::sleep;
use chrono::TimeDelta;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{Acquire, PgPool, Postgres, query};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(40);

pub async fn periodically_clean_up_unverified_users(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    grace_period_in_day: u32,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(TimeDelta::days(grace_period_in_day.into())) {
        Ok(grace_period) => {
            while let Ok(permit) = housekeeping_semaphore.acquire().await {
                if let Err(e) = clean_up_unverified_users(db, &grace_period, delete).await {
                    error!("Could not clean up unverified users: {e}");
                }
                drop(permit);

                sleep(period).await;
            }
        }
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

    let total_deleted_unverified_users = delete_unverified_users(&mut *tx, grace_period).await?;

    let total_deleted_unreachable_organizations =
        deleted_unreachable_organizations(&mut *tx, grace_period).await?;

    if delete {
        tx.commit().await?;

        if total_deleted_unverified_users + total_deleted_unreachable_organizations > 0 {
            debug!("Running vacuum analyze and reindexing...");
            vacuum_analyze_and_reindex(db).await?;
        }

        info!(
            "Cleaned up {total_deleted_unverified_users} unverified users and {total_deleted_unreachable_organizations} unreachable organizations in {:?}",
            start.elapsed()
        );
    } else {
        tx.rollback().await?;
        info!(
            "Could clean up {total_deleted_unverified_users} unverified users and {total_deleted_unreachable_organizations} unreachable organizations in {:?} (but transaction was rolled back)",
            start.elapsed()
        );
    }
    Ok(())
}

async fn delete_unverified_users<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM iam.user
            WHERE email_verified_at IS NULL
                AND created_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn deleted_unreachable_organizations<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
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
                AND price__id IS NULL
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

    query!("VACUUM ANALYZE iam.user, iam.organization")
        .execute(&mut *db)
        .await?;

    query!("REINDEX TABLE CONCURRENTLY iam.user")
        .execute(&mut *db)
        .await?;

    query!("REINDEX TABLE CONCURRENTLY iam.organization")
        .execute(&mut *db)
        .await?;

    Ok(())
}
