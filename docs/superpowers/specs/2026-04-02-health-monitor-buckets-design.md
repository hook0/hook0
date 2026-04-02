# Health Monitor Bucketed Evaluation — Design Spec

## Goal

Replace the all-time cumulative counters (`subscription_health`) with time-bucketed aggregation driven entirely by the health monitor. The output-worker returns to zero extra writes. Health evaluation detects suspects from bucket data, computes failure rates over a sliding window, and feeds the existing state machine.

## Architecture

Three components:

1. **Delta ingestion** — the health monitor reads new completed `request_attempt` rows since a watermark (via expression index on `COALESCE(succeeded_at, failed_at)`), aggregates them into open buckets.
2. **Suspect detection** — from the bucket data itself (SUM of failed_count), identify subscriptions with enough failures to warrant evaluation. Also includes subscriptions currently in `warning` state for recovery detection.
3. **Evaluation** — compute failure rate for suspects, feed the existing state machine (warning → disabled → resolved). Reset `failure_percent` for non-suspects.

### Write path (output-worker)

Zero change. The output-worker writes to `request_attempt` as before. No UPSERT, no SAVEPOINT, no extra table.

### Read path (health monitor, periodic)

1. Read new completed `request_attempt` rows since watermark (expression index)
2. Upsert aggregates into open buckets (atomic CTE)
3. Close buckets that hit the duration or message count threshold
4. Detect suspects from bucket data + subscriptions currently in `warning` state
5. Compute failure rate for suspects from buckets in the sliding window
6. Run existing state machine (warning/disable/recover) + sync `subscription.failure_percent` for frontend + reset `failure_percent` to NULL for non-suspects
7. Advance watermark to `MAX(COALESCE(succeeded_at, failed_at))` of processed rows
8. Cleanup old buckets (daily)

All steps 1-7 run within the advisory-locked transaction. Step 8 runs in a separate daily cycle (existing cleanup pattern).

## Data Model

### New: `subscription_health_bucket`

```sql
CREATE TABLE webhook.subscription_health_bucket (
    subscription__id UUID NOT NULL
        REFERENCES webhook.subscription(subscription__id) ON DELETE CASCADE,
    bucket_start     TIMESTAMPTZ NOT NULL,
    bucket_end       TIMESTAMPTZ,  -- NULL = open bucket
    total_count      INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
    failed_count     INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0 AND failed_count <= total_count),
    PRIMARY KEY (subscription__id, bucket_start)
);

-- For cleanup queries (DELETE WHERE bucket_start < threshold)
CREATE INDEX idx_subscription_health_bucket_start
    ON webhook.subscription_health_bucket(bucket_start);

-- For finding the open bucket per subscription (Step 2) and closing buckets (Step 3)
CREATE INDEX idx_subscription_health_bucket_open
    ON webhook.subscription_health_bucket(subscription__id)
    WHERE bucket_end IS NULL;
```

A bucket is "open" when `bucket_end IS NULL`. The health monitor closes it (sets `bucket_end`) when either threshold is reached. Each subscription has at most one open bucket.

Buckets are kept at re-enable (historical data). The `failure_percent` on the subscription is reset to NULL, and new buckets post-re-enable will be healthy — the subscription won't appear as a suspect.

### New: expression index for delta scan

```sql
CREATE INDEX CONCURRENTLY idx_request_attempt_completed_at
    ON webhook.request_attempt (COALESCE(succeeded_at, failed_at))
    WHERE succeeded_at IS NOT NULL OR failed_at IS NOT NULL;
```

Partial expression index — only contains completed rows. Not updated at INSERT (both columns NULL), only at completion (UPDATE). Supports the watermark-based delta scan in Step 1. Prevents losing in-flight rows (unlike a `created_at`-based watermark).

Must be created with `CREATE INDEX CONCURRENTLY` (non-transactional) in a separate migration file to avoid locking the high-write `request_attempt` table. After migration, verify index is not INVALID: `SELECT * FROM pg_indexes WHERE indexname = 'idx_request_attempt_completed_at' AND indexdef LIKE '%INVALID%'`.

