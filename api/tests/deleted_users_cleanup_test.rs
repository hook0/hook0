//! Integration tests for GDPR user deletion cleanup
//!
//! These tests verify that when a user requests account deletion:
//! 1. After 30 days, their personal data is anonymized
//! 2. Organizations where they are the sole member are deleted (CASCADE to all Hook0 data)
//! 3. They are removed from shared organizations
//!
//! Run with: DATABASE_URL="postgres://postgres:postgres@localhost:5432/hook0" cargo test --test deleted_users_cleanup_test --ignored

use chrono::{TimeDelta, Utc};
use sqlx::postgres::types::PgInterval;
use sqlx::{PgPool, query, query_scalar};
use std::env;
use uuid::Uuid;

async fn get_db_pool() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set to run tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

/// Creates a test user and returns their user_id
async fn create_test_user(db: &PgPool, email: &str) -> Uuid {
    query_scalar!(
        "
            INSERT INTO iam.user (email, password, first_name, last_name)
            VALUES ($1, 'test_password_hash', 'Test', 'User')
            RETURNING user__id
        ",
        email
    )
    .fetch_one(db)
    .await
    .expect("Failed to create test user")
}

/// Creates a test organization and returns its organization_id
async fn create_test_organization(db: &PgPool, name: &str) -> Uuid {
    // Get or create a test price
    let price_id = query_scalar!("SELECT price__id FROM pricing.price LIMIT 1")
        .fetch_optional(db)
        .await
        .expect("Failed to query prices");

    let price_id = match price_id {
        Some(id) => id,
        None => {
            // Create a test plan first
            let plan_id = query_scalar!(
                "
                    INSERT INTO pricing.plan (name, label)
                    VALUES ('Test Plan', 'Test')
                    RETURNING plan__id
                "
            )
            .fetch_one(db)
            .await
            .expect("Failed to create test plan");

            // Create a test price
            query_scalar!(
                "
                    INSERT INTO pricing.price (plan__id, amount, time_basis)
                    VALUES ($1, 0.00, 'month')
                    RETURNING price__id
                ",
                plan_id
            )
            .fetch_one(db)
            .await
            .expect("Failed to create test price")
        }
    };

    query_scalar!(
        "
            INSERT INTO iam.organization (name, price__id)
            VALUES ($1, $2)
            RETURNING organization__id
        ",
        name,
        price_id
    )
    .fetch_one(db)
    .await
    .expect("Failed to create test organization")
}

/// Links a user to an organization
async fn link_user_to_organization(db: &PgPool, user_id: &Uuid, org_id: &Uuid) {
    query!(
        "
            INSERT INTO iam.user__organization (user__id, organization__id, role)
            VALUES ($1, $2, 'editor')
        ",
        user_id,
        org_id
    )
    .execute(db)
    .await
    .expect("Failed to link user to organization");
}

/// Creates a test application in an organization
async fn create_test_application(db: &PgPool, org_id: &Uuid, name: &str) -> Uuid {
    query_scalar!(
        "
            INSERT INTO event.application (organization__id, name)
            VALUES ($1, $2)
            RETURNING application__id
        ",
        org_id,
        name
    )
    .fetch_one(db)
    .await
    .expect("Failed to create test application")
}

/// Marks a user as having requested deletion N days ago
async fn set_deletion_requested(db: &PgPool, user_id: &Uuid, days_ago: i64) {
    let requested_at = Utc::now() - TimeDelta::days(days_ago);

    query!(
        "
            UPDATE iam.user
            SET deletion_requested_at = $1
            WHERE user__id = $2
        ",
        requested_at,
        user_id
    )
    .execute(db)
    .await
    .expect("Failed to set deletion_requested_at");
}

