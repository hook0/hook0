# Manual Retry — Design Spec

**Date**: 2026-04-03
**Feature**: Manual retry of webhook deliveries
**Branch**: `feat/retry-schedule` (extends existing work)

---

## Problem

When a webhook delivery fails and exhausts all automatic retries, the event is silently dropped. Users cannot recover missed events after a disable-then-re-enable cycle. This is the #1 complaint on Svix's GitHub and the biggest gap in Hook0's retry feature.

## Solution

Add a manual retry endpoint that creates a new one-shot delivery attempt for a specific subscription. The user clicks a button on any delivery row (failed or successful) and the system immediately re-delivers the event payload.

---

## API

### Endpoint

```
POST /api/v1/request_attempts/{request_attempt_id}/retry
```

Follows the existing pattern: `/events/{event_id}/replay` and `/request_attempts` scope in `main.rs`.

### Route registration

In `main.rs`, add inside the existing `web::scope("/request_attempts")`:

```rust
.service(
    web::resource("/{request_attempt_id}/retry")
        .route(web::post().to(handlers::request_attempts::retry)),
)
```

### IAM

Add `RequestAttemptRetry` variant to the `Action` enum in `api/src/iam.rs`:
- Add to enum definition
- Add `action_name()` mapping (e.g. `"request_attempt.retry"`)
- Add to biscuit policy rules and authorizer logic
- Follow the existing pattern of `EventReplay` as reference

### Guards

The handler takes only `request_attempt_id` in the URL. Auth happens AFTER fetching the attempt to determine the application. To prevent information leakage:

1. **Fetch attempt** — read `request_attempt` to get `event_id`, `subscription_id`, `application_id`
2. **Auth** — biscuit token, IAM action `RequestAttemptRetry` scoped to the application
3. **On auth failure OR attempt not found: return 404 uniformly** — never distinguish "not found" from "forbidden" to prevent cross-org attempt ID enumeration
4. **Subscription not deleted** — verify `subscription.deleted_at IS NULL`, return 404 if deleted
5. **Event payload exists** — the event payload must still be available (DB `payload` column or object storage). Return 400 `"payload_expired"` if missing.

### No guards on

- Attempt status (failed or success — both allowed, matching Svix behavior)
- Pending retries (no dedup — two concurrent retries produce two independent delivery attempts, this is acceptable and matches Svix)
- Subscription enabled state (the user explicitly chose to retry; note: the worker query filters on `is_enabled`, so if the subscription is disabled the attempt will sit as "Pending" until re-enabled)

### Behavior

1. Fetch the source `request_attempt` to get `event_id`, `subscription_id`, `application_id`
2. Authorize (return 404 on failure — see guard chain above)
3. Verify subscription exists and is not deleted
4. Verify event payload exists (DB or object storage)
5. Extract caller identity: if `AuthorizedToken::User`, use `user_id` for `triggered_by`. If `AuthorizedToken::Service` or `AuthorizedToken::Master`, set `triggered_by = NULL` (no user to attribute)
6. Insert a new `request_attempt` row:
   - `application_id`, `event_id`, `subscription_id` from source
   - `retry_count = 0`
   - `delay_until = NULL` (immediate pickup)
   - `attempt_trigger = 'manual_retry'`
   - `triggered_by = user_id or NULL` (see step 5)
7. For Pulsar workers: send ONLY this new attempt to Pulsar (see Pulsar section below)
8. Return `202 Accepted`

### Response

```json
{
  "request_attempt_id": "<new_attempt_uuid>"
}
```

### Pulsar: single-attempt send

The existing `send_request_attempts_to_pulsar` function (`events.rs:737`) sends ALL pending attempts for an event. It CANNOT be reused — it would re-send auto-retry attempts waiting on `delay_until`, causing duplicate deliveries.

A new function `send_single_attempt_to_pulsar` is needed:

```rust
async fn send_single_attempt_to_pulsar(
    tx: &mut Transaction<'_, Postgres>,
    pulsar_producer: &MultiTopicProducer,
    request_attempt_id: Uuid,
) -> Result<(), Hook0Problem>
```

This function:
1. JOINs `request_attempt` → `subscription` → `subscription__worker` to resolve the worker_name/topic
2. Reads the event payload
3. Sends a single Pulsar message for this specific attempt

Follow the pattern in `send_request_attempts_to_pulsar` but with a `WHERE ra.request_attempt__id = $1` filter instead of `WHERE ra.event__id = $1`.

