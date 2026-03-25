# Configurable Retry Schedule — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow organizations to define named retry schedules (exponential, linear, custom) and assign them to subscriptions, falling back to the existing Svix/Stripe default when none is assigned.

**Architecture:** New `webhook.retry_schedule` table with CRUD API following existing flat-scope pattern (service_token style). Worker's `compute_next_retry` modified to resolve the subscription's schedule via JOIN and compute delay per strategy. Hook0 client events for observability.

**Tech Stack:** Rust, actix-web, paperclip (OpenAPI), sqlx (Postgres), clap, strum, Biscuit auth, validator crate

**Spec:** `docs/superpowers/specs/2026-03-25-retry-schedule-design.md`

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `api/migrations/20260325120000_add_retry_schedule.up.sql` | Create | Migration: new table + subscription FK |
| `api/migrations/20260325120000_add_retry_schedule.down.sql` | Create | Rollback migration |
| `api/src/iam.rs` | Modify | Add `RetrySchedule*` Action variants |
| `api/src/problems.rs` | Modify | Add `RetryScheduleInUse` error variant |
| `api/src/handlers/retry_schedules.rs` | Create | CRUD handlers for retry schedules |
| `api/src/handlers/mod.rs` | Modify | Register `retry_schedules` module |
| `api/src/handlers/subscriptions.rs` | Modify | Add `retry_schedule_id` to post/response structs |
| `api/src/main.rs` | Modify | Register `/retry_schedules` routes |
| `api/src/hook0_client.rs` | Modify | Add 3 event types + structs |
| `output-worker/src/main.rs` | Modify | New retry computation with schedule support |
| `output-worker/src/pg.rs` | Modify | Add schedule fields to fetch query |
| `output-worker/src/pulsar.rs` | Modify | Add LEFT JOIN for schedule in status query, pass schedule info through retry path |

---

## Task 1: Database Migration

**Files:**
- Create: `api/migrations/20260325120000_add_retry_schedule.up.sql`
- Create: `api/migrations/20260325120000_add_retry_schedule.down.sql`

- [ ] **Step 1: Write up migration**

```sql
-- api/migrations/20260325120000_add_retry_schedule.up.sql

create table webhook.retry_schedule (
    retry_schedule__id uuid not null default gen_random_uuid(),
    organization__id uuid not null references iam.organization(organization__id) on update cascade on delete cascade,
    name text not null check (length(name) >= 1),
    strategy text not null check (strategy in ('exponential', 'linear', 'custom')),
    intervals integer[] not null check (
        array_length(intervals, 1) > 0
        and 1 <= all(intervals)
        and 604800 >= all(intervals)
    ),
    max_attempts integer not null default 8 check (max_attempts > 0 and max_attempts <= 100),
    created_at timestamptz not null default statement_timestamp(),
    updated_at timestamptz not null default statement_timestamp(),
    constraint retry_schedule_pkey primary key (retry_schedule__id),
    constraint retry_schedule_org_name_unique unique (organization__id, name)
);

alter table webhook.subscription
    add column retry_schedule__id uuid references webhook.retry_schedule(retry_schedule__id) on delete set null;
```

Notes:
- `on delete cascade` on `organization__id`: org deletion cascades to schedule deletion
- `on delete set null` on `subscription.retry_schedule__id`: schedule deletion reverts subscription to default
- `1 <= all(intervals) and 604800 >= all(intervals)`: enforces per-element bounds at DB level
- No index on `subscription.retry_schedule__id` — the usage-count query in the delete handler is rare; the UNIQUE constraint on `(organization__id, name)` covers the list query via prefix scan

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

Run: `psql $DATABASE_URL -c "\d webhook.retry_schedule"` and `psql $DATABASE_URL -c "\d webhook.subscription" | grep retry`
Expected: Table exists with correct columns; subscription has `retry_schedule__id` column

- [ ] **Step 5: Commit**

```bash
git add api/migrations/20260325120000_add_retry_schedule.up.sql api/migrations/20260325120000_add_retry_schedule.down.sql
git commit -m "feat(db): add retry_schedule table and subscription FK"
```

