//! FK-chain fixture builder for health monitor tests.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Inserts the minimum FK-chain for a subscription + request_attempts:
///   iam.organization -> event.application -> event.service -> event.resource_type
///   -> event.verb -> event.event_type -> event.application_secret -> event.event
///   -> webhook.subscription (with target) -> webhook.request_attempt
///
/// Returns (org_id, app_id, sub_id).
pub(in crate::subscription_health_monitor::evaluation) async fn insert_test_fixtures(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    num_succeeded: i32,
    num_failed: i32,
    attempts_timestamp: DateTime<Utc>,
) -> (Uuid, Uuid, Uuid) {
    let org_id = Uuid::now_v7();
    let app_id = Uuid::now_v7();
    let sub_id = Uuid::now_v7();
    let target_id = sub_id; // target__id = subscription target__id (UNIQUE)
    let secret_token = Uuid::now_v7();

    // 1. Organization
    sqlx::query!(
        "INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, $2, $3)",
        org_id,
        "test-org-health",
        Uuid::nil(),
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 2. Application
    sqlx::query!(
        "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, $3)",
        app_id,
        org_id,
        "test-app",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 3. Service, resource_type, verb (for event_type FK chain)
    sqlx::query!(
        "INSERT INTO event.service (service__name, application__id) VALUES ($1, $2)",
        "svc",
        app_id,
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO event.resource_type (resource_type__name, application__id, service__name) VALUES ($1, $2, $3)",
        "res",
        app_id,
        "svc",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO event.verb (verb__name, application__id) VALUES ($1, $2)",
        "created",
        app_id,
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 4. Event type (generated column: svc.res.created)
    sqlx::query!(
        "INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name) VALUES ($1, $2, $3, $4)",
        app_id,
        "svc",
        "res",
        "created",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 5. Application secret
    sqlx::query!(
        "INSERT INTO event.application_secret (token, application__id) VALUES ($1, $2)",
        secret_token,
        app_id,
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 6. Subscription (labels required, target__id must be unique)
    // Indentation of continuation lines below is load-bearing: sqlx caches
    // queries by exact SQL text. Keep 15 leading spaces on continuations.
    sqlx::query!(
        r#"INSERT INTO webhook.subscription
               (subscription__id, application__id, target__id, is_enabled, labels)
               VALUES ($1, $2, $3, true, '{"env":"test"}'::jsonb)"#,
        sub_id,
        app_id,
        target_id,
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 7. Target HTTP (inherits webhook.target; FK to subscription.target__id)
    sqlx::query!(
        "INSERT INTO webhook.target_http (target__id, method, url) VALUES ($1, $2, $3)",
        target_id,
        "POST",
        "https://example.com/webhook",
    )
    .execute(&mut **tx)
    .await
    .unwrap();

    // 8. Insert events + request_attempts
    // Disable the dispatch trigger once to avoid side-effects
    sqlx::query!("ALTER TABLE event.event DISABLE TRIGGER event_dispatch")
        .execute(&mut **tx)
        .await
        .unwrap();

    for i in 0..(num_succeeded + num_failed) {
        let event_id = Uuid::now_v7();
        let attempt_id = Uuid::now_v7();
        let is_failed = i >= num_succeeded;

        // Indentation of continuation lines below is load-bearing: sqlx
        // caches by exact SQL text. Keep 19 leading spaces on continuations.
        sqlx::query!(
            r#"INSERT INTO event.event
                   (event__id, application__id, event_type__name, payload_content_type, ip, occurred_at, application_secret__token, labels)
                   VALUES ($1, $2, 'svc.res.created', 'application/json', '127.0.0.1'::inet, $3, $4, '{"env":"test"}'::jsonb)"#,
            event_id,
            app_id,
            attempts_timestamp,
            secret_token,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        // Indentation of continuation lines below is load-bearing: sqlx
        // caches by exact SQL text. Keep 23 leading spaces on continuations.
        if is_failed {
            sqlx::query!(
                r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, failed_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                attempt_id,
                event_id,
                sub_id,
                app_id,
                attempts_timestamp,
            )
            .execute(&mut **tx)
            .await
            .unwrap();
        } else {
            sqlx::query!(
                r#"INSERT INTO webhook.request_attempt
                       (request_attempt__id, event__id, subscription__id, application__id, succeeded_at)
                       VALUES ($1, $2, $3, $4, $5)"#,
                attempt_id,
                event_id,
                sub_id,
                app_id,
                attempts_timestamp,
            )
            .execute(&mut **tx)
            .await
            .unwrap();
        }
    }

    sqlx::query!("ALTER TABLE event.event ENABLE TRIGGER event_dispatch")
        .execute(&mut **tx)
        .await
        .unwrap();

    (org_id, app_id, sub_id)
}
