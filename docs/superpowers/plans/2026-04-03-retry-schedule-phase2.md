# Retry Schedule Phase 2 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add subscription detail page with health timeline, improve slider UX with human-readable text input, add health badge icons + click affordance, add retry jitter, and fix backend issues.

**Architecture:** 6 independent workstreams — 3 frontend (subscription detail page, slider input, badge improvements), 2 backend (jitter, SQL fixes), 1 cleanup (remove logs subscription filter). Each produces working, testable software independently.

**Tech Stack:** Vue 3, TanStack Query, VeeValidate + Zod, Rust (actix-web, sqlx, clap), PostgreSQL

---

## Design Decisions (from grill review)

| # | Decision | Choice |
|---|----------|--------|
| 1 | Click name in subscription list | Goes to detail page. Edit button (Pencil icon) per row in actions column. |
| 2 | Double empty state on detail | Merge into single empty state when both deliveries AND health events are empty. |
| 3 | Health chart | Simple vertical timeline with colored event markers — NO ECharts. |
| 4 | Slider invalid input | Input stays red + error message with expected range. No auto-revert. |
| 5 | Jitter in preview chips | Ignore — chips show theoretical delay. Jitter is a worker implementation detail. |
| 6 | Health badge click target | Goes to SubscriptionsDetail (not LogsList). |
| 7 | Edit/Delete in subscription list | Icon buttons (Pencil + Trash2) in same cell. Target column hidden on mobile. |
| 8 | Migration name length | Modify the initial migration directly (not a new migration). |
| 9 | Health badge icons | Keep all 4 (accessibility for colorblind users). |
| 10 | Deliveries on detail page | Cursor-based pagination. |

---

## File Structure

### New files
- `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue` — subscription detail page (header + deliveries + health timeline)
- `frontend/src/pages/organizations/applications/subscriptions/SubscriptionHealthTimeline.vue` — vertical timeline of health events
- `frontend/src/pages/organizations/applications/subscriptions/useSubscriptionHealthQueries.ts` — TanStack Query for health events API
- `frontend/src/pages/organizations/applications/subscriptions/SubscriptionHealthService.ts` — API layer for health events
- `frontend/src/utils/parseDuration.ts` — human-readable duration parser ("1h30min" → 5400)

### Modified files
- `frontend/src/components/Hook0Slider.vue` — add editable text input mode
- `frontend/src/components/Hook0HealthBadge.vue` — add icons per status
- `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue` — name links to detail, add Edit icon button, health badge links to detail, hide target on mobile
- `frontend/src/pages/organizations/applications/logs/LogList.vue` — remove subscription filter
- `frontend/src/pages/organizations/applications/logs/LogService.ts` — revert to single param, add listBySubscription
- `frontend/src/pages/organizations/applications/logs/useLogQueries.ts` — revert to single param, add useLogListBySubscription
- `frontend/src/queries/keys.ts` — revert logKeys, add healthEventKeys
- `frontend/src/routes.ts` — add SubscriptionsDetail route
- `frontend/src/composables/useNavigationTabs.ts` — add SubscriptionsDetail to active check
- `frontend/src/pages/OrganizationAndApplicationSelector.vue` — add SubscriptionsDetail mapping
- `frontend/src/composables/useCommandPalette.ts` — add SubscriptionsDetail mapping
- `frontend/src/locales/en.json` — add subscriptionDetail.* keys
- `output-worker/src/retry.rs` — add jitter after delay computation
- `output-worker/src/main.rs` — add retry_jitter_factor config field
- `output-worker/Cargo.toml` — add rand dependency
- `api/src/health_monitor/queries.rs` — replace NOT EXISTS with DISTINCT ON
- `api/migrations/20260325120000_add_retry_schedule.up.sql` — add length(name) <= 200 to CHECK

---

## Task 1: parseDuration utility

**Files:**
- Create: `frontend/src/utils/parseDuration.ts`

- [ ] **Step 1: Write parseDuration**

