# Phase 1: Configurable Retry Schedule

**Ticket**: [#42 — Customizable delay algorithm strategy on retry](https://gitlab.com/hook0/hook0/-/work_items/42)
**Date**: 2026-03-25
**Status**: Draft

---

## 1. Goal

Replace the hardcoded retry logic (30 fast + 30 slow = 60 attempts) with a configurable retry schedule system. Organizations can define named retry policies with one of three strategies and assign them to subscriptions. Subscriptions without a schedule use the Svix/Stripe default.

This is Phase 1 of ticket #42. Phases 2-4 (auto-deactivation, email notifications, manual retry/recovery) are out of scope.

## 2. Scope

### In scope

- New `webhook.retry_schedule` table
- CRUD API for retry schedules (org-level)
- `retry_schedule_id` FK on `webhook.subscription`
- Worker uses assigned schedule or Svix/Stripe default
- Hook0 client operational events for schedule CRUD

### Out of scope

- Automatic endpoint deactivation (Phase 2)
- Email notifications for failing endpoints (Phase 3)
- Manual retry/recover/replay APIs (Phase 4)
- Frontend UI for retry schedule management

## 3. Design

### 3.1. Database Schema

#### New table: `webhook.retry_schedule`

| Column | Type | Constraints |
|---|---|---|
| `retry_schedule__id` | UUID | PK, DEFAULT gen_random_uuid() |
| `organization__id` | UUID | FK → iam.organization, NOT NULL |
| `name` | TEXT | NOT NULL, CHECK length >= 1 |
| `strategy` | TEXT | NOT NULL, CHECK IN ('exponential', 'linear', 'custom') |
| `intervals` | INTEGER[] | NOT NULL, CHECK array_length > 0 |
| `max_attempts` | INTEGER | NOT NULL, DEFAULT 8, CHECK > 0 AND <= 100 |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT statement_timestamp() |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT statement_timestamp() |

- UNIQUE constraint on `(organization__id, name)`
- Validation on intervals: each value >= 1 and <= 604800 (1 week)

#### Alter: `webhook.subscription`

- ADD `retry_schedule__id UUID REFERENCES webhook.retry_schedule(retry_schedule__id)` (nullable)
- Index on `retry_schedule__id`

### 3.2. API Routes

All routes require Biscuit authentication. Authorization checks organization membership.

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/retry_schedules` | Create a schedule |
| GET | `/api/v1/retry_schedules` | List schedules (query param `organization_id`) |
| GET | `/api/v1/retry_schedules/{schedule_id}` | Get a schedule |
| PUT | `/api/v1/retry_schedules/{schedule_id}` | Update a schedule |
| DELETE | `/api/v1/retry_schedules/{schedule_id}` | Delete a schedule |

Flat top-level scope, consistent with existing routes (`event_types`, `service_token`, `request_attempts`). Organization resolved via body/query param + Biscuit auth, not URL nesting.

#### Create request body

```json
{
    "organization_id": "uuid",
    "name": "Production Retry Policy",
    "strategy": "exponential",
    "intervals": [5, 300, 1800, 7200, 18000, 36000, 36000],
    "max_attempts": 8
}
```

#### Update request body

```json
{
    "name": "Production Retry Policy",
    "strategy": "exponential",
    "intervals": [5, 300, 1800, 7200, 18000, 36000, 36000],
    "max_attempts": 8
}
```

#### Validation rules

- `name`: non-empty string
- `strategy`: one of `exponential`, `linear`, `custom`
- `intervals`: non-empty array, each value 1..=604800 seconds
- `max_attempts`: 1..=100
- Organization must exist and caller must be a member

#### Delete behavior

- Fails with 409 Conflict if the schedule is still assigned to any subscription
- Caller must unassign it from all subscriptions first

#### Response body

```json
{
    "retry_schedule_id": "uuid",
    "organization_id": "uuid",
    "name": "Production Retry Policy",
    "strategy": "exponential",
    "intervals": [5, 300, 1800, 7200, 18000, 36000, 36000],
    "max_attempts": 8,
    "created_at": "2026-03-25T10:00:00Z",
    "updated_at": "2026-03-25T10:00:00Z"
}
```

### 3.3. Subscription Modifications

#### SubscriptionPost / SubscriptionPut

- Add optional field `retry_schedule_id: Option<Uuid>`
- Validation: if provided, the schedule must belong to the same organization as the subscription's application
- Setting to `null` removes the assignment (reverts to default)

#### Subscription response

- Add `retry_schedule_id: Option<Uuid>` to the response body

### 3.4. Worker Retry Logic

#### Current behavior (to be replaced)

```
fast retries: 30 attempts, delay = min(5s * count, 300s)
slow retries: 30 attempts, delay = 3600s
total: 60 attempts max
```

#### New behavior

When a request attempt fails, the worker:

1. Queries the subscription's `retry_schedule_id` via JOIN
2. If schedule exists, computes delay based on strategy:
   - **exponential**: `intervals[min(retry_count, len(intervals) - 1)]` — last interval repeats until `max_attempts`
   - **linear**: `intervals[0]` fixed delay for every retry until `max_attempts`
   - **custom**: `intervals[min(retry_count, len(intervals) - 1)]` — last interval repeats until `max_attempts`
3. Stops retrying when `retry_count >= max_attempts`
4. If no schedule assigned, uses **hardcoded default**:
   - intervals: `[0, 5, 300, 1800, 7200, 18000, 36000, 36000]`
   - max_attempts: 8
   - strategy: exponential (last interval repeats)

Note: `strategy` controls delay computation. `exponential` and `custom` share the same index-based lookup, but the distinction is a **UI/semantic hint** — `exponential` signals increasing intervals, `custom` signals arbitrary user-defined ones. `linear` uses only `intervals[0]` as a fixed delay. The worker branches on `strategy` to select the computation.

#### retry_count to delay mapping (default schedule)

| retry_count | delay | cumulative elapsed |
|---|---|---|
| 0 | 0s (immediate) | 0s |
| 1 | 5s | 5s |
| 2 | 5min | ~5min |
| 3 | 30min | ~35min |
| 4 | 2h | ~2h35 |
| 5 | 5h | ~7h35 |
| 6 | 10h | ~17h35 |
| 7 | 10h | ~27h35 |

#### `max_attempts` vs `intervals` length

These are independent. `max_attempts` controls when to stop retrying. `intervals` controls the delay progression. If `max_attempts > len(intervals)`, the last interval repeats. If `max_attempts < len(intervals)`, extra intervals are unused.

#### Breaking change

Default retry behavior changes from 60 attempts over ~30h to 8 attempts over ~20h. This is intentional per the ticket specification.

#### Schedule resolution — no extra query

The schedule data (`strategy`, `intervals`, `max_attempts`) must be fetched in the **initial attempt fetch query** via `LEFT JOIN webhook.retry_schedule`, not as a separate query. This avoids an extra DB round-trip per retry decision. The fields are carried through in `RequestAttemptWithOptionalPayload` (or equivalent struct).

The existing subscription check in `compute_next_retry` (is_enabled, deleted_at) already queries the subscription — the schedule JOIN is added there.

#### Natural "no impact on in-progress retries"

Modifying a schedule does not affect already-created `request_attempt` rows — they keep their `delay_until`. Only the *next* retry after a failure consults the current schedule.

### 3.5. Hook0 Client Events

New operational event types to register on startup:

- `api.retry_schedule.created`
- `api.retry_schedule.updated`
- `api.retry_schedule.removed`

Each event includes: `instance_id`, `organization_id`, `retry_schedule_id`, `name`, `strategy`.

### 3.6. Files to Create/Modify

| File | Action |
|---|---|
| `api/migrations/TIMESTAMP_add_retry_schedule.up.sql` | New migration |
| `api/migrations/TIMESTAMP_add_retry_schedule.down.sql` | Rollback: DROP FK column from subscription first, then DROP table |
| `api/src/handlers/retry_schedules.rs` | New handler module |
| `api/src/handlers/subscriptions.rs` | Add retry_schedule_id field |
| `api/src/handlers/mod.rs` | Register new module |
| `api/src/main.rs` | Add routes |
| `api/src/hook0_client.rs` | Add 3 event types |
| `output-worker/src/main.rs` | Replace compute_next_retry_duration |
| `output-worker/src/pg.rs` | Pass schedule info to retry computation |
| `output-worker/src/pulsar.rs` | Pass schedule info to retry computation |

Note: The existing `--max-fast-retries` and `--max-slow-retries` CLI params become dead config after this change. They should be removed as part of the worker modifications.

Note: `updated_at` is maintained at application level (explicit `SET updated_at = statement_timestamp()` in UPDATE handler), consistent with the existing `webhook.subscription.updated_at` pattern. No DB trigger needed.

Note: Interval values are in **seconds** (documented in CHECK constraint comment and API docs).

## 4. Security

- All retry schedule APIs require Biscuit auth
- Authorization: caller must be org member (same pattern as other org-scoped resources)
- Interval validation bounds prevent abuse (min 1s, max 1 week)
- max_attempts capped at 100

## 5. Testing Strategy

- Unit tests for delay computation per strategy (exponential, linear, custom edge cases)
- Integration tests for CRUD API (create, list, get, update, delete, conflict on delete)
- Integration test for subscription assignment + validation (cross-org rejection)
- Worker integration test: verify schedule is used for retry delay computation