### New: `health_monitor_watermark`

```sql
CREATE TABLE webhook.health_monitor_watermark (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_processed_at TIMESTAMPTZ NOT NULL DEFAULT '-infinity'
);
INSERT INTO webhook.health_monitor_watermark DEFAULT VALUES
    ON CONFLICT DO NOTHING;
```

Singleton row tracking the last processed `COALESCE(succeeded_at, failed_at)` in `request_attempt`. The Rust code must assert `rows_affected == 1` when advancing the watermark to detect a missing singleton row.

### New CLI flags

| Flag | Default | Description |
|------|---------|-------------|
| `--health-monitor-bucket-duration` | `5m` | Max time duration of a bucket before closing |
| `--health-monitor-bucket-max-messages` | `100` | Max message count per bucket before closing |
| `--health-monitor-bucket-retention-days` | `8` | Bucket retention (default aligned with `--max-retry-window` of 8 days) |

### Reused existing CLI flags

| Flag | Used in | Purpose |
|------|---------|---------|
| `--health-monitor-time-window` | Step 4, 5 | Sliding window for suspect detection and failure rate |
| `--health-monitor-min-sample-size` | Step 4, 5 | Coarse pre-filter for suspects + minimum total attempts to evaluate |
| `--health-monitor-retention-period-days` | Step 8 | Health event retention (separate from bucket retention) |

### Removed

- Table `webhook.subscription_health` (cumulative counters)
- Index `idx_subscription_health_percent`
- `record_delivery_health()` function in output-worker
- SAVEPOINT logic in `pg.rs` and `pulsar.rs`
- Calls to `record_delivery_health` in `pg.rs` and `pulsar.rs`

### Kept (unchanged)

- `webhook.subscription.failure_percent` — synced by health monitor for frontend display (set for suspects, reset to NULL for non-suspects)
- `webhook.subscription_health_event` — transition history (warning/disabled/resolved)
- `idx_subscription_health_event_sub_id` — covering index for lateral join
- `idx_subscription_health_event_cleanup` — partial index for event cleanup
- State machine logic (warning → disabled → resolved)
- Email notifications, Hook0 client events
- Advisory lock for cross-instance mutual exclusion

## Health Monitor Flow (per tick)

All steps run within the advisory-locked transaction. If the advisory lock is not acquired, the tick is skipped (existing behavior).

### Step 1: Ingest deltas

Read new completed attempts since the watermark using the expression index:

```sql
WITH capped AS (
    SELECT subscription__id, failed_at, succeeded_at
    FROM webhook.request_attempt
    WHERE COALESCE(succeeded_at, failed_at) > $watermark
      AND (succeeded_at IS NOT NULL OR failed_at IS NOT NULL)
    ORDER BY COALESCE(succeeded_at, failed_at)
    LIMIT 50000  -- safety valve: caps raw rows scanned, not output groups
)
SELECT subscription__id,
       COUNT(*) AS total,
       COUNT(failed_at) AS failed,
       MAX(COALESCE(succeeded_at, failed_at)) AS max_completed_at
FROM capped
GROUP BY subscription__id
```

The inner `LIMIT 50000` bounds the actual number of raw rows read from the index, preventing lock hold explosion after extended downtime. `ORDER BY COALESCE(succeeded_at, failed_at)` ensures the watermark advances monotonically. Using `COALESCE(succeeded_at, failed_at)` instead of `created_at` prevents losing in-flight rows — a row picked up before the watermark but completed after it will be captured when it completes.

Note: rows where `succeeded_at IS NULL AND failed_at IS NULL` (still in-flight) are excluded. They will be picked up in a future tick when they complete.

### Step 2: Upsert into open buckets

Atomic CTE that resolves each subscription's open bucket (or creates a new one):

