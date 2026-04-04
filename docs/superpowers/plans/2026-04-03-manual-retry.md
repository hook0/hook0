# Manual Retry — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `POST /request_attempts/{id}/retry` endpoint that creates a one-shot manual retry, with worker support and a frontend button.

**Architecture:** Migration adds `attempt_trigger` + `triggered_by` columns to `request_attempt`. API handler creates a new attempt and optionally sends to Pulsar. Worker checks `attempt_trigger` to skip successor creation for manual retries. Frontend adds a retry button on each delivery row in the subscription detail page.

**Tech Stack:** Rust (actix-web, sqlx, prost), PostgreSQL, Protobuf, Vue 3, TanStack Query

**Spec:** `docs/superpowers/specs/2026-04-03-manual-retry-design.md`

---

## File Structure

### New files
- `api/migrations/20260403130000_add_attempt_trigger.up.sql`
- `api/migrations/20260403130000_add_attempt_trigger.down.sql`
- `api/src/event_payload.rs` — shared `fetch_event_payload` function (DB or S3 fallback)

### Modified files
- `protobuf/proto/request_attempt.proto` — add `attempt_trigger` field
- `api/src/iam.rs` — add `RequestAttemptRetry` action
- `api/src/handlers/request_attempts.rs` — add `retry` handler with `send_single_attempt_to_pulsar` inline, add `attempt_trigger` to list response
- `api/src/handlers/events.rs` — refactor `replay` to use `event_payload::fetch_event_payload`
- `api/src/main.rs` — register retry route, register `event_payload` module
- `api/src/problems.rs` — add `PayloadExpired` problem
- `output-worker/src/main.rs` — add `attempt_trigger` to `RequestAttemptWithOptionalPayload` struct (defined at line 285)
- `output-worker/src/pg.rs` — add `attempt_trigger` to look_for_work SELECT, write `auto_retry` on successors, one-shot check, propagate to protobuf
- `output-worker/src/pulsar.rs` — one-shot check for manual retries (read from deserialized protobuf)
- `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue` — add retry button column (local, NOT in useLogColumns)
- `frontend/src/pages/organizations/applications/logs/LogService.ts` — add `retry()` function
- `frontend/src/pages/organizations/applications/logs/useLogQueries.ts` — add `useRetryDelivery` mutation
- `frontend/src/pages/organizations/applications/logs/useLogColumns.ts` — add "Retry" badge for manual attempts (badge only, not button)
- `frontend/src/types.ts` — regenerated via `npm run generate:types` (not manually edited)
- `frontend/src/locales/en.json` — add retry i18n keys
- `frontend/src/types.ts` — add `attempt_trigger` to RequestAttempt type

---

## Task 1: Migration

**Files:**
- Create: `api/migrations/20260403130000_add_attempt_trigger.up.sql`
- Create: `api/migrations/20260403130000_add_attempt_trigger.down.sql`

- [ ] **Step 1: Write up migration**

```sql
SET lock_timeout = '5s';

-- Step 1: add column with default (instant on PG 11+ — stored in catalog, no rewrite)
ALTER TABLE webhook.request_attempt
  ADD COLUMN attempt_trigger TEXT NOT NULL DEFAULT 'dispatch';

-- Step 2: CHECK as NOT VALID — no table scan, no ACCESS EXCLUSIVE hold
ALTER TABLE webhook.request_attempt
  ADD CONSTRAINT request_attempt_trigger_check
    CHECK (attempt_trigger IN ('dispatch', 'auto_retry', 'manual_retry'))
    NOT VALID;

-- Step 3: validate separately — SHARE UPDATE EXCLUSIVE lock, concurrent DML OK
VALIDATE CONSTRAINT request_attempt_trigger_check;

-- Step 4: nullable FK for user attribution (NULL for system/service-token callers)
ALTER TABLE webhook.request_attempt
  ADD COLUMN triggered_by UUID REFERENCES iam.user;

RESET lock_timeout;
```

