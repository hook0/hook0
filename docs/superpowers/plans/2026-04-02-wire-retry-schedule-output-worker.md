# Wire Retry Schedule in Output Worker — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the output-worker use the subscription's assigned retry schedule (increasing/linear/custom) instead of the hardcoded backoff table when computing retry delays.

**Architecture:** Modify `compute_next_retry` to LEFT JOIN the retry schedule from the DB. Dispatch delay computation by strategy. Fallback to the existing hardcoded backoff when no schedule is assigned. The function signature stays the same — callers (`pg.rs`, `pulsar.rs`) are unchanged.

**Tech Stack:** Rust, sqlx (compile-time checked queries), PostgreSQL

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `output-worker/src/main.rs` | Add `RetryScheduleRow` struct, rewrite `compute_next_retry` query, add `compute_scheduled_retry_delay`, update `compute_next_retry_duration` to `compute_default_retry_delay`, add tests |

No other files change — `pg.rs` and `pulsar.rs` call `compute_next_retry` with the same signature.

---

## Task 1: Add `RetryScheduleRow` struct and update the query

**Files:**
- Modify: `output-worker/src/main.rs`

- [ ] **Step 1: Add the `RetryScheduleRow` struct**

After the existing `RequestAttemptWithOptionalPayload` struct, add:

```rust
/// Retry schedule data fetched alongside the subscription check in `compute_next_retry`.
/// All fields are optional because the LEFT JOIN returns NULLs when no schedule is assigned.
#[allow(non_snake_case)]
#[derive(Debug, sqlx::FromRow)]
struct SubscriptionRetryInfo {
    /// Always true when the subscription is active — used to distinguish "no row" (sub deleted/disabled) from "no schedule"
    is_active: bool,
    strategy: Option<String>,
    max_retries: Option<i32>,
    custom_intervals: Option<Vec<i32>>,
    linear_delay: Option<i32>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
}
```

- [ ] **Step 2: Replace the query in `compute_next_retry`**

Replace the existing `query!("SELECT true AS whatever ...")` block (lines ~848-861) with:

```rust
let sub = sqlx::query_as::<_, SubscriptionRetryInfo>(
    r#"
    SELECT
        true AS is_active,
        rs.strategy,
        rs.max_retries,
        rs.custom_intervals,
        rs.linear_delay,
        rs.increasing_base_delay,
        rs.increasing_wait_factor
    FROM webhook.subscription AS s
    INNER JOIN event.application AS a ON a.application__id = s.application__id
    LEFT JOIN webhook.retry_schedule AS rs ON rs.retry_schedule__id = s.retry_schedule__id
    WHERE s.subscription__id = $1
        AND s.deleted_at IS NULL
        AND s.is_enabled
        AND a.deleted_at IS NULL
    "#,
)
.bind(attempt.subscription_id)
.fetch_optional(conn)
.await?;
```

- [ ] **Step 3: Update the dispatch logic after the query**

Replace the `if sub.is_some() { ... }` block with:

```rust
match sub {
    Some(info) => {
        // Subscription is active — compute delay from schedule or fallback
        Ok(compute_scheduled_retry_delay(&info, attempt.retry_count, max_retries))
    }
    None => {
        // Subscription deleted/disabled — do not retry
        Ok(None)
    }
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cd output-worker && SQLX_OFFLINE=true cargo check 2>&1 | tail -5
```

Expected: compilation error for missing `compute_scheduled_retry_delay` — that's Task 2.

- [ ] **Step 5: Commit (WIP)**

```bash
git add output-worker/src/main.rs
git commit -m "wip: add SubscriptionRetryInfo struct and query in compute_next_retry"
```

---

## Task 2: Implement `compute_scheduled_retry_delay`

**Files:**
- Modify: `output-worker/src/main.rs`

- [ ] **Step 1: Rename `compute_next_retry_duration` to `compute_default_retry_delay`**

This is the existing hardcoded backoff. Rename it and update its single call site in `evaluate_retry_policy`.

- [ ] **Step 2: Add `compute_scheduled_retry_delay`**

Add after the `compute_next_retry` function:

