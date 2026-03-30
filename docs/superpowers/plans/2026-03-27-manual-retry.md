# Manual Retry — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `POST /api/v1/request_attempts/{id}/retry` endpoint that creates a one-shot manual retry, bypasses disabled subscriptions, and is excluded from the health monitor's failure ratio.

**Architecture:** Migration adds `source` + `user__id` columns to `request_attempt`. New API handler inserts a retry attempt + publishes to Pulsar if configured. Worker skips auto-retry for `source = 'user'` and picks up manual attempts on disabled subscriptions. Health monitor excludes `source = 'user'` from its ratio.

**Tech Stack:** Rust, actix-web, sqlx (Postgres), Pulsar (optional), k6 (tests)

**Spec:** `docs/superpowers/specs/2026-03-27-manual-retry-design.md`

**Prerequisite:** Phase 2 (feat/auto-deactivation) must be merged. This branch starts from Phase 2.

---

## Conventions

**Build check:** After every task, run `cargo check -p hook0-api` (or appropriate crate).

**Commit:** After every task, commit with the message provided.

**SQL style:** Lowercase keywords, matching existing migrations.

**Anchors:** File locations use pattern anchors, not line numbers.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `api/migrations/20260327120000_add_request_attempt_source.up.sql` | Create | Add `source`, `user__id`, constraint to `request_attempt` |
| `api/migrations/20260327120000_add_request_attempt_source.down.sql` | Create | Rollback |
| `api/src/iam.rs` | Modify | Add `RequestAttemptRetry` action variant |
| `api/src/handlers/request_attempts.rs` | Modify | Add `retry` handler + `source`/`user_id` in list response |
| `api/src/main.rs` | Modify | Register retry route |
| `output-worker/src/main.rs` | Modify | Add `source` field to struct, skip retry for manual attempts |
| `output-worker/src/pg.rs` | Modify | Add `ra.source` to SELECT, bypass `is_enabled` for manual |
| `output-worker/src/pulsar.rs` | Modify | Modify `not_cancelled` to allow manual attempts on disabled subs |
| `api/src/health_monitor.rs` | Modify | Add `and ra.source = 'system'` to evaluation query |
| `tests-api-integrations/src/retry/retry_request_attempt.js` | Create | k6 test for retry endpoint |
| `tests-api-integrations/src/main.js` | Modify | Wire retry test scenario |

---

## Task 1: Database Migration

- [ ] **Step 1: Write up migration**

```sql
-- api/migrations/20260327120000_add_request_attempt_source.up.sql

-- 'system' = automatic (dispatch trigger, worker retry)
-- 'user' = manual retry via API
-- default 'system' handles existing rows and dispatch trigger INSERTs
alter table webhook.request_attempt
    add column source text not null default 'system'
        check (source in ('system', 'user'));

-- NULL = system or service token, NOT NULL = action by this user
alter table webhook.request_attempt
    add column user__id uuid
        references iam.user(user__id)
        on delete set null;

-- source = 'system' must have user__id = NULL
alter table webhook.request_attempt
    add constraint request_attempt_source_user_check
        check (source != 'system' or user__id is null);
```

- [ ] **Step 2: Write down migration**

```sql
-- api/migrations/20260327120000_add_request_attempt_source.down.sql

alter table webhook.request_attempt drop constraint if exists request_attempt_source_user_check;
alter table webhook.request_attempt drop column if exists user__id;
alter table webhook.request_attempt drop column if exists source;
```

- [ ] **Step 3: Verify compilation**

Commit: `feat(db): add source and user__id columns to request_attempt for manual retry`

---

## Task 2: IAM Action — `RequestAttemptRetry`

- [ ] **Step 1: Add action variant**

In `api/src/iam.rs`, find `RequestAttemptList` in the `Action` enum. After it, add:

```rust
RequestAttemptRetry {
    application_id: &'a Uuid,
},
```

- [ ] **Step 2: Add 5 match arms**

Following the existing pattern (search for `RequestAttemptList` in each match):

- `action_name()`: `Self::RequestAttemptRetry { .. } => "request_attempt:retry",`
- `allowed_roles()`: `Self::RequestAttemptRetry { .. } => vec![],` — Empty vec means Editor-only (Editor is always included as the base role in the IAM system)
- `application_id()`: `Self::RequestAttemptRetry { application_id, .. } => Some(**application_id),`
- `generate_facts()`: `Self::RequestAttemptRetry { .. } => vec![],`
- `can_work_without_organization()`: falls through to default `false` — no change needed