- [ ] **Step 2: Write down migration**

```sql
SET lock_timeout = '5s';
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS triggered_by;
ALTER TABLE webhook.request_attempt DROP CONSTRAINT IF EXISTS request_attempt_trigger_check;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS attempt_trigger;
RESET lock_timeout;
```

- [ ] **Step 3: Commit**

```bash
git add api/migrations/20260403130000_*
git commit -m "feat(api): add attempt_trigger and triggered_by columns to request_attempt"
```

---

## Task 2: Protobuf

**Files:**
- Modify: `protobuf/proto/request_attempt.proto`

- [ ] **Step 1: Add attempt_trigger field**

Add after the last field (field number 15):
```protobuf
  string attempt_trigger = 15;
```

- [ ] **Step 2: Rebuild protobuf**

Run: `cd protobuf && cargo build`

- [ ] **Step 3: Commit**

```bash
git add protobuf/
git commit -m "feat(protobuf): add attempt_trigger field to RequestAttempt message"
```

---

## Task 3: Create `event_payload.rs` + refactor replay

**Files:**
- Create: `api/src/event_payload.rs`
- Modify: `api/src/handlers/events.rs`
- Modify: `api/src/main.rs` (add `mod event_payload;`)

- [ ] **Step 1: Create event_payload.rs**

Extract the inline payload fetch logic from `events.rs` lines 665-710 into a new shared module:

```rust
// api/src/event_payload.rs
//! Fetch event payload from DB or object storage — shared by replay and manual retry handlers.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fetch event payload from the DB column or fall back to object storage.
/// Returns None if the payload is unavailable (expired or storage error).
pub async fn fetch_event_payload(
    db_payload: Option<Vec<u8>>,
    object_storage: Option<&crate::ObjectStorageConfig>,
    application_id: Uuid,
    event_id: Uuid,
    received_at: DateTime<Utc>,
) -> Option<Vec<u8>> {
    if let Some(p) = db_payload {
        return Some(p);
    }
    if let Some(os) = object_storage {
        let key = format!(
            "{}/event/{}/{event_id}",
            application_id,
            received_at.naive_utc().date(),
        );
        // Copy the existing S3 get_object logic from events.rs lines 674-710
        match os.client.get_object().bucket(&os.bucket).key(&key).send().await {
            Ok(obj) => match obj.body.collect().await {
                Ok(ab) => return Some(ab.to_vec()),
                Err(e) => {
                    tracing::warn!("S3 GET body collect failed for {key}: {e}");
                }
            },
            Err(e) => {
                tracing::warn!("S3 GET failed for {key}: {e}");
            }
        }
    }
    None
}
```

- [ ] **Step 2: Register module in main.rs**

Add `mod event_payload;` in `api/src/main.rs` alongside the other module declarations.

- [ ] **Step 3: Refactor replay handler**

In `events.rs`, replace the inline payload fetch (lines 665-710) with:
```rust
let payload = crate::event_payload::fetch_event_payload(
    event.payload, state.object_storage.as_ref(),
    body.application_id, event_id, event.received_at,
).await;
```

- [ ] **Step 4: Verify replay still builds**

Run: `cargo build -p hook0-api`

- [ ] **Step 5: Commit**

```bash
git add api/src/event_payload.rs api/src/handlers/events.rs api/src/main.rs
git commit -m "refactor(api): extract fetch_event_payload into shared module"
```

---

## Task 4: IAM action + API handler

**Files:**
- Modify: `api/src/iam.rs`
- Modify: `api/src/handlers/request_attempts.rs`
- Modify: `api/src/main.rs`
- Modify: `api/src/problems.rs`

- [ ] **Step 1: Add PayloadExpired problem**

In `problems.rs`, add a new problem variant:
```rust
PayloadExpired => (StatusCode::BAD_REQUEST, "payload_expired", "Event payload is no longer available")
```

Follow the existing pattern of other problem definitions.

