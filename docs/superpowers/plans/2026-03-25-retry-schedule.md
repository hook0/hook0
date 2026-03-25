# Configurable Retry Schedule — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow organizations to define named retry schedules (exponential, linear, custom) and assign them to subscriptions, falling back to David's default when none is assigned.

**Architecture:** New `webhook.retry_schedule` table with strategy-specific fields. CRUD API following flat-scope pattern (service_token style). Worker's `compute_next_retry` modified to resolve schedule via JOIN and dispatch to strategy-specific delay computation via `ScheduleConfig` enum. Hook0 client events with full payload.

**Tech Stack:** Rust, actix-web, paperclip (OpenAPI), sqlx (Postgres), clap, strum, Biscuit auth, validator crate, k6 (JS integration tests)

**Spec:** `docs/superpowers/specs/2026-03-25-retry-schedule-design.md`

**Base:** David's commit `5b44bbc8` cherry-picked onto this branch.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `api/migrations/20260325120000_add_retry_schedule.up.sql` | Create | Migration (expand-and-contract) |
| `api/migrations/20260325120000_add_retry_schedule.down.sql` | Create | Rollback |
| `api/src/iam.rs` | Modify | Add `RetrySchedule*` Action variants |
| `api/src/problems.rs` | Modify | Add `TooManyRetrySchedulesPerOrganization` variant |
| `api/src/handlers/retry_schedules.rs` | Create | CRUD handlers + unit tests for validation |
| `api/src/handlers/mod.rs` | Modify | Register module |
| `api/src/handlers/subscriptions.rs` | Modify | Add `retry_schedule_id` field |
| `api/src/main.rs` | Modify | Routes + `max_retry_schedules_per_org` env var |
| `api/src/hook0_client.rs` | Modify | 3 event types with full payload |
| `output-worker/src/main.rs` | Modify | `ScheduleConfig` enum + `compute_delay_from_schedule` + unit tests |
| `output-worker/src/pg.rs` | Modify | LEFT JOIN in fetch query |
| `output-worker/src/pulsar.rs` | Modify | LEFT JOIN in status query |
| `tests-api-integrations/src/retry_schedules/` | Create | k6 integration tests |

---

## Task 1: Database Migration

**Files:**
- Create: `api/migrations/20260325120000_add_retry_schedule.up.sql`
- Create: `api/migrations/20260325120000_add_retry_schedule.down.sql`

- [ ] **Step 1: Write up migration (expand-and-contract for zero-downtime)**

```sql
-- api/migrations/20260325120000_add_retry_schedule.up.sql

create table webhook.retry_schedule (
    retry_schedule__id uuid not null default public.gen_random_uuid(),
    organization__id uuid not null
        constraint retry_schedule_organization__id_fkey
        references iam.organization(organization__id)
        on update cascade on delete cascade,
    name text not null check (length(name) > 1),
    strategy text not null check (strategy in ('exponential', 'linear', 'custom')),
    max_retries integer not null check (max_retries > 0 and max_retries <= 100),
    custom_intervals integer[],
    linear_delay integer,
    created_at timestamptz not null default statement_timestamp(),
    updated_at timestamptz not null default statement_timestamp(),
    constraint retry_schedule_pkey primary key (retry_schedule__id),
    constraint retry_schedule_org_name_unique unique (organization__id, name),
    constraint retry_schedule_strategy_fields_check check (
        case strategy
            when 'exponential' then
                custom_intervals is null and linear_delay is null
            when 'linear' then
                custom_intervals is null
                and linear_delay is not null
                and linear_delay >= 1 and linear_delay <= 604800
            when 'custom' then
                linear_delay is null
                and custom_intervals is not null
                and array_length(custom_intervals, 1) = max_retries
                and 1 <= all(custom_intervals)
                and 604800 >= all(custom_intervals)
            else false
        end
    )
);

-- Expand-and-contract: add column without FK first (instant, minimal lock)
alter table webhook.subscription add column retry_schedule__id uuid;

-- Add FK with NOT VALID (no full scan, brief lock)
alter table webhook.subscription
    add constraint subscription_retry_schedule__id_fkey
    foreign key (retry_schedule__id)
    references webhook.retry_schedule(retry_schedule__id)
    on update cascade on delete set null
    not valid;

-- Validate separately (SHARE UPDATE EXCLUSIVE, not exclusive)
alter table webhook.subscription
    validate constraint subscription_retry_schedule__id_fkey;

-- Index on FK column to prevent full table scan on cascade delete
create index subscription_retry_schedule__id_idx
    on webhook.subscription (retry_schedule__id);
```