```typescript
const UNITS: Record<string, number> = {
  s: 1,
  sec: 1,
  min: 60,
  h: 3600,
  d: 86400,
};

const TOKEN_REGEX = /(\d+(?:\.\d+)?)\s*(d|h|min|sec|s)/gi;
const VALID_CHARS_REGEX = /^[\d\s.dhmseinc]+$/i;

export function parseDuration(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) return null;

  const asNumber = Number(trimmed);
  if (!Number.isNaN(asNumber) && asNumber >= 0) return Math.round(asNumber);

  if (!VALID_CHARS_REGEX.test(trimmed)) return null;

  let total = 0;
  let matched = false;
  let totalMatchedLength = 0;
  let match: RegExpExecArray | null;

  TOKEN_REGEX.lastIndex = 0;
  while ((match = TOKEN_REGEX.exec(trimmed)) !== null) {
    const value = parseFloat(match[1]);
    const unit = match[2].toLowerCase();
    const multiplier = UNITS[unit];
    if (multiplier === undefined) return null;
    total += value * multiplier;
    totalMatchedLength += match[0].length;
    matched = true;
  }

  const nonWhitespaceLength = trimmed.replace(/\s/g, '').length;
  if (!matched || totalMatchedLength < nonWhitespaceLength) return null;

  return Math.round(total);
}
```

- [ ] **Step 2: Verify**

Run: `cd frontend && npx tsx -e "import('./src/utils/parseDuration.ts').then(m => { const p = m.parseDuration; console.log(p('1h30min'), p('3s'), p('90'), p('invalid'), p('1habc')); })"`
Expected: `5400 3 90 null null`

- [ ] **Step 3: Commit**

```bash
git add frontend/src/utils/parseDuration.ts
git commit -m "feat(frontend): add parseDuration utility for human-readable duration input"
```

---

## Task 2: Hook0Slider — editable text input

**Files:**
- Modify: `frontend/src/components/Hook0Slider.vue`
- Modify: `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue`

- [ ] **Step 1: Add imports, props, state, and functions**

Add `ref` to vue import. Add `parseDuration` import. Add `editable?: boolean` prop (default `false`).

```typescript
import { parseDuration } from '@/utils/parseDuration';

const isEditing = ref(false);
const editText = ref('');
const editError = ref(false);
const editErrorMessage = ref('');

function startEditing() {
  if (!props.editable) return;
  editText.value = displayValue.value;
  editError.value = false;
  editErrorMessage.value = '';
  isEditing.value = true;
}

function confirmEdit() {
  const parsed = parseDuration(editText.value);
  if (parsed === null) {
    editError.value = true;
    editErrorMessage.value = `Expected: 1s, 5min, 1h30min, 2d`;
    return;
  }
  if (parsed < props.min || parsed > props.max) {
    editError.value = true;
    editErrorMessage.value = `Range: ${props.min}s – ${props.max}s`;
    return;
  }
  emit('update:modelValue', parsed);
  isEditing.value = false;
  editError.value = false;
}

function cancelEdit() {
  isEditing.value = false;
  editError.value = false;
}
```

- [ ] **Step 2: Update template**

Replace the `<span class="hook0-slider__value">` with:
```html
<div v-if="isEditing" class="hook0-slider__edit-wrapper">
  <input
    v-model="editText"
    class="hook0-slider__edit-input"
    :class="{ 'hook0-slider__edit-input--error': editError }"
    @keydown.enter="confirmEdit"
    @keydown.escape="cancelEdit"
    autofocus
  />
  <p v-if="editError" class="hook0-slider__edit-error">{{ editErrorMessage }}</p>
</div>
<span
  v-else
  class="hook0-slider__value"
  :class="{ 'hook0-slider__value--editable': editable }"
  @click="startEditing"
>
  {{ displayValue }}
</span>
```

Note: NO `@blur="confirmEdit"` — the user must press Enter or Escape. Blur with error state would cause UX confusion.

- [ ] **Step 3: Add styles**

```css
.hook0-slider__value--editable {
  cursor: text;
  border-bottom: 1px dashed var(--color-border);
}

.hook0-slider__value--editable:hover {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.hook0-slider__edit-wrapper {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.125rem;
}

.hook0-slider__edit-input {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-primary);
  font-variant-numeric: tabular-nums;
  background: none;
  border: none;
  border-bottom: 2px solid var(--color-primary);
  outline: none;
  width: 6rem;
  text-align: right;
  padding: 0;
}

.hook0-slider__edit-input--error {
  border-bottom-color: var(--color-error);
  color: var(--color-error);
}

.hook0-slider__edit-error {
  font-size: 0.6875rem;
  color: var(--color-error);
  margin: 0;
}
```