---

## Migration

### New columns on `webhook.request_attempt`

Column named `attempt_trigger` (not `trigger` — `trigger` is a PostgreSQL reserved keyword).

```sql
SET lock_timeout = '5s';

-- Step 1: add column with default (instant on PG 11+)
ALTER TABLE webhook.request_attempt
  ADD COLUMN attempt_trigger TEXT NOT NULL DEFAULT 'dispatch';

-- Step 2: add CHECK as NOT VALID (no table scan, no lock)
ALTER TABLE webhook.request_attempt
  ADD CONSTRAINT request_attempt_trigger_check
    CHECK (attempt_trigger IN ('dispatch', 'auto_retry', 'manual_retry'))
    NOT VALID;

-- Step 3: validate separately (SHARE UPDATE EXCLUSIVE lock, concurrent reads/writes OK)
VALIDATE CONSTRAINT request_attempt_trigger_check;

-- Step 4: add triggered_by FK (nullable, no rewrite)
-- NULL when caller is a service token or system (no user to attribute)
ALTER TABLE webhook.request_attempt
  ADD COLUMN triggered_by UUID REFERENCES iam.user;

RESET lock_timeout;
```

- `attempt_trigger`: who created this attempt. `dispatch` = initial delivery from event trigger, `auto_retry` = worker successor after failure, `manual_retry` = user clicked retry.
- `triggered_by`: the user who initiated a manual retry. NULL for system-created attempts AND for service-token-initiated retries.
- `DEFAULT 'dispatch'` makes the migration backward-compatible — existing rows get the correct default.
- Expand-and-contract pattern on CHECK prevents full table scan under ACCESS EXCLUSIVE lock.

### Deployment ordering

The worker INSERT for successors (`pg.rs:302-316`) must be updated to write `attempt_trigger = 'auto_retry'` BEFORE or AT the same time as the migration. Otherwise, auto-retry successors created after the migration will be mislabeled as `'dispatch'` (the DEFAULT). Deploy the worker code and migration atomically, or accept a brief window of mislabeled rows.

### Down migration

```sql
SET lock_timeout = '5s';
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS triggered_by;
ALTER TABLE webhook.request_attempt DROP CONSTRAINT IF EXISTS request_attempt_trigger_check;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS attempt_trigger;
RESET lock_timeout;
```

---

## Worker changes

### Fetching attempt_trigger

The worker's `look_for_work` query in `pg.rs` must SELECT `attempt_trigger` from the `request_attempt` row. Add the field to the struct that carries attempt data.

### Creating auto-retry successors

When the worker creates a successor after a failed delivery (`pg.rs` INSERT), add `attempt_trigger = 'auto_retry'` to the INSERT statement.

### One-shot behavior for manual retries

At `pg.rs:292`, BEFORE calling `compute_next_retry`, check `attempt_trigger`:

```rust
if attempt.attempt_trigger == "manual_retry" {
    info!("Manual retry failed; not re-queuing (one-shot)");
    // skip compute_next_retry and successor INSERT entirely
} else if let Some(retry_in) = compute_next_retry(...).await? {
    // existing auto-retry logic
}
```

This short-circuits before the DB query in `compute_next_retry`, avoiding unnecessary work.

Note: for a manual retry on a disabled subscription, `compute_next_retry` would already return `None` (it filters `AND s.is_enabled`). The explicit `attempt_trigger` check above makes the one-shot behavior clear and independent of subscription state.

### Disabled subscription edge case

The existing worker query (`pg.rs:76`) filters `AND s.is_enabled`. A manual retry on a disabled subscription will sit as "Pending" until the subscription is re-enabled. This is acceptable — the user chose to retry knowing the subscription state.

---

## Frontend

### Retry button in SubscriptionsDetail.vue

- Icon: `RotateCcw` from lucide-vue-next
- Placement: in the delivery table, as a ghost button on every row
- Visible on all rows (failed AND success)
- Disabled while mutation is in-flight (`isPending` from `useMutation`) — prevents spam-click
- `aria-label`: `t('subscriptionDetail.retryDelivery')`

### Interaction flow

1. User clicks `RotateCcw` button on a delivery row
2. Button shows spinner (disabled during mutation)
3. Call `POST /request_attempts/{id}/retry`
4. On success: `toast.success(t('subscriptionDetail.retryQueued'))`, `invalidateQueries(logKeys.bySubscription(...))`
5. On error (400 payload_expired): `toast.error(t('subscriptionDetail.payloadExpired'))`
6. On error (404): `toast.error(t('common.notFound'))`
7. The table refetches and shows the new attempt as "Pending"

