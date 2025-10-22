//! Integration tests for FIFO subscription mode
//!
//! These tests verify that FIFO (First-In-First-Out) mode works correctly:
//! 1. Basic FIFO ordering - events processed in order
//! 2. FIFO with retries - subsequent events wait for retries to complete
//! 3. FIFO with multiple subscriptions - FIFO and non-FIFO work independently
//! 4. Worker crash recovery - orphaned states can be detected and cleaned
//!
//! NOTE: These tests require a PostgreSQL database connection.
//! Set DATABASE_URL environment variable to run these tests.

use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Helper struct for test database setup
struct TestDb {
    pool: PgPool,
    application_id: Uuid,
    _organization_id: Uuid,
}

impl TestDb {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/hook0".to_string());

        let pool = PgPool::connect(&database_url).await?;

        // Create test organization and application
        let organization_id = Uuid::new_v4();
        let application_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO iam.organization (organization__id, name, created_at)
             VALUES ($1, 'test-org', statement_timestamp())
             ON CONFLICT (organization__id) DO NOTHING",
        )
        .bind(organization_id)
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO event.application (application__id, name, organization__id, created_at)
             VALUES ($1, 'test-app', $2, statement_timestamp())
             ON CONFLICT (application__id) DO NOTHING",
        )
        .bind(application_id)
        .bind(organization_id)
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            application_id,
            _organization_id: organization_id,
        })
    }

    /// Create a test subscription
    async fn create_subscription(
        &self,
        label_value: &str,
        fifo_mode: bool,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let subscription_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        // Create service, resource_type, and verb for event type
        sqlx::query(
            "INSERT INTO event.service (application__id, service__name)
             VALUES ($1, 'test')
             ON CONFLICT DO NOTHING",
        )
        .bind(self.application_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO event.verb (application__id, verb__name)
             VALUES ($1, 'action')
             ON CONFLICT DO NOTHING",
        )
        .bind(self.application_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO event.resource_type (application__id, service__name, resource_type__name)
             VALUES ($1, 'test', 'resource')
             ON CONFLICT DO NOTHING",
        )
        .bind(self.application_id)
        .execute(&self.pool)
        .await?;

        // Create event type (generated name: test.resource.action)
        sqlx::query(
            "INSERT INTO event.event_type (
                application__id, service__name, resource_type__name, verb__name
             )
             VALUES ($1, 'test', 'resource', 'action')
             ON CONFLICT DO NOTHING",
        )
        .bind(self.application_id)
        .execute(&self.pool)
        .await?;

        // Create subscription with labels (not name) and target__id
        sqlx::query(
            "INSERT INTO webhook.subscription (
                subscription__id, application__id, target__id,
                is_enabled, fifo_mode, labels, created_at
             )
             VALUES ($1, $2, $3, true, $4, $5, statement_timestamp())",
        )
        .bind(subscription_id)
        .bind(self.application_id)
        .bind(target_id)
        .bind(fifo_mode)
        .bind(serde_json::json!({"test": label_value}))
        .execute(&self.pool)
        .await?;

        // Create target (parent table)
        sqlx::query(
            "INSERT INTO webhook.target (target__id)
             VALUES ($1)",
        )
        .bind(target_id)
        .execute(&self.pool)
        .await?;

        // Create target_http (child table with actual webhook config)
        sqlx::query(
            "INSERT INTO webhook.target_http (target__id, method, url, headers)
             VALUES ($1, 'POST', 'http://localhost:8080/webhook', '{}')",
        )
        .bind(target_id)
        .execute(&self.pool)
        .await?;

        // Link subscription to event type
        sqlx::query(
            "INSERT INTO webhook.subscription__event_type (
                subscription__id, application__id, event_type__name
             )
             VALUES ($1, $2, 'test.resource.action')",
        )
        .bind(subscription_id)
        .bind(self.application_id)
        .execute(&self.pool)
        .await?;

        // Initialize FIFO state if FIFO mode enabled
        if fifo_mode {
            sqlx::query(
                "INSERT INTO webhook.fifo_subscription_state (subscription__id, updated_at)
                 VALUES ($1, statement_timestamp())
                 ON CONFLICT (subscription__id) DO NOTHING",
            )
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(subscription_id)
    }

    /// Create a test event
    async fn create_event(
        &self,
        payload: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let event_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO event.event (
                event__id, application__id, event_type__name, payload,
                payload_content_type, ip, occurred_at
             )
             VALUES ($1, $2, 'test.resource.action', $3, 'application/json', '127.0.0.1', statement_timestamp())",
        )
        .bind(event_id)
        .bind(self.application_id)
        .bind(serde_json::to_vec(&serde_json::json!({"message": payload}))?)
        .execute(&self.pool)
        .await?;

        Ok(event_id)
    }

    /// Create request attempt for event and subscription
    async fn create_request_attempt(
        &self,
        event_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let request_attempt_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO webhook.request_attempt (
                request_attempt__id, event__id, subscription__id, created_at
             )
             VALUES ($1, $2, $3, statement_timestamp())",
        )
        .bind(request_attempt_id)
        .bind(event_id)
        .bind(subscription_id)
        .execute(&self.pool)
        .await?;

        Ok(request_attempt_id)
    }

    /// Get FIFO state for subscription
    async fn get_fifo_state(&self, subscription_id: Uuid) -> Result<Option<FifoState>, Box<dyn std::error::Error>> {
        let row = sqlx::query(
            "SELECT current_request_attempt__id, last_completed_event_created_at, updated_at
             FROM webhook.fifo_subscription_state
             WHERE subscription__id = $1",
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| FifoState {
            current_request_attempt_id: r.get(0),
            last_completed_event_created_at: r.get(1),
            _updated_at: r.get(2),
        }))
    }

    /// Check if request attempt is blocked by FIFO
    async fn is_request_blocked(
        &self,
        subscription_id: Uuid,
        request_attempt_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let row = sqlx::query(
            "SELECT
                s.fifo_mode,
                fss.current_request_attempt__id
             FROM webhook.subscription s
             LEFT JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
             WHERE s.subscription__id = $1",
        )
        .bind(subscription_id)
        .fetch_one(&self.pool)
        .await?;

        let fifo_mode: bool = row.get(0);
        let current_attempt: Option<Uuid> = row.get(1);

        if !fifo_mode {
            return Ok(false);
        }

        // Blocked if there's a different request attempt currently in flight
        Ok(current_attempt.is_some() && current_attempt != Some(request_attempt_id))
    }

    /// Simulate picking a request attempt (what worker does)
    async fn simulate_pickup(
        &self,
        request_attempt_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if blocked
        if self.is_request_blocked(subscription_id, request_attempt_id).await? {
            return Ok(false);
        }

        // Update picked_at
        sqlx::query(
            "UPDATE webhook.request_attempt
             SET picked_at = statement_timestamp()
             WHERE request_attempt__id = $1",
        )
        .bind(request_attempt_id)
        .execute(&self.pool)
        .await?;

        // Create or update FIFO state
        sqlx::query(
            "INSERT INTO webhook.fifo_subscription_state (
                subscription__id, current_request_attempt__id, updated_at
             )
             VALUES ($1, $2, statement_timestamp())
             ON CONFLICT (subscription__id)
             DO UPDATE SET
                current_request_attempt__id = $2,
                updated_at = statement_timestamp()",
        )
        .bind(subscription_id)
        .bind(request_attempt_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    /// Simulate successful completion
    async fn simulate_success(
        &self,
        request_attempt_id: Uuid,
        subscription_id: Uuid,
        event_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Mark as succeeded
        sqlx::query(
            "UPDATE webhook.request_attempt
             SET succeeded_at = statement_timestamp()
             WHERE request_attempt__id = $1",
        )
        .bind(request_attempt_id)
        .execute(&self.pool)
        .await?;

        // Clear FIFO state
        sqlx::query(
            "UPDATE webhook.fifo_subscription_state
             SET
                current_request_attempt__id = NULL,
                last_completed_event_created_at = (SELECT e.occurred_at FROM event.event e WHERE e.event__id = $2),
                updated_at = statement_timestamp()
             WHERE subscription__id = $1",
        )
        .bind(subscription_id)
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up test data
    async fn cleanup(&self, subscription_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        // Get target__id first
        let target_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT target__id FROM webhook.subscription WHERE subscription__id = $1"
        )
        .bind(subscription_id)
        .fetch_optional(&self.pool)
        .await?;

        // Delete in correct order due to foreign keys
        sqlx::query("DELETE FROM webhook.request_attempt WHERE subscription__id = $1")
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM webhook.fifo_subscription_state WHERE subscription__id = $1")
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM webhook.subscription__event_type WHERE subscription__id = $1")
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM webhook.subscription WHERE subscription__id = $1")
            .bind(subscription_id)
            .execute(&self.pool)
            .await?;

        // Clean up target tables
        if let Some(tid) = target_id {
            sqlx::query("DELETE FROM webhook.target_http WHERE target__id = $1")
                .bind(tid)
                .execute(&self.pool)
                .await?;

            sqlx::query("DELETE FROM webhook.target WHERE target__id = $1")
                .bind(tid)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct FifoState {
    current_request_attempt_id: Option<Uuid>,
    last_completed_event_created_at: Option<chrono::DateTime<chrono::Utc>>,
    _updated_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::test]
#[ignore] // Run with: cargo test --test fifo_integration -- --ignored
async fn test_basic_fifo_ordering() {
    let db = TestDb::new().await.expect("Failed to connect to database");
    let subscription_id = db
        .create_subscription("test-fifo-basic", true)
        .await
        .expect("Failed to create subscription");

    // Create 3 events in order
    let event1 = db.create_event("event-1").await.unwrap();
    let event2 = db.create_event("event-2").await.unwrap();
    let _event3 = db.create_event("event-3").await.unwrap();

    // Create request attempts
    let ra1 = db.create_request_attempt(event1, subscription_id).await.unwrap();
    let ra2 = db.create_request_attempt(event2, subscription_id).await.unwrap();
    let _ra3 = db.create_request_attempt(_event3, subscription_id).await.unwrap();

    // Verify initial state - no blocking
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert!(state.current_request_attempt_id.is_none(), "Initially no request in flight");

    // Pick first request - should succeed
    assert!(db.simulate_pickup(ra1, subscription_id).await.unwrap(), "First pickup should succeed");

    // Try to pick second request - should be blocked
    assert!(!db.simulate_pickup(ra2, subscription_id).await.unwrap(), "Second pickup should be blocked");

    // Verify FIFO state shows ra1 in flight
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert_eq!(state.current_request_attempt_id, Some(ra1), "FIFO state should show ra1 in flight");

    // Complete first request
    db.simulate_success(ra1, subscription_id, event1).await.unwrap();

    // Verify FIFO state cleared
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert!(state.current_request_attempt_id.is_none(), "FIFO state should be cleared after success");

    // Now second request should be pickable
    assert!(db.simulate_pickup(ra2, subscription_id).await.unwrap(), "Second pickup should succeed after first completes");

    // Cleanup
    db.cleanup(subscription_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_fifo_vs_non_fifo_independence() {
    let db = TestDb::new().await.expect("Failed to connect to database");

    // Create FIFO and non-FIFO subscriptions
    let fifo_sub = db.create_subscription("test-fifo", true).await.unwrap();
    let normal_sub = db.create_subscription("test-normal", false).await.unwrap();

    // Create event
    let event = db.create_event("test-event").await.unwrap();

    // Create request attempts for both
    let fifo_ra1 = db.create_request_attempt(event, fifo_sub).await.unwrap();
    let fifo_ra2 = db.create_request_attempt(event, fifo_sub).await.unwrap();
    let normal_ra1 = db.create_request_attempt(event, normal_sub).await.unwrap();
    let normal_ra2 = db.create_request_attempt(event, normal_sub).await.unwrap();

    // Pick FIFO request
    assert!(db.simulate_pickup(fifo_ra1, fifo_sub).await.unwrap());

    // FIFO should block second request
    assert!(!db.simulate_pickup(fifo_ra2, fifo_sub).await.unwrap(), "FIFO should block second request");

    // Non-FIFO should allow both
    assert!(db.simulate_pickup(normal_ra1, normal_sub).await.unwrap(), "Non-FIFO should allow first request");
    assert!(db.simulate_pickup(normal_ra2, normal_sub).await.unwrap(), "Non-FIFO should allow second request concurrently");

    // Cleanup
    db.cleanup(fifo_sub).await.unwrap();
    db.cleanup(normal_sub).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_fifo_state_tracking() {
    let db = TestDb::new().await.expect("Failed to connect to database");
    let subscription_id = db.create_subscription("test-fifo-state", true).await.unwrap();

    let event = db.create_event("test-event").await.unwrap();
    let ra = db.create_request_attempt(event, subscription_id).await.unwrap();

    // Check initial state
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert!(state.current_request_attempt_id.is_none());
    assert!(state.last_completed_event_created_at.is_none());

    // Pickup
    db.simulate_pickup(ra, subscription_id).await.unwrap();

    // Check state updated
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert_eq!(state.current_request_attempt_id, Some(ra));

    // Complete
    db.simulate_success(ra, subscription_id, event).await.unwrap();

    // Check state cleared but timestamp set
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert!(state.current_request_attempt_id.is_none());
    assert!(state.last_completed_event_created_at.is_some(), "Completion timestamp should be set");

    // Cleanup
    db.cleanup(subscription_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_orphaned_fifo_state_detection() {
    let db = TestDb::new().await.expect("Failed to connect to database");
    let subscription_id = db.create_subscription("test-orphan", true).await.unwrap();

    let event = db.create_event("test-event").await.unwrap();
    let ra = db.create_request_attempt(event, subscription_id).await.unwrap();

    // Simulate pickup
    db.simulate_pickup(ra, subscription_id).await.unwrap();

    // Simulate time passing (in real scenario, worker crashed)
    // Update picked_at to be old
    sqlx::query(
        "UPDATE webhook.request_attempt
         SET picked_at = statement_timestamp() - INTERVAL '10 minutes'
         WHERE request_attempt__id = $1",
    )
    .bind(ra)
    .execute(&db.pool)
    .await
    .unwrap();

    // Query for orphaned states (from monitoring queries)
    let orphaned = sqlx::query(
        "SELECT fss.subscription__id, fss.current_request_attempt__id
         FROM webhook.fifo_subscription_state fss
         INNER JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
         WHERE ra.picked_at IS NOT NULL
           AND ra.succeeded_at IS NULL
           AND ra.failed_at IS NULL
           AND ra.picked_at < statement_timestamp() - INTERVAL '5 minutes'",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();

    assert_eq!(orphaned.len(), 1, "Should detect 1 orphaned FIFO state");

    // Cleanup orphaned state (from monitoring queries)
    sqlx::query(
        "UPDATE webhook.fifo_subscription_state
         SET current_request_attempt__id = NULL, updated_at = statement_timestamp()
         WHERE subscription__id = $1",
    )
    .bind(subscription_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // Verify cleared
    let state = db.get_fifo_state(subscription_id).await.unwrap().unwrap();
    assert!(state.current_request_attempt_id.is_none(), "Orphaned state should be cleared");

    // Cleanup
    db.cleanup(subscription_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_fifo_with_retries() {
    let db = TestDb::new().await.expect("Failed to connect to database");
    let subscription_id = db.create_subscription("test-fifo-retry", true).await.unwrap();

    // Create 2 events
    let event1 = db.create_event("event-1").await.unwrap();
    let event2 = db.create_event("event-2").await.unwrap();

    // Create request attempts
    let ra1 = db.create_request_attempt(event1, subscription_id).await.unwrap();
    let ra2 = db.create_request_attempt(event2, subscription_id).await.unwrap();

    // Pick first request
    assert!(db.simulate_pickup(ra1, subscription_id).await.unwrap());

    // Verify second is blocked
    assert!(!db.simulate_pickup(ra2, subscription_id).await.unwrap(), "Second should be blocked");

    // Simulate failure - mark as failed (not success)
    sqlx::query(
        "UPDATE webhook.request_attempt
         SET failed_at = statement_timestamp()
         WHERE request_attempt__id = $1",
    )
    .bind(ra1)
    .execute(&db.pool)
    .await
    .unwrap();

    // Create retry attempt (what worker would do)
    let ra1_retry = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO webhook.request_attempt (
            request_attempt__id, event__id, subscription__id, retry_count, delay_until
         )
         VALUES ($1, $2, $3, 1, statement_timestamp() + INTERVAL '5 seconds')",
    )
    .bind(ra1_retry)
    .bind(event1)
    .bind(subscription_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // Update FIFO state to point to retry
    sqlx::query(
        "UPDATE webhook.fifo_subscription_state
         SET current_request_attempt__id = $1, updated_at = statement_timestamp()
         WHERE subscription__id = $2",
    )
    .bind(ra1_retry)
    .bind(subscription_id)
    .execute(&db.pool)
    .await
    .unwrap();

    // Verify ra2 still blocked (waiting for retry)
    assert!(!db.simulate_pickup(ra2, subscription_id).await.unwrap(), "Second should still be blocked during retry");

    // Simulate retry success
    db.simulate_success(ra1_retry, subscription_id, event1).await.unwrap();

    // Now ra2 should be pickable
    assert!(db.simulate_pickup(ra2, subscription_id).await.unwrap(), "Second should be pickable after retry succeeds");

    // Cleanup
    db.cleanup(subscription_id).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_fifo_performance_comparison() {
    let db = TestDb::new().await.expect("Failed to connect to database");

    // Create FIFO and non-FIFO subscriptions
    let fifo_sub = db.create_subscription("perf-fifo", true).await.unwrap();
    let normal_sub = db.create_subscription("perf-normal", false).await.unwrap();

    // Create same number of events for both
    let mut fifo_events = Vec::new();
    let mut normal_events = Vec::new();
    for i in 0..10 {
        let event = db.create_event(&format!("event-{}", i)).await.unwrap();
        fifo_events.push(event);
        normal_events.push(event);
    }

    // Create request attempts
    let mut fifo_attempts = Vec::new();
    let mut normal_attempts = Vec::new();
    for event_id in &fifo_events {
        let ra = db.create_request_attempt(*event_id, fifo_sub).await.unwrap();
        fifo_attempts.push(ra);
    }
    for event_id in &normal_events {
        let ra = db.create_request_attempt(*event_id, normal_sub).await.unwrap();
        normal_attempts.push(ra);
    }

    // Test FIFO: Only first can be picked
    let mut fifo_pickable = Vec::new();
    for &ra in &fifo_attempts {
        if db.simulate_pickup(ra, fifo_sub).await.unwrap() {
            fifo_pickable.push(ra);
        }
    }
    assert_eq!(fifo_pickable.len(), 1, "FIFO should allow only 1 concurrent pickup");

    // Test normal: All should be pickable concurrently
    let mut normal_pickable = Vec::new();
    for &ra in &normal_attempts {
        if db.simulate_pickup(ra, normal_sub).await.unwrap() {
            normal_pickable.push(ra);
        }
    }
    assert_eq!(normal_pickable.len(), 10, "Normal mode should allow all concurrent pickups");

    // Verify throughput difference
    let fifo_throughput = fifo_pickable.len() as f64;
    let normal_throughput = normal_pickable.len() as f64;
    let throughput_ratio = fifo_throughput / normal_throughput;

    assert!(throughput_ratio < 0.2, "FIFO throughput should be <20% of normal mode (was {:.1}%)", throughput_ratio * 100.0);

    // Cleanup
    db.cleanup(fifo_sub).await.unwrap();
    db.cleanup(normal_sub).await.unwrap();
}