- [ ] **Step 4: Enable editable on duration sliders in RetrySchedulesEdit.vue**

Add `:editable="true"` to Hook0Slider instances for base delay and linear delay (those with `:format-value="formatDuration"`). NOT on wait factor or max retries.

- [ ] **Step 5: Type check + lint**

Run: `cd frontend && npx vue-tsc --noEmit && npx eslint src/components/Hook0Slider.vue src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue --fix --max-warnings=0`

- [ ] **Step 6: Commit**

```bash
git add frontend/src/components/Hook0Slider.vue frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue
git commit -m "feat(frontend): add editable text input to slider for human-readable duration entry"
```

---

## Task 3: Health badge icons + subscription list improvements

**Files:**
- Modify: `frontend/src/components/Hook0HealthBadge.vue`
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue`

- [ ] **Step 1: Add icons to Hook0HealthBadge**

```typescript
import { CheckCircle, AlertTriangle, XCircle, Minus } from 'lucide-vue-next';

const statusIcon = computed(() => {
  switch (status.value) {
    case 'healthy': return CheckCircle;
    case 'warning': return AlertTriangle;
    case 'critical': return XCircle;
    default: return Minus;
  }
});
```

Template:
```html
<Hook0Badge :variant="variant" size="sm">
  <component :is="statusIcon" :size="12" aria-hidden="true" />
  {{ label }}
