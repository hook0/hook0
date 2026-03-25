# Configurable Retry Schedule — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow organizations to define named retry schedules (exponential, linear, custom) and assign them to subscriptions, falling back to David's default when none is assigned.

**Architecture:** New `webhook.retry_schedule` table with strategy-specific fields. CRUD API following flat-scope pattern (service_token style). Worker's `compute_next_retry` modified to resolve schedule via JOIN and dispatch to strategy-specific delay computation. Hook0 client events with full payload.

**Tech Stack:** Rust, actix-web, paperclip (OpenAPI), sqlx (Postgres), clap, strum, Biscuit auth, validator crate

**Spec:** `docs/superpowers/specs/2026-03-25-retry-schedule-design.md`

**Base:** David's commit `5b44bbc8` cherry-picked onto this branch.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `api/migrations/20260325120000_add_retry_schedule.up.sql` | Create | Migration: new table + subscription FK |
| `api/migrations/20260325120000_add_retry_schedule.down.sql` | Create | Rollback migration |
| `api/src/iam.rs` | Modify | Add `RetrySchedule*` Action variants (Viewer for read, Editor for write) |
| `api/src/problems.rs` | Modify | Add `TooManyRetrySchedulesPerOrganization` error variant |
| `api/src/handlers/retry_schedules.rs` | Create | CRUD handlers for retry schedules |
| `api/src/handlers/mod.rs` | Modify | Register `retry_schedules` module |
| `api/src/handlers/subscriptions.rs` | Modify | Add `retry_schedule_id` to post/response structs |
| `api/src/main.rs` | Modify | Register `/retry_schedules` routes + `max_retry_schedules_per_org` env var |
| `api/src/hook0_client.rs` | Modify | Add 3 event types with full payload |
| `output-worker/src/main.rs` | Modify | ScheduleConfig + compute_delay_from_schedule + boot log note |
| `output-worker/src/pg.rs` | Modify | Add schedule fields to fetch query via LEFT JOIN |
| `output-worker/src/pulsar.rs` | Modify | Add LEFT JOIN in status query, pass schedule to retry logic |

---

## Task 1: Database Migration

**Files:**
- Create: `api/migrations/20260325120000_add_retry_schedule.up.sql`
- Create: `api/migrations/20260325120000_add_retry_schedule.down.sql`

- [ ] **Step 1: Write up migration**

```sql
-- api/migrations/20260325120000_add_retry_schedule.up.sql

create table webhook.retry_schedule (
    retry_schedule__id uuid not null default public.gen_random_uuid(),
    organization__id uuid not null references iam.organization(organization__id) on update cascade on delete cascade,
    name text not null check (length(name) >= 1),
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
        end
    )
);

alter table webhook.subscription
    add column retry_schedule__id uuid
    references webhook.retry_schedule(retry_schedule__id)
    on update cascade on delete set null;
```

Notes:
- FK on `organization__id` inline in CREATE TABLE (not separate ALTER TABLE)
- FK on `subscription.retry_schedule__id` via ALTER TABLE (existing table)
- `on delete cascade` on org FK: org deletion cascades to schedule deletion
- `on delete set null` on subscription FK: schedule deletion reverts subscription to default
- Cross-field CHECK validates strategy-specific field requirements
- `604800` = MAX_INTERVAL_SECONDS (1 week). Must match the Rust constant.
- No index on `subscription.retry_schedule__id` — rare queries only

- [ ] **Step 2: Write down migration**

```sql
-- api/migrations/20260325120000_add_retry_schedule.down.sql

alter table webhook.subscription drop column if exists retry_schedule__id;
drop table if exists webhook.retry_schedule;
```

- [ ] **Step 3: Run migration**

Run: `cd api && sqlx migrate run`
Expected: Migration applied successfully

- [ ] **Step 4: Verify schema**

Run: `psql $DATABASE_URL -c "\d webhook.retry_schedule"`
Expected: Table with all columns and constraints

- [ ] **Step 5: Commit**

```bash
git add api/migrations/20260325120000_add_retry_schedule.*
git commit -m "feat(db): add retry_schedule table and subscription FK"
```

---

