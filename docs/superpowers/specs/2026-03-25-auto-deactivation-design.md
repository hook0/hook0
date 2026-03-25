# Phase 2: Automatic Subscription Deactivation

**Ticket**: [#42 — Customizable delay algorithm strategy on retry](https://gitlab.com/hook0/hook0/-/work_items/42)
**Date**: 2026-03-25
**Status**: Draft
**Depends on**: Phase 1 (Configurable Retry Schedule)

---

## 1. Goal

Automatically disable subscriptions whose retries have been fully exhausted, and notify organization members via email at key moments. This is Phase 2 of ticket #42.

Phase 1 introduced configurable retry schedules (exponential, linear, custom). Phase 2 builds on top: when all retries for a message are exhausted, the subscription is automatically disabled. Warning and recovery emails keep organization members informed.

## 2. Scope

### In scope

- Auto-disable subscription when retries are exhausted (worker-side)
- `auto_disabled_at` and `warning_sent_at` columns on `webhook.subscription`
- Warning email at configurable % of retries (default 50%), deduplicated per subscription
- Deactivation email when subscription is disabled
- Recovery email when a retry succeeds after a warning was sent
- Hook0 event `api.subscription.disabled`
- `auto_disabled_at` exposed in subscription API response
- Extract `hook0-mailer` crate (shared between `api` and `output-worker`)
- Extract `hook0-client` crate (shared between `api` and `output-worker`)
- Feature flag to disable the entire feature

### Out of scope

- Manual retry/recover/replay APIs (Phase 4)
- Frontend UI for deactivation status
- Per-subscription warning threshold override (global env var only)
- Hook0 events for warning or recovery (email only)
- `message.attempt.exhausted` or `message.attempt.failed` events

## 3. Design

### 3.1. Database

Single migration on `webhook.subscription`:

```sql
ALTER TABLE webhook.subscription ADD COLUMN auto_disabled_at timestamptz;
ALTER TABLE webhook.subscription ADD COLUMN warning_sent_at timestamptz;
```

- `auto_disabled_at IS NOT NULL` → disabled by the system (retries exhausted)
- `auto_disabled_at IS NULL AND is_enabled = false` → disabled by the user
- `warning_sent_at IS NOT NULL` → a warning email has been sent for the current failure sequence
- No new table. No index on these columns (never queried directly).

**Worker sets on warning:**
```sql
UPDATE webhook.subscription
SET warning_sent_at = statement_timestamp()
WHERE subscription__id = $1 AND warning_sent_at IS NULL
-- Returns 1 row affected → send email. 0 rows → skip (already sent).
```

**Worker sets on deactivation:**
```sql
UPDATE webhook.subscription
SET is_enabled = false, auto_disabled_at = statement_timestamp()
WHERE subscription__id = $1 AND is_enabled = true
```

**Worker sets on recovery (retry succeeds after warning):**
```sql
UPDATE webhook.subscription
SET warning_sent_at = NULL
WHERE subscription__id = $1 AND warning_sent_at IS NOT NULL
-- Returns 1 row affected → send recovery email. 0 rows → skip (no warning was sent).
```

**Manual re-activation (API PUT, only when is_enabled transitions false → true):**
```sql
SET is_enabled = true, auto_disabled_at = NULL, warning_sent_at = NULL
```

### 3.2. Worker Logic

Three new behaviors added to the existing retry flow. The worker needs to resolve `application_id → organization_id → member emails` via DB queries (same pattern as `api/src/quotas.rs` lines 462-472).

#### 3.2.1. Warning email

When a retry fails and the per-message `retry_count == warning_threshold`:

```
warning_threshold = ceil(max_retries * SUBSCRIPTION_WARNING_AT_RETRY_PERCENT / 100)
```

Each message has its own `retry_count` (0, 1, 2, ...) and will independently cross the threshold. The `warning_sent_at` atomic UPDATE deduplicates across messages: if the UPDATE affects 1 row (first message to cross the threshold for this subscription), send the `SubscriptionWarning` email. If 0 rows (another message already triggered it), skip.

The threshold is computed from the schedule's `max_retries` (or the worker's default if no schedule assigned).

#### 3.2.2. Auto-deactivation

When retries are exhausted (existing "give up" code path), if `--disable-subscription-on-retries-exhausted` is `true`:

1. `UPDATE webhook.subscription SET is_enabled = false, auto_disabled_at = statement_timestamp() WHERE subscription__id = $1 AND is_enabled = true`
2. If 1 row affected: send `SubscriptionDisabled` email to all organization members + emit `api.subscription.disabled` Hook0 event
3. If 0 rows affected: subscription already disabled (by user or by another message's exhaustion), skip

If the feature flag is `false`, the worker behaves exactly as today (logs "giving up", moves on).

**Known trade-off**: A single message exhausting its retries disables the subscription for all subsequent messages. With an aggressive schedule (e.g., `max_retries: 1`), a transient failure can cause deactivation after just 2 delivery attempts. This is by design — the user chose the schedule and accepts the consequences.

#### 3.2.3. Recovery email

When any delivery **succeeds** (including initial delivery at `retry_count=0`) and the subscription has `warning_sent_at IS NOT NULL`:

The worker clears `warning_sent_at` and sends a `SubscriptionRecovered` email. This covers the case where a warning was sent (subscription was struggling) but the target recovered before exhaustion — even if the recovery is detected by a different message succeeding on first attempt. No deactivation occurs. Deduplicated the same way: `UPDATE SET warning_sent_at = NULL WHERE warning_sent_at IS NOT NULL` — 1 row = send, 0 rows = skip.

#### 3.2.4. Default schedule behavior

When no retry schedule is assigned, David's default applies (`max_retries` = worker's `--max-retries`, default 25). The warning is sent at `ceil(25 * 0.50) = 13`. Deactivation at retry 25.

#### 3.2.5. Both code paths

The deactivation + warning + recovery logic must be implemented in both the PG polling path (`output-worker/src/pg.rs`) and the Pulsar path (`output-worker/src/pulsar.rs`). A shared function should be extracted to avoid duplicating the logic.

#### 3.2.6. Email sending is best-effort

Email sending failures (SMTP unreachable, timeout) do not block deactivation or Hook0 event emission. The worker logs a warning and proceeds. The DB state change (deactivation) is the source of truth, not the email.

### 3.3. Crate Extraction

Two new crates, one per concern:

```
hook0/
├── api/
├── output-worker/
├── hook0-mailer/       # new
├── hook0-client/       # new
└── Cargo.toml          # workspace members
```

#### `hook0-mailer`

Extracted from `api/src/mailer.rs`:
- `Mailer` struct (async SMTP via `lettre`, connection-pooled)
- `Mail` enum with all variants (existing: `VerifyUserEmail`, `ResetPassword`, `QuotaEventsPerDayWarning`, `QuotaEventsPerDayReached` + new: `SubscriptionWarning`, `SubscriptionDisabled`, `SubscriptionRecovered`)
- MJML template pipeline (`include_str!` + `mrml` rendering + `html2text` fallback)
- Dependencies: `lettre`, `mrml`, `html2text`

#### `hook0-client`

Extracted from `api/src/hook0_client.rs`:
- `Hook0Client` struct, `Hook0ClientEvent` enum, `mk_hook0_event`, `initialize()`
- All existing event types + new `EventSubscriptionDisabled`
- `"api.subscription.disabled"` added to `EVENT_TYPES`

Both crates consumed by `api` and `output-worker`.

### 3.4. Emails

Three new MJML templates. All sent to **all members of the organization** (same SQL pattern as quota notification emails in `api/src/quotas.rs`: `application_id → organization_id → user emails`).

**Language:** English (consistent with existing templates).

Note: the `webhook.subscription` table has no `name` field. Email templates use `description` as the subscription display name, falling back to `subscription_id` if description is null.

#### `SubscriptionWarning`

- **Trigger:** `retry_count == warning_threshold` AND `warning_sent_at` was NULL (first message to cross threshold)
- **Subject:** `[Hook0] Subscription failing: {subscription_description}`
- **Content:** application name, subscription description/ID, target URL, number of failed attempts, retries remaining, instructions to verify the target

#### `SubscriptionDisabled`

- **Trigger:** retries exhausted, subscription auto-disabled (first message to exhaust)
- **Subject:** `[Hook0] Subscription disabled: {subscription_description}`
- **Content:** application name, subscription description/ID, target URL, deactivation date, total attempts, instructions to re-activate (API PUT `is_enabled: true`)

#### `SubscriptionRecovered`

- **Trigger:** retry succeeds AND `warning_sent_at` was NOT NULL
- **Subject:** `[Hook0] Subscription recovered: {subscription_description}`
- **Content:** application name, subscription description/ID, target URL, confirmation that delivery succeeded

### 3.5. Hook0 Event

New event type: `api.subscription.disabled`

Payload (nested objects, not flattened):

```json
{
  "subscription": {
    "subscription_id": "uuid",
    "organization_id": "uuid",
    "application_id": "uuid",
    "description": "string|null",
    "target": "url",
    "auto_disabled_at": "timestamptz"
  },
  "retry_schedule": {
    "retry_schedule_id": "uuid",
    "name": "string",
    "strategy": "exponential|linear|custom",
    "max_retries": 25,
    "custom_intervals": "int[]|null",
    "linear_delay": "int|null"
  }
}
```

`retry_schedule` is `null` when the worker's default schedule was used.

Emitted by the worker via the `hook0-client` crate at the moment of deactivation.

No Hook0 events for warning or recovery — those are email-only.

### 3.6. API Changes

**Subscription response:** Add `auto_disabled_at: Option<DateTime<Utc>>` to the `Subscription` response struct. This allows API consumers to distinguish user-disabled from system-disabled subscriptions.

