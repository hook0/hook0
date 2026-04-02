# Health Monitor Bucketed Evaluation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace cumulative health counters with time-bucketed aggregation driven by the health monitor. Zero output-worker changes. Watermark-based delta ingestion. Suspect detection from buckets.

**Architecture:** The health monitor ingests completed `request_attempt` rows via an expression index, aggregates into time/count-bounded buckets, detects suspects from bucket data, computes failure rates, and feeds the existing state machine. The output-worker writes nothing extra.

**Tech Stack:** Rust, sqlx (runtime queries with `FromRow`), PostgreSQL, actix-web

**Spec:** `docs/superpowers/specs/2026-04-02-health-monitor-buckets-design.md`

---

## File Map

### New Files
| File | Responsibility |
|------|----------------|
| `api/migrations/20260402120000_add_health_buckets.up.sql` | Create `subscription_health_bucket`, `health_monitor_watermark` tables + indexes |
| `api/migrations/20260402120000_add_health_buckets.down.sql` | Rollback |
| `api/migrations/20260402120001_add_completed_at_index.up.sql` | `CREATE INDEX CONCURRENTLY` expression index (non-transactional) |
| `api/migrations/20260402120001_add_completed_at_index.down.sql` | Drop expression index |

### Modified Files
| File | Change |
|------|--------|
| `api/src/health_monitor/mod.rs` | Add 3 new fields to `HealthMonitorConfig` |
| `api/src/health_monitor/evaluation.rs` | Rewrite `fetch_subscription_health_stats` (7-step flow), update `cleanup_resolved_health_events` to also clean buckets |
| `api/src/main.rs` | Add 3 CLI flags, wire into config struct |
| `output-worker/src/pg.rs` | Remove `record_delivery_health` calls |
| `output-worker/src/pulsar.rs` | Remove `record_delivery_health` calls |
| `output-worker/src/main.rs` | Remove `record_delivery_health` function |

### Removed
| File/Artifact | Reason |
|---------------|--------|
| `webhook.subscription_health` table | Replaced by buckets (drop in migration) |
| `idx_subscription_health_percent` | Table dropped |

---

## Task 1: Migrations

**Files:**
- Create: `api/migrations/20260402120000_add_health_buckets.up.sql`
- Create: `api/migrations/20260402120000_add_health_buckets.down.sql`
- Create: `api/migrations/20260402120001_add_completed_at_index.up.sql`
- Create: `api/migrations/20260402120001_add_completed_at_index.down.sql`

- [ ] **Step 1: Create transactional migration (up)**

```sql
-- api/migrations/20260402120000_add_health_buckets.up.sql

-- Bucketed health aggregation (replaces subscription_health cumulative counters)
CREATE TABLE webhook.subscription_health_bucket (
    subscription__id UUID NOT NULL
        REFERENCES webhook.subscription(subscription__id) ON DELETE CASCADE,
    bucket_start     TIMESTAMPTZ NOT NULL,
    bucket_end       TIMESTAMPTZ,
    total_count      INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
    failed_count     INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0 AND failed_count <= total_count),
    PRIMARY KEY (subscription__id, bucket_start)
);

CREATE INDEX idx_subscription_health_bucket_start
    ON webhook.subscription_health_bucket(bucket_start);

CREATE INDEX idx_subscription_health_bucket_open
    ON webhook.subscription_health_bucket(subscription__id)
    WHERE bucket_end IS NULL;

-- Watermark singleton for delta processing
CREATE TABLE webhook.health_monitor_watermark (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_processed_at TIMESTAMPTZ NOT NULL DEFAULT '-infinity'
);
INSERT INTO webhook.health_monitor_watermark DEFAULT VALUES
    ON CONFLICT DO NOTHING;

-- Drop old cumulative counters
DROP TABLE IF EXISTS webhook.subscription_health;
```

- [ ] **Step 2: Create transactional migration (down)**

```sql
-- api/migrations/20260402120000_add_health_buckets.down.sql

-- Recreate old table
CREATE TABLE webhook.subscription_health (
    subscription__id UUID NOT NULL
        CONSTRAINT subscription_health_pkey PRIMARY KEY
        REFERENCES webhook.subscription(subscription__id) ON DELETE CASCADE,
    total_count INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
    failed_count INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0 AND failed_count <= total_count),
    failure_percent DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (failure_percent >= 0 AND failure_percent <= 100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp()
);

CREATE INDEX idx_subscription_health_percent
    ON webhook.subscription_health(failure_percent)
    WHERE failure_percent > 0;

DROP TABLE IF EXISTS webhook.health_monitor_watermark;
DROP TABLE IF EXISTS webhook.subscription_health_bucket;
```

- [ ] **Step 3: Create non-transactional migration for expression index (up)**

