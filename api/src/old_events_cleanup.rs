use actix::clock::sleep;
use futures_util::TryStreamExt;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{query, query_scalar, Acquire, PgPool, Postgres};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::quotas::{Quota, Quotas};

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(20);

pub async fn periodically_clean_up_old_events(
    db: &PgPool,
    quotas: &Quotas,
    period: Duration,
    grace_period_in_day: u16,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    loop {
        if let Err(e) = clean_up_old_events(db, quotas, grace_period_in_day, delete).await {
            error!("Could not clean up old events: {e}");
        }

        sleep(period).await;
    }
}

async fn clean_up_old_events(
    db: &PgPool,
    quotas: &Quotas,
    grace_period_in_day: u16,
    delete: bool,
) -> Result<(), sqlx::Error> {
    trace!("Start cleaning up old events...");
    let start = Instant::now();
    let mut tx = db.begin().await?;
    let mut total_deleted_events = 0;

    let mut applications = query_scalar!("SELECT application__id from event.application").fetch(db);
    while let Some(application_id) = applications.try_next().await? {
        let deleted_events = delete_old_events_for_application(
            &mut tx,
            quotas,
            i32::from(grace_period_in_day),
            &application_id,
        )
        .await?;
        debug!("Found {deleted_events} old events to clean up for application '{application_id}'");
        total_deleted_events += deleted_events;
    }

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

async fn delete_old_events_for_application<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    quotas: &Quotas,
    grace_period_in_day: i32,
    application_id: &Uuid,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let retention_in_days = quotas
        .get_limit_for_application(&mut *db, Quota::DaysOfEventsRetention, application_id)
        .await?;
    trace!("Event retention for application '{application_id}' is {retention_in_days} days (plus {grace_period_in_day} days grace period)");

    let retention = PgInterval {
        days: retention_in_days + grace_period_in_day,
        ..Default::default()
    };
    let res = query!(
        "
            DELETE FROM event.event
            WHERE application__id = $1
                AND received_at + $2 < statement_timestamp()
        ",
        application_id,
        retention,
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