/// Runs the cleanup logic for specific user IDs only (test isolation)
async fn run_cleanup_for_users(
    db: &PgPool,
    grace_period_days: i64,
    user_ids: &[Uuid],
) -> (u64, u64, u64) {
    let grace_period = PgInterval::try_from(TimeDelta::days(grace_period_days))
        .expect("Failed to convert grace period");

    let mut tx = db.begin().await.expect("Failed to start transaction");

    // Only consider users from the provided list
    let users_to_delete: Vec<Uuid> = query_scalar!(
        "
            SELECT user__id
            FROM iam.user
            WHERE user__id = ANY($1)
                AND deletion_requested_at IS NOT NULL
                AND deleted_at IS NULL
                AND deletion_requested_at + $2 < statement_timestamp()
        ",
        user_ids,
        &grace_period,
    )
    .fetch_all(&mut *tx)
    .await
    .expect("Failed to get users to delete");

    let mut total_orgs_deleted = 0u64;
    let mut total_memberships_removed = 0u64;

    for user_id in &users_to_delete {
        let orgs = query!(
            "
                DELETE FROM iam.organization
                WHERE organization__id IN (
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
        .execute(&mut *tx)
        .await
        .expect("Failed to delete organizations");
        total_orgs_deleted += orgs.rows_affected();

        let memberships = query!(
            "DELETE FROM iam.user__organization WHERE user__id = $1",
            user_id,
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to remove memberships");
        total_memberships_removed += memberships.rows_affected();
    }

    // Only anonymize users from the provided list
    let erased = query!(
        "
            UPDATE iam.user
            SET
                deleted_at = statement_timestamp(),
                email = 'deleted-' || user__id::text || '@deleted.invalid',
                first_name = '',
                last_name = '',
                password = ''
            WHERE user__id = ANY($1)
                AND deletion_requested_at IS NOT NULL
                AND deleted_at IS NULL
                AND deletion_requested_at + $2 < statement_timestamp()
        ",
        user_ids,
        &grace_period,
    )
    .execute(&mut *tx)
    .await
    .expect("Failed to anonymize users");

    tx.commit().await.expect("Failed to commit transaction");

    (
        total_orgs_deleted,
        total_memberships_removed,
        erased.rows_affected(),
    )
}

async fn user_exists_active(db: &PgPool, user_id: &Uuid) -> bool {
    let count: i64 = query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM iam.user WHERE user__id = $1 AND deleted_at IS NULL"#,
        user_id
    )
    .fetch_one(db)
    .await
    .expect("Failed to check user");

    count > 0
}

async fn user_is_anonymized(db: &PgPool, user_id: &Uuid) -> bool {
    let email: String = query_scalar!(r#"SELECT email FROM iam.user WHERE user__id = $1"#, user_id)
        .fetch_one(db)
        .await
        .expect("Failed to get user email");

    email.ends_with("@deleted.invalid")
}

async fn organization_exists(db: &PgPool, org_id: &Uuid) -> bool {
    let count: i64 = query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM iam.organization WHERE organization__id = $1"#,
        org_id
    )
    .fetch_one(db)
    .await
    .expect("Failed to check organization");

    count > 0
}

async fn application_exists(db: &PgPool, app_id: &Uuid) -> bool {
    let count: i64 = query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM event.application WHERE application__id = $1"#,
        app_id
    )
    .fetch_one(db)
    .await
    .expect("Failed to check application");

    count > 0
}

async fn cleanup_test_user(db: &PgPool, user_id: &Uuid) {
    let _ = query!("DELETE FROM iam.user WHERE user__id = $1", user_id)
        .execute(db)
        .await;
}

#[ignore]
#[tokio::test]
async fn test_user_deletion_after_grace_period_deletes_sole_member_org_and_data() {
    let db = get_db_pool().await;
    let test_id = Uuid::new_v4();

    let user_id = create_test_user(&db, &format!("test_deletion_{test_id}@example.com")).await;
    let org_id = create_test_organization(&db, &format!("Test Org {test_id}")).await;
    link_user_to_organization(&db, &user_id, &org_id).await;
    let app_id = create_test_application(&db, &org_id, "Test App").await;

    set_deletion_requested(&db, &user_id, 31).await;

    assert!(
        user_exists_active(&db, &user_id).await,
        "User should exist before cleanup"
    );
    assert!(
        organization_exists(&db, &org_id).await,
        "Org should exist before cleanup"
    );
    assert!(
        application_exists(&db, &app_id).await,
        "App should exist before cleanup"
    );

    let (orgs_deleted, _, users_anonymized) = run_cleanup_for_users(&db, 30, &[user_id]).await;

    assert_eq!(orgs_deleted, 1, "Should delete 1 organization");
    assert_eq!(users_anonymized, 1, "Should anonymize 1 user");
    assert!(
        !organization_exists(&db, &org_id).await,
        "Org should be deleted"
    );
    assert!(
        !application_exists(&db, &app_id).await,
        "App should be deleted (CASCADE)"
    );
    assert!(
        user_is_anonymized(&db, &user_id).await,
        "User should be anonymized"
    );

    cleanup_test_user(&db, &user_id).await;
}

#[ignore]
#[tokio::test]
async fn test_user_not_deleted_before_grace_period() {
    let db = get_db_pool().await;
    let test_id = Uuid::new_v4();

    let user_id = create_test_user(&db, &format!("test_not_deleted_{test_id}@example.com")).await;
    let org_id = create_test_organization(&db, &format!("Test Org Not Deleted {test_id}")).await;
    link_user_to_organization(&db, &user_id, &org_id).await;

    set_deletion_requested(&db, &user_id, 10).await;

    let (orgs_deleted, _, users_anonymized) = run_cleanup_for_users(&db, 30, &[user_id]).await;

    assert_eq!(orgs_deleted, 0, "Should not delete any organization");
    assert_eq!(users_anonymized, 0, "Should not anonymize any user");
    assert!(
        user_exists_active(&db, &user_id).await,
        "User should still exist"
    );
    assert!(
        organization_exists(&db, &org_id).await,
        "Org should still exist"
    );

    // Cleanup: cancel deletion and delete test data
    query!(
        "UPDATE iam.user SET deletion_requested_at = NULL WHERE user__id = $1",
        &user_id
    )
    .execute(&db)
    .await
    .expect("Failed to cancel deletion");
    cleanup_test_user(&db, &user_id).await;
}

#[ignore]
#[tokio::test]
async fn test_shared_org_preserved_when_user_deleted() {
    let db = get_db_pool().await;
    let test_id = Uuid::new_v4();

    let user1_id = create_test_user(&db, &format!("test_shared1_{test_id}@example.com")).await;
    let user2_id = create_test_user(&db, &format!("test_shared2_{test_id}@example.com")).await;
    let shared_org_id = create_test_organization(&db, &format!("Shared Org {test_id}")).await;
    link_user_to_organization(&db, &user1_id, &shared_org_id).await;
    link_user_to_organization(&db, &user2_id, &shared_org_id).await;
    let app_id = create_test_application(&db, &shared_org_id, "Shared App").await;

    set_deletion_requested(&db, &user1_id, 31).await;

    let (orgs_deleted, memberships_removed, users_anonymized) =
        run_cleanup_for_users(&db, 30, &[user1_id, user2_id]).await;

    assert_eq!(orgs_deleted, 0, "Should not delete shared organization");
    assert_eq!(memberships_removed, 1, "Should remove 1 membership");
    assert_eq!(users_anonymized, 1, "Should anonymize 1 user");
    assert!(
        organization_exists(&db, &shared_org_id).await,
        "Shared org should be preserved"
    );
    assert!(
        application_exists(&db, &app_id).await,
        "Shared app should be preserved"
    );
    assert!(
        user_is_anonymized(&db, &user1_id).await,
        "User1 should be anonymized"
    );
    assert!(
        user_exists_active(&db, &user2_id).await,
        "User2 should be unaffected"
    );

    cleanup_test_user(&db, &user1_id).await;
    cleanup_test_user(&db, &user2_id).await;
}