- [ ] **Step 2: Add RequestAttemptRetry IAM action**

In `iam.rs`:
- Add variant: `RequestAttemptRetry { application_id: &'a Uuid }`
- Add `action_name()`: `"request_attempt:retry"`
- Add to role mappings (follow `EventReplay` pattern — empty roles = org-owner only)
- Add `application_id` extraction in the match arm

- [ ] **Step 3: Add retry handler**

In `request_attempts.rs`, add:

```rust
pub async fn retry(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    path: Path<Uuid>,
) -> Result<HttpResponse, Hook0Problem> {
    let request_attempt_id = path.into_inner();

    // 1. Fetch the source attempt with subscription and event info
    let source = query_as!(...)  // JOIN request_attempt, subscription, event
        .fetch_optional(&state.db)
        .await?
        .ok_or(Hook0Problem::NotFound)?;

    // 2. Authorize — return 404 uniformly for both not-found and forbidden
    //    Also extracts user_id from the token (None for service/master tokens)
    let authorized = authorize_for_application(
        &state.db, &biscuit,
        Action::RequestAttemptRetry { application_id: &source.application_id },
        state.max_authorization_time_in_ms, state.debug_authorizer,
    ).await;
    let authorized = match authorized {
        Ok(token) => token,
        Err(_) => return Err(Hook0Problem::NotFound), // 404, not 403 — prevent enumeration
    };
    let user_id = match &authorized {
        AuthorizedToken::User(aut) => Some(aut.user_id),
        _ => None, // Service tokens and master tokens have no user_id
    };

    // 3. Check subscription not deleted
    if source.subscription_deleted_at.is_some() {
        return Err(Hook0Problem::NotFound);
    }

    // 4. Check payload exists
    let payload = fetch_event_payload(
        source.payload, state.object_storage.as_ref(),
        source.application_id, source.event_id, source.received_at,
    ).await.ok_or(Hook0Problem::PayloadExpired)?;

    // 6. INSERT new attempt
    let mut tx = state.db.begin().await?;
    let new_id = query_scalar!(
        "INSERT INTO webhook.request_attempt
           (application__id, event__id, subscription__id, retry_count, attempt_trigger, triggered_by)
         VALUES ($1, $2, $3, 0, 'manual_retry', $4)
         RETURNING request_attempt__id",
        source.application_id, source.event_id, source.subscription_id, user_id,
    ).fetch_one(&mut *tx).await?;

    // 7. Pulsar (optional)
    if let Some(pulsar) = &state.pulsar {
        send_single_attempt_to_pulsar(
            &mut tx, pulsar, new_id,
            &payload, &source.payload_content_type,
            source.received_at, &source.event_type,
        ).await?;
    }

    tx.commit().await?;

    // Return 202 Accepted (not 200) — the retry is queued, not completed
    Ok(HttpResponse::Accepted().json(RetryResponse { request_attempt_id: new_id }))
}
```

- [ ] **Step 4: Register route**

In `main.rs`, inside `web::scope("/request_attempts")`, add:
```rust
.service(
    web::resource("/{request_attempt_id}/retry")
        .route(web::post().to(handlers::request_attempts::retry)),
)
```

- [ ] **Step 5: Add attempt_trigger to list response**

In `request_attempts.rs`, add `attempt_trigger` to the SELECT in `list()` and to the response struct `RequestAttemptStatus`.

- [ ] **Step 6: Update sqlx offline data**

Check if `.sqlx/` directory exists at the project root. If it does, offline mode is active and this step is mandatory:
Run: `cd api && cargo sqlx prepare`
If `.sqlx/` does not exist, skip this step.

- [ ] **Step 7: Build**

Run: `cargo build -p hook0-api`

- [ ] **Step 8: Commit**

```bash
git add api/src/
git commit -m "feat(api): add POST /request_attempts/{id}/retry endpoint for manual delivery retry"
```

---

## Task 5: Worker — one-shot + auto_retry trigger