## Task 2: IAM Action Variants + Error Variant

**Files:**
- Modify: `api/src/iam.rs`
- Modify: `api/src/problems.rs`

- [ ] **Step 1: Add Action variants to iam.rs**

Add to the `Action` enum:

```rust
    //
    RetryScheduleList,
    RetryScheduleCreate,
    RetryScheduleGet,
    RetryScheduleEdit,
    RetryScheduleDelete,
```

Flat variants (no fields), same as `ServiceTokenList`. Add the 5 match arms in:
- `action_name()` → `"retry_schedule:list"`, etc.
- `allowed_roles()` → `vec![Role::Viewer]` for List/Get, `vec![]` (Editor only) for Create/Edit/Delete
- `can_work_without_organization()` → `false` (via default `_ =>` arm)
- `application_id()` → `None`
- `generate_facts()` → `vec![]`

- [ ] **Step 2: Add error variant to problems.rs**

```rust
    TooManyRetrySchedulesPerOrganization(QuotaValue),
```

With mapping:
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

- [ ] **Step 3: Verify compilation**

Run: `cd api && cargo check`

- [ ] **Step 4: Commit**

```bash
git add api/src/iam.rs api/src/problems.rs
git commit -m "feat(api): add RetrySchedule IAM actions and error variant"
```

---

## Task 3: Retry Schedule CRUD Handler

**Files:**
- Create: `api/src/handlers/retry_schedules.rs`
- Modify: `api/src/handlers/mod.rs`

- [ ] **Step 1: Register the module in mod.rs**

```rust
pub mod retry_schedules;
```

- [ ] **Step 2: Define types and structs**

Create `api/src/handlers/retry_schedules.rs`:

```rust
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

/// Maximum interval value in seconds (1 week). Must match the DB CHECK constraint.
pub const MAX_INTERVAL_SECONDS: i32 = 604_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Apiv2Schema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

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
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    #[validate(custom(function = "validate_custom_intervals"))]
    pub custom_intervals: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePut {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    #[validate(custom(function = "validate_custom_intervals"))]
    pub custom_intervals: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}
```

- [ ] **Step 3: Implement validation functions**

```rust
fn validate_custom_intervals(intervals: &[i32]) -> Result<(), validator::ValidationError> {
    if intervals.iter().any(|&i| i < 1 || i > MAX_INTERVAL_SECONDS) {
        return Err(validator::ValidationError::new("interval_out_of_range"));
    }
    Ok(())
}

/// Cross-field validation. Call after struct-level validate().
fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    custom_intervals: &Option<Vec<i32>>,
    linear_delay: &Option<i32>,
) -> Result<(), Hook0Problem> {
    match strategy {
        RetryStrategy::Exponential => {
            if custom_intervals.is_some() || linear_delay.is_some() {
                return Err(Hook0Problem::Validation(/* "exponential: custom_intervals and linear_delay must be null" */));
            }
        }
        RetryStrategy::Linear => {
            if custom_intervals.is_some() {
                return Err(Hook0Problem::Validation(/* "linear: custom_intervals must be null" */));
            }
            if linear_delay.is_none() {
                return Err(Hook0Problem::Validation(/* "linear: linear_delay is required" */));
            }
        }
        RetryStrategy::Custom => {
            if linear_delay.is_some() {
                return Err(Hook0Problem::Validation(/* "custom: linear_delay must be null" */));
            }
            match custom_intervals {
                None => return Err(Hook0Problem::Validation(/* "custom: custom_intervals is required" */)),
                Some(intervals) if intervals.len() != max_retries as usize => {
                    return Err(Hook0Problem::Validation(/* "custom: custom_intervals.len() must equal max_retries" */));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
```

Note: The exact error construction depends on how `Hook0Problem::Validation` wraps validation errors. The implementer should check the existing pattern in other handlers and adapt. The DB CHECK constraint is the final safety net.

- [ ] **Step 4: Implement `list` handler**