</Hook0Badge>
```

- [ ] **Step 2: Update SubscriptionsList — Edit icon button + health links to detail**

In the actions column, add Edit (Pencil) icon button before Delete (Trash2). Both as `Hook0Button variant="ghost"`. Import `Pencil` from lucide.

Change health badge `RouterLink` destination from `routes.LogsList` to `routes.SubscriptionsDetail`:
```typescript
to: {
  name: routes.SubscriptionsDetail,
  params: {
    organization_id: route.params.organization_id,
    application_id: route.params.application_id,
    subscription_id: row.original.subscription_id,
  },
},
```

Remove `subscription_name` query param (no longer needed — detail page fetches its own data).

- [ ] **Step 3: Hide target column on mobile**

Add to the target column definition: `meta: { hideOnMobile: true }` (or use CSS). Check how Hook0Table handles responsive columns — if it doesn't support column hiding, add a CSS class:

```css
@media (max-width: 768px) {
  .subscription-table :deep(td:nth-child(3)),
  .subscription-table :deep(th:nth-child(3)) {
    display: none;
  }
}
```

- [ ] **Step 4: Enhance health badge click affordance**

```css
.subscription__health-link:hover {
  opacity: 0.85;
  transform: scale(1.05);
  transition: all 0.15s ease;
}
```

- [ ] **Step 5: Type check + lint**

Run: `cd frontend && npx vue-tsc --noEmit && npx eslint src/components/Hook0HealthBadge.vue src/pages/organizations/applications/subscriptions/SubscriptionsList.vue --fix --max-warnings=0`

- [ ] **Step 6: Commit**

```bash
git add frontend/src/components/Hook0HealthBadge.vue frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue
git commit -m "feat(frontend): add status icons to health badges, edit button in subscription list, responsive target column"
```

---

## Task 4: Remove subscription filter from LogList + add listBySubscription

**Files:**
- Modify: `frontend/src/pages/organizations/applications/logs/LogList.vue`
- Modify: `frontend/src/pages/organizations/applications/logs/LogService.ts`
- Modify: `frontend/src/pages/organizations/applications/logs/useLogQueries.ts`
- Modify: `frontend/src/queries/keys.ts`
- Modify: `frontend/src/locales/en.json`

- [ ] **Step 1: Revert LogService.list + add listBySubscription**

Revert `list()` to single `application_id` param (remove `ListParams` type). Add:

```typescript
export function listBySubscription(
  application_id: UUID,
  subscription_id: UUID
): Promise<Array<RequestAttemptTypeFixed>> {
  return unwrapResponse(
    http.get<Array<RequestAttemptTypeFixed>>('/request_attempts', {
      params: {
        application_id,
        subscription_id,
        min_created_at: subDays(new Date(), 7).toISOString(),
      },
    })
  );
}
```

- [ ] **Step 2: Revert useLogQueries + add useLogListBySubscription**

Revert `useLogList` to single param. Add:

```typescript
export function useLogListBySubscription(
  applicationId: Ref<string>,
  subscriptionId: Ref<string>
) {
  return useQuery({
    queryKey: computed(() => logKeys.bySubscription(applicationId.value, subscriptionId.value)),
    queryFn: () => LogService.listBySubscription(applicationId.value, subscriptionId.value),
    enabled: computed(() => !!applicationId.value && !!subscriptionId.value),
  });
}
```

- [ ] **Step 3: Update logKeys in keys.ts**

```typescript
export const logKeys = {
  all: ['logs'] as const,
  lists: () => [...logKeys.all, 'list'] as const,
  list: (applicationId: string) => [...logKeys.lists(), applicationId] as const,
  bySubscription: (applicationId: string, subscriptionId: string) =>
    [...logKeys.lists(), applicationId, 'subscription', subscriptionId] as const,
};
```

- [ ] **Step 4: Strip filter UI from LogList.vue**

Remove: `subscriptionIdFilter`, `subscriptionNameFilter`, `clearSubscriptionFilter`, filter badge template, `Filter`/`X`/`RouterLink` imports, `Hook0Badge` import, all `.logs-filter-*` CSS. Revert `useLogList` call to single param.

- [ ] **Step 5: Remove `logs.filteredBySubscription` from en.json**

- [ ] **Step 6: Type check + lint**

- [ ] **Step 7: Commit**

```bash
git add frontend/src/pages/organizations/applications/logs/ frontend/src/queries/keys.ts frontend/src/locales/en.json
git commit -m "refactor(frontend): remove subscription filter from logs, add listBySubscription for detail page"
```

---

## Task 5: Subscription detail page

**Files:**
- Create: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue`
- Create: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionHealthTimeline.vue`
- Create: `frontend/src/pages/organizations/applications/subscriptions/useSubscriptionHealthQueries.ts`
- Create: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionHealthService.ts`
- Modify: `frontend/src/routes.ts`
- Modify: `frontend/src/composables/useNavigationTabs.ts`
- Modify: `frontend/src/pages/OrganizationAndApplicationSelector.vue`
- Modify: `frontend/src/composables/useCommandPalette.ts`
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue`
- Modify: `frontend/src/queries/keys.ts`
- Modify: `frontend/src/locales/en.json`

- [ ] **Step 1: Add route and nav**

In `routes.ts`, add `SubscriptionsDetail: 'SubscriptionsDetail'` to the routes const.

Add route definition:
```typescript
{
  name: routes.SubscriptionsDetail,
  path: '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id',
  component: () =>
    import('@/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue'),
  meta: { title: 'Subscription' },
},
```

Add to `useNavigationTabs.ts` subscriptions active check.
Add to `OrganizationAndApplicationSelector.vue`: `[routes.SubscriptionsDetail]: routes.SubscriptionsList`.
Add to `useCommandPalette.ts`: `[routes.SubscriptionsDetail]: routes.SubscriptionsList`.

- [ ] **Step 2: Create SubscriptionHealthService**

```typescript
import http, { type UUID } from '@/http';
import { unwrapResponse } from '@/utils/unwrapResponse';

export type HealthEvent = {
  health_event_id: string;
  subscription_id: string;
  status: 'warning' | 'disabled' | 'resolved';
  source: 'system' | 'user';
  user_id: string | null;
  created_at: string;
};

// Backend returns paginated response — first page (50 items) is sufficient for the timeline.
export function listHealthEvents(
  subscriptionId: UUID,
  organizationId: UUID
): Promise<HealthEvent[]> {
  return unwrapResponse(
    http.get<HealthEvent[]>(`/subscriptions/${subscriptionId}/health_events`, {
      params: { organization_id: organizationId },
    })
  );
}
```

- [ ] **Step 3: Add healthEventKeys + useSubscriptionHealthQueries**

In `keys.ts`:
```typescript
export const healthEventKeys = {
  all: ['healthEvents'] as const,
  list: (subscriptionId: string) => [...healthEventKeys.all, subscriptionId] as const,
};
```

```typescript
// useSubscriptionHealthQueries.ts
import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as SubscriptionHealthService from './SubscriptionHealthService';
import { healthEventKeys } from '@/queries/keys';