### Service + query

- Add `retry(requestAttemptId: UUID)` to `LogService.ts` → `POST /request_attempts/${id}/retry`
- Mutation via `useMutation` in a new `useRetryDelivery` composable, with invalidation on `logKeys.bySubscription`

### i18n keys

```json
"subscriptionDetail": {
  "retryDelivery": "Retry this delivery",
  "retryQueued": "Retry queued",
  "payloadExpired": "Cannot retry — event payload has expired"
}
```

---

## Protobuf

Add `attempt_trigger` field (string, optional) to the `RequestAttempt` protobuf message in `protobuf/`. Both PG and Pulsar worker paths use this protobuf. Without it, the Pulsar worker cannot determine one-shot behavior after deserializing the message.

---

## Refactor: extract `fetch_event_payload`

The `replay` handler in `events.rs` contains ~30 lines of inline logic to fetch event payload from DB or object storage. Extract into a reusable function:

```rust
async fn fetch_event_payload(
    db: &PgPool,
    object_storage: Option<&ObjectStorageConfig>,
    event_id: Uuid,
    application_id: Uuid,
    received_at: DateTime<Utc>,
    db_payload: Option<Vec<u8>>,
) -> Option<Vec<u8>>
```

Used by both `replay` (refactored) and the new `retry` handler. Avoids duplicating the S3 fallback logic.

---

## Frontend: "Retry" badge on manual attempts

The API response for `GET /request_attempts` must include `attempt_trigger` in the response. On the frontend, rows with `attempt_trigger === 'manual_retry'` display a small `Hook0Badge variant="info" size="sm"` with text "Retry" next to the status badge. This helps users distinguish manual retries from automatic deliveries.

Add `attempt_trigger` to the `RequestAttempt` TypeScript type in `types.ts`.

---

## Observability note

Manual retries have `retry_count = 0`, which looks identical to a fresh dispatch in metrics. The `attempt_trigger = 'manual_retry'` column distinguishes them. Any monitoring that keys on `retry_count` should also filter on `attempt_trigger` to avoid counting manual retries as first-attempt failures.

---

## Decisions log

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | Retry scope | Per-subscription | Standard across Stripe, Svix, Convoy |
| 2 | Retry behavior | One-shot, no auto-retry chain | Verified in Svix source (`attempt.rs:582-586`) |
| 3 | retry_count | 0 (fresh start) | Verified in Svix source (`queue/mod.rs:84`) |
| 4 | Auto retries in flight | Not cancelled | Independent attempts |
| 5 | Status restriction | None (retry failed AND success) | Verified in Svix source — no status check |
| 6 | Duplicate guard | None (button disabled during inflight) | Verified in Svix source — no dedup |
| 7 | Tracking | `attempt_trigger` + `triggered_by` on request_attempt | Worker needs to know one-shot behavior |
| 8 | UI | Button per row, toast + refetch | Existing mutation pattern |
| 9 | API shape | `POST /request_attempts/{id}/retry` | Follows existing route structure |
| 10 | Column name | `attempt_trigger` not `trigger` | `trigger` is a PG reserved word |
| 11 | Deleted subscription | Block retry (404) | Orphaned attempts waste worker resources |
| 12 | Disabled subscription | Allow retry (sits as Pending) | User's explicit choice |
| 13 | Service token caller | `triggered_by = NULL` | Service tokens have no user_id |
| 14 | Security: error responses | 404 for both not-found and forbidden | Prevent cross-org attempt enumeration |
| 15 | Pulsar | New `send_single_attempt_to_pulsar` fn | Existing fn sends ALL pending, would cause duplicates |
| 16 | Deployment ordering | Worker code + migration atomic | Avoid mislabeled auto-retry successors |
| 17 | Protobuf | Add `attempt_trigger` to RequestAttempt protobuf | Pulsar worker needs the field without DB re-query |
| 18 | Pulsar optionnel | `if let Some(pulsar)` pattern | Same as replay handler, PG worker polls DB directly |
| 19 | Badge "Retry" in UI | Info badge on manual_retry rows | Distinguish manual from automatic deliveries |
| 20 | Payload fetch | Extract `fetch_event_payload` from replay handler | Reuse in retry handler, avoid 30-line duplication |