```rust
#[api_v2_operation(summary = "List retry schedules", operation_id = "retry_schedules.list", tags(retry_schedules))]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<Vec<RetrySchedule>>, Hook0Problem> {
    let org_id = qs.organization_id;

    if authorize(&biscuit, Some(org_id), Action::RetryScheduleList,
        state.max_authorization_time_in_ms, state.debug_authorizer).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let schedules = query_as!(RetrySchedule,
        "SELECT retry_schedule__id AS retry_schedule_id, organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay, created_at, updated_at
         FROM webhook.retry_schedule WHERE organization__id = $1 ORDER BY created_at ASC",
        &org_id
    ).fetch_all(&state.db).await.map_err(Hook0Problem::from)?;

    Ok(Json(schedules))
}
```

- [ ] **Step 5: Implement `create` handler**

Auth first, per-org limit (atomic INSERT...SELECT...WHERE count < N), `CreatedJson` return.

```rust
#[api_v2_operation(summary = "Create a retry schedule", operation_id = "retry_schedules.create", tags(retry_schedules))]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<RetrySchedulePost>,
) -> Result<CreatedJson<RetrySchedule>, Hook0Problem> {
    if let Err(e) = body.validate() { return Err(Hook0Problem::Validation(e)); }
    validate_strategy_fields(body.strategy, body.max_retries, &body.custom_intervals, &body.linear_delay)?;

    let org_id = body.organization_id;
    if authorize(&biscuit, Some(org_id), Action::RetryScheduleCreate,
        state.max_authorization_time_in_ms, state.debug_authorizer).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let strategy_str = body.strategy.to_string();
    let limit = i64::from(state.max_retry_schedules_per_org);

    let schedule = query_as!(RetrySchedule,
        "INSERT INTO webhook.retry_schedule (organization__id, name, strategy, max_retries, custom_intervals, linear_delay)
         SELECT $1, $2, $3, $4, $5::integer[], $6
         WHERE (SELECT count(*) FROM webhook.retry_schedule WHERE organization__id = $1) < $7
         RETURNING retry_schedule__id AS retry_schedule_id, organization__id AS organization_id,
                   name, strategy, max_retries, custom_intervals, linear_delay, created_at, updated_at",
        &org_id, &body.name, &strategy_str, &body.max_retries,
        &body.custom_intervals as &Option<Vec<i32>>, &body.linear_delay, limit,
    ).fetch_optional(&state.db).await.map_err(Hook0Problem::from)?;

    let schedule = match schedule {
        Some(s) => s,
        None => return Err(Hook0Problem::TooManyRetrySchedulesPerOrganization(state.max_retry_schedules_per_org)),
    };

    // Hook0 client event
    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let evt: Hook0ClientEvent = EventRetryScheduleCreated {
            organization_id: schedule.organization_id,
            retry_schedule_id: schedule.retry_schedule_id,
            name: schedule.name.clone(),
            strategy: schedule.strategy.clone(),
            max_retries: schedule.max_retries,
            custom_intervals: schedule.custom_intervals.clone(),
            linear_delay: schedule.linear_delay,
        }.into();
        if let Err(e) = hook0_client.send_event(&evt.mk_hook0_event()).await {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(schedule))
}
```

- [ ] **Step 6: Implement `get` handler**

Auth-first, SQL scoped to org_id + schedule_id.

```rust
#[api_v2_operation(summary = "Get a retry schedule", operation_id = "retry_schedules.get", tags(retry_schedules))]
pub async fn get(
    state: Data<crate::State>, _: OaBiscuit, biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>, qs: Query<RetryScheduleQs>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let org_id = qs.organization_id;
    if authorize(&biscuit, Some(org_id), Action::RetryScheduleGet,
        state.max_authorization_time_in_ms, state.debug_authorizer).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let schedule = query_as!(RetrySchedule,
        "SELECT retry_schedule__id AS retry_schedule_id, organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay, created_at, updated_at
         FROM webhook.retry_schedule WHERE retry_schedule__id = $1 AND organization__id = $2",
        schedule_id.as_ref(), &org_id,
    ).fetch_optional(&state.db).await.map_err(Hook0Problem::from)?.ok_or(Hook0Problem::NotFound)?;

    Ok(Json(schedule))
}
```

- [ ] **Step 7: Implement `edit` handler**

Auth-first, UPDATE...RETURNING scoped to org_id. Hook0 event with full payload.

