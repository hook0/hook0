# Feature Specification: FIFO Mode for Subscriptions

## Executive Summary

This specification defines how to implement an optional FIFO (First In First Out) delivery mode for webhooks at the subscription level. When enabled, this feature guarantees that webhooks for a given subscription are delivered in strict event order, even in the presence of retries and concurrent workers.

## Problem Statement

Currently, Hook0 sends webhooks in the same order as events are received, but there are two scenarios where ordering can be lost:

1. **Retry Scheduling**: When a webhook fails and needs to be retried, it's scheduled for later while the system continues sending subsequent webhooks from other events
2. **Concurrent Processing**: Multiple workers process request attempts concurrently, which can cause webhooks from closely-received events to arrive out of order

### Customer Use Cases

Customers requiring FIFO mode typically have:
- **State Machine Systems**: Sequential state transitions that must be applied in order
- **Financial Operations**: Transaction sequences where order affects account balances
- **Workflow Orchestration**: Multi-step processes where each step depends on the previous one
- **Audit Trails**: Systems requiring chronological event processing for compliance

## Current Architecture Analysis

### Event Flow
```
Event Received → Event Dispatch (trigger) → Request Attempts Created → Workers Pick & Execute
```

### Key Components

1. **Event Dispatch Trigger** (`event.dispatch()`)
   - Creates `request_attempt` records for matching subscriptions
   - Runs synchronously when events are inserted
   - Orders by event creation time

2. **Worker Types**
   - **PG Workers**: Poll PostgreSQL using `ORDER BY ra.created_at ASC` with `SKIP LOCKED`
   - **Pulsar Workers**: Use Pulsar's message queue with shared subscription type

3. **Retry Mechanism**
   - Fast retries: 5s to 5min exponential backoff (30 attempts default)
   - Slow retries: 1h intervals (30 attempts default)
   - New `request_attempt` created with `delay_until` timestamp

4. **Concurrency**
   - PG workers: Configurable concurrent units per worker
   - Pulsar workers: Semaphore-controlled concurrency
   - Multiple workers can process same subscription simultaneously

## Proposed Solution

### 1. Database Schema Changes

#### Add FIFO field to subscription table

```sql
-- Migration: add_fifo_mode_to_subscription.up.sql
ALTER TABLE webhook.subscription
ADD COLUMN fifo_mode BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX idx_subscription_fifo_mode
ON webhook.subscription(fifo_mode)
WHERE fifo_mode = true;

COMMENT ON COLUMN webhook.subscription.fifo_mode IS
'When true, webhooks for this subscription are delivered in strict event order.
The next webhook is only sent after the current one succeeds or exhausts all retries.';
```

#### Add tracking for in-flight FIFO requests

```sql
-- Track the current "blocking" request attempt for FIFO subscriptions
CREATE TABLE webhook.fifo_subscription_state (
    subscription__id UUID NOT NULL PRIMARY KEY REFERENCES webhook.subscription(subscription__id) ON DELETE CASCADE,
    current_request_attempt__id UUID REFERENCES webhook.request_attempt(request_attempt__id) ON DELETE SET NULL,
    last_completed_event_created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),

    CONSTRAINT fk_fifo_subscription
        FOREIGN KEY (subscription__id)
        REFERENCES webhook.subscription(subscription__id)
        ON DELETE CASCADE
);

CREATE INDEX idx_fifo_state_current_attempt
ON webhook.fifo_subscription_state(current_request_attempt__id)
WHERE current_request_attempt__id IS NOT NULL;

COMMENT ON TABLE webhook.fifo_subscription_state IS
'Tracks the current in-flight request attempt for FIFO subscriptions to enforce ordering';
```

### 2. API Changes

#### Update Subscription Creation/Update Endpoints

