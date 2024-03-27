use actix::clock::sleep;
use log::{debug, error, info, trace};
use sqlx::{query, Acquire, PgPool, Postgres};
use std::time::{Duration, Instant};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(20);

pub async fn periodically_clean_up_old_events(
    db: &PgPool,
    period: Duration,
    global_days_of_events_retention_limit: i32,
    grace_period_in_day: u16,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
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
        info!("Cleaned up {total_deleted_events} old events and {total_dangling_responses} dangling responses in {:?}", start.elapsed());
    } else {
        tx.rollback().await?;
        info!("Could clean up {total_deleted_events} old events and {total_dangling_responses} dangling responses in {:?} (but transaction was rolled back)", start.elapsed());
    }
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
            WITH retention AS (
                SELECT a.application__id, MAKE_INTERVAL(days => COALESCE(LEAST(a.days_of_events_retention_limit, p.days_of_events_retention_limit), $1) + $2) AS events_retention_limit
                FROM event.application AS a
                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
            )
            DELETE FROM event.event
            WHERE event__id IN (
                SELECT e.event__id
                FROM event.event AS e
                INNER JOIN retention AS r ON r.application__id = e.application__id
                WHERE e.received_at + r.events_retention_limit < statement_timestamp()
            );
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