```rust
/// Computes the retry delay for a subscription based on its assigned retry schedule.
/// Falls back to the default hardcoded backoff when no schedule is assigned.
fn compute_scheduled_retry_delay(
    info: &SubscriptionRetryInfo,
    retry_count: i16,
    global_max_retries: u8,
) -> Option<Duration> {
    match info.strategy.as_deref() {
        Some("increasing") => {
            let max = info.max_retries.unwrap_or(0);
            if retry_count >= max as i16 {
                return None;
            }
            let base = info.increasing_base_delay.unwrap_or(3) as f64;
            let factor = info.increasing_wait_factor.unwrap_or(3.0);
            Some(Duration::from_secs_f64(base * factor.powi(retry_count as i32)))
        }
        Some("linear") => {
            let max = info.max_retries.unwrap_or(0);
            if retry_count >= max as i16 {
                return None;
            }
            let delay = info.linear_delay.unwrap_or(60) as u64;
            Some(Duration::from_secs(delay))
        }
        Some("custom") => {
            let intervals = info.custom_intervals.as_deref().unwrap_or(&[]);
            intervals
                .get(retry_count as usize)
                .map(|&d| Duration::from_secs(d as u64))
        }
        _ => {
            // No schedule assigned — use hardcoded default backoff
            compute_default_retry_delay(global_max_retries, retry_count)
        }
    }
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd output-worker && SQLX_OFFLINE=true cargo check 2>&1 | tail -5
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add output-worker/src/main.rs
git commit -m "feat(output-worker): compute retry delay from assigned retry schedule"
```

---

## Task 3: Add unit tests

**Files:**
- Modify: `output-worker/src/main.rs` (in the `#[cfg(test)]` module)

- [ ] **Step 1: Add tests for all 3 strategies + fallback**

```rust
#[test]
fn scheduled_increasing_delays() {
    let info = SubscriptionRetryInfo {
        is_active: true,
        strategy: Some("increasing".to_string()),
        max_retries: Some(5),
        custom_intervals: None,
        linear_delay: None,
        increasing_base_delay: Some(3),
        increasing_wait_factor: Some(3.0),
    };
    // retry 0: 3 * 3^0 = 3s
    assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(3)));
    // retry 1: 3 * 3^1 = 9s
    assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(9)));
    // retry 2: 3 * 3^2 = 27s
    assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(27)));
    // retry 5: exceeds max_retries → None
    assert_eq!(compute_scheduled_retry_delay(&info, 5, 25), None);
}

#[test]
fn scheduled_linear_delays() {
    let info = SubscriptionRetryInfo {
        is_active: true,
        strategy: Some("linear".to_string()),
        max_retries: Some(3),
        custom_intervals: None,
        linear_delay: Some(120),
        increasing_base_delay: None,
        increasing_wait_factor: None,
    };
    assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(120)));
    assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(120)));
    assert_eq!(compute_scheduled_retry_delay(&info, 3, 25), None);
}

#[test]
fn scheduled_custom_delays() {
    let info = SubscriptionRetryInfo {
        is_active: true,
        strategy: Some("custom".to_string()),
        max_retries: Some(3),
        custom_intervals: Some(vec![10, 60, 300]),
        linear_delay: None,
        increasing_base_delay: None,
        increasing_wait_factor: None,
    };
    assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(10)));
    assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(60)));
    assert_eq!(compute_scheduled_retry_delay(&info, 2, 25), Some(Duration::from_secs(300)));
    assert_eq!(compute_scheduled_retry_delay(&info, 3, 25), None);
}

#[test]
fn no_schedule_falls_back_to_default() {
    let info = SubscriptionRetryInfo {
        is_active: true,
        strategy: None,
        max_retries: None,
        custom_intervals: None,
        linear_delay: None,
        increasing_base_delay: None,
        increasing_wait_factor: None,
    };
    // Falls back to hardcoded: retry 0 = 3s, retry 1 = 10s
    assert_eq!(compute_scheduled_retry_delay(&info, 0, 25), Some(Duration::from_secs(3)));
    assert_eq!(compute_scheduled_retry_delay(&info, 1, 25), Some(Duration::from_secs(10)));
    // Global max respected for fallback
    assert_eq!(compute_scheduled_retry_delay(&info, 25, 25), None);
}
```

- [ ] **Step 2: Run tests**

```bash
cd output-worker && SQLX_OFFLINE=true cargo test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 3: Run clippy**

```bash
SQLX_OFFLINE=true cargo clippy -p hook0-output-worker -- -D warnings 2>&1 | tail -5
```

Expected: no errors.

- [ ] **Step 4: Regenerate sqlx cache**

```bash
DATABASE_URL="postgres://postgres:postgres@127.0.0.1:5432/hook0" cargo sqlx prepare 2>&1 | tail -3
```

- [ ] **Step 5: Commit**

```bash
git add output-worker/
git commit -m "feat(output-worker): wire retry schedule into delivery retries"
```
