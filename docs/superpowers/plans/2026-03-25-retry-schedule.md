# Configurable Retry Schedule — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow organizations to define named retry schedules (exponential, linear, custom) and assign them to subscriptions, falling back to the existing Svix/Stripe default when none is assigned.

**Architecture:** New `webhook.retry_schedule` table with CRUD API following existing flat-scope pattern. Worker's `compute_next_retry` modified to resolve the subscription's schedule via JOIN and compute delay per strategy. Hook0 client events for observability.

**Tech Stack:** Rust, actix-web, paperclip (OpenAPI), sqlx (Postgres), clap, Biscuit auth, validator crate

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

CREATE TABLE webhook.retry_schedule (
    retry_schedule__id UUID NOT NULL DEFAULT gen_random_uuid(),
    organization__id UUID NOT NULL REFERENCES iam.organization(organization__id),
    name TEXT NOT NULL CHECK (length(name) >= 1),
    strategy TEXT NOT NULL CHECK (strategy IN ('exponential', 'linear', 'custom')),
    intervals INTEGER[] NOT NULL CHECK (array_length(intervals, 1) > 0),
    max_attempts INTEGER NOT NULL DEFAULT 8 CHECK (max_attempts > 0 AND max_attempts <= 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    CONSTRAINT retry_schedule_pkey PRIMARY KEY (retry_schedule__id),
    CONSTRAINT retry_schedule_org_name_unique UNIQUE (organization__id, name)
);

ALTER TABLE webhook.subscription
    ADD COLUMN retry_schedule__id UUID REFERENCES webhook.retry_schedule(retry_schedule__id);

CREATE INDEX subscription_retry_schedule_idx ON webhook.subscription(retry_schedule__id);
```

- [ ] **Step 2: Write down migration**

```sql
-- api/migrations/20260325120000_add_retry_schedule.down.sql

DROP INDEX IF EXISTS webhook.subscription_retry_schedule_idx;
ALTER TABLE webhook.subscription DROP COLUMN IF EXISTS retry_schedule__id;
DROP TABLE IF EXISTS webhook.retry_schedule;
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

Then add the corresponding Datalog rules in the `authorize()` function's authorizer builder. Follow the pattern of `ServiceToken*` actions (org-scoped, no application_id). The rules should check `right("retry_schedule", "list")`, etc. Look at how `ServiceTokenList` / `ServiceTokenCreate` rules are structured and replicate.

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
Expected: Compiles (new Action variants may have unused warnings — that's OK, they'll be used in Task 3)

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

- [ ] **Step 2: Define structs in retry_schedules.rs**

Create `api/src/handlers/retry_schedules.rs`:

```rust
use actix_web::web::{self, Data, Json, Path, Query, ReqData};
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
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

// --- Request structs ---

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(custom(function = "validate_strategy"))]
    pub strategy: String,
    #[validate(length(min = 1, max = 100), custom(function = "validate_intervals"))]
    pub intervals: Vec<i32>,
    #[validate(range(min = 1, max = 100))]
    pub max_attempts: i32,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePut {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(custom(function = "validate_strategy"))]
    pub strategy: String,
    #[validate(length(min = 1, max = 100), custom(function = "validate_intervals"))]
    pub intervals: Vec<i32>,
    #[validate(range(min = 1, max = 100))]
    pub max_attempts: i32,
}

#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryScheduleListQuery {
    pub organization_id: Uuid,
}

// --- Validation functions ---

fn validate_strategy(strategy: &str) -> Result<(), validator::ValidationError> {
    match strategy {
        "exponential" | "linear" | "custom" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_strategy")),
    }
}

fn validate_intervals(intervals: &[i32]) -> Result<(), validator::ValidationError> {
    if intervals.iter().any(|&i| i < 1 || i > 604800) {
        return Err(validator::ValidationError::new("invalid_interval_value"));
    }
    Ok(())
}
```

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
    qs: Query<RetryScheduleListQuery>,
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
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
            FROM webhook.retry_schedule
            WHERE organization__id = $1
            ORDER BY created_at ASC
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
) -> Result<Json<RetrySchedule>, Hook0Problem> {
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

    let schedule = query_as!(
        RetrySchedule,
        "
            INSERT INTO webhook.retry_schedule (organization__id, name, strategy, intervals, max_attempts)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
        ",
        &org_id,
        &body.name,
        &body.strategy,
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

    Ok(Json(schedule))
}
```

- [ ] **Step 5: Implement `get` handler**

Append to `api/src/handlers/retry_schedules.rs`:

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
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let schedule = query_as!(
        RetrySchedule,
        "
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name,
                strategy,
                intervals,
                max_attempts,
                created_at,
                updated_at
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
        ",
        schedule_id.as_ref()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    if authorize(
        &biscuit,
        Some(schedule.organization_id),
        Action::RetryScheduleGet,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    Ok(Json(schedule))
}
```

- [ ] **Step 6: Implement `edit` handler**

Append to `api/src/handlers/retry_schedules.rs`:

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
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    // Fetch first to check org access
    let existing = query_as!(
        RetrySchedule,
        "
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, intervals, max_attempts, created_at, updated_at
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
        ",
        schedule_id.as_ref()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    if authorize(
        &biscuit,
        Some(existing.organization_id),
        Action::RetryScheduleEdit,
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
            UPDATE webhook.retry_schedule
            SET name = $2, strategy = $3, intervals = $4, max_attempts = $5,
                updated_at = statement_timestamp()
            WHERE retry_schedule__id = $1
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, intervals, max_attempts, created_at, updated_at
        ",
        schedule_id.as_ref(),
        &body.name,
        &body.strategy,
        &body.intervals,
        &body.max_attempts,
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

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

Append to `api/src/handlers/retry_schedules.rs`:

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
) -> Result<Json<()>, Hook0Problem> {
    let existing = query_as!(
        RetrySchedule,
        "
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, intervals, max_attempts, created_at, updated_at
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
        ",
        schedule_id.as_ref()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    if authorize(
        &biscuit,
        Some(existing.organization_id),
        Action::RetryScheduleDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // Check no subscriptions are using this schedule
    let usage_count = sqlx::query_scalar!(
        "
            SELECT COUNT(*) AS count
            FROM webhook.subscription
            WHERE retry_schedule__id = $1
              AND deleted_at IS NULL
        ",
        schedule_id.as_ref()
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if usage_count.unwrap_or(0) > 0 {
        return Err(Hook0Problem::RetryScheduleInUse);
    }

    sqlx::query!(
        "DELETE FROM webhook.retry_schedule WHERE retry_schedule__id = $1",
        schedule_id.as_ref()
    )
    .execute(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleRemoved {
            organization_id: existing.organization_id,
            retry_schedule_id: existing.retry_schedule_id,
            name: existing.name.clone(),
            strategy: existing.strategy.clone(),
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

- [ ] **Step 8: Verify compilation**

Run: `cd api && cargo check`
Expected: May fail on missing imports/types (Hook0Problem::Conflict, hook0_client events) — those are created in Tasks 3 and 4. Verify the handler module itself has no syntax errors.

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
Expected: Compiles (after Task 4 for hook0_client types)

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

- [ ] **Step 3: Update all SQL queries that SELECT from subscription**

Every `list`, `get`, `create`, `edit` query that returns a `Subscription` needs to include:
```sql
s.retry_schedule__id AS retry_schedule_id,
```

Find each `query_as!(Subscription, ...)` call and add the column to the SELECT.

- [ ] **Step 4: Update `create` INSERT to include retry_schedule_id**

In the `create` handler's INSERT query, add the column:
```sql
INSERT INTO webhook.subscription (..., retry_schedule__id)
VALUES (..., $N)
```

Add validation: if `retry_schedule_id` is provided, verify it belongs to the same org:
```rust
if let Some(schedule_id) = &body.retry_schedule_id {
    let schedule_org = sqlx::query_scalar!(
        "SELECT organization__id FROM webhook.retry_schedule WHERE retry_schedule__id = $1",
        schedule_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // Compare with the subscription's application org
    let app_org = sqlx::query_scalar!(
        "SELECT organization__id FROM event.application WHERE application__id = $1 AND deleted_at IS NULL",
        &body.application_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    if schedule_org != app_org {
        return Err(Hook0Problem::Forbidden);
    }
}
```

- [ ] **Step 5: Update `edit` UPDATE to include retry_schedule_id**

In the `edit` handler's UPDATE query, add:
```sql
retry_schedule__id = $N,
```

Same org validation as create.

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
- Modify: `output-worker/src/pulsar.rs` (where compute_next_retry is called)

- [ ] **Step 1: Add schedule fields to `RequestAttemptWithOptionalPayload`**

In `output-worker/src/main.rs`, add to the struct (around line 285-302):

```rust
    // Retry schedule fields (from LEFT JOIN)
    pub retry_strategy: Option<String>,
    pub retry_intervals: Option<Vec<i32>>,
    pub retry_max_attempts: Option<i32>,
```

- [ ] **Step 2: Update pg.rs fetch query to JOIN retry_schedule**

In `output-worker/src/pg.rs`, modify the main SELECT query (lines 46-92):

Add to SELECT columns:
```sql
            rs.strategy AS retry_strategy,
            rs.intervals AS retry_intervals,
            rs.max_attempts AS retry_max_attempts,
```

Add to JOIN clauses:
```sql
        LEFT JOIN webhook.retry_schedule AS rs ON rs.retry_schedule__id = s.retry_schedule__id
```

- [ ] **Step 3: Write `compute_delay_from_schedule` function**

In `output-worker/src/main.rs`, add a new function:

```rust
fn compute_delay_from_schedule(
    strategy: &str,
    intervals: &[i32],
    max_attempts: i32,
    retry_count: i16,
) -> Option<Duration> {
    let count = retry_count as i32;
    if count >= max_attempts {
        return None; // exhausted
    }
    if intervals.is_empty() {
        return None;
    }

    let delay_secs = match strategy {
        "linear" => intervals[0],
        // "exponential" and "custom" both use index-based lookup
        _ => {
            let idx = (count as usize).min(intervals.len() - 1);
            intervals[idx]
        }
    };

    Some(Duration::from_secs(delay_secs.max(0) as u64))
}
```

- [ ] **Step 4: Modify `compute_next_retry` to use schedule data**

Change the signature of `compute_next_retry` to accept schedule info:

```rust
async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_fast_retries: u32,
    max_slow_retries: u32,
    retry_strategy: Option<&str>,
    retry_intervals: Option<&[i32]>,
    retry_max_attempts: Option<i32>,
) -> Result<Option<Duration>, sqlx::Error> {
```

In the retry computation branch (after subscription check), replace the call to `compute_next_retry_duration`:

```rust
if sub.is_some() {
    match (retry_strategy, retry_intervals, retry_max_attempts) {
        (Some(strategy), Some(intervals), Some(max_attempts)) => {
            Ok(compute_delay_from_schedule(strategy, intervals, max_attempts, attempt.retry_count))
        }
        _ => {
            // No custom schedule — use existing default
            Ok(compute_next_retry_duration(max_fast_retries, max_slow_retries, attempt.retry_count))
        }
    }
} else {
    Ok(None)
}
```

- [ ] **Step 5: Update call sites in pg.rs**

In `output-worker/src/pg.rs`, where `compute_next_retry` is called (around line 292), pass the schedule fields from the fetched attempt:

```rust
compute_next_retry(
    &mut tx,
    &attempt,
    &response,
    config.max_fast_retries,
    config.max_slow_retries,
    next_attempt.retry_strategy.as_deref(),
    next_attempt.retry_intervals.as_deref(),
    next_attempt.retry_max_attempts,
)
```

- [ ] **Step 6: Update Pulsar mode for schedule support**

In Pulsar mode, the attempt payload comes from the Pulsar message (not a DB query), but there IS a lightweight DB query at `output-worker/src/pulsar.rs:399-440` that checks attempt status. This query JOINs `webhook.subscription` and `event.application`.

Two changes needed:

1. **Add LEFT JOIN to the status check query** (around line 399): Add `LEFT JOIN webhook.retry_schedule AS rs ON rs.retry_schedule__id = s.retry_schedule__id` and select `rs.strategy`, `rs.intervals`, `rs.max_attempts` in the SELECT clause. Extend the `RawRequestAttemptStatus` struct to carry these fields.

2. **Pass schedule fields to `compute_next_retry`** at the call site (around line 580): Extract the schedule fields from the status query result and pass them through. The schedule data flows: status query → local variables → `compute_next_retry()` call.

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
Expected: Clear sequence of commits covering migration, handler, routes, hook0 events, subscription modification, worker logic

---

## Notes for Implementation

- **sqlx compile-time checking**: All queries are checked at compile time. Ensure the migration has been applied to the dev database before running `cargo check`.
- **Biscuit Datalog rules**: When adding `Action::RetrySchedule*` variants, study the existing `ServiceToken*` rules in `authorize()` as a model — they are org-scoped without application_id, same as retry schedules.
- **`SubscriptionPost` reuse**: The codebase reuses `SubscriptionPost` for both create and edit. The `retry_schedule_id` field added in Task 6 applies to both operations.