- [ ] **Step 2: Write down migration**

```sql
-- api/migrations/20260325120000_add_retry_schedule.down.sql

drop index if exists webhook.subscription_retry_schedule__id_idx;
alter table webhook.subscription drop constraint if exists subscription_retry_schedule__id_fkey;
alter table webhook.subscription drop column if exists retry_schedule__id;
drop table if exists webhook.retry_schedule;
```

- [ ] **Step 3: Run migration, verify, commit**

```bash
cd api && sqlx migrate run
psql $DATABASE_URL -c "\d webhook.retry_schedule"
git add api/migrations/20260325120000_add_retry_schedule.*
git commit -m "feat(db): add retry_schedule table and subscription FK"
```

---

## Task 2: IAM Action Variants + Error Variant

**Files:** `api/src/iam.rs`, `api/src/problems.rs`

- [ ] **Step 1: Add Action variants**

```rust
    RetryScheduleList,
    RetryScheduleCreate,
    RetryScheduleGet,
    RetryScheduleEdit,
    RetryScheduleDelete,
```

5 match arms:
- `action_name()` → `"retry_schedule:list"`, `"retry_schedule:create"`, etc.
- `allowed_roles()` → `vec![Role::Viewer]` for List/Get, `vec![]` for Create/Edit/Delete
- `can_work_without_organization()` → falls through to `false`
- `application_id()` → `None`
- `generate_facts()` → `vec![]`

- [ ] **Step 2: Add error variant**

```rust
TooManyRetrySchedulesPerOrganization(QuotaValue),
```

Mapping (follow `TooManySubscriptionsPerApplication` pattern exactly):
```rust
Hook0Problem::TooManyRetrySchedulesPerOrganization(limit) => {
    let detail = format!("This organization cannot have more than {limit} retry schedules.");
    Problem {
        id: Hook0Problem::TooManyRetrySchedulesPerOrganization(limit),
        title: "Exceeded number of retry schedules per organization",
        detail: detail.into(),
        validation: None,
        status: StatusCode::TOO_MANY_REQUESTS,
    }
},
```

- [ ] **Step 3: Verify, commit**

---

## Task 3: Retry Schedule CRUD Handler

**Files:** `api/src/handlers/retry_schedules.rs`, `api/src/handlers/mod.rs`

- [ ] **Step 1: Register module, define types**

```rust
// api/src/handlers/mod.rs
pub mod retry_schedules;
```

```rust
// api/src/handlers/retry_schedules.rs
use actix_web::web::{Data, Json, Path, Query, ReqData};
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::{api_v2_operation, CreatedJson, Apiv2Schema};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use strum::{Display, EnumString};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventRetryScheduleCreated, EventRetryScheduleRemoved, EventRetryScheduleUpdated,
    Hook0ClientEvent,
};
use crate::iam::{authorize, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::quotas::QuotaValue;

pub const MAX_INTERVAL_SECS: i32 = 604_800; // 1 week. Must match DB CHECK.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Apiv2Schema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

// Response struct — strategy as String from DB (sqlx maps TEXT → String)
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: String,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy, // enum — invalid values → 400 at deserialization
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePut {
    #[validate(length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}
```

- [ ] **Step 2: Cross-field validation function**