**Files:**
- Modify: `output-worker/src/main.rs` — add `attempt_trigger` to `RequestAttemptWithOptionalPayload` struct (defined at line 285)
- Modify: `output-worker/src/pg.rs` — SELECT, successor INSERT, one-shot check, protobuf construction
- Modify: `output-worker/src/pulsar.rs` — one-shot check for Pulsar path

- [ ] **Step 1: Add attempt_trigger to RequestAttemptWithOptionalPayload struct**

In `output-worker/src/main.rs`, find the `RequestAttemptWithOptionalPayload` struct (line 285). Add:
```rust
pub attempt_trigger: String,
```

- [ ] **Step 2: Add attempt_trigger to look_for_work SELECT**

In `pg.rs`, add `ra.attempt_trigger` to the SELECT in `look_for_work` (around line 46-78). The field will be read into the struct from Step 1.

- [ ] **Step 3: Write auto_retry on successor INSERT**

In `pg.rs` around line 302, update the successor INSERT to include `attempt_trigger`:

```sql
INSERT INTO webhook.request_attempt
  (application__id, event__id, subscription__id, delay_until, retry_count, attempt_trigger)
VALUES ($1, $2, $3, statement_timestamp() + $4, $5, 'auto_retry')
```

- [ ] **Step 4: Add one-shot check before compute_next_retry (PG path)**

In `pg.rs` around line 292, before calling `compute_next_retry`:

```rust
if attempt.attempt_trigger == "manual_retry" {
    info!(
        request_attempt_id = %attempt.request_attempt_id,
        "Manual retry failed; not re-queuing (one-shot)"
    );
} else if let Some(retry_in) = compute_next_retry(...).await? {
    // existing successor INSERT logic
}
```

- [ ] **Step 5: Propagate attempt_trigger to protobuf construction**

In `pg.rs` around line 165 where `RequestAttempt` protobuf is built from `RequestAttemptWithOptionalPayload`, add:
```rust
attempt_trigger: attempt.attempt_trigger.clone(),
```

- [ ] **Step 6: Add one-shot check on Pulsar path**

In `output-worker/src/pulsar.rs`, find where the worker processes a failed delivery and decides whether to create a successor. Add the same one-shot check:

```rust
if attempt.attempt_trigger == "manual_retry" {
    info!(
        request_attempt_id = %attempt.request_attempt_id,
        "Manual retry failed; not re-queuing (one-shot)"
    );
} else if let Some(retry_in) = compute_next_retry(...).await? {
    // existing successor logic
}
```

The `attempt_trigger` field is available from the deserialized protobuf `RequestAttempt` message (added in Task 2).

- [ ] **Step 7: Build + test**

Run: `cargo build -p hook0-output-worker && cargo test -p hook0-output-worker`

Update existing tests that construct `RequestAttemptWithOptionalPayload` or `RequestAttempt` to include `attempt_trigger: "dispatch".to_string()`.

- [ ] **Step 8: Commit**

```bash
git add output-worker/src/main.rs output-worker/src/pg.rs output-worker/src/pulsar.rs
git commit -m "feat(output-worker): support attempt_trigger — auto_retry on successors, one-shot for manual_retry on both PG and Pulsar paths"
```

---

## Task 6: Frontend — retry button + badge

**Files:**
- Regenerate: `frontend/src/types.ts` (via `npm run generate:types`)
- Modify: `frontend/src/pages/organizations/applications/logs/LogService.ts`
- Modify: `frontend/src/pages/organizations/applications/logs/useLogQueries.ts`
- Modify: `frontend/src/pages/organizations/applications/logs/useLogColumns.ts`
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue`
- Modify: `frontend/src/locales/en.json`

**Prerequisite:** The API must be running locally (Task 4 deployed) so swagger.json includes `attempt_trigger`.

- [ ] **Step 1: Regenerate TypeScript types from OpenAPI**

Start the API if not running, then:
```bash
cd frontend && npm run generate:types
```

This regenerates `types.ts` from `http://localhost:8081/api/v1/swagger.json`. The `RequestAttempt` type will automatically include `attempt_trigger`. Do NOT edit `types.ts` manually — it is auto-generated.