```rust
#[api_v2_operation(summary = "Update a retry schedule", operation_id = "retry_schedules.edit", tags(retry_schedules))]
pub async fn edit(
    state: Data<crate::State>, _: OaBiscuit, biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>, qs: Query<RetryScheduleQs>, body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    if let Err(e) = body.validate() { return Err(Hook0Problem::Validation(e)); }
    validate_strategy_fields(body.strategy, body.max_retries, &body.custom_intervals, &body.linear_delay)?;

    let org_id = qs.organization_id;
    if authorize(&biscuit, Some(org_id), Action::RetryScheduleEdit,
        state.max_authorization_time_in_ms, state.debug_authorizer).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let strategy_str = body.strategy.to_string();

    let schedule = query_as!(RetrySchedule,
        "UPDATE webhook.retry_schedule
         SET name = $3, strategy = $4, max_retries = $5, custom_intervals = $6::integer[],
             linear_delay = $7, updated_at = statement_timestamp()
         WHERE retry_schedule__id = $1 AND organization__id = $2
         RETURNING retry_schedule__id AS retry_schedule_id, organization__id AS organization_id,
                   name, strategy, max_retries, custom_intervals, linear_delay, created_at, updated_at",
        schedule_id.as_ref(), &org_id, &body.name, &strategy_str, &body.max_retries,
        &body.custom_intervals as &Option<Vec<i32>>, &body.linear_delay,
    ).fetch_optional(&state.db).await.map_err(Hook0Problem::from)?.ok_or(Hook0Problem::NotFound)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let evt: Hook0ClientEvent = EventRetryScheduleUpdated {
            organization_id: schedule.organization_id,
            retry_schedule_id: schedule.retry_schedule_id,
            name: schedule.name.clone(),
            strategy: schedule.strategy.clone(),
            max_retries: schedule.max_retries,
            custom_intervals: schedule.custom_intervals.clone(),
            linear_delay: schedule.linear_delay,
        }.into();
        if let Err(e) = hook0_client.send_event(&evt.mk_hook0_event()).await {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(Json(schedule))
}
```

- [ ] **Step 8: Implement `delete` handler**

Auth-first. DELETE RETURNING (no 409 — SET NULL handles subscriptions). Disambiguate NotFound on 0 rows.

```rust
#[api_v2_operation(summary = "Delete a retry schedule", operation_id = "retry_schedules.delete", tags(retry_schedules))]
pub async fn delete(
    state: Data<crate::State>, _: OaBiscuit, biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>, qs: Query<RetryScheduleQs>,
) -> Result<Json<()>, Hook0Problem> {
    let org_id = qs.organization_id;
    if authorize(&biscuit, Some(org_id), Action::RetryScheduleDelete,
        state.max_authorization_time_in_ms, state.debug_authorizer).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    struct Deleted { name: String, strategy: String, max_retries: i32,
                     custom_intervals: Option<Vec<i32>>, linear_delay: Option<i32> }

    let deleted = sqlx::query_as!(Deleted,
        "DELETE FROM webhook.retry_schedule
         WHERE retry_schedule__id = $1 AND organization__id = $2
         RETURNING name, strategy, max_retries, custom_intervals, linear_delay",
        schedule_id.as_ref(), &org_id,
    ).fetch_optional(&state.db).await.map_err(Hook0Problem::from)?
     .ok_or(Hook0Problem::NotFound)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let evt: Hook0ClientEvent = EventRetryScheduleRemoved {
            organization_id: org_id,
            retry_schedule_id: *schedule_id,
            name: deleted.name,
            strategy: deleted.strategy,
            max_retries: deleted.max_retries,
            custom_intervals: deleted.custom_intervals,
            linear_delay: deleted.linear_delay,
        }.into();
        if let Err(e) = hook0_client.send_event(&evt.mk_hook0_event()).await {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(Json(()))
}
```

- [ ] **Step 9: Verify compilation, commit**

```bash
cd api && cargo check
git add api/src/handlers/retry_schedules.rs api/src/handlers/mod.rs
git commit -m "feat(api): add retry_schedules CRUD handler"
```

---

## Task 4: Register Routes + Config