```rust
fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    custom_intervals: &Option<Vec<i32>>,
    linear_delay: &Option<i32>,
) -> Result<(), Hook0Problem> {
    match strategy {
        RetryStrategy::Exponential => {
            if custom_intervals.is_some() || linear_delay.is_some() {
                return Err(Hook0Problem::BadRequest(
                    "exponential strategy must not have custom_intervals or linear_delay".into(),
                ));
            }
        }
        RetryStrategy::Linear => {
            if custom_intervals.is_some() {
                return Err(Hook0Problem::BadRequest(
                    "linear strategy must not have custom_intervals".into(),
                ));
            }
            if linear_delay.is_none() {
                return Err(Hook0Problem::BadRequest(
                    "linear strategy requires linear_delay".into(),
                ));
            }
        }
        RetryStrategy::Custom => {
            if linear_delay.is_some() {
                return Err(Hook0Problem::BadRequest(
                    "custom strategy must not have linear_delay".into(),
                ));
            }
            match custom_intervals {
                None => {
                    return Err(Hook0Problem::BadRequest(
                        "custom strategy requires custom_intervals".into(),
                    ));
                }
                Some(intervals) => {
                    if intervals.len() != max_retries as usize {
                        return Err(Hook0Problem::BadRequest(format!(
                            "custom_intervals length ({}) must equal max_retries ({})",
                            intervals.len(), max_retries
                        )));
                    }
                    if intervals.iter().any(|&i| i < 1 || i > MAX_INTERVAL_SECS) {
                        return Err(Hook0Problem::BadRequest(format!(
                            "each custom_interval must be between 1 and {MAX_INTERVAL_SECS}"
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}
```

Note: Check if `Hook0Problem::BadRequest(String)` exists. If not, find the equivalent variant in `problems.rs` for generic 400 errors. The DB CHECK is the final safety net.

- [ ] **Step 3-7: Implement list, create, get, edit, delete handlers**

All handlers follow these patterns:
- **Auth first** (authorize before validate)
- **SQL scoped** to `organization__id`
- **`strategy_str = body.strategy.to_string()`** for SQL insert/update
- **`CreatedJson`** for create, `Json` for others
- **Hook0 event** with full payload (name, strategy, max_retries, custom_intervals, linear_delay)
- **Delete uses `DELETE...RETURNING`** to capture data for the event

Create handler uses atomic INSERT with per-org limit:
```sql
INSERT INTO webhook.retry_schedule (...)
SELECT $1, $2, $3, $4, $5::integer[], $6
WHERE (SELECT count(*) FROM webhook.retry_schedule WHERE organization__id = $1) < $7
```

- [ ] **Step 8: Unit tests for validate_strategy_fields (in `#[cfg(test)]`)**

20 tests covering all cross-field validation rules:
- `test_validate_exponential_ok` — no intervals, no delay → Ok
- `test_validate_exponential_rejects_intervals` → BadRequest
- `test_validate_exponential_rejects_delay` → BadRequest
- `test_validate_linear_ok` — delay present, no intervals → Ok
- `test_validate_linear_rejects_intervals` → BadRequest
- `test_validate_linear_requires_delay` → BadRequest
- `test_validate_custom_ok` — intervals.len == max_retries → Ok
- `test_validate_custom_rejects_delay` → BadRequest
- `test_validate_custom_requires_intervals` → BadRequest
- `test_validate_custom_len_mismatch` — len != max_retries → BadRequest
- `test_validate_custom_interval_below_min` — [0] → BadRequest
- `test_validate_custom_interval_above_max` — [604801] → BadRequest
- `test_validate_custom_interval_at_min` — [1] → Ok
- `test_validate_custom_interval_at_max` — [604800] → Ok
- `test_validate_exponential_rejects_both` — intervals + delay → BadRequest
- `test_validate_linear_rejects_both` — intervals + delay → BadRequest (intervals present)
- `test_validate_custom_empty_intervals` — [] with max_retries=1 → BadRequest (len mismatch)
- `test_validate_custom_single` — [60] with max_retries=1 → Ok
- `test_validate_custom_mixed_valid` — [1, 300, 604800] with max_retries=3 → Ok
- `test_validate_custom_mixed_invalid` — [1, 0, 300] → BadRequest (0 < 1)

- [ ] **Step 9: Verify compilation, commit**

---

## Task 4: Register Routes + Config

**Files:** `api/src/main.rs`

- [ ] **Step 1: Add env var to Config**

```rust
#[clap(long, env, default_value_t = 50)]
max_retry_schedules_per_org: QuotaValue,
```

Pass through to `State`.

- [ ] **Step 2: Add route scope**