Commit: `feat(api): add RequestAttemptRetry IAM action`

---

## Task 3: Retry Handler + Route Registration

- [ ] **Step 1: Add the retry handler to `api/src/handlers/request_attempts.rs`**

After the `list` handler, add:

```rust
#[api_v2_operation(
    summary = "Retry a request attempt",
    operation_id = "request_attempts.retry",
    consumes = "application/json",
    produces = "application/json",
    tags("Request Attempts")
)]
pub async fn retry(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    attempt_id: Path<Uuid>,
) -> Result<CreatedJson<RetryResponse>, Hook0Problem> {
    let attempt_id = attempt_id.into_inner();

    // 1. Fetch original attempt + resolve application_id for auth
    let original = sqlx::query_as::<_, OriginalAttempt>(
        "select ra.event__id, ra.subscription__id, ra.application__id
         from webhook.request_attempt ra
         inner join webhook.subscription s on s.subscription__id = ra.subscription__id
         inner join event.application a on a.application__id = ra.application__id
         where ra.request_attempt__id = $1
           and s.deleted_at is null
           and a.deleted_at is null"
    )
    .bind(attempt_id)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    // 2. Auth + extract user_id
    let authorized_token = authorize_for_application(
        &state.db, &biscuit,
        Action::RequestAttemptRetry { application_id: &original.application__id },
        state.max_authorization_time_in_ms, state.debug_authorizer,
    ).await.map_err(|_| Hook0Problem::Forbidden)?;

    let auth_user_id = match authorized_token {
        AuthorizedToken::User(u) => Some(u.user_id),
        _ => None, // Service token or master key
    };

    // 4. INSERT new attempt
    let new_attempt = sqlx::query_as::<_, RetryResponse>(
        "insert into webhook.request_attempt (event__id, subscription__id, application__id, source, user__id)
         values ($1, $2, $3, 'user', $4)
         returning request_attempt__id as request_attempt_id, event__id as event_id,
                  subscription__id as subscription_id, source, user__id as user_id,
                  created_at, retry_count"
    )
    .bind(original.event__id)
    .bind(original.subscription__id)
    .bind(original.application__id)
    .bind(auth_user_id)
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // 5. Pulsar publish if configured
    // Follow the pattern from events.rs send_request_attempts_to_pulsar
    // This requires fetching subscription target details for the Pulsar message

    Ok(CreatedJson(new_attempt))
}
```

Define the helper structs:

```rust
#[derive(sqlx::FromRow)]
struct OriginalAttempt {
    event__id: Uuid,
    subscription__id: Uuid,
    application__id: Uuid,
}

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct RetryResponse {
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    pub subscription_id: Uuid,
    pub source: String,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
}
```

> **Note:** `retry_count` defaults to 0 (fresh attempt, not a continuation of the original retry chain). This is intentional.

- [ ] **Step 2: Register the route in `api/src/main.rs`**

Find the `request_attempts` scope. Add the retry route:

```rust
.service(
    web::scope("/request_attempts")
        .wrap(Compat::new(rate_limiters.token()))
        .wrap(biscuit_auth.clone())
        .service(
            web::resource("")
                .route(web::get().to(handlers::request_attempts::list)),
        )
        .service(
            web::resource("/{request_attempt_id}/retry")
                .route(web::post().to(handlers::request_attempts::retry)),
        ),
)
```

> **Note:** Prefer `query_as!` macro for compile-time checking where possible; fall back to runtime `query_as` only if the offline sqlx cache is stale.

- [ ] **Step 3: Verify compilation + tests**

Commit: `feat(api): add POST /request_attempts/{id}/retry endpoint`

---

## Task 4: Worker — Skip Auto-Retry + Bypass is_enabled

- [ ] **Step 1: Add `source` to `RequestAttemptWithOptionalPayload`**

In `output-worker/src/main.rs`, find the struct. Add:

```rust
pub source: String,
```

- [ ] **Step 2: Add `ra.source` to PG fetch query**

In `output-worker/src/pg.rs`, find the fetch query SELECT. Add `ra.source` after `s.secret`.

- [ ] **Step 3: Modify PG WHERE clause for manual bypass**

Change `AND s.is_enabled` to `AND (s.is_enabled OR ra.source = 'user')`.