**Files:**
- Modify: `api/src/main.rs`

- [ ] **Step 1: Add `max_retry_schedules_per_org` to Config**

In the `Config` struct (or equivalent CLI args struct), add:

```rust
    /// Maximum number of retry schedules per organization
    #[clap(long, env, default_value_t = 50)]
    max_retry_schedules_per_org: QuotaValue,
```

Ensure it's available in `State` (or passed through to handlers via `state.max_retry_schedules_per_org`).

- [ ] **Step 2: Add retry_schedules route scope**

```rust
.service(
    web::scope("/retry_schedules")
        .wrap(Compat::new(rate_limiters.token()))
        .wrap(biscuit_auth.clone())
        .service(
            web::resource("")
                .route(web::get().to(handlers::retry_schedules::list))
                .route(web::post().to(handlers::retry_schedules::create)),
        )
        .service(
            web::resource("/{retry_schedule_id}")
                .route(web::get().to(handlers::retry_schedules::get))
                .route(web::put().to(handlers::retry_schedules::edit))
                .route(web::delete().to(handlers::retry_schedules::delete)),
        ),
)
```

- [ ] **Step 3: Verify compilation, commit**

```bash
cd api && cargo check
git add api/src/main.rs
git commit -m "feat(api): register /retry_schedules routes and config"
```

---

## Task 5: Hook0 Client Events

**Files:**
- Modify: `api/src/hook0_client.rs`

- [ ] **Step 1: Add event type strings to EVENT_TYPES**

```rust
    "api.retry_schedule.created",
    "api.retry_schedule.updated",
    "api.retry_schedule.removed",
```

- [ ] **Step 2: Add event structs with full payload**

Three structs (Created/Updated/Removed), each with:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct EventRetryScheduleCreated {
    pub organization_id: Uuid,
    pub retry_schedule_id: Uuid,
    pub name: String,
    pub strategy: String,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
}
```

Each implements `Event` trait (`event_type()` → string, `labels()` → instance + org labels) and `From<...> for Hook0ClientEvent`. Same pattern for Updated and Removed.

- [ ] **Step 3: Add variants to Hook0ClientEvent enum + mk_hook0_event match arms**

```rust
    RetryScheduleCreated(EventRetryScheduleCreated),
    RetryScheduleUpdated(EventRetryScheduleUpdated),
    RetryScheduleRemoved(EventRetryScheduleRemoved),
```

In `mk_hook0_event`:
```rust
    Self::RetryScheduleCreated(e) => to_event(e, None),
    Self::RetryScheduleUpdated(e) => to_event(e, None),
    Self::RetryScheduleRemoved(e) => to_event(e, None),
```

- [ ] **Step 4: Verify compilation, commit**

```bash
cd api && cargo check
git add api/src/hook0_client.rs
git commit -m "feat(api): add retry_schedule hook0 client events with full payload"
```

---

## Task 6: Modify Subscription Handler

**Files:**
- Modify: `api/src/handlers/subscriptions.rs`

- [ ] **Step 1: Add `retry_schedule_id` to Subscription response + SubscriptionPost**

```rust
// In Subscription (response):
pub retry_schedule_id: Option<Uuid>,

// In SubscriptionPost (request, used for both create and edit):
pub retry_schedule_id: Option<Uuid>,
```

- [ ] **Step 2: Update all SELECT queries to include `s.retry_schedule__id AS retry_schedule_id`**

- [ ] **Step 3: Update create INSERT with atomic cross-org enforcement**

Use subquery to resolve retry_schedule_id only if it belongs to the same org:

```sql
INSERT INTO webhook.subscription (..., retry_schedule__id)
VALUES (..., (
    SELECT retry_schedule__id FROM webhook.retry_schedule
    WHERE retry_schedule__id = $N
      AND organization__id = (
          SELECT organization__id FROM event.application
          WHERE application__id = $APP_ID AND deleted_at IS NULL
      )
))
```

After INSERT, check if `body.retry_schedule_id.is_some() && created.retry_schedule_id.is_none()` → return NotFound.

- [ ] **Step 4: Update edit UPDATE with same cross-org enforcement**

- [ ] **Step 5: Verify compilation, commit**

```bash
cd api && cargo check
git add api/src/handlers/subscriptions.rs
git commit -m "feat(api): add retry_schedule_id to subscription CRUD"
```

---

## Task 7: Modify Worker Retry Logic

**Files:**
- Modify: `output-worker/src/main.rs`
- Modify: `output-worker/src/pg.rs`
- Modify: `output-worker/src/pulsar.rs`

- [ ] **Step 1: Add ScheduleConfig + RetryStrategy to worker**

In `output-worker/src/main.rs`:

```rust
use std::str::FromStr;