```rust
.service(
    web::scope("/retry_schedules")
        .wrap(Compat::new(rate_limiters.token()))
        .wrap(biscuit_auth.clone())
        .service(web::resource("")
            .route(web::get().to(handlers::retry_schedules::list))
            .route(web::post().to(handlers::retry_schedules::create)))
        .service(web::resource("/{retry_schedule_id}")
            .route(web::get().to(handlers::retry_schedules::get))
            .route(web::put().to(handlers::retry_schedules::edit))
            .route(web::delete().to(handlers::retry_schedules::delete))),
)
```

- [ ] **Step 3: Verify, commit**

---

## Task 5: Hook0 Client Events

**Files:** `api/src/hook0_client.rs`

- [ ] **Step 1: Add EVENT_TYPES strings, event structs with full payload, enum variants, mk_hook0_event arms**

Each event struct has: `organization_id`, `retry_schedule_id`, `name`, `strategy`, `max_retries`, `custom_intervals: Option<Vec<i32>>`, `linear_delay: Option<i32>`.

Pattern: `impl Event` (event_type + labels), `impl From<...> for Hook0ClientEvent`, `to_event(e, None)` in mk_hook0_event.

- [ ] **Step 2: Verify, commit**

---

## Task 6: Modify Subscription Handler

**Files:** `api/src/handlers/subscriptions.rs`

- [ ] **Step 1: Add `retry_schedule_id: Option<Uuid>` to Subscription + SubscriptionPost**
- [ ] **Step 2: Update all SELECT queries** to include `s.retry_schedule__id AS retry_schedule_id`
- [ ] **Step 3: Atomic cross-org enforcement** in create/edit via SQL subquery
- [ ] **Step 4: Verify, commit**

---

## Task 7: Modify Worker Retry Logic

**Files:** `output-worker/src/main.rs`, `output-worker/src/pg.rs`, `output-worker/src/pulsar.rs`

- [ ] **Step 1: Define `ScheduleConfig` as an enum (make invalid states unrepresentable)**

```rust
use std::str::FromStr;

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

/// Schedule configuration — enum eliminates impossible field combinations.
#[derive(Debug, Clone)]
pub enum ScheduleConfig {
    Exponential { max_retries: i32 },
    Linear { max_retries: i32, delay_secs: i32 },
    Custom { max_retries: i32, intervals_secs: Vec<i32> },
}

impl ScheduleConfig {
    fn max_retries(&self) -> i32 {
        match self {
            Self::Exponential { max_retries } => *max_retries,
            Self::Linear { max_retries, .. } => *max_retries,
            Self::Custom { max_retries, .. } => *max_retries,
        }
    }
}
```

Add raw fields to `RequestAttemptWithOptionalPayload`:

```rust
    pub retry_strategy: Option<String>,
    pub retry_max_retries: Option<i32>,
    pub retry_custom_intervals: Option<Vec<i32>>,
    pub retry_linear_delay: Option<i32>,
```

Build helper:

```rust
impl RequestAttemptWithOptionalPayload {
    pub fn schedule_config(&self) -> Option<ScheduleConfig> {
        let strategy_str = self.retry_strategy.as_ref()?;
        let strategy = RetryStrategy::from_str(strategy_str).ok()?;
        let max_retries = self.retry_max_retries?;
        match strategy {
            RetryStrategy::Exponential => Some(ScheduleConfig::Exponential { max_retries }),
            RetryStrategy::Linear => {
                let delay_secs = self.retry_linear_delay?;
                Some(ScheduleConfig::Linear { max_retries, delay_secs })
            }
            RetryStrategy::Custom => {
                let intervals_secs = self.retry_custom_intervals.clone()?;
                // Validate invariant: intervals length must match max_retries
                if intervals_secs.len() != usize::try_from(max_retries).unwrap_or(0) {
                    warn!("Custom schedule has intervals len {} != max_retries {}, falling back to default",
                        intervals_secs.len(), max_retries);
                    return None;
                }
                Some(ScheduleConfig::Custom { max_retries, intervals_secs })
            }
        }
    }
}
```

Note: if any required field is `None` or strategy parse fails → returns `None` → falls back to default. Caller logs `warn!`.

- [ ] **Step 2: Write `compute_delay_from_schedule`**

