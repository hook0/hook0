use actix_web::rt::time::sleep;
use sqlx::{Acquire, PgPool, Postgres, query};
use std::time::{Duration, Instant};
use thousands::Separable;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, trace};

use crate::humanize::humanize_duration;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(30);

pub async fn periodically_clean_up_old_events(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    global_days_of_events_retention_limit: i32,
    grace_period_in_day: u16,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    while let Ok(permit) = housekeeping_semaphore.acquire().await {
        if let Err(e) = clean_up_old_events_and_responses(
            db,
            global_days_of_events_retention_limit,
            grace_period_in_day,
            delete,
        )
        .await
        {
            error!("Could not clean up old events: {e}");
        }
        drop(permit);

        sleep(period).await;
    }
}

async fn clean_up_old_events_and_responses(
    db: &PgPool,
    global_days_of_events_retention_limit: i32,
    grace_period_in_day: u16,
    delete: bool,
) -> Result<(), sqlx::Error> {
    trace!("Start cleaning up old events...");
    let start = Instant::now();

    debug!("Backup up events per day...");
    backup_events_per_day(db).await?;

    let mut tx = db.begin().await?;

    debug!("Removing old events...");
    let total_deleted_events = delete_old_events(
        &mut *tx,
        global_days_of_events_retention_limit,
        i32::from(grace_period_in_day),
    )
    .await?;

    debug!("Removing dangling responses...");
    let total_dangling_responses = delete_dangling_responses(&mut *tx).await?;

    if delete {
        tx.commit().await?;

        if total_deleted_events + total_dangling_responses > 0 {
            debug!("Running vacuum analyze...");
            vacuum_analyze(db).await?;
        }

        info!(
            "Cleaned up {} old events and {} dangling responses in {}",
            total_deleted_events.separate_with_commas(),
            total_dangling_responses.separate_with_commas(),
            humanize_duration(start.elapsed()),
        );
    } else {
        tx.rollback().await?;
        info!(
            "Could clean up {} old events and {} dangling responses in {} (but transaction was rolled back)",
            total_deleted_events.separate_with_commas(),
            total_dangling_responses.separate_with_commas(),
            humanize_duration(start.elapsed()),
        );
    }

    Ok(())
}

async fn backup_events_per_day<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    // A day is only counted an hour after it ends, so ingest transactions still open at midnight
    // have committed before we snapshot. Rows here are never updated, so an undercount is permanent.
    query!(
        "
            INSERT INTO event.all_time_events_per_day (application__id, date, amount)
            SELECT application__id, received_at::date AS date, COUNT(event__id)::integer AS amount
            FROM event.event
            WHERE received_at >= COALESCE(
                    ((SELECT MAX(date) FROM event.all_time_events_per_day) + 1)::timestamptz,
                    '-infinity'::timestamptz
                )
                AND received_at < date_trunc('day', statement_timestamp() - interval '1 hour')
            GROUP BY application__id, received_at::date
            ON CONFLICT (application__id, date) DO NOTHING
        ",
    )
    .execute(&mut *db)
    .await?;

    Ok(())
}

async fn delete_old_events<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    global_days_of_events_retention_limit: i32,
    grace_period_in_day: i32,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            WITH retention AS MATERIALIZED (
                SELECT a.application__id, statement_timestamp() - MAKE_INTERVAL(days => COALESCE(LEAST(a.days_of_events_retention_limit, p.days_of_events_retention_limit), $1) + $2) AS cutoff
                FROM event.application AS a
                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
            )
            -- Performance-critical query shape; do NOT simplify. event.event can contain millions of rows.
            -- The per-application retention cutoff is precomputed as a timestamp in the CTE so `received_at < cutoff` is sargable, and `MATERIALIZED` stops the CTE from being inlined.
            -- The `OFFSET 0` below is an intentional optimizer fence: without it Postgres de-correlates the LATERAL into a hash join over a full seq scan of event.event.
            -- With it, the subquery runs once per application and uses event_application__id_received_at_idx for a per-app index range scan.
            DELETE FROM event.event AS e
            USING retention AS r,
            LATERAL (
                SELECT e2.event__id
                FROM event.event AS e2
                WHERE e2.application__id = r.application__id
                  AND e2.received_at < r.cutoff
                OFFSET 0
            ) AS old
            WHERE e.event__id = old.event__id;
        ",
        global_days_of_events_retention_limit,
        grace_period_in_day,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn delete_dangling_responses<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM webhook.response
            USING webhook.response AS r
            LEFT OUTER JOIN webhook.request_attempt AS ra ON ra.response__id = r.response__id
            WHERE webhook.response.response__id = r.response__id
                AND ra.response__id IS NULL
        "
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn vacuum_analyze<'a, A: Acquire<'a, Database = Postgres>>(db: A) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    trace!(
        "Running VACUUM ANALYZE on big tables: event.event, webhook.request_attempt, webhook.response"
    );
    query!("VACUUM ANALYZE event.event, webhook.request_attempt, webhook.response")
        .execute(&mut *db)
        .await?;

    Ok(())
}