/// Retry schedule configuration resolved from the subscription's assigned schedule.
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub strategy: RetryStrategy,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
}

#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}
```

Add schedule fields to `RequestAttemptWithOptionalPayload`:

```rust
    pub retry_strategy: Option<String>,
    pub retry_max_retries: Option<i32>,
    pub retry_custom_intervals: Option<Vec<i32>>,
    pub retry_linear_delay: Option<i32>,
```

Add helper:

```rust
impl RequestAttemptWithOptionalPayload {
    pub fn schedule_config(&self) -> Option<ScheduleConfig> {
        let strategy_str = self.retry_strategy.as_ref()?;
        let strategy = RetryStrategy::from_str(strategy_str).ok()?;
        Some(ScheduleConfig {
            strategy,
            max_retries: self.retry_max_retries?,
            custom_intervals: self.retry_custom_intervals.clone(),
            linear_delay: self.retry_linear_delay,
        })
    }
}
```

Note: if `from_str` fails (unknown strategy in DB), returns `None` → falls back to default. A `warn!` log should be added in the caller when this happens.

- [ ] **Step 2: Write `compute_delay_from_schedule`**

```rust
fn compute_delay_from_schedule(config: &ScheduleConfig, retry_count: i16) -> Option<Duration> {
    let count = i32::from(retry_count);
    if count >= config.max_retries {
        return None;
    }

    match config.strategy {
        RetryStrategy::Exponential => {
            // Delegate to David's hardcoded table with schedule's max_retries
            compute_next_retry_duration(
                u8::try_from(config.max_retries).unwrap_or(u8::MAX),
                retry_count,
            )
        }
        RetryStrategy::Linear => {
            let delay = config.linear_delay.unwrap_or(60);
            Some(Duration::from_secs(u64::try_from(delay.max(0)).unwrap_or(0)))
        }
        RetryStrategy::Custom => {
            let intervals = config.custom_intervals.as_ref()?;
            let idx = usize::try_from(count).unwrap_or(0)
                .min(intervals.len().saturating_sub(1));
            let delay = intervals[idx];
            Some(Duration::from_secs(u64::try_from(delay.max(0)).unwrap_or(0)))
        }
    }
}
```

- [ ] **Step 3: Modify `compute_next_retry` signature**

David's current signature:

```rust
async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
) -> Result<Option<Duration>, sqlx::Error>
```

Add `schedule: Option<&ScheduleConfig>`:

```rust
async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
    schedule: Option<&ScheduleConfig>,
) -> Result<Option<Duration>, sqlx::Error>
```

In the retry branch (after subscription check):

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

- [ ] **Step 4: Update pg.rs fetch query**

Add to SELECT:
```sql
rs.strategy AS retry_strategy,
rs.max_retries AS retry_max_retries,
rs.custom_intervals AS retry_custom_intervals,
rs.linear_delay AS retry_linear_delay,
```

Add JOIN:
```sql
LEFT JOIN webhook.retry_schedule AS rs ON rs.retry_schedule__id = s.retry_schedule__id
```

Update call site:
```rust
let schedule_config = next_attempt.schedule_config();
if schedule_config.is_none() && next_attempt.retry_strategy.is_some() {
    warn!("Unknown retry strategy {:?}, falling back to default", next_attempt.retry_strategy);
}
compute_next_retry(&mut tx, &attempt, &response, config.max_retries, schedule_config.as_ref())
```

- [ ] **Step 5: Update pulsar.rs**

Add LEFT JOIN and schedule fields to the status check query (around line 399). Build `ScheduleConfig` from the query result and pass to `compute_next_retry`.

- [ ] **Step 6: Update boot log**

In the log message from `evaluate_retry_policy`, add a note:

```rust
info!("Default retry policy: {effective_retries} retries over {cumulative_duration:?} (can be overridden per-subscription via custom retry schedules)");
```

- [ ] **Step 7: Verify compilation, commit**

```bash
cd output-worker && cargo check
git add output-worker/src/main.rs output-worker/src/pg.rs output-worker/src/pulsar.rs
git commit -m "feat(worker): support configurable retry schedules"
```

---

## Task 8: Tests

**Files:**
- Modify: `output-worker/src/main.rs` (unit tests in `#[cfg(test)]` module)
- Create or modify: API integration test files (TBD based on actix_web::test setup)

