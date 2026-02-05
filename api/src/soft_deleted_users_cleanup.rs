use actix_web::rt::time::sleep;
use chrono::TimeDelta;
use log::{debug, error, info, trace};
use sqlx::postgres::types::PgInterval;
use sqlx::{Acquire, PgPool, Postgres, query, query_scalar};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

const STARTUP_GRACE_PERIOD: Duration = Duration::from_secs(60);
const DELETION_GRACE_PERIOD_DAYS: i64 = 30;

/// Periodically erases users who requested account deletion more than 30 days ago.
/// This implements the GDPR right to erasure (Art. 17) with a grace period allowing users
/// to cancel their deletion request.
///
/// The deletion process:
/// 1. Deletes organizations where the user is the sole member (cascades to all Hook0 data)
/// 2. Removes user from organizations with multiple members
/// 3. Anonymizes user personal data (email, name, password)
pub async fn periodically_clean_up_soft_deleted_users(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(TimeDelta::days(DELETION_GRACE_PERIOD_DAYS)) {
        Ok(grace_period) => {
            while let Ok(permit) = housekeeping_semaphore.acquire().await {
                if let Err(e) = clean_up_soft_deleted_users(db, &grace_period, delete).await {
                    error!("Could not clean up deleted users: {e}");
                }
                drop(permit);

                sleep(period).await;
            }
        }
        Err(e) => {
            error!(
                "Could not convert deletion grace period ({DELETION_GRACE_PERIOD_DAYS} days) to a PG interval: {e}"
            )
        }
    }
}

async fn clean_up_soft_deleted_users(
    db: &PgPool,
    grace_period: &PgInterval,
    delete: bool,
) -> anyhow::Result<()> {
    trace!("Cleaning up users who requested deletion...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    // Step 1: Get users eligible for deletion
    let users_to_delete = get_users_to_delete(&mut *tx, grace_period).await?;

    if users_to_delete.is_empty() {
        tx.rollback().await?;
        trace!("No users to clean up");
        Ok(())
    } else {
        // Step 2: Remove each user from their organizations
        let total_memberships_removed =
            remove_users_from_organizations(&mut *tx, &users_to_delete).await?;

        // Step 3: Anonymize user personal data
        let total_erased = erase_user_personal_data(&mut *tx, grace_period).await?;

        if delete {
            tx.commit().await?;

            if users_to_delete.is_empty() {
                debug!("Running vacuum analyze...");
                vacuum_analyze(db).await?;
            }

            info!(
                "Deleted {} users in {:?}: {total_memberships_removed} memberships removed, {total_erased} users anonymized",
                users_to_delete.len(),
                start.elapsed(),
            );
        } else {
            tx.rollback().await?;
            info!(
                "Would delete {} users in {:?}: {total_memberships_removed} memberships (rolled back)",
                users_to_delete.len(),
                start.elapsed(),
            );
        }
        Ok(())
    }
}

/// Returns user IDs eligible for deletion (requested > grace_period ago, not yet deleted)
async fn get_users_to_delete<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<Vec<Uuid>, sqlx::Error> {
    let mut db = db.acquire().await?;

    let users = query_scalar!(
        "
            SELECT user__id
            FROM iam.user
            WHERE deletion_requested_at IS NOT NULL
                AND deleted_at IS NULL
                AND deletion_requested_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .fetch_all(&mut *db)
    .await?;

    Ok(users)
}

/// Removes users from all organizations.
/// This preserves the organization and its data for other members.
/// If the user was the sole member of the organization, it will be deleted by the `periodically_clean_up_unverified_users` task.
async fn remove_users_from_organizations<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    user_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM iam.user__organization
            WHERE user__id = ANY($1)
        ",
        user_ids,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

/// Erases personal data of users who requested account deletion more than `grace_period` ago.
/// This implements GDPR Article 17 (Right to Erasure) by:
/// 1. Setting `deleted_at` to mark the account as deleted
/// 2. Anonymizing email with a unique identifier (to prevent reuse conflicts)
/// 3. Clearing first_name and last_name (set to empty string)
/// 4. Clearing password (account can no longer be used)
///
/// The user record is preserved for audit trail but no longer contains personal data.
async fn erase_user_personal_data<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    grace_period: &PgInterval,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            UPDATE iam.user
            SET
                deleted_at = statement_timestamp(),
                email = 'deleted-' || user__id::text || '-' || extract(epoch from statement_timestamp()) || '@deleted.invalid',
                first_name = '',
                last_name = '',
                password = ''
            WHERE deletion_requested_at IS NOT NULL
                AND deleted_at IS NULL
                AND deletion_requested_at + $1 < statement_timestamp()
        ",
        grace_period,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

async fn vacuum_analyze<'a, A: Acquire<'a, Database = Postgres>>(db: A) -> Result<(), sqlx::Error> {
    let mut db = db.acquire().await?;

    query!("VACUUM ANALYZE iam.user, iam.user__organization")
        .execute(&mut *db)
        .await?;

    Ok(())
}