```rust
fn compute_delay_from_schedule(config: &ScheduleConfig, retry_count: i16) -> Option<Duration> {
    let count = i32::from(retry_count);
    if count >= config.max_retries() {
        return None;
    }

    match config {
        ScheduleConfig::Exponential { max_retries } => {
            compute_next_retry_duration(
                u8::try_from(*max_retries).unwrap_or(100).min(100),
                retry_count,
            )
        }
        ScheduleConfig::Linear { delay_secs, .. } => {
            // delay_secs guaranteed 1..=604800 by DB CHECK; try_from is defensive
            Some(Duration::from_secs(u64::try_from(*delay_secs).unwrap_or(0)))
        }
        ScheduleConfig::Custom { intervals_secs, .. } => {
            let idx = usize::try_from(count).unwrap_or(0);
            let delay = intervals_secs.get(idx)?; // safe: returns None if out of bounds
            // delay guaranteed 1..=604800 by DB CHECK; try_from is defensive
            Some(Duration::from_secs(u64::try_from(*delay).unwrap_or(0)))
        }
    }
}
```

Key fixes vs previous version:
- `ScheduleConfig` is an enum — no `Option` fields, no magic fallbacks
- `intervals_secs.get(idx)?` — no panic on empty/short vec
- `u8::try_from(*max_retries).unwrap_or(100).min(100)` — clamps to valid range
- `delay_secs` guaranteed present by enum variant (Linear always has it)

- [ ] **Step 3: Modify `compute_next_retry` signature**

```rust
async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
    schedule: Option<&ScheduleConfig>,
) -> Result<Option<Duration>, sqlx::Error>
```

Dispatch:
```rust
if sub.is_some() {
    match schedule {
        Some(config) => Ok(compute_delay_from_schedule(config, attempt.retry_count)),
        None => Ok(compute_next_retry_duration(max_retries, attempt.retry_count)),
    }
} else {
    Ok(None)
}
```

- [ ] **Step 4: Update pg.rs** — LEFT JOIN + pass schedule_config
- [ ] **Step 5: Update pulsar.rs** — LEFT JOIN in status query + pass schedule_config
- [ ] **Step 6: Update boot log** — add "(default, may be overridden per-subscription)"

- [ ] **Step 7: Unit tests for worker (28 tests in `#[cfg(test)]`)**

**compute_delay_from_schedule — Exponential (7):**
- `test_exponential_first_retry` — count=0 → `Some(3s)`
- `test_exponential_mid_table` — count=3 → `Some(30min)`
- `test_exponential_past_table` — count=9, max=10 → `Some(10h)`
- `test_exponential_at_max` — count=5, max=5 → `None`
- `test_exponential_past_max` — count=6, max=5 → `None`
- `test_exponential_max_1` — max=1, count=0 → `Some(3s)`
- `test_exponential_max_100` — max=100, count=99 → `Some(10h)`

**compute_delay_from_schedule — Linear (5):**
- `test_linear_first_retry` — delay=300, count=0 → `Some(300s)`
- `test_linear_every_retry_same` — count=4 → `Some(300s)`
- `test_linear_at_max` — count=5, max=5 → `None`
- `test_linear_delay_1s` → `Some(1s)`
- `test_linear_delay_max` → `Some(604800s)`

**compute_delay_from_schedule — Custom (6):**
- `test_custom_first_interval` — [3,30,300], count=0 → `Some(3s)`
- `test_custom_last_interval` — count=2 → `Some(300s)`
- `test_custom_at_max` — count=3, max=3 → `None`
- `test_custom_single` — [60], max=1, count=0 → `Some(60s)`
- `test_custom_empty_intervals` — [], count=0 → `None` (get returns None)
- `test_custom_all_same` — [10,10,10] → all `Some(10s)`

**schedule_config() helper (6):**
- `test_config_none_when_no_strategy` → `None`
- `test_config_exponential` → `Exponential { max_retries: 10 }`
- `test_config_linear` → `Linear { max_retries: 5, delay_secs: 300 }`
- `test_config_custom` → `Custom { max_retries: 3, intervals_secs: [1,2,3] }`
- `test_config_unknown_strategy` — "fibonacci" → `None`
- `test_config_missing_max_retries` → `None`