export function useSubscriptionHealthEvents(
  subscriptionId: Ref<string>,
  organizationId: Ref<string>
) {
  return useQuery({
    queryKey: computed(() => healthEventKeys.list(subscriptionId.value)),
    queryFn: () =>
      SubscriptionHealthService.listHealthEvents(subscriptionId.value, organizationId.value),
    enabled: computed(() => !!subscriptionId.value && !!organizationId.value),
  });
}
```

- [ ] **Step 4: Create SubscriptionHealthTimeline.vue**

Simple vertical timeline — NO ECharts. Props: `events: HealthEvent[]`. ~80 lines.

Each event rendered as a row: colored dot (green=resolved, yellow=warning, red=disabled), status badge, source badge (system/user), relative date.

Use scoped CSS with BEM: `.health-timeline`, `.health-timeline__item`, `.health-timeline__dot`, `.health-timeline__dot--warning` etc.

Empty state: text "No health events recorded" (handled by parent).

- [ ] **Step 5: Create SubscriptionsDetail.vue**

Three sections in `Hook0PageLayout`:

**Section 1 — Header card:**
- Subscription name (h3), target URL (method + url), enabled toggle (`Hook0Switch`), health badge (`Hook0HealthBadge`)
- "Edit" button linking to `routes.SubscriptionsEdit`
- Uses `useSubscriptionDetail(subscriptionId)` from existing queries

**Section 2 — Deliveries card:**
- Reuse `useLogColumns` from `useLogColumns.ts` — REMOVE the "Subscription" column (redundant)
- Uses `useLogListBySubscription(applicationId, subscriptionId)` from Task 4
- `Hook0Table` with cursor-based pagination
- Includes `EventSidePanel` on row click (same pattern as LogList)

**Section 3 — Health timeline card:**
- `SubscriptionHealthTimeline` with data from `useSubscriptionHealthEvents`

**Merged empty state:** When BOTH deliveries and health events are empty, show a single `Hook0EmptyState` with title "No activity yet" and description "Deliveries and health events will appear here once events are sent to this subscription."

Follow page pattern: error-first, skeleton, then content. Use `Hook0Card`/`Hook0CardHeader`/`Hook0CardContent`.

- [ ] **Step 6: Update SubscriptionsList — name links to detail**

Change `routes.SubscriptionsEdit` → `routes.SubscriptionsDetail` in the name column cell.

- [ ] **Step 7: Add i18n keys**

```json
"subscriptionDetail": {
  "title": "Subscription",
  "edit": "Edit",
  "deliveries": "Recent Deliveries",
  "deliveriesEmpty": "No deliveries in the last 7 days",
  "healthTimeline": "Health Timeline",
  "healthEmpty": "No health events recorded",
  "noActivityTitle": "No activity yet",
  "noActivityDescription": "Deliveries and health events will appear here once events are sent to this subscription.",
  "healthEventDate": "Date",
  "healthEventStatus": "Status",
  "healthEventSource": "Source"
}
```

- [ ] **Step 8: Type check + lint**

- [ ] **Step 9: Commit**

```bash
git add frontend/src/pages/organizations/applications/subscriptions/ frontend/src/routes.ts frontend/src/composables/ frontend/src/pages/OrganizationAndApplicationSelector.vue frontend/src/queries/keys.ts frontend/src/locales/en.json
git commit -m "feat(frontend): add subscription detail page with deliveries table and health timeline"
```

---

## Task 6: Backend — jitter

**Files:**
- Modify: `output-worker/Cargo.toml` — add `rand = "0.8"`
- Modify: `output-worker/src/main.rs` — add config field
- Modify: `output-worker/src/retry.rs` — add jitter

- [ ] **Step 1: Add rand dependency**

`output-worker/Cargo.toml`: add `rand = "0.8"` to dependencies.

- [ ] **Step 2: Add config field**

In `Config` struct in `main.rs`:
```rust
/// Jitter factor applied to retry delays (0.0 = no jitter, 0.2 = up to +20%).
/// Prevents thundering-herd when many subscriptions recover simultaneously.
#[clap(long, env, default_value = "0.2")]
retry_jitter_factor: f64,
```

- [ ] **Step 3: Add jitter to retry.rs**

Add `jitter_factor: f64` to both `compute_next_retry` (public) AND `compute_scheduled_retry_delay` (private). Thread from caller in `main.rs`/`pg.rs`/`pulsar.rs`.

```rust
use rand::Rng;