**SubscriptionPost struct** (`api/src/handlers/subscriptions.rs`):
```rust
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct SubscriptionPost {
    application_id: Uuid,
    is_enabled: bool,
    event_types: Vec<String>,
    description: Option<String>,
    metadata: Option<HashMap<String, String>>,
    labels: Option<HashMap<String, String>>,
    target: Target,
    dedicated_workers: Option<Vec<String>>,

    /// When true, webhooks are delivered in strict event order.
    /// The next webhook is only sent after the current one succeeds or exhausts all retries.
    /// This may significantly reduce throughput for this subscription.
    #[serde(default)]
    fifo_mode: bool,
}
```

**Subscription struct** (response):
```rust
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Subscription {
    pub application_id: Uuid,
    pub subscription_id: Uuid,
    pub is_enabled: bool,
    pub event_types: Vec<String>,
    pub description: Option<String>,
    pub secret: Uuid,
    pub metadata: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub target: Target,
    pub created_at: DateTime<Utc>,
    pub dedicated_workers: Vec<String>,

    /// FIFO delivery mode flag
    pub fifo_mode: bool,
}
```

#### Database Operations Updates

**Create subscription** (`api/src/handlers/subscriptions.rs:create`):
```rust
let subscription = query_as!(
    RawSubscription,
    "
        INSERT INTO webhook.subscription (
            subscription__id, application__id, is_enabled, description,
            secret, metadata, labels, target__id, fifo_mode, created_at
        )
        VALUES (
            public.gen_random_uuid(), $1, $2, $3,
            public.gen_random_uuid(), $4, $5, public.gen_random_uuid(), $6, statement_timestamp()
        )
        RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, fifo_mode, created_at
    ",
    &body.application_id,
    &body.is_enabled,
    body.description,
    metadata,
    labels,
    &body.fifo_mode,
)
.fetch_one(&mut *tx)
.await?;

// Initialize FIFO state if needed
if body.fifo_mode {
    query!(
        "
            INSERT INTO webhook.fifo_subscription_state (subscription__id)
            VALUES ($1)
        ",
        &subscription.subscription__id,
    )
    .execute(&mut *tx)
    .await?;
}
```

**Update subscription** (`api/src/handlers/subscriptions.rs:edit`):
```rust
// Note: Changing fifo_mode from false->true is allowed
// Changing from true->false requires no in-flight requests
let subscription = query_as!(
    RawSubscription,
    "
        UPDATE webhook.subscription
        SET is_enabled = $1, description = $2, metadata = $3, labels = $4, fifo_mode = $5
        WHERE subscription__id = $6 AND application__id = $7 AND deleted_at IS NULL
        RETURNING subscription__id, is_enabled, description, secret, metadata, labels, target__id, fifo_mode, created_at
    ",
    &body.is_enabled,
    body.description,
    metadata,
    labels,
    &body.fifo_mode,
    &subscription_id,
    &body.application_id
)
.fetch_optional(&mut *tx)
.await?;

// Handle FIFO mode changes
if body.fifo_mode {
    query!(
        "
            INSERT INTO webhook.fifo_subscription_state (subscription__id)
            VALUES ($1)
            ON CONFLICT (subscription__id) DO NOTHING
        ",
        &subscription.subscription__id,
    )
    .execute(&mut *tx)
    .await?;
}
```

### 3. Event Dispatch Logic

The dispatch trigger remains unchanged - it creates request attempts in event order as before. The FIFO enforcement happens during worker pickup.

### 4. Worker Changes

#### PG Worker (`output-worker/src/pg.rs`)

**Query Modification for FIFO Subscriptions**:

```rust
// New query that respects FIFO ordering
let next_attempt = match worker.scope {
    WorkerScope::Public { worker_id } => {
        query_as!(
            RequestAttempt,
            "
                SELECT
                    ra.request_attempt__id AS request_attempt_id,
                    ra.event__id AS event_id,
                    ra.subscription__id AS subscription_id,
                    ra.created_at,
                    ra.retry_count,
                    t_http.method AS http_method,
                    t_http.url AS http_url,
                    t_http.headers AS http_headers,
                    e.event_type__name AS event_type_name,
                    e.payload AS payload,
                    e.payload_content_type AS payload_content_type,
                    s.secret,
                    s.fifo_mode
                FROM webhook.request_attempt AS ra
                INNER JOIN webhook.subscription AS s ON s.subscription__id = ra.subscription__id
                LEFT JOIN webhook.subscription__worker AS sw ON sw.subscription__id = s.subscription__id
                INNER JOIN event.application AS a ON a.application__id = s.application__id AND a.deleted_at IS NULL
                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                LEFT JOIN iam.organization__worker AS ow ON ow.organization__id = o.organization__id AND ow.default = true
                INNER JOIN webhook.target_http AS t_http ON t_http.target__id = s.target__id
                INNER JOIN event.event AS e ON e.event__id = ra.event__id
                LEFT JOIN webhook.fifo_subscription_state AS fss ON fss.subscription__id = s.subscription__id
                WHERE
                    ra.succeeded_at IS NULL
                    AND ra.failed_at IS NULL
                    AND (ra.delay_until IS NULL OR ra.delay_until <= statement_timestamp())
                    AND (COALESCE(sw.worker__id, ow.worker__id) IS NULL OR COALESCE(sw.worker__id, ow.worker__id) = $1)
                    -- FIFO constraint: only pick if no other request is in flight for this subscription
                    AND (
                        s.fifo_mode = false
                        OR fss.current_request_attempt__id IS NULL
                        OR fss.current_request_attempt__id = ra.request_attempt__id
                    )
                ORDER BY
                    s.subscription__id ASC,  -- Group by subscription for better FIFO performance
                    ra.created_at ASC
                LIMIT 1
                FOR UPDATE OF ra
                SKIP LOCKED
            ",
            worker_id.to_owned(),
        )
        .fetch_optional(&mut *tx)
        .await?
    }
    // Similar for Private scope...
};

// After picking, update FIFO state if needed
if let Some(attempt) = &next_attempt {
    query!(
        "
            UPDATE webhook.request_attempt
            SET picked_at = statement_timestamp(), worker_name = $1, worker_version = $2
            WHERE request_attempt__id = $3
        ",
        &worker.name,
        &worker_version,
        attempt.request_attempt_id
    )
    .execute(&mut *tx)
    .await?;

    // Mark this attempt as current for FIFO tracking
    query!(
        "
            UPDATE webhook.fifo_subscription_state
            SET current_request_attempt__id = $1, updated_at = statement_timestamp()
            WHERE subscription__id = $2
        ",
        &attempt.request_attempt_id,
        &attempt.subscription_id,
    )
    .execute(&mut *tx)
    .await?;
}
```

**Completion Handling**:

```rust
if response.is_success() {
    // Mark attempt as completed
    query!(
        "UPDATE webhook.request_attempt SET succeeded_at = statement_timestamp() WHERE request_attempt__id = $1",
        attempt.request_attempt_id
    )
    .execute(&mut *tx)
    .await?;

    // Clear FIFO state to allow next webhook
    query!(
        "
            UPDATE webhook.fifo_subscription_state
            SET
                current_request_attempt__id = NULL,
                last_completed_event_created_at = (SELECT e.occurred_at FROM event.event e WHERE e.event__id = $2),
                updated_at = statement_timestamp()
            WHERE subscription__id = $1
        ",
        &attempt.subscription_id,
        &attempt.event_id,
    )
    .execute(&mut *tx)
    .await?;
} else {
    // Mark attempt as failed
    query!(
        "UPDATE webhook.request_attempt SET failed_at = statement_timestamp() WHERE request_attempt__id = $1",
        attempt.request_attempt_id
    )
    .execute(&mut *tx)
    .await?;

    // Creating a retry request or giving up
    if let Some(retry_in) = compute_next_retry(...).await? {
        let retry_id = query!(
            "
                INSERT INTO webhook.request_attempt (event__id, subscription__id, delay_until, retry_count)
                VALUES ($1, $2, statement_timestamp() + $3, $4)
                RETURNING request_attempt__id
            ",
            attempt.event_id,
            attempt.subscription_id,
            PgInterval::try_from(retry_in).unwrap(),
            next_retry_count,
        )
        .fetch_one(&mut *tx)
        .await?
        .request_attempt__id;

        // For FIFO: update to point to the retry attempt
        query!(
            "
                UPDATE webhook.fifo_subscription_state
                SET current_request_attempt__id = $1, updated_at = statement_timestamp()
                WHERE subscription__id = $2
            ",
            &retry_id,
            &attempt.subscription_id,
        )
        .execute(&mut *tx)
        .await?;
    } else {
        // No more retries - clear FIFO state
        query!(
            "
                UPDATE webhook.fifo_subscription_state
                SET
                    current_request_attempt__id = NULL,
                    last_completed_event_created_at = (SELECT e.occurred_at FROM event.event e WHERE e.event__id = $2),
                    updated_at = statement_timestamp()
                WHERE subscription__id = $1
            ",
            &attempt.subscription_id,
            &attempt.event_id,
        )
        .execute(&mut *tx)
        .await?;
    }
}
```