```sql
WITH open_buckets AS (
    SELECT subscription__id, bucket_start
    FROM webhook.subscription_health_bucket
    WHERE subscription__id = ANY($subscription_ids)
      AND bucket_end IS NULL
),
deltas (subscription__id, total, failed) AS (
    VALUES ($sub1, $total1, $failed1), ($sub2, $total2, $failed2), ...
)
INSERT INTO webhook.subscription_health_bucket
    (subscription__id, bucket_start, total_count, failed_count)
SELECT d.subscription__id,
       COALESCE(ob.bucket_start, now()),
       d.total,
       d.failed
FROM deltas d
LEFT JOIN open_buckets ob USING (subscription__id)
ON CONFLICT (subscription__id, bucket_start)
DO UPDATE SET
    total_count = subscription_health_bucket.total_count + EXCLUDED.total_count,
    failed_count = subscription_health_bucket.failed_count + EXCLUDED.failed_count;
```

Uses `idx_subscription_health_bucket_open` for the open bucket lookup. One round trip, handles both "open bucket exists" and "no bucket yet".

Note: the UPSERT is NOT idempotent (additive). The advisory lock is the sole correctness guard — if two monitors ran concurrently, counts would double. This is acceptable since the advisory lock prevents concurrent runs by design.

### Step 3: Close full buckets

```sql
UPDATE webhook.subscription_health_bucket
SET bucket_end = now()
WHERE bucket_end IS NULL
  AND (bucket_start < now() - $bucket_duration
       OR total_count >= $bucket_max_messages);
```

Note: `now()` in PostgreSQL returns the transaction start time, so `bucket_start < now() - 5min` is consistent within the tick. A bucket opened in this same tick will NOT be prematurely closed.

### Step 4: Detect suspects

From bucket data (no `request_attempt` re-scan):

```sql
SELECT subscription__id
FROM webhook.subscription_health_bucket
WHERE bucket_start > now() - $time_window
GROUP BY subscription__id
HAVING SUM(failed_count) > $min_failures_threshold
```

Uses `idx_subscription_health_bucket_start` for the time range.

Union with subscriptions currently in `warning` state (for recovery detection — a warned subscription whose failure rate drops must be re-evaluated):

```sql
UNION
SELECT DISTINCT she.subscription__id
FROM webhook.subscription_health_event she
WHERE she.status = 'warning'
  AND NOT EXISTS (
    SELECT 1 FROM webhook.subscription_health_event newer
    WHERE newer.subscription__id = she.subscription__id
      AND newer.created_at > she.created_at
  )
```

The `$min_failures_threshold` reuses the existing `--health-monitor-min-sample-size` CLI flag as a coarse pre-filter. This is intentionally loose — a subscription with 6 failures out of 10,000 will appear as a suspect but Step 5 will compute its real 0.06% failure rate, and the state machine won't act. The actual evaluation threshold is applied in Step 5 via the `warning_failure_percent` config.

### Step 5: Compute failure rate for suspects

```sql
SELECT subscription__id,
       SUM(failed_count)::float8 / SUM(total_count) * 100.0 AS failure_percent,
       SUM(total_count) AS sample_size
FROM webhook.subscription_health_bucket
WHERE subscription__id = ANY($suspect_ids)
  AND bucket_start > now() - $time_window
GROUP BY subscription__id
HAVING SUM(total_count) >= $min_sample_size
```

Uses PK `(subscription__id, bucket_start)` for index range scan per suspect.

### Step 6: State machine + notifications + failure_percent sync

Unchanged state machine. For each suspect above threshold, run `process_subscription` (existing state machine). For suspects whose failure rate is below the warning threshold AND who are currently in `warning` state, the state machine transitions them to `resolved`.

Sync `subscription.failure_percent` for frontend display:
- Set `failure_percent` for each evaluated suspect (existing UPDATE by PK)
- Reset to NULL for subscriptions that have a non-null `failure_percent` but are NOT in the suspect list:

```sql
UPDATE webhook.subscription
SET failure_percent = NULL
WHERE failure_percent IS NOT NULL
  AND subscription__id NOT IN (SELECT unnest($suspect_ids));
```