**Fallback (2):**
- `test_no_schedule_uses_default` — count=0 → `Some(3s)`
- `test_schedule_overrides_default` — Exponential max=2, count=2 → `None`

**Negative/edge (2):**
- `test_negative_retry_count` — count=-1 → behavior depends on strategy (exponential delegates to David's fn)
- `test_retry_count_zero_all_strategies` — count=0 returns correct first delay for each

- [ ] **Step 8: Verify, commit**

---

## Task 8: Integration Tests (k6)

**Files:** `tests-api-integrations/src/retry_schedules/`

Follow the existing k6 pattern: JS modules, auth via `SERVICE_TOKEN`, assertions via `check()`.

- [ ] **Step 1: Create test module files**

```
tests-api-integrations/src/retry_schedules/
├── create_retry_schedule.js
├── list_retry_schedules.js
├── get_retry_schedule.js
├── update_retry_schedule.js
├── delete_retry_schedule.js
└── assign_to_subscription.js
```

- [ ] **Step 2: CRUD tests (in create/list/get/update/delete .js)**

**Create:**
- Create exponential → 201, strategy="exponential", custom_intervals=null, linear_delay=null
- Create linear → 201, linear_delay present
- Create custom → 201, custom_intervals present, len == max_retries
- Create with invalid strategy → 400
- Create exponential with custom_intervals → 400
- Create linear without linear_delay → 400
- Create custom with len mismatch → 400
- Create custom with interval=0 → 400
- Create max_retries=0 → 400
- Create empty name → 400
- Create duplicate name same org → constraint error
- Create same name different org → both 201

**List:**
- List empty → []
- List after create → contains created
- List scoped to org → other org's schedules not visible

**Get:**
- Get existing → 200, all fields
- Get wrong org → 404
- Get nonexistent → 404

**Update:**
- Update name → 200, updated_at changed
- Change strategy → 200
- Update with invalid cross-fields → 400
- Update wrong org → 404

**Delete:**
- Delete existing → 200, then get → 404
- Delete wrong org → 404
- Delete twice → second 404

- [ ] **Step 3: Subscription assignment tests (assign_to_subscription.js)**

- Assign schedule → subscription has retry_schedule_id
- Assign cross-org schedule → rejected
- Remove schedule (null) → reverts
- Delete schedule → subscription.retry_schedule_id becomes null

- [ ] **Step 4: Role tests**

- Viewer can list → 200
- Viewer can get → 200
- Viewer cannot create → 403
- Viewer cannot edit → 403
- Viewer cannot delete → 403

- [ ] **Step 5: Wire into main test runner, commit**

Add to `tests-api-integrations/src/main.js` or a separate scenario. Commit:
```bash
git add tests-api-integrations/
git commit -m "test: add k6 integration tests for retry schedules"
```

---

## Task 9: Full Build Verification

- [ ] **Step 1: Build workspace** — `cargo build`
- [ ] **Step 2: Run unit tests** — `cargo test`
- [ ] **Step 3: Run sqlx prepare** — `cd api && cargo sqlx prepare && cd ../output-worker && cargo sqlx prepare`
- [ ] **Step 4: Commit sqlx metadata**

---

## Notes for Implementation

- **sqlx compile-time checking**: Ensure migration applied to dev DB before `cargo check`.
- **Biscuit Datalog**: Follow `ServiceToken*` pattern. List/Get add `allowed_role("viewer")`.
- **RetryStrategy enum**: Defined in both API and worker. Acceptable duplication for now.
- **MAX_INTERVAL_SECS**: Rust const `604_800`. SQL CHECK uses literal `604800`. Comment in both referencing the other.
- **SQL convention**: Migrations lowercase, handler/worker UPPERCASE.
- **Concurrent INSERT race**: Documented, accepted (soft quota + rate limiter).
- **`ScheduleConfig` enum**: Makes invalid states unrepresentable. `Exponential` has no intervals/delay, `Linear` always has `delay_secs`, `Custom` always has `intervals_secs`.
- **`intervals.get(idx)?`**: Safe indexing, no panic on empty vec.
- **Expand-and-contract migration**: Column added without FK, FK added NOT VALID, then validated separately. Minimizes lock window.