#### Pulsar Worker (`output-worker/src/pulsar.rs`)

Similar changes needed for the Pulsar worker:

1. **Topic Structure**: Consider using per-subscription topics for FIFO subscriptions (alternative approach)
2. **State Tracking**: Same FIFO state table logic
3. **Retry Handling**: Ensure retries block subsequent messages for same subscription

**Alternative Pulsar Approach**: Use Pulsar's built-in ordering guarantees by creating exclusive subscriptions per FIFO-enabled subscription. This is more complex but leverages Pulsar's native capabilities.

### 5. Configuration & Validation

#### Retry Policy Considerations

FIFO mode works with existing retry configuration but customers should understand:

- **Throughput Impact**: One failing webhook blocks all subsequent webhooks for that subscription
- **Latency Impact**: Total delivery time = sum of all retry delays
- **Recommended Settings for FIFO**:
  - Lower `max_fast_retries` (e.g., 5 instead of 30)
  - Shorter `max_slow_retries` (e.g., 10 instead of 30)
  - Document that customers should implement idempotency even with FIFO

#### Validation Rules

1. Cannot enable FIFO on subscriptions with `dedicated_workers` spanning multiple workers (potential future enhancement)
2. Warn customers about throughput implications in API documentation
3. Add metrics to track FIFO queue depth per subscription

### 6. Observability & Monitoring

#### New Metrics

```rust
// Prometheus metrics to add
fifo_subscription_queue_depth{subscription_id}: Gauge
    // Number of pending request attempts for FIFO subscriptions

fifo_subscription_blocked_duration_seconds{subscription_id}: Histogram
    // How long webhooks wait due to FIFO blocking

fifo_subscription_current_retry_count{subscription_id}: Gauge
    // Current retry count of blocking request
```

#### Logging

```rust
info!("[FIFO] Subscription {subscription_id} blocked - waiting for request attempt {current_attempt} to complete");
warn!("[FIFO] Subscription {subscription_id} blocked for {duration}s - current attempt on retry {retry_count}/{max_retries}");
```

#### Database Queries for Monitoring

```sql
-- FIFO subscriptions with long-running blocks
SELECT
    s.subscription__id,
    s.application__id,
    fss.current_request_attempt__id,
    ra.retry_count,
    ra.created_at,
    ra.picked_at,
    EXTRACT(EPOCH FROM (NOW() - ra.created_at)) as age_seconds,
    COUNT(*) FILTER (WHERE ra2.succeeded_at IS NULL AND ra2.failed_at IS NULL) as pending_count
FROM webhook.subscription s
INNER JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
LEFT JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
LEFT JOIN webhook.request_attempt ra2 ON ra2.subscription__id = s.subscription__id
WHERE s.fifo_mode = true
    AND fss.current_request_attempt__id IS NOT NULL
GROUP BY s.subscription__id, s.application__id, fss.current_request_attempt__id, ra.retry_count, ra.created_at, ra.picked_at
HAVING EXTRACT(EPOCH FROM (NOW() - ra.created_at)) > 300  -- blocked for > 5 minutes
ORDER BY age_seconds DESC;
```