Verify `attempt_trigger` is present:
```bash
grep "attempt_trigger" src/types.ts
```

- [ ] **Step 2: Add retry function to LogService**

```typescript
export function retry(request_attempt_id: UUID): Promise<{ request_attempt_id: string }> {
  return unwrapResponse(
    http.post<{ request_attempt_id: string }>(`/request_attempts/${request_attempt_id}/retry`)
  );
}
```

- [ ] **Step 3: Add useRetryDelivery mutation to useLogQueries**

```typescript
export function useRetryDelivery(
  applicationId: Ref<string>,
  subscriptionId: Ref<string>
) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (requestAttemptId: string) => LogService.retry(requestAttemptId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: logKeys.bySubscription(applicationId.value, subscriptionId.value),
      });
    },
  });
}
```

- [ ] **Step 4: Add "Retry" badge to useLogColumns**

In `useLogColumns.ts`, in the status cell renderer, after the existing status badge, add:
```typescript
if (info.row.original.attempt_trigger === 'manual_retry') {
  return h('div', { class: 'log-status-wrapper' }, [
    statusBadge,
    h(Hook0Badge, { variant: 'info', size: 'sm' }, () => t('subscriptionDetail.manualRetryBadge')),
  ]);
}
```

- [ ] **Step 5: Add retry button column in SubscriptionsDetail (LOCAL, not in useLogColumns)**

The retry button is specific to the subscription detail page. Do NOT add it to `useLogColumns.ts` — it would appear on the global Deliveries page too. Instead, extend the columns locally in `SubscriptionsDetail.vue`:

```typescript
const baseColumns = useLogColumns(); // shared columns (minus Subscription column)
const columns = [...baseColumns, retryColumn]; // add retry locally
```

The retry column definition:
```typescript
{
  id: 'retry',
  header: '',
  size: 40,
  cell: ({ row }) =>
    h(Hook0Button, {
      variant: 'ghost',
      type: 'button',
      disabled: retryMutation.isPending.value,
      'aria-label': t('subscriptionDetail.retryDelivery'),
      onClick: () => {
        retryMutation.mutate(row.original.request_attempt_id, {
          onSuccess: () => toast.success(t('subscriptionDetail.retryQueued')),
          onError: (err) => handleMutationError(err),
        });
      },
    }, () => h(RotateCcw, { size: 16, 'aria-hidden': 'true' })),
}
```

Import `RotateCcw` from lucide-vue-next. Import and call `useRetryDelivery`.

- [ ] **Step 6: Add i18n keys**

In `en.json`, add to `subscriptionDetail`:
```json
"retryDelivery": "Retry this delivery",
"retryQueued": "Retry queued",
"payloadExpired": "Cannot retry — event payload has expired",
"manualRetryBadge": "Retry"
```

- [ ] **Step 7: Type check + lint**

Run: `cd frontend && npx vue-tsc --noEmit && npx eslint src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue src/pages/organizations/applications/logs/ --fix --max-warnings=0`

- [ ] **Step 8: Commit**

```bash
git add frontend/src/
git commit -m "feat(frontend): add manual retry button on deliveries, retry badge for manual attempts"
```

---

## Execution Order

```
Task 1 (migration) → Task 2 (protobuf) → Task 3 (extract helpers) → Task 4 (API handler) → Task 5 (worker) → Task 6 (frontend)
```

Sequential — each task builds on the previous. Task 1-2 are foundations, Task 3-4 are API, Task 5 is worker, Task 6 is frontend.

With 2 agents: Agent 1 does Tasks 1-4 (API), Agent 2 does Task 5 (worker, starts after Task 2) + Task 6 (frontend, starts after Task 4).