- [ ] **Step 4: Add source check before compute_next_retry in PG path**

In `pg.rs`, find where `compute_next_retry` is called (after a failed attempt). Before the call, add:

```rust
if attempt.source == "user" {
    // Manual retry — one-shot, no automatic re-retry
    info!("Manual retry failed, no automatic re-retry");
    // Mark failed and continue (existing failed_at logic already handles this)
} else {
    // Existing compute_next_retry logic
}
```

- [ ] **Step 5: Modify Pulsar status check**

In `output-worker/src/pulsar.rs`, find the `not_cancelled` computation. Change:

```sql
(s.is_enabled AND a.deleted_at IS NULL) AS "not_cancelled!"
```
to:
```sql
((s.is_enabled OR ra.source = 'user') AND a.deleted_at IS NULL) AS "not_cancelled!"
```

Also add `ra.source` to this SELECT — specifically, add `source: String` to the `RawRequestAttemptStatus` struct in pulsar.rs, and `ra.source AS "source!"` to the SELECT. The source check in the Pulsar path uses this DB-fetched value, NOT the protobuf `RequestAttempt`.

The Pulsar path will also need the `source` check before `compute_next_retry`.

- [ ] **Step 6: Update `load_waiting_request_attempts_from_db` in `pulsar.rs`**

Its SELECT also needs `ra.source` since it builds `RequestAttemptWithOptionalPayload`.

- [ ] **Step 7: Verify compilation + worker unit tests**

Commit: `feat(worker): support manual retry — skip auto-retry, bypass is_enabled`

---

## Task 5: Health Monitor — Exclude Manual Attempts

- [ ] **Step 1: Add `and ra.source = 'system'` to the health evaluation query**

In `api/src/health_monitor.rs`, find the `attempt_stats` CTE. In the WHERE clause, after the line filtering completed attempts, add:

```sql
and ra.source = 'system'
```

This ensures manual retries don't affect the health ratio.

- [ ] **Step 2: Verify compilation**

Commit: `feat(api): exclude manual retries from health monitor ratio`

---

## Task 6: Request Attempt List — Add `source` and `user_id` to Response

- [ ] **Step 1: Add fields to `RequestAttempt` response struct**

In `api/src/handlers/request_attempts.rs`, add to the `RequestAttempt` struct:

```rust
pub source: String,
pub user_id: Option<Uuid>,
```

- [ ] **Step 2: Add columns to the list query SELECT**

Add `ra.source` and `ra.user__id AS user_id` to the list query.

- [ ] **Step 3: Verify compilation**

Commit: `feat(api): add source and user_id to request attempt list response`

---

## Task 7: Build Verification

- [ ] **Step 1: Build entire workspace**

```bash
cargo build
```

- [ ] **Step 2: Run unit tests**

```bash
cargo test
```

Commit: `chore: verify full workspace build for manual retry`

---

## Task 8: k6 Integration Tests

- [ ] **Step 1: Create retry test helper**

Create `tests-api-integrations/src/retry/retry_request_attempt.js`:

Tests to include:
- Retry a failed attempt → 201, new attempt created with `source = 'user'`
- Retry a succeeded attempt → 201 (allowed)
- Retry nonexistent attempt → 404
- Retry attempt on disabled subscription → 201 (bypass)
- Verify the new attempt is picked up by worker (poll until succeeded/failed)
- Verify `source` and `user_id` appear in list response

- [ ] **Step 2: Wire into main.js as a parallel scenario**

- [ ] **Step 3: Run tests locally to verify**

Commit: `test: add k6 integration tests for manual retry endpoint`

---

## Notes for Implementation

- **Auth pattern**: This is a fetch-before-auth handler (the only input is `request_attempt_id`). The original attempt must be fetched first to get `application_id` for authorization.
- **Pulsar publish**: Follow the exact pattern in `events.rs:send_request_attempts_to_pulsar`. The retry handler needs to fetch subscription target details (http_method, http_url, etc.) to build the Pulsar message.
- **`source` field on `RequestAttemptWithOptionalPayload`**: Since the column has `DEFAULT 'system'`, existing sqlx offline cache will need to be updated (`cargo sqlx prepare`).
- **Protobuf struct**: If Pulsar is used, the `RequestAttempt` protobuf struct in `protobuf/src/request_attempt.rs` does NOT need a `source` field — the worker can read `source` from the DB status-check query.