```sql
-- api/migrations/20260402120001_add_completed_at_index.up.sql
-- no-transaction
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_request_attempt_completed_at
    ON webhook.request_attempt (COALESCE(succeeded_at, failed_at))
    WHERE succeeded_at IS NOT NULL OR failed_at IS NOT NULL;
```

- [ ] **Step 4: Create non-transactional migration (down)**

```sql
-- api/migrations/20260402120001_add_completed_at_index.down.sql
-- no-transaction
DROP INDEX CONCURRENTLY IF EXISTS webhook.idx_request_attempt_completed_at;
```

- [ ] **Step 5: Reset DB, run migrations, verify**

```bash
psql "postgres://postgres:postgres@127.0.0.1:5432/postgres" -c "DROP DATABASE hook0 WITH (FORCE);"
psql "postgres://postgres:postgres@127.0.0.1:5432/postgres" -c "CREATE DATABASE hook0;"
cd api && DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/hook0" sqlx migrate run --ignore-missing
```

Verify: `\dt webhook.subscription_health_bucket` exists, `\dt webhook.subscription_health` does NOT exist, index `idx_request_attempt_completed_at` exists and is VALID.

- [ ] **Step 6: Commit**

```bash
git add api/migrations/20260402120000_* api/migrations/20260402120001_*
git commit -m "feat(db): add health bucket tables, watermark, expression index; drop subscription_health"
```

---

## Task 2: CLI flags + config

**Files:**
- Modify: `api/src/health_monitor/mod.rs` (lines 18-30)
- Modify: `api/src/main.rs` (lines 497-501 for new flags, lines 1108-1117 for config construction)

- [ ] **Step 1: Add fields to `HealthMonitorConfig`**

In `api/src/health_monitor/mod.rs`, add after `retention_period_days`:

```rust
    pub bucket_duration: Duration,
    pub bucket_max_messages: u32,
    pub bucket_retention_days: u32,
```

- [ ] **Step 2: Add CLI flags in `main.rs`**

After the `health_monitor_retention_period_days` flag (~line 501), add:

```rust
    /// Duration of a health bucket before closing
    #[clap(long, env, value_parser = humantime::parse_duration, default_value = "5m")]
    health_monitor_bucket_duration: Duration,

    /// Maximum message count per health bucket before closing
    #[clap(long, env, default_value_t = 100)]
    health_monitor_bucket_max_messages: u32,

    /// Health bucket retention in days (default aligned with max-retry-window)
    #[clap(long, env, default_value_t = 8)]
    health_monitor_bucket_retention_days: u32,
```

- [ ] **Step 3: Wire into config construction**

In `main.rs` config construction (~line 1108-1117), add the 3 new fields:

```rust
    bucket_duration: config.health_monitor_bucket_duration,
    bucket_max_messages: config.health_monitor_bucket_max_messages,
    bucket_retention_days: config.health_monitor_bucket_retention_days,
```

- [ ] **Step 4: Verify compilation**

```bash
SQLX_OFFLINE=true cargo check -p hook0-api 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
git add api/src/health_monitor/mod.rs api/src/main.rs
git commit -m "feat(api): add health bucket CLI flags (duration, max-messages, retention)"
```

---

## Task 3: Rewrite evaluation.rs

**Files:**
- Modify: `api/src/health_monitor/evaluation.rs` (full rewrite)

This is the core task. The file currently has `fetch_subscription_health_stats` (reads from `subscription_health`) and `cleanup_resolved_health_events`. Replace `fetch_subscription_health_stats` with the 7-step flow. Keep `cleanup_resolved_health_events` and add bucket cleanup.

- [ ] **Step 1: Rewrite `fetch_subscription_health_stats`**

The function signature changes — it now returns `(Vec<SubscriptionHealth>, Option<DateTime<Utc>>)` where the second element is `max_completed_at` for the watermark. The `SubscriptionHealth` struct stays the same (the state machine consumes it).

The function implements Steps 1-5 of the spec:
1. Read watermark
2. Ingest delta from `request_attempt` via expression index (CTE with LIMIT 50000)
3. Upsert into open buckets (atomic CTE)
4. Close full buckets
5. Detect suspects (from buckets + warned subscriptions)
6. Compute failure rate for suspects
7. Return `Vec<SubscriptionHealth>` + `max_completed_at`

Since this is complex SQL, use `sqlx::query_as::<_, T>()` (runtime, not macro) for the final query, and `sqlx::query()` for the intermediate steps.

- [ ] **Step 2: Add `advance_watermark` function**

```rust
pub async fn advance_watermark(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    max_completed_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        "UPDATE webhook.health_monitor_watermark SET last_processed_at = $1 WHERE id = 1 AND $1 > last_processed_at"
    )
    .bind(max_completed_at)
    .execute(&mut **tx)
    .await?;

    if result.rows_affected() != 1 {
        tracing::warn!("Health monitor: watermark row missing or not advanced");
    }
    Ok(())
}
```

- [ ] **Step 3: Add `reset_healthy_failure_percent` function**