---

## Task 2: IAM Action Variants + Error Variant

**Files:**
- Modify: `api/src/iam.rs` (Action enum, around line 343-464)
- Modify: `api/src/problems.rs` (Hook0Problem enum)

- [ ] **Step 1: Add Action variants to iam.rs**

In `api/src/iam.rs`, add to the `Action` enum (after the `EventsPerDayOrganization` variant, around line 464):

```rust
    //
    RetryScheduleList,
    RetryScheduleCreate,
    RetryScheduleGet,
    RetryScheduleEdit,
    RetryScheduleDelete,
```

These are flat (no fields) like `ServiceTokenList`, `ServiceTokenCreate`, `ServiceTokenGet`. No schedule ID needed in Datalog rules — authorization is org-scoped only.

Then add the corresponding Datalog rules in the `authorize()` function's authorizer builder. Follow the pattern of `ServiceTokenList` / `ServiceTokenCreate` rules (org-scoped, no application_id). The rules should check `right("retry_schedule", "list")`, etc.

- [ ] **Step 2: Add `RetryScheduleInUse` to Hook0Problem**

In `api/src/problems.rs`, add to the `Hook0Problem` enum (in the functional errors section):

```rust
    RetryScheduleInUse,
```

Then add its `Problem` mapping in the `impl From<Hook0Problem> for Problem` block, following the existing conflict pattern (e.g., `OrganizationIsNotEmpty`):

```rust
Hook0Problem::RetryScheduleInUse => Problem {
    id: Uuid::nil(),
    title: "Retry schedule is in use".to_owned(),
    detail: "This retry schedule is still assigned to one or more subscriptions. Unassign it first.".to_owned(),
    status: StatusCode::CONFLICT,
    ..Problem::default()
},
```

- [ ] **Step 3: Verify compilation**

Run: `cd api && cargo check`
Expected: Compiles (new Action variants may have unused warnings — OK, used in Task 3)

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

Add to `api/src/handlers/mod.rs`:
```rust
pub mod retry_schedules;
```

- [ ] **Step 2: Define structs and types in retry_schedules.rs**

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

// --- Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Apiv2Schema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

// --- Response struct ---

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: String,
    pub intervals: Vec<i32>,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Note: `strategy` is String in the response because sqlx maps TEXT columns to String.
// The enum is used in request bodies for validation; the DB value is trusted on read.

// --- Request structs ---

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(length(min = 1, max = 100))]
    pub intervals: Vec<i32>,
    #[validate(range(min = 1, max = 100))]
    pub max_attempts: i32,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePut {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(length(min = 1, max = 100))]
    pub intervals: Vec<i32>,
    #[validate(range(min = 1, max = 100))]
    pub max_attempts: i32,
}

// Query struct used by list, get, edit, delete — org_id always required
#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}

