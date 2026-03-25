# Phase 1: Configurable Retry Schedule

**Ticket**: [#42 — Customizable delay algorithm strategy on retry](https://gitlab.com/hook0/hook0/-/work_items/42)
**Date**: 2026-03-25
**Status**: Draft

---

## 1. Goal

Add configurable retry schedules on top of the existing default retry policy (commit `5b44bbc8` by David Sferruzza: `3s, 10s, 3min, 30min, 1h, 3h, 5h, 10h repeating`, max 25 retries, window 8d). Organizations can define named retry policies with one of three strategies and assign them to subscriptions. Subscriptions without a schedule keep the worker's default.

This is Phase 1 of ticket #42. Phases 2-4 (auto-deactivation, email notifications, manual retry/recovery) are out of scope.

## 2. Scope

### In scope

- New `webhook.retry_schedule` table with three strategies (exponential, linear, custom)
- CRUD API for retry schedules (org-level, flat route `/retry_schedules`)
- `retry_schedule__id` FK on `webhook.subscription` (nullable, ON DELETE SET NULL)
- Worker uses assigned schedule or falls back to David's default
- Hook0 client operational events for schedule CRUD (full payload)
- Unit tests for delay computation + integration tests (HTTP, actix_web::test)

### Out of scope

- Automatic endpoint deactivation (Phase 2)
- Email notifications for failing endpoints (Phase 3)
- Manual retry/recover/replay APIs (Phase 4)
- Frontend UI for retry schedule management

## 3. Design

### 3.1. Strategies

Three strategies, each with different fields:

| Strategy | `custom_intervals` | `linear_delay` | Delay computation |
|---|---|---|---|
| **exponential** | NULL | NULL | Uses David's hardcoded table (`3s, 10s, 3min, 30min, 1h, 3h, 5h, 10h...`), gated by `max_retries` from the schedule |
| **linear** | NULL | required (1..=MAX_INTERVAL_SECONDS) | Fixed delay = `linear_delay` at every retry, gated by `max_retries` |
| **custom** | required, `len == max_retries` | NULL | `custom_intervals[retry_count]` — the array is the source of truth |

- `exponential` and `custom` share the same index-based lookup in the worker, but `exponential` uses the hardcoded table while `custom` uses the user-provided array.
- The `strategy` field is both a semantic label (for UI/UX) and a behavioral discriminator (controls which fields are required and how the delay is computed).
- `MAX_INTERVAL_SECONDS = 604800` (1 week) — defined as a Rust constant, also enforced in the DB CHECK constraint. Both must be updated together.

### 3.2. Database Schema

#### New table: `webhook.retry_schedule`

| Column | Type | Constraints |
|---|---|---|
| `retry_schedule__id` | UUID | PK, DEFAULT public.gen_random_uuid() |
| `organization__id` | UUID | FK → iam.organization (ON DELETE CASCADE ON UPDATE CASCADE), NOT NULL |
| `name` | TEXT | NOT NULL, CHECK length >= 1 |
| `strategy` | TEXT | NOT NULL, CHECK IN ('exponential', 'linear', 'custom') |
| `max_retries` | INTEGER | NOT NULL, CHECK > 0 AND <= 100 |
| `custom_intervals` | INTEGER[] | Nullable — required only for `custom` strategy |
| `linear_delay` | INTEGER | Nullable — required only for `linear` strategy |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT statement_timestamp() |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT statement_timestamp() |

- UNIQUE constraint on `(organization__id, name)`
- Named CHECK constraint `retry_schedule_strategy_fields_check` enforcing cross-field validation:
  - `exponential`: `custom_intervals IS NULL AND linear_delay IS NULL`
  - `linear`: `custom_intervals IS NULL AND linear_delay IS NOT NULL AND linear_delay >= 1 AND linear_delay <= MAX_INTERVAL_SECONDS`
  - `custom`: `linear_delay IS NULL AND custom_intervals IS NOT NULL AND array_length(custom_intervals, 1) = max_retries AND 1 <= ALL(custom_intervals) AND MAX_INTERVAL_SECONDS >= ALL(custom_intervals)`
- `updated_at` maintained at application level (explicit `SET updated_at = statement_timestamp()` in UPDATE handler)

#### Alter: `webhook.subscription`

- ADD `retry_schedule__id UUID` (nullable)
- FK → `webhook.retry_schedule(retry_schedule__id)` ON DELETE SET NULL ON UPDATE CASCADE
- No index — the usage queries are rare (delete handler) and the UNIQUE constraint on `(organization__id, name)` covers list queries

#### Naming: `max_retries` not `max_attempts`

Aligned with David's worker code where `retry_count = 0` is the initial delivery attempt and `retry_count = 1` is the first retry. The comparison `retry_count < max_retries` works directly with no off-by-one. `max_retries = 25` means 26 total delivery attempts (1 initial + 25 retries).

### 3.3. API Routes

All routes require Biscuit authentication.

- **Viewer** role can list and get (read-only)
- **Editor** role required for create, edit, delete

| Method | Path | Description | Role |
|---|---|---|---|
| POST | `/api/v1/retry_schedules` | Create a schedule | Editor |
| GET | `/api/v1/retry_schedules` | List schedules (query param `organization_id`) | Viewer |
| GET | `/api/v1/retry_schedules/{schedule_id}` | Get a schedule (query param `organization_id`) | Viewer |
| PUT | `/api/v1/retry_schedules/{schedule_id}` | Update a schedule (query param `organization_id`) | Editor |
| DELETE | `/api/v1/retry_schedules/{schedule_id}` | Delete a schedule (query param `organization_id`) | Editor |

All item routes (get, edit, delete) require `organization_id` as query parameter. Auth is checked first, then the SQL query is scoped to both `retry_schedule__id` and `organization__id` (no fetch-before-auth, no IDOR).

#### Create request bodies

```json
// Exponential (uses David's hardcoded table, custom max_retries)
{
    "organization_id": "uuid",
    "name": "Default with fewer retries",
    "strategy": "exponential",
    "max_retries": 10
}

// Linear (fixed delay)
{
    "organization_id": "uuid",
    "name": "Every 5 minutes",
    "strategy": "linear",
    "max_retries": 20,
    "linear_delay": 300
}

// Custom (array = source of truth, len == max_retries)
{
    "organization_id": "uuid",
    "name": "Aggressive then patient",
    "strategy": "custom",
    "max_retries": 5,
    "custom_intervals": [3, 30, 300, 3600, 36000]
}
```

#### Validation rules

- `name`: min 2 chars, max 200 chars (consistent with organization/application name constraints)
- `strategy`: one of `exponential`, `linear`, `custom` (Rust enum, invalid values → 400)
- `max_retries`: 1..=100
- `custom_intervals`: required if custom, `len == max_retries`, each value 1..=MAX_INTERVAL_SECONDS. Must be absent/null for exponential and linear.
- `linear_delay`: required if linear, 1..=MAX_INTERVAL_SECONDS. Must be absent/null for exponential and custom.
- Organization must exist and caller must be a member
- Per-org limit: configurable via env var `MAX_RETRY_SCHEDULES_PER_ORG` (default 50)

#### Delete behavior

Delete is allowed even if the schedule is assigned to subscriptions. The DB `ON DELETE SET NULL` reverts affected subscriptions to the default retry policy.

**Frontend note (Phase 2+)**: The UI should display a confirmation warning before deletion, listing:
- Subscriptions currently using this schedule (and that they will revert to default)
- Any retries currently in progress on those subscriptions with this policy, and their next scheduled retry times

#### Response body

```json
{
    "retry_schedule_id": "uuid",
    "organization_id": "uuid",
    "name": "Aggressive then patient",
    "strategy": "custom",
    "max_retries": 5,
    "custom_intervals": [3, 30, 300, 3600, 36000],
    "linear_delay": null,
    "created_at": "2026-03-25T10:00:00Z",
    "updated_at": "2026-03-25T10:00:00Z"
}
```

### 3.4. Subscription Modifications

#### SubscriptionPost (used for both create and edit)

- Add optional field `retry_schedule_id: Option<Uuid>`
- Validation: if provided, the schedule must belong to the same organization as the subscription's application — enforced atomically via SQL subquery (no TOCTOU)
- Setting to `null` removes the assignment (reverts to default)

#### Subscription response

- Add `retry_schedule_id: Option<Uuid>` to the response body (ID only, not inline schedule)

### 3.5. Worker Retry Logic

#### Default behavior (David's commit `5b44bbc8`)

The worker uses a hardcoded escalating schedule: `3s, 10s, 3min, 30min, 1h, 3h, 5h, 10h (repeating)`, gated by `--max-retries` (default 25) and `--max-retry-window` (default 8d, informational only at boot). This remains the fallback when `retry_schedule__id IS NULL`.

#### New behavior with custom schedule

When a request attempt fails, the worker:

1. Retrieves schedule data from the initial fetch query (LEFT JOIN, no extra round-trip)
2. Parses `strategy` from `String` to `RetryStrategy` enum (via strum `FromStr`). If parse fails → `warn!` + give up (no retry).
3. Computes delay based on strategy:
   - **exponential**: calls David's `compute_next_retry_duration(max_retries, retry_count)` with the schedule's `max_retries` instead of the worker's `--max-retries`
   - **linear**: delay = `linear_delay` seconds, stop when `retry_count >= max_retries`
   - **custom**: delay = `custom_intervals[retry_count]`, stop when `retry_count >= max_retries`
4. If no schedule assigned → calls David's function with the worker's default `--max-retries`

#### Schedule changes during in-progress retries

The current schedule always applies. Already-created `request_attempt` rows keep their `delay_until`, but the *next* retry after a failure consults the current schedule. If a schedule is changed or removed mid-retry, the new configuration takes effect on the next failure.

Example: subscription had `max_retries: 50`, changed to a schedule with `max_retries: 3`. If current `retry_count = 5`, the worker sees `5 >= 3` → gives up. This is intentional.

#### Worker boot log

David's `evaluate_retry_policy` logs the effective default schedule at startup. Add a note that this is the **default** schedule and may be overridden by per-subscription custom schedules.

### 3.6. Hook0 Client Events

New operational event types to register on startup:

- `api.retry_schedule.created`
- `api.retry_schedule.updated`
- `api.retry_schedule.removed`

Each event includes the full schedule payload: `organization_id`, `retry_schedule_id`, `name`, `strategy`, `max_retries`, `custom_intervals`, `linear_delay`.

### 3.7. Files to Create/Modify

| File | Action |
|---|---|
| `api/migrations/TIMESTAMP_add_retry_schedule.up.sql` | New migration |
| `api/migrations/TIMESTAMP_add_retry_schedule.down.sql` | Rollback |
| `api/src/iam.rs` | Add RetrySchedule* Action variants |
| `api/src/problems.rs` | Add TooManyRetrySchedulesPerOrganization error variant |
| `api/src/handlers/retry_schedules.rs` | New handler module |
| `api/src/handlers/subscriptions.rs` | Add retry_schedule_id field |
| `api/src/handlers/mod.rs` | Register new module |
| `api/src/main.rs` | Add routes + MAX_RETRY_SCHEDULES_PER_ORG env var |
| `api/src/hook0_client.rs` | Add 3 event types with full payload |
| `output-worker/src/main.rs` | ScheduleConfig struct + compute_delay_from_schedule |
| `output-worker/src/pg.rs` | Add LEFT JOIN for schedule in fetch query |
| `output-worker/src/pulsar.rs` | Add LEFT JOIN for schedule in status query |

## 4. Security

- All retry schedule APIs require Biscuit auth (Viewer for read, Editor for write)
- Auth-first pattern on all endpoints (no fetch-before-auth)
- All SQL queries scoped to `organization__id` (no IDOR)
- Interval validation bounds prevent abuse (min 1s, max 1 week)
- max_retries capped at 100
- Per-org schedule count limit (env var, default 50)
- Cross-org schedule assignment prevented atomically via SQL subquery

## 5. Testing Strategy

- Unit tests for `compute_delay_from_schedule` per strategy (exponential, linear, custom, edge cases)
- Integration tests (HTTP via `actix_web::test` + test DB) for:
  - CRUD API (create all 3 strategies, list, get, update, delete)
  - Subscription assignment + cross-org rejection
  - Delete with assigned subscriptions (verify SET NULL behavior)
- Worker unit tests for schedule resolution (with/without schedule, unknown strategy)

## 6. Design Decisions

Decisions made during the design interview:

| # | Question | Decision | Rationale |
|---|---|---|---|
| 1 | Base code | Cherry-pick David's commit `5b44bbc8` | His new default is the baseline; our feature adds custom schedules on top |
| 2 | Quota system | Hardcoded limit as env var (default 50) | Proper Quotas integration is over-engineering for Phase 1; easy to migrate later |
| 3 | Biscuit roles | Viewer for list/get, Editor for create/edit/delete | Retry schedules are config, but viewable by all org members |
| 4 | Pulsar support | Yes, fully supported | Pulsar is actively maintained and used in production |
| 5 | exponential vs custom behavior | Exponential uses David's hardcoded table; custom uses user array | The strategy field is both a semantic label and a behavioral discriminator |
| 6 | Default fallback | Call David's function unchanged | Zero duplication, zero conflict |
| 7 | Tests | Unit + integration HTTP (actix_web::test) | No existing test DB infra; actix_web::test is the most complete approach |
| 8 | Interval minimum | >= 1 second | Prevents retry spam; David's default starts at 3s |
| 9 | ON DELETE SET NULL | Correct for subscription FK | Schedule deletion reverts subscriptions to default; PostgreSQL handles cascade ordering |
| 10 | Subscription response | Just retry_schedule_id, not inline | Schedule is shared N:1, unlike target which is 1:1 |
| 11 | Schedule change mid-retry | Current schedule applies | Already-planned attempts keep their delay_until; new retries use current schedule |
| 12 | Field naming | max_retries (not max_attempts) | Aligned with David's code; retry_count < max_retries works directly |
| 13 | Schema fields | custom_intervals, linear_delay, max_retries | Strategy-specific nullable fields with cross-field CHECK constraint |
| 14 | linear validation | custom_intervals must be absent | Prevents confusion where extra intervals are silently ignored |
| 15 | Route naming | /retry_schedules (underscore) | Consistent with event_types, service_token, request_attempts |
| 16 | Delete behavior | Allow delete, SET NULL, no 409 | Frontend should show warning with affected subscriptions and in-progress retries |
| 17 | Worker boot log | Keep David's evaluate_retry_policy, note "default" | Custom schedules override per-subscription |
| 18 | Hook0 events | Full payload (all fields) | Audit trail should be self-contained |

## 7. Open Questions for Future Phases

1. **Email notifications (Phase 2-3) vs custom schedules**: The ticket defines email warning at 3 days and auto-deactivation at 5 days of continuous failures. These are calendar-time thresholds, independent of the retry schedule. A custom schedule with `linear_delay: 300, max_retries: 20` exhausts in ~1.7 hours — the 3-day warning never triggers. A schedule with `max_retries: 50` may still be retrying at day 5 when deactivation kicks in. Phase 2 must decide: use fixed calendar thresholds, make them configurable per schedule, or compute them dynamically from the schedule's total duration.

2. **Frontend warning on delete**: The delete API allows deletion of assigned schedules (SET NULL). The frontend should show a confirmation dialog listing affected subscriptions and in-progress retries. This requires an endpoint (or query param on DELETE) that returns the list of affected subscriptions — to be designed in the frontend phase.

3. **Quota system integration**: The per-org limit is currently an env var, not integrated with the pricing plan / Quotas system. If retry schedules become a paid feature, this needs migration to the Quotas system (add column to `pricing.plan`, use `state.quotas.get_limit_for_organization()`).

4. **Manual retry (Phase 4)**: When a user manually retries a message, should it use the subscription's custom schedule or always use the default? To be decided in Phase 4 spec.

5. **`max_retry_window` interaction**: David's `max_retry_window` is currently informational only (logged at boot). If it becomes enforced at runtime in the future, it would apply as a global cap even on custom schedules. This interaction should be considered if `max_retry_window` enforcement is planned.