```rust
pub async fn reset_healthy_failure_percent(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    suspect_ids: &[Uuid],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE webhook.subscription SET failure_percent = NULL WHERE failure_percent IS NOT NULL AND subscription__id != ALL($1)"
    )
    .bind(suspect_ids)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected())
}
```

- [ ] **Step 4: Add `cleanup_old_buckets` function**

```rust
pub async fn cleanup_old_buckets(
    db: &PgPool,
    config: &HealthMonitorConfig,
) -> Result<u64, sqlx::Error> {
    let retention_days = config.bucket_retention_days as i32;
    let result = sqlx::query(
        "DELETE FROM webhook.subscription_health_bucket WHERE bucket_start < now() - make_interval(days => $1)"
    )
    .bind(retention_days)
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}
```

- [ ] **Step 5: Verify compilation**

```bash
SQLX_OFFLINE=true cargo check -p hook0-api 2>&1 | tail -10
```

- [ ] **Step 6: Commit**

```bash
git add api/src/health_monitor/evaluation.rs
git commit -m "feat(api): rewrite health evaluation with bucketed aggregation"
```

---

## Task 4: Update mod.rs to call new functions

**Files:**
- Modify: `api/src/health_monitor/mod.rs`

- [ ] **Step 1: Update `run_health_check` to call `advance_watermark` and `reset_healthy_failure_percent`**

In `run_health_check` (~line 83), after the `process_subscription` loop and before `tx.commit()`:

1. Call `advance_watermark(&mut tx, max_completed_at)` if `max_completed_at` is `Some`
2. Collect suspect_ids from the subscriptions that were evaluated
3. Call `reset_healthy_failure_percent(&mut tx, &suspect_ids)`

- [ ] **Step 2: Update cleanup cycle to also clean buckets**

In `run_health_monitor` (~line 61), after `cleanup_resolved_health_events`, add:

```rust
match evaluation::cleanup_old_buckets(db, config).await {
    Ok(n) if n > 0 => info!("Health monitor: cleaned up {n} old health buckets"),
    Ok(_) => debug!("Health monitor: bucket cleanup tick, none to remove"),
    Err(e) => warn!("Health monitor: bucket cleanup error: {e}"),
}
```

- [ ] **Step 3: Verify compilation**

```bash
SQLX_OFFLINE=true cargo check -p hook0-api 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
git add api/src/health_monitor/mod.rs
git commit -m "feat(api): wire watermark advancement, failure_percent reset, bucket cleanup"
```

---

## Task 5: Remove output-worker health writes

**Files:**
- Modify: `output-worker/src/pg.rs` (lines 281, 295)
- Modify: `output-worker/src/pulsar.rs` (lines 557, 583)
- Modify: `output-worker/src/main.rs` (line 773 — `record_delivery_health` function)

- [ ] **Step 1: Remove calls in `pg.rs`**

Remove the two `if let Err(e) = record_delivery_health(...)` blocks (success path ~line 281, failure path ~line 295). Remove the `record_delivery_health` import from the `use crate::{ ... }` block.

- [ ] **Step 2: Remove calls in `pulsar.rs`**

Same — remove both `if let Err(e) = record_delivery_health(...)` blocks and the import.

- [ ] **Step 3: Remove `record_delivery_health` function in `main.rs`**

Delete the entire function (~line 773-819) including the doc comment and the `MAX_RETRY_DELAY` constant if it's only used there. Also remove the `SubscriptionRetryInfo` `is_active` field usage if it was only needed for the health check (verify first).

- [ ] **Step 4: Verify compilation + tests**

```bash
SQLX_OFFLINE=true cargo check -p hook0-output-worker
SQLX_OFFLINE=true cargo test -p hook0-output-worker
```

- [ ] **Step 5: Commit**

```bash
git add output-worker/
git commit -m "refactor(output-worker): remove record_delivery_health — health monitor handles aggregation"
```

---

## Task 6: Regenerate sqlx cache + final verification

**Files:**
- Modify: `api/.sqlx/` (regenerated)
- Modify: `output-worker/.sqlx/` (regenerated)

- [ ] **Step 1: Regenerate sqlx cache for both crates**

```bash
cd api && DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/hook0" cargo sqlx prepare
cd ../output-worker && DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/hook0" cargo sqlx prepare
```

- [ ] **Step 2: Verify offline build**

```bash
SQLX_OFFLINE=true cargo check --workspace
```

- [ ] **Step 3: Run all tests**

```bash
SQLX_OFFLINE=true cargo test -p hook0-api -p hook0-output-worker
```

- [ ] **Step 4: Run clippy**

```bash
SQLX_OFFLINE=true cargo clippy -p hook0-api -p hook0-output-worker -- -D warnings -A non_snake_case
```

- [ ] **Step 5: Commit**

```bash
git add api/.sqlx/ output-worker/.sqlx/
git commit -m "chore: regenerate sqlx offline cache after health bucket migration"
```
