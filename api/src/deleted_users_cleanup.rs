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
pub async fn periodically_clean_up_deleted_users(
    housekeeping_semaphore: &Semaphore,
    db: &PgPool,
    period: Duration,
    delete: bool,
) {
    sleep(STARTUP_GRACE_PERIOD).await;

    match PgInterval::try_from(TimeDelta::days(DELETION_GRACE_PERIOD_DAYS)) {
        Ok(grace_period) => {
            while let Ok(permit) = housekeeping_semaphore.acquire().await {
                if let Err(e) = clean_up_deleted_users(db, &grace_period, delete).await {
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

async fn clean_up_deleted_users(
    db: &PgPool,
    grace_period: &PgInterval,
    delete: bool,
) -> anyhow::Result<()> {
    trace!("Cleaning up users who requested deletion...");
    let start = Instant::now();

    let mut tx = db.begin().await?;

    // Step 1: Get users eligible for deletion
    let users_to_delete = get_users_to_delete(&mut *tx, grace_period).await?;
    let total_users = users_to_delete.len();

    if total_users == 0 {
        tx.rollback().await?;
        trace!("No users to clean up");
        return Ok(());
    }

    // Step 2: For each user, delete their associated data
    let mut total_orgs_deleted = 0u64;
    let mut total_memberships_removed = 0u64;

    for user_id in &users_to_delete {
        // Delete organizations where user is sole member (CASCADE handles all Hook0 data)
        let orgs_deleted = delete_sole_member_organizations(&mut *tx, user_id).await?;
        total_orgs_deleted += orgs_deleted;

        // Remove user from organizations with multiple members
        let memberships_removed = remove_user_from_organizations(&mut *tx, user_id).await?;
        total_memberships_removed += memberships_removed;
    }

    // Step 3: Anonymize user personal data
    let total_erased = erase_user_personal_data(&mut *tx, grace_period).await?;

    if delete {
        tx.commit().await?;

        if total_users > 0 {
            debug!("Running vacuum analyze...");
            vacuum_analyze(db).await?;
        }

        info!(
            "Deleted {total_users} users in {:?}: {} orgs deleted, {} memberships removed, {} users anonymized",
            start.elapsed(),
            total_orgs_deleted,
            total_memberships_removed,
            total_erased
        );
    } else {
        tx.rollback().await?;
        info!(
            "Would delete {total_users} users in {:?}: {} orgs, {} memberships (rolled back)",
            start.elapsed(),
            total_orgs_deleted,
            total_memberships_removed
        );
    }
    Ok(())
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

/// Deletes organizations where the user is the sole member.
/// Due to CASCADE constraints, this also deletes:
/// - Applications (and their secrets, event_types, events, services, verbs, resource_types)
/// - Subscriptions (and their event_type mappings, worker assignments)
/// - Request attempts
/// - Quota notifications
async fn delete_sole_member_organizations<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    user_id: &Uuid,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM iam.organization
            WHERE organization__id IN (
                -- Organizations where this user is the ONLY member
                SELECT uo.organization__id
                FROM iam.user__organization uo
                WHERE uo.user__id = $1
                AND NOT EXISTS (
                    SELECT 1 FROM iam.user__organization other
                    WHERE other.organization__id = uo.organization__id
                    AND other.user__id != $1
                )
            )
        ",
        user_id,
    )
    .execute(&mut *db)
    .await?;

    Ok(res.rows_affected())
}

/// Removes user from organizations where they are not the sole member.
/// This preserves the organization and its data for other members.
async fn remove_user_from_organizations<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    user_id: &Uuid,
) -> Result<u64, sqlx::Error> {
    let mut db = db.acquire().await?;

    let res = query!(
        "
            DELETE FROM iam.user__organization
            WHERE user__id = $1
        ",
        user_id,
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
                email = 'deleted-' || user__id::text || '@deleted.invalid',
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

    query!("VACUUM ANALYZE iam.user").execute(&mut *db).await?;

    Ok(())
}