fn apply_jitter(delay: Duration, jitter_factor: f64) -> Duration {
    if jitter_factor <= 0.0 {
        return delay;
    }
    let mut rng = rand::thread_rng();
    let factor = 1.0 + rng.gen::<f64>() * jitter_factor;
    delay.mul_f64(factor)
}
```

Apply in each strategy branch, THEN cap at MAX:
- increasing: `Some(apply_jitter(delay, jitter_factor).min(MAX_RETRY_DELAY_SECS))`
- linear: `Some(apply_jitter(Duration::from_secs(delay_secs as u64), jitter_factor).min(MAX_RETRY_DELAY_SECS))`
- custom: `apply_jitter(...).min(MAX_RETRY_DELAY_SECS)`
- default fallback: do NOT apply jitter (preserve legacy behavior)

- [ ] **Step 4: Update all 12+ unit tests**

Add `0.0` as jitter_factor to every test call. Add:

```rust
#[test]
fn test_jitter_stays_within_bounds() {
    let info = SubscriptionRetrySchedule {
        strategy: "increasing".to_string(),
        max_retries: Some(5),
        increasing_base_delay: Some(3),
        increasing_wait_factor: Some(1.0),
        linear_delay: None,
        custom_intervals: None,
    };
    for _ in 0..100 {
        let delay = compute_scheduled_retry_delay(&info, 0, 5, 0.2).unwrap();
        assert!(delay >= Duration::from_secs(3));
        assert!(delay <= Duration::from_millis(3600));
    }
}
```

- [ ] **Step 5: Clippy**

Run: `cargo clippy --all --all-features --all-targets -- -D warnings`

- [ ] **Step 6: Commit**

```bash
git add output-worker/
git commit -m "feat(output-worker): add configurable retry jitter (RETRY_JITTER_FACTOR env var, default 0.2)"
```

---

## Task 7: Backend — SQL fixes

**Files:**
- Modify: `api/src/health_monitor/queries.rs`
- Modify: `api/migrations/20260325120000_add_retry_schedule.up.sql`

- [ ] **Step 1: Replace NOT EXISTS with DISTINCT ON in find_suspects**

Replace the second UNION branch in `queries.rs`:

```sql
UNION
SELECT subscription__id
FROM (
    SELECT DISTINCT ON (subscription__id) subscription__id, status
    FROM webhook.subscription_health_event
    ORDER BY subscription__id, created_at DESC
) latest
WHERE latest.status = 'warning'
```

- [ ] **Step 2: Fix name length CHECK in initial migration**

In `api/migrations/20260325120000_add_retry_schedule.up.sql`, change:
```sql
name text not null check (length(name) > 1),
```
to:
```sql
name text not null check (length(name) > 1 and length(name) <= 200),
```

- [ ] **Step 3: Clippy**

Run: `cargo clippy --all --all-features --all-targets -- -D warnings`

- [ ] **Step 4: Commit**

```bash
git add api/src/health_monitor/queries.rs api/migrations/20260325120000_add_retry_schedule.up.sql
git commit -m "fix(api): replace NOT EXISTS with DISTINCT ON, add SQL length cap on retry_schedule.name"
```

---

## Execution Order

```
Parallel group A (frontend):
  Task 1 → Task 2 (parseDuration → slider)
  Task 3 (badge icons + list improvements)
  Task 4 (log filter removal) → Task 5 (subscription detail page)

Parallel group B (backend):
  Task 6 (jitter)
  Task 7 (SQL fixes)
```

Optimal with 3 agents:
- Agent 1: Task 1 → Task 2
- Agent 2: Task 3 → Task 4 → Task 5
- Agent 3: Task 6 → Task 7