## Implementation Plan

### Phase 1: Database & API Foundation
- [ ] Migration: Add `fifo_mode` column to `subscription` table
- [ ] Migration: Create `fifo_subscription_state` table
- [ ] Update API models: `SubscriptionPost` and `Subscription` structs
- [ ] Update subscription CREATE endpoint
- [ ] Update subscription UPDATE endpoint
- [ ] Update subscription GET/LIST endpoints
- [ ] Add API documentation with FIFO warnings

### Phase 2: PG Worker Implementation
- [ ] Update `RequestAttempt` struct to include `fifo_mode`
- [ ] Modify PG worker query to respect FIFO constraints
- [ ] Update request pickup logic to set FIFO state
- [ ] Update success handler to clear FIFO state
- [ ] Update failure handler to manage FIFO state during retries
- [ ] Add FIFO-specific logging

### Phase 3: Pulsar Worker Implementation
- [ ] Evaluate Pulsar-native FIFO approach vs. state-table approach
- [ ] Implement chosen approach for Pulsar workers
- [ ] Update retry logic for FIFO subscriptions in Pulsar
- [ ] Ensure consistency between PG and Pulsar behavior

### Phase 4: Observability
- [ ] Add Prometheus metrics for FIFO monitoring
- [ ] Create Grafana dashboard for FIFO subscriptions
- [ ] Add alerting for stuck FIFO subscriptions
- [ ] Document monitoring queries

### Phase 5: Testing & Documentation
- [ ] Unit tests for FIFO constraint logic
- [ ] Integration tests: FIFO ordering guarantee
- [ ] Integration tests: FIFO with retries
- [ ] Integration tests: FIFO with concurrent workers
- [ ] Load tests: throughput impact
- [ ] API documentation updates
- [ ] User guide: when to use FIFO mode
- [ ] Migration guide for existing subscriptions

### Phase 6: Deployment & Monitoring
- [ ] Deploy to staging environment
- [ ] Monitor FIFO behavior in staging
- [ ] Gradual rollout to production
- [ ] Monitor performance impact on non-FIFO subscriptions

## Performance Considerations

### Impact on Non-FIFO Subscriptions
- Minimal: Query adds a LEFT JOIN and WHERE condition that's only evaluated for FIFO subscriptions
- Index on `fifo_mode` (partial WHERE clause) ensures non-FIFO subscriptions are unaffected

### FIFO Subscription Throughput
- **Worst Case**: Single-threaded processing per subscription
- **Best Case**: Immediate success = event rate × (1 / (network_latency + processing_time))
- **Realistic**: Factor in retries and delays

### Database Load
- Additional table `fifo_subscription_state` is small (one row per FIFO subscription)
- Updates to this table happen in same transaction as request attempt updates (no additional roundtrips)
- Indexes ensure queries remain efficient

## Edge Cases & Error Handling

### Worker Crashes
If a worker crashes while processing a FIFO request:
- `current_request_attempt__id` remains set in `fifo_subscription_state`
- `picked_at` is set but neither `succeeded_at` nor `failed_at` is set
- **Solution**: Background job to detect and reset orphaned FIFO states:

```sql
-- Detect orphaned FIFO states (picked but not completed/failed for > 5 minutes)
UPDATE webhook.fifo_subscription_state fss
SET current_request_attempt__id = NULL, updated_at = statement_timestamp()
FROM webhook.request_attempt ra
WHERE fss.current_request_attempt__id = ra.request_attempt__id
    AND ra.picked_at IS NOT NULL
    AND ra.succeeded_at IS NULL
    AND ra.failed_at IS NULL
    AND ra.picked_at < NOW() - INTERVAL '5 minutes';
```

### Subscription Disable/Delete
- Subscription disable: No new request attempts created, existing ones complete
- Subscription delete: CASCADE deletes FIFO state automatically
- FIFO state table has `ON DELETE CASCADE` constraint