// --- Per-org schedule limit ---
const MAX_RETRY_SCHEDULES_PER_ORG: i64 = 50;
```

Note: `RetryStrategy` enum replaces string validation. Serde handles deserialization — invalid values return a 400 automatically. The `validate_intervals` custom function is no longer needed since DB CHECK constraint handles bounds and the `validator` length attribute handles array length. Interval element bounds (1..=604800) are enforced by the DB CHECK constraint `1 <= ALL(intervals) AND 604800 >= ALL(intervals)`.

- [ ] **Step 3: Implement `list` handler**

Append to `api/src/handlers/retry_schedules.rs`:

```rust
#[api_v2_operation(
    summary = "List retry schedules",
    description = "List all retry schedules for an organization",
    operation_id = "retry_schedules.list",
    tags(retry_schedules)
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<Vec<RetrySchedule>>, Hook0Problem> {
    let org_id = qs.organization_id;

    if authorize(
        &biscuit,
        Some(org_id),
        Action::RetryScheduleList,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let schedules = query_as!(
        RetrySchedule,
        "
            select
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
            from webhook.retry_schedule
            where organization__id = $1
            order by created_at asc
        ",
        &org_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(schedules))
}
```

- [ ] **Step 4: Implement `create` handler**

Append to `api/src/handlers/retry_schedules.rs`:

```rust
#[api_v2_operation(
    summary = "Create a retry schedule",
    description = "Create a new retry schedule for an organization",
    operation_id = "retry_schedules.create",
    tags(retry_schedules)
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<RetrySchedulePost>,
) -> Result<CreatedJson<RetrySchedule>, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let org_id = body.organization_id;

    if authorize(
        &biscuit,
        Some(org_id),
        Action::RetryScheduleCreate,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // Per-org schedule count limit
    let count = sqlx::query_scalar!(
        "select count(*) from webhook.retry_schedule where organization__id = $1",
        &org_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .unwrap_or(0);

    if count >= MAX_RETRY_SCHEDULES_PER_ORG {
        return Err(Hook0Problem::RetryScheduleInUse); // TODO: add a proper TooManyRetrySchedules variant
    }

    let strategy_str = body.strategy.to_string();

    let schedule = query_as!(
        RetrySchedule,
        "
            insert into webhook.retry_schedule (organization__id, name, strategy, intervals, max_attempts)
            values ($1, $2, $3, $4, $5)
            returning
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
        ",
        &org_id,
        &body.name,
        &strategy_str,
        &body.intervals,
        &body.max_attempts,
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleCreated {
            organization_id: schedule.organization_id,
            retry_schedule_id: schedule.retry_schedule_id,
            name: schedule.name.clone(),
            strategy: schedule.strategy.clone(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(schedule))
}
```

- [ ] **Step 5: Implement `get` handler**

Auth-first pattern (service_token style). `organization_id` required as query param. SQL scoped to both IDs.

```rust
#[api_v2_operation(
    summary = "Get a retry schedule",
    description = "Get a retry schedule by ID",
    operation_id = "retry_schedules.get",
    tags(retry_schedules)
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let org_id = qs.organization_id;

    if authorize(
        &biscuit,
        Some(org_id),
        Action::RetryScheduleGet,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let schedule = query_as!(
        RetrySchedule,
        "
            select
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
            from webhook.retry_schedule
            where retry_schedule__id = $1
              and organization__id = $2
        ",
        schedule_id.as_ref(),
        &org_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    Ok(Json(schedule))
}
```

- [ ] **Step 6: Implement `edit` handler**

Auth-first, SQL scoped, no TOCTOU.

```rust
#[api_v2_operation(
    summary = "Update a retry schedule",
    description = "Update a retry schedule by ID",
    operation_id = "retry_schedules.edit",
    tags(retry_schedules)
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let org_id = qs.organization_id;

    if authorize(
        &biscuit,
        Some(org_id),
        Action::RetryScheduleEdit,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let strategy_str = body.strategy.to_string();

    let schedule = query_as!(
        RetrySchedule,
        "
            update webhook.retry_schedule
            set name = $3, strategy = $4, intervals = $5, max_attempts = $6,
                updated_at = statement_timestamp()
            where retry_schedule__id = $1
              and organization__id = $2
            returning
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name, strategy, intervals, max_attempts, created_at, updated_at
        ",
        schedule_id.as_ref(),
        &org_id,
        &body.name,
        &strategy_str,
        &body.intervals,
        &body.max_attempts,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleUpdated {
            organization_id: schedule.organization_id,
            retry_schedule_id: schedule.retry_schedule_id,
            name: schedule.name.clone(),
            strategy: schedule.strategy.clone(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(Json(schedule))
}
```

- [ ] **Step 7: Implement `delete` handler**

Auth-first. Atomic delete with NOT EXISTS check (no TOCTOU).

```rust
#[api_v2_operation(
    summary = "Delete a retry schedule",
    description = "Delete a retry schedule by ID. Fails if still assigned to subscriptions.",
    operation_id = "retry_schedules.delete",
    tags(retry_schedules)
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<()>, Hook0Problem> {
    let org_id = qs.organization_id;

    if authorize(
        &biscuit,
        Some(org_id),
        Action::RetryScheduleDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // Atomic: delete only if not assigned to any active subscription
    let result = sqlx::query!(
        "
            delete from webhook.retry_schedule
            where retry_schedule__id = $1
              and organization__id = $2
              and not exists (
                  select 1 from webhook.subscription
                  where retry_schedule__id = $1
                    and deleted_at is null
              )
        ",
        schedule_id.as_ref(),
        &org_id,
    )
    .execute(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if result.rows_affected() == 0 {
        // Disambiguate: does the schedule exist?
        let exists = sqlx::query_scalar!(
            "select exists(
                select 1 from webhook.retry_schedule
                where retry_schedule__id = $1 and organization__id = $2
            ) as \"exists!\"",
            schedule_id.as_ref(),
            &org_id,
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        if exists {
            return Err(Hook0Problem::RetryScheduleInUse);
        } else {
            return Err(Hook0Problem::NotFound);
        }
    }

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleRemoved {
            organization_id: org_id,
            retry_schedule_id: *schedule_id,
            name: String::new(), // Schedule already deleted; name not available
            strategy: String::new(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(Json(()))
}
```

Note on delete event: since the row is already deleted when we fire the event, name/strategy are unavailable. If audit trail requires full data, refactor to SELECT + DELETE in a transaction instead.

- [ ] **Step 8: Verify compilation**

Run: `cd api && cargo check`
Expected: Compiles (requires Tasks 4 and 5 for routes and hook0_client types)

- [ ] **Step 9: Commit**

```bash
git add api/src/handlers/retry_schedules.rs api/src/handlers/mod.rs
git commit -m "feat(api): add retry_schedules CRUD handler"
```

---

## Task 4: Register Routes in main.rs

**Files:**
- Modify: `api/src/main.rs` (route registration around lines 1404-1418)

- [ ] **Step 1: Add retry_schedules route scope**

Add a new `.service()` block adjacent to the existing `/subscriptions` scope (after line ~1418 in `api/src/main.rs`):

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

- [ ] **Step 2: Verify compilation**

Run: `cd api && cargo check`
Expected: Compiles (after Task 5 for hook0_client types)

- [ ] **Step 3: Commit**

```bash
git add api/src/main.rs
git commit -m "feat(api): register /retry_schedules routes"
```

---

## Task 5: Hook0 Client Events

**Files:**
- Modify: `api/src/hook0_client.rs`

- [ ] **Step 1: Add event type strings to EVENT_TYPES array**

In `api/src/hook0_client.rs`, add to the `EVENT_TYPES` const array (around line 39, before the closing `];`):

```rust
    "api.retry_schedule.created",
    "api.retry_schedule.updated",
    "api.retry_schedule.removed",
```

- [ ] **Step 2: Add event structs**

Add these structs following the existing pattern (after the subscription event structs, around line ~730):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct EventRetryScheduleCreated {
    pub organization_id: Uuid,
    pub retry_schedule_id: Uuid,
    pub name: String,
    pub strategy: String,
}

impl Event for EventRetryScheduleCreated {
    fn event_type(&self) -> &'static str {
        "api.retry_schedule.created"
    }
    fn labels(&self) -> Vec<(String, String)> {
        vec![
            (INSTANCE_LABEL.to_owned(), INSTANCE_VALUE.to_owned()),
            (ORGANIZATION_LABEL.to_owned(), self.organization_id.to_string()),
        ]
    }
}

impl From<EventRetryScheduleCreated> for Hook0ClientEvent {
    fn from(e: EventRetryScheduleCreated) -> Self {
        Self::RetryScheduleCreated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventRetryScheduleUpdated {
    pub organization_id: Uuid,
    pub retry_schedule_id: Uuid,
    pub name: String,
    pub strategy: String,
}

impl Event for EventRetryScheduleUpdated {
    fn event_type(&self) -> &'static str {
        "api.retry_schedule.updated"
    }
    fn labels(&self) -> Vec<(String, String)> {
        vec![
            (INSTANCE_LABEL.to_owned(), INSTANCE_VALUE.to_owned()),
            (ORGANIZATION_LABEL.to_owned(), self.organization_id.to_string()),
        ]
    }
}

impl From<EventRetryScheduleUpdated> for Hook0ClientEvent {
    fn from(e: EventRetryScheduleUpdated) -> Self {
        Self::RetryScheduleUpdated(e)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventRetryScheduleRemoved {
    pub organization_id: Uuid,
    pub retry_schedule_id: Uuid,
    pub name: String,
    pub strategy: String,
}

impl Event for EventRetryScheduleRemoved {
    fn event_type(&self) -> &'static str {
        "api.retry_schedule.removed"
    }
    fn labels(&self) -> Vec<(String, String)> {
        vec![
            (INSTANCE_LABEL.to_owned(), INSTANCE_VALUE.to_owned()),
            (ORGANIZATION_LABEL.to_owned(), self.organization_id.to_string()),
        ]
    }
}

impl From<EventRetryScheduleRemoved> for Hook0ClientEvent {
    fn from(e: EventRetryScheduleRemoved) -> Self {
        Self::RetryScheduleRemoved(e)
    }
}
```

- [ ] **Step 3: Add variants to Hook0ClientEvent enum**

In the `Hook0ClientEvent` enum (around line 134), add:

```rust
    RetryScheduleCreated(EventRetryScheduleCreated),
    RetryScheduleUpdated(EventRetryScheduleUpdated),
    RetryScheduleRemoved(EventRetryScheduleRemoved),
```

- [ ] **Step 4: Add match arms in `mk_hook0_event` method**

Find the `impl Hook0ClientEvent` block and its `mk_hook0_event` method (around line 137-186). Add match arms at the end of the `match self { ... }` block:

```rust
            Self::RetryScheduleCreated(e) => to_event(e, None),
            Self::RetryScheduleUpdated(e) => to_event(e, None),
            Self::RetryScheduleRemoved(e) => to_event(e, None),
```

- [ ] **Step 5: Verify compilation**

Run: `cd api && cargo check`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add api/src/hook0_client.rs
git commit -m "feat(api): add retry_schedule hook0 client events"
```

---

## Task 6: Modify Subscription Handler

**Files:**
- Modify: `api/src/handlers/subscriptions.rs`

- [ ] **Step 1: Add `retry_schedule_id` to `Subscription` response struct**

In `api/src/handlers/subscriptions.rs`, add to the `Subscription` struct (around line 31-49):

```rust
    pub retry_schedule_id: Option<Uuid>,
```

- [ ] **Step 2: Add `retry_schedule_id` to `SubscriptionPost` request struct**

In the `SubscriptionPost` struct (around line 445-467), add:

```rust
    pub retry_schedule_id: Option<Uuid>,
```

Note: `SubscriptionPost` is reused for both create and edit. This field applies to both operations.

- [ ] **Step 3: Update all SQL queries that SELECT from subscription**

Every `list`, `get`, `create`, `edit` query that returns a `Subscription` needs to include:
```sql
s.retry_schedule__id as retry_schedule_id,
```

Find each `query_as!(Subscription, ...)` call and add the column to the SELECT.

- [ ] **Step 4: Update `create` INSERT with SQL-level cross-org enforcement**

In the `create` handler's INSERT query, use a subquery to enforce same-org ownership atomically (no TOCTOU):

```sql
insert into webhook.subscription (..., retry_schedule__id)
values (..., (
    select retry_schedule__id from webhook.retry_schedule
    where retry_schedule__id = $N
      and organization__id = (
          select organization__id from event.application
          where application__id = $APP_ID and deleted_at is null
      )
))
```

If `retry_schedule_id` is `None`, pass `NULL` directly. If the subquery returns NULL (schedule doesn't exist or wrong org), the column gets NULL — detect this and return an error:

```rust
if body.retry_schedule_id.is_some() && created.retry_schedule_id.is_none() {
    return Err(Hook0Problem::NotFound); // schedule not found or cross-org
}
```

- [ ] **Step 5: Update `edit` UPDATE with same cross-org enforcement**

Same pattern as create for the UPDATE query.

- [ ] **Step 6: Verify compilation**

Run: `cd api && cargo check`
Expected: Compiles

- [ ] **Step 7: Commit**

```bash
git add api/src/handlers/subscriptions.rs
git commit -m "feat(api): add retry_schedule_id to subscription CRUD"
```

---

## Task 7: Modify Worker Retry Logic

**Files:**
- Modify: `output-worker/src/main.rs` (lines 262-268 constants, lines 285-302 struct, lines 769-843 retry functions)
- Modify: `output-worker/src/pg.rs` (lines 46-92 fetch query)
- Modify: `output-worker/src/pulsar.rs` (status query + retry call site)

- [ ] **Step 1: Add `ScheduleConfig` struct and schedule fields to `RequestAttemptWithOptionalPayload`**

In `output-worker/src/main.rs`, add:

```rust
/// Retry schedule configuration resolved from the subscription's assigned schedule.
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub strategy: String,
    pub intervals: Vec<i32>,
    pub max_attempts: i32,
}
```

Add to `RequestAttemptWithOptionalPayload` (around line 285-302):

```rust
    // Retry schedule fields (from LEFT JOIN)
    pub retry_strategy: Option<String>,
    pub retry_intervals: Option<Vec<i32>>,
    pub retry_max_attempts: Option<i32>,
```

Add a helper method:

```rust
impl RequestAttemptWithOptionalPayload {
    pub fn schedule_config(&self) -> Option<ScheduleConfig> {
        match (&self.retry_strategy, &self.retry_intervals, &self.retry_max_attempts) {
            (Some(strategy), Some(intervals), Some(max_attempts)) => Some(ScheduleConfig {
                strategy: strategy.clone(),
                intervals: intervals.clone(),
                max_attempts: *max_attempts,
            }),
            _ => None,
        }
    }
}
```

- [ ] **Step 2: Update pg.rs fetch query to JOIN retry_schedule**

In `output-worker/src/pg.rs`, modify the main SELECT query (lines 46-92):

Add to SELECT columns:
```sql
            rs.strategy as retry_strategy,
            rs.intervals as retry_intervals,
            rs.max_attempts as retry_max_attempts,
```

Add to JOIN clauses:
```sql
        left join webhook.retry_schedule as rs on rs.retry_schedule__id = s.retry_schedule__id
```

- [ ] **Step 3: Write `compute_delay_from_schedule` function**

In `output-worker/src/main.rs`, add a new function:

```rust
fn compute_delay_from_schedule(config: &ScheduleConfig, retry_count: i16) -> Option<Duration> {
    let count = i32::from(retry_count);
    if count >= config.max_attempts {
        return None; // exhausted
    }
    if config.intervals.is_empty() {
        return None;
    }

    let delay_secs = match config.strategy.as_str() {
        "linear" => config.intervals[0],
        "exponential" | "custom" => {
            let idx = (count as usize).min(config.intervals.len() - 1);
            config.intervals[idx]
        }
        unknown => {
            warn!("Unknown retry strategy: {unknown}, giving up");
            return None;
        }
    };

    Some(Duration::from_secs(u64::from(delay_secs.max(0) as u32)))
}
```

- [ ] **Step 4: Modify `compute_next_retry` to use `ScheduleConfig`**

Change the signature of `compute_next_retry`:

```rust
async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_fast_retries: u32,
    max_slow_retries: u32,
    schedule: Option<&ScheduleConfig>,
) -> Result<Option<Duration>, sqlx::Error> {
```

In the retry computation branch (after subscription check), replace the call to `compute_next_retry_duration`:

```rust
if sub.is_some() {
    match schedule {
        Some(config) => Ok(compute_delay_from_schedule(config, attempt.retry_count)),
        None => Ok(compute_next_retry_duration(max_fast_retries, max_slow_retries, attempt.retry_count)),
    }
} else {
    Ok(None)
}
```

- [ ] **Step 5: Update call sites in pg.rs**

In `output-worker/src/pg.rs`, where `compute_next_retry` is called (around line 292), extract and pass the schedule:

```rust
let schedule_config = next_attempt.schedule_config();
compute_next_retry(
    &mut tx,
    &attempt,
    &response,
    config.max_fast_retries,
    config.max_slow_retries,
    schedule_config.as_ref(),
)
```

- [ ] **Step 6: Update Pulsar mode for schedule support**

In Pulsar mode, the attempt payload comes from the Pulsar message, but there IS a lightweight DB query at `output-worker/src/pulsar.rs:399-440` that checks attempt status via JOINs on `webhook.subscription` and `event.application`.

Two changes:

1. **Add LEFT JOIN to the status check query** (around line 399): Add `left join webhook.retry_schedule as rs on rs.retry_schedule__id = s.retry_schedule__id` and select `rs.strategy as retry_strategy`, `rs.intervals as retry_intervals`, `rs.max_attempts as retry_max_attempts`. Extend `RawRequestAttemptStatus` to carry these fields.

2. **Build `ScheduleConfig` from status query result** and pass to `compute_next_retry`:

```rust
let schedule_config = match (&status.retry_strategy, &status.retry_intervals, &status.retry_max_attempts) {
    (Some(strategy), Some(intervals), Some(max_attempts)) => Some(ScheduleConfig {
        strategy: strategy.clone(),
        intervals: intervals.clone(),
        max_attempts: *max_attempts,
    }),
    _ => None,
};
```

- [ ] **Step 7: Verify compilation**

Run: `cd output-worker && cargo check`
Expected: Compiles

- [ ] **Step 8: Commit**

```bash
git add output-worker/src/main.rs output-worker/src/pg.rs output-worker/src/pulsar.rs
git commit -m "feat(worker): support configurable retry schedules"
```

---

## Task 8: Full Build Verification

- [ ] **Step 1: Build the entire workspace**

Run: `cargo build`
Expected: All crates compile

- [ ] **Step 2: Run existing tests**

Run: `cargo test`
Expected: All existing tests pass (no regressions)

- [ ] **Step 3: Run sqlx prepare if needed**

Run: `cd api && cargo sqlx prepare && cd ../output-worker && cargo sqlx prepare`
Expected: sqlx offline query data updated

- [ ] **Step 4: Commit sqlx metadata if changed**

```bash
git add -A .sqlx/ */.sqlx/
git commit -m "chore: update sqlx offline query metadata"
```

---

## Task 9: Create Feature Branch and Squash

- [ ] **Step 1: Create feature branch from current work**

All commits should be on a feature branch `feat/retry-schedule` for the MR. If not already on a branch, create one and cherry-pick/rebase the commits.

- [ ] **Step 2: Verify all changes**

Run: `git log --oneline master..HEAD`
Expected: Clear sequence of commits covering migration, IAM, handler, routes, hook0 events, subscription modification, worker logic

---

## Notes for Implementation

- **sqlx compile-time checking**: All queries are checked at compile time. Ensure the migration has been applied to the dev database before running `cargo check`.
- **Biscuit Datalog rules**: When adding `Action::RetrySchedule*` variants, study the existing `ServiceToken*` rules in `authorize()` as a model — they are org-scoped without application_id, same as retry schedules.
- **`SubscriptionPost` reuse**: The codebase reuses `SubscriptionPost` for both create and edit. The `retry_schedule_id` field added in Task 6 applies to both operations.
- **`RetryStrategy` enum**: Used in request bodies for type-safe deserialization. In response structs and DB, strategy remains `String` (sqlx maps TEXT to String). The enum is the write-side guard; the DB CHECK constraint is the storage-side guard.
- **Per-org limit**: `MAX_RETRY_SCHEDULES_PER_ORG = 50`. Adjust if needed. Consider adding a proper `TooManyRetrySchedulesPerOrganization` variant to `Hook0Problem` (following `TooManySubscriptionsPerApplication` pattern) instead of reusing `RetryScheduleInUse`.
- **SQL convention**: All SQL in migrations and queries uses lowercase keywords, matching existing codebase style.