### Step 7: Advance watermark

```sql
UPDATE webhook.health_monitor_watermark
SET last_processed_at = $max_completed_at
WHERE id = 1
  AND $max_completed_at > last_processed_at;
```

`$max_completed_at` is the `MAX(COALESCE(succeeded_at, failed_at))` returned by Step 1. If Step 1 returned zero rows, the watermark is not advanced. This prevents skipping rows from in-flight output-worker transactions.

The Rust code must assert `rows_affected == 1` when the watermark is expected to advance, to detect a missing singleton row.

### Step 8: Cleanup (daily)

```sql
DELETE FROM webhook.subscription_health_bucket
WHERE bucket_start < now() - make_interval(days => $bucket_retention_days);
```

Uses `idx_subscription_health_bucket_start` for efficient range scan. Runs in the existing daily cleanup cycle alongside `cleanup_resolved_health_events`.

## Scalability Analysis

| Operation | Cost | Index used |
|-----------|------|-----------|
| Delta scan (Step 1) | O(new rows since last tick) | `idx_request_attempt_completed_at` expression index |
| Bucket upsert (Step 2) | O(subscriptions with activity) | PK + `idx_subscription_health_bucket_open` |
| Close buckets (Step 3) | O(open buckets past threshold) | `idx_subscription_health_bucket_open` |
| Suspects from buckets (Step 4) | O(buckets in window) | `idx_subscription_health_bucket_start` |
| Warning recovery (Step 4) | O(warning events) | `idx_subscription_health_event_sub_id` |
| Failure rate (Step 5) | O(suspects × buckets in window) | PK range scan |
| State machine (Step 6) | O(suspects above threshold) | PK lookups |
| failure_percent reset (Step 6) | O(previously-suspect subs) | Seq scan on non-null failure_percent (small set) |
| Watermark advance (Step 7) | O(1) | PK |
| Cleanup (Step 8) | O(expired buckets) | `idx_subscription_health_bucket_start` |

At 100k subscriptions with 1% failing: Step 4 finds ~1000 suspects, Step 5 aggregates ~1000 × 12 buckets (1 hour / 5 min) = 12,000 rows. All via index. Estimated tick duration: ~30ms.

Safety valve: Step 1 LIMIT caps delta processing at 50,000 raw rows per tick, bounding lock hold time even after extended downtime.

## Migration Path

1. Create `subscription_health_bucket`, `health_monitor_watermark` tables + bucket indexes (transactional migration)
2. Create `idx_request_attempt_completed_at` expression index (separate non-transactional migration with `CREATE INDEX CONCURRENTLY`)
3. Add CLI flags for bucket duration, max messages, and bucket retention
4. Rewrite `evaluation.rs` to use the new 7-step flow
5. Remove `record_delivery_health` from output-worker, revert `pg.rs` and `pulsar.rs`
6. Drop `subscription_health` table and `idx_subscription_health_percent`
7. Update integration tests

## Out of Scope

- **Bucket rollup** — consolidating old fine-grained buckets (5min) into coarser buckets (hourly) after N days. Would reduce storage for long retention periods. Currently buckets are simply deleted after `bucket_retention_days`.
- **Manual retry API** (`POST /messages/{id}/retry`)
- **Bulk replay/recover API** (`POST /applications/{app_id}/replay`, `/recover`)
- **`message.attempt.exhausted` operational webhook** — event when all retries for a message are exhausted
- **`endpoint.warning` operational webhook** — currently only `endpoint.disabled` fires a Hook0 event; warnings send email only
- **Notification tracking table** (`endpoint_health_notification`) — deduplication of notifications; current design uses the health event history for this
- **Per-subscription configurable thresholds** — currently thresholds (warning %, disable %) are global CLI flags
- **Automatic recovery probing** — re-testing disabled endpoints after a cooldown period
- **Dashboard historical charts** — displaying failure rate over time from bucket data (the data model supports it, the frontend doesn't render it yet)