- [ ] **Step 1: Unit tests for `compute_delay_from_schedule`**

Add to the existing `#[cfg(test)]` module in `output-worker/src/main.rs`:

Tests to cover:
- Exponential strategy: delegates to David's table, respects max_retries
- Linear strategy: returns linear_delay for every retry, stops at max_retries
- Custom strategy: returns custom_intervals[retry_count], stops at max_retries
- Custom with retry_count at boundary (== max_retries → None)
- Custom with retry_count 0 (first retry)
- schedule_config() helper: returns None for None fields, parses strategy correctly, returns None for unknown strategy

- [ ] **Step 2: Integration tests for CRUD API**

Setup: `actix_web::test` with a test DB (migration applied). Tests to cover:
- Create all 3 strategies with valid bodies → 201
- Create with invalid strategy → 400
- Create exponential with custom_intervals present → 400 (cross-field validation)
- Create custom with len(custom_intervals) != max_retries → 400
- List schedules for org → returns created schedules
- Get schedule by ID → 200
- Get with wrong org_id → 404
- Update schedule → 200, fields changed
- Delete schedule → 200
- Delete assigned schedule → 200 (SET NULL, subscription.retry_schedule_id becomes null)
- Per-org limit exceeded → 429

- [ ] **Step 3: Integration test for subscription assignment**

- Assign schedule to subscription → OK
- Assign cross-org schedule → fails (retry_schedule_id becomes null)
- Remove schedule (set null) → subscription reverts to default

- [ ] **Step 4: Commit tests**

```bash
git add -A
git commit -m "test: add unit and integration tests for retry schedules"
```

---

## Task 9: Full Build Verification

- [ ] **Step 1: Build entire workspace**

Run: `cargo build`

- [ ] **Step 2: Run all tests**

Run: `cargo test`

- [ ] **Step 3: Run sqlx prepare**

Run: `cd api && cargo sqlx prepare && cd ../output-worker && cargo sqlx prepare`

- [ ] **Step 4: Commit sqlx metadata**

```bash
git add -A .sqlx/ */.sqlx/
git commit -m "chore: update sqlx offline query metadata"
```

---

## Notes for Implementation

- **sqlx compile-time checking**: Ensure migration applied to dev DB before `cargo check`.
- **Biscuit Datalog rules**: Follow `ServiceToken*` pattern. `RetryScheduleList`/`RetryScheduleGet` add `allowed_role("viewer")` via `allowed_roles() -> vec![Role::Viewer]`.
- **RetryStrategy enum**: Defined in both API (`api/src/handlers/retry_schedules.rs`) and worker (`output-worker/src/main.rs`). Consider extracting to a shared crate if this becomes a pattern. For now, duplication is acceptable (2 files).
- **MAX_INTERVAL_SECONDS**: Rust const in handler (`604_800`). SQL CHECK uses the literal `604800`. Document the coupling with a comment in both locations.
- **SQL convention**: Migrations use lowercase keywords; handler/worker queries use UPPERCASE keywords.
- **Concurrent INSERT race**: The atomic `INSERT...SELECT...WHERE count < N` is not strictly race-safe under `READ COMMITTED` (two concurrent inserts could both pass). Accepted risk: rate limiter + soft quota, blast radius is low.
- **`SubscriptionPost` reuse**: The codebase reuses `SubscriptionPost` for both create and edit. The `retry_schedule_id` field applies to both.