**Subscription PUT handler:** When `is_enabled` transitions from `false` to `true` (re-activation), clear both `auto_disabled_at` and `warning_sent_at`. A PUT that does not change `is_enabled` (e.g., updating description while subscription is active) must NOT clear these columns — doing so would silently reset an in-progress warning sequence.

No new API endpoints.

### 3.7. Configuration

**New worker args (clap, all with env var equivalents):**

| Arg | Env var | Type | Default | Description |
|---|---|---|---|---|
| `--disable-subscription-on-retries-exhausted` | `DISABLE_SUBSCRIPTION_ON_RETRIES_EXHAUSTED` | bool | `true` | Enable/disable the entire auto-deactivation feature |
| `--subscription-warning-at-retry-percent` | `SUBSCRIPTION_WARNING_AT_RETRY_PERCENT` | u8 | `50` | Retry % threshold at which warning email is sent |

**New worker args for hook0-client (optional — if absent, no events emitted):**
- `--hook0-client-api-url`
- `--hook0-client-application-id`
- `--hook0-client-token`

**New worker args for mailer (optional — if absent, no emails sent, warning logged):**
- `--smtp-connection-url`
- `--smtp-timeout`
- `--email-sender-address`
- `--email-sender-name`

## 4. Security

- Auto-deactivation is a write from the worker to the DB — uses the same connection pool already trusted for attempt updates
- Email recipients resolved via SQL scoped to `organization__id` (same pattern as quota emails)
- Hook0 event emitted via authenticated hook0-client (same auth as API)
- Feature flag allows operators to disable the behavior entirely
- No new API endpoints — no new attack surface

## 5. Design Decisions

| # | Question | Decision | Ticket divergence |
|---|---|---|---|
| 1 | Deactivation trigger | Retries exhausted (not 5 calendar days) | **Yes** — ticket says 5 days; we trigger on retries exhausted because configurable schedules make fixed calendar thresholds meaningless |
| 2 | Number of messages to trigger | Single message with retries exhausted is enough | **Yes** — ticket implies continuous failure pattern over days |
| 3 | Minimum duration safeguard | None — trust the user's schedule configuration | — |
| 4 | Subscription status model | Keep `is_enabled` bool + add `auto_disabled_at` column | Additive, no breaking change |
| 5 | Feature flag | Worker clap arg `--disable-subscription-on-retries-exhausted` (default true) | Aligned (ticket says env var toggle) |
| 6 | Warning email trigger | At `ceil(max_retries * percent / 100)`, percent configurable (default 50%) | **Yes** — ticket says 3 calendar days |
| 7 | Deactivation email | Sent when subscription is auto-disabled | Aligned |
| 8 | Recovery email | Sent when any delivery succeeds (including retry_count=0) after warning was sent (warning_sent_at IS NOT NULL) | Aligned (ticket's optional recovery notification) |
| 9 | Email deduplication | `warning_sent_at` column on subscription — atomic UPDATE deduplicates across concurrent messages | Simplified vs ticket's approach |
| 10 | Hook0 events | Only `api.subscription.disabled` (no warning event, no attempt events) | Simplified |
| 11 | Event naming | `api.subscription.disabled` (not `endpoint.disabled`) | Aligned with existing `api.subscription.*` naming |
| 12 | Event payload | Nested `subscription` + `retry_schedule` objects | — |
| 13 | Crate architecture | Separate `hook0-mailer` and `hook0-client` crates (not a single shared crate) | — |
| 14 | Email recipients | All organization members | Aligned with existing quota email pattern |
| 15 | Worker as email sender | Worker gets its own mailer via `hook0-mailer` crate | New — currently only API sends emails |
| 16 | Subscription display name | Use `description` field (no `name` column on subscription) | — |
| 17 | Email sending failures | Best-effort, do not block deactivation | — |
| 18 | PUT clearing semantics | Clear `auto_disabled_at` + `warning_sent_at` only when `is_enabled` transitions false → true, not on every PUT | — |

## 6. Open Questions for Future Phases

1. **Per-subscription warning threshold**: Currently global (env var). Could become a field on `retry_schedule` if users want different warning levels per schedule.
2. **Frontend deactivation UI**: Display `auto_disabled_at` in subscription detail, show re-activation button with explanation. Requires frontend work.
3. **Manual retry (Phase 4)**: When manually retrying a failed message on a disabled subscription, should it auto-re-enable? Or require explicit re-activation first?
4. **`max_retry_window` interaction**: If David's `max_retry_window` becomes enforced at runtime, it could cause retries to stop before `max_retries` is reached. The warning threshold would still be computed from `max_retries`, but deactivation might trigger earlier than expected — resulting in deactivation without warning. This is a known limitation to address if `max_retry_window` enforcement is planned.
5. **Phase numbering**: Phase 1 spec lists Phase 3 as "Email notifications" separately. This Phase 2 spec merges deactivation + emails. Phase 1's out-of-scope section should be updated to reflect the consolidated phasing.