### Changing FIFO Mode
- **false → true**: Safe, starts enforcing FIFO from that point
- **true → false**: Safe, but warn if there are pending requests

### Multiple Events, Same Timestamp
- Events are ordered by `created_at`, which uses `statement_timestamp()`
- PostgreSQL's timestamp has microsecond precision
- If timestamps are identical, order is implementation-dependent but stable within a transaction

## Future Enhancements

1. **Per-Event-Type FIFO**: FIFO within each event type, concurrent across types
2. **FIFO Groups**: Multiple subscriptions sharing FIFO constraint
3. **Configurable Retry Policy per Subscription**: Allow FIFO subscriptions to have different retry policies
4. **Dead Letter Queue**: Separate queue for permanently failed FIFO webhooks
5. **Manual FIFO Reset**: API endpoint to manually release FIFO block (with audit log)

## API Documentation Example

```markdown
### FIFO Mode

When `fifo_mode` is set to `true` on a subscription, Hook0 guarantees that webhooks are delivered in the exact order events were received. This means:

✅ **Guarantees:**
- Webhooks arrive in event creation order
- Next webhook only sent after current one succeeds or exhausts retries
- Retries don't cause out-of-order delivery

⚠️ **Important Considerations:**
- **Reduced Throughput**: Only one webhook in-flight per subscription at a time
- **Increased Latency**: Failures block subsequent webhooks during retry delays
- **Not Recommended For**: High-volume subscriptions or unreliable webhook endpoints
- **Recommended For**: Systems requiring strict ordering (state machines, financial transactions, workflow orchestration)

**Best Practices:**
- Implement idempotency even with FIFO enabled
- Use lower retry counts for FIFO subscriptions
- Monitor FIFO queue depth metrics
- Consider separate subscriptions for order-sensitive vs. order-independent events
```

## Testing Strategy

### Unit Tests
- FIFO state creation on subscription creation
- FIFO state update on request attempt pickup
- FIFO state clearing on success/failure/exhaustion
- Query filtering logic for FIFO vs. non-FIFO

### Integration Tests
1. **Basic FIFO Ordering**
   - Create FIFO subscription
   - Send 10 events rapidly
   - Verify webhooks arrive in order

2. **FIFO with Retries**
   - Create FIFO subscription
   - Send event A (will fail and retry)
   - Send event B immediately after
   - Verify B waits for A to complete all retries

3. **FIFO with Multiple Subscriptions**
   - Create FIFO subscription S1
   - Create regular subscription S2
   - Send events
   - Verify S1 processes sequentially, S2 processes concurrently

4. **Concurrent Workers**
   - Multiple workers running
   - FIFO subscription receives events
   - Verify only one worker processes at a time per subscription

### Load Tests
- Measure throughput impact of FIFO on single subscription
- Measure impact of FIFO subscriptions on overall system throughput
- Verify non-FIFO subscriptions maintain performance

## Open Questions

1. **Should FIFO be compatible with dedicated workers?**
   - Current spec: No restriction
   - Consideration: Dedicated workers + FIFO = very specific use case

2. **Should we support changing FIFO mode on subscriptions with pending requests?**
   - Current spec: Allowed but warn
   - Alternative: Block until pending requests clear

3. **Should FIFO subscriptions use separate Pulsar topics?**
   - Current spec: Same topic, state table enforcement
   - Alternative: Exclusive subscription per FIFO subscription (more complex)

4. **Should retry policy be configurable per subscription?**
   - Current spec: Use global retry policy
   - Enhancement: Per-subscription retry configuration

## Success Metrics

- FIFO subscriptions maintain 100% ordering guarantee
- Non-FIFO subscription performance unchanged (<5% impact)
- FIFO subscription throughput meets documented expectations
- No orphaned FIFO states after worker crashes
- Clear monitoring and alerting for FIFO issues

## References

- Original GitHub issue context (provided by user)
- Hook0 codebase architecture analysis
- PostgreSQL row-level locking documentation
- Apache Pulsar ordering guarantees documentation
