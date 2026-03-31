# Frontend Retry Schedule & Subscription Health — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add frontend CRUD for retry schedules, retry schedule assignment on subscriptions, subscription detail page, and delivery health indicator in subscription list.

**Architecture:** Three feature blocks: (A) Retry schedule CRUD pages at org level, (B) retry schedule selector in subscription edit form + new subscription detail page, (C) delivery health (`failure_percent`) persisted by health monitor and displayed in subscription list. Requires merging `feat/auto-deactivation` into `feat/retry-schedule` first, then a small backend migration + Rust changes, then OpenAPI type regeneration, then all frontend work.

**Tech Stack:** Vue 3 Composition API, TanStack Query/Table, VeeValidate + Zod 4, vue-i18n, Lucide icons, scoped CSS with BEM naming, Axios service layer, OpenAPI-generated types.

---

## Prerequisites

Before starting any task, execute:

```bash
git checkout feat/retry-schedule
git merge feat/auto-deactivation --ff-only
```

This brings in the health monitor, `subscription_health_event` table, and health evaluation logic.

---

## File Map

### Backend (Rust) — New/Modified

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `api/migrations/20260330120000_add_subscription_failure_percent.up.sql` | Add `failure_percent` column to `webhook.subscription` |
| Create | `api/migrations/20260330120000_add_subscription_failure_percent.down.sql` | Rollback |
| Modify | `api/src/health_monitor.rs` | Persist `failure_percent` to subscription table during evaluation |
| Modify | `api/src/handlers/subscriptions.rs` | Add `failure_percent` to `Subscription` response struct + all SQL queries |

### Frontend — New Files

| File | Responsibility |
|------|----------------|
| `frontend/src/pages/organizations/retry_schedules/RetryScheduleService.ts` | API service layer for retry schedule CRUD |
| `frontend/src/pages/organizations/retry_schedules/useRetryScheduleQueries.ts` | TanStack Query composables |
| `frontend/src/pages/organizations/retry_schedules/RetrySchedulesList.vue` | List page with table |
| `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue` | Create/edit form with conditional fields + preview |
| `frontend/src/pages/organizations/retry_schedules/retrySchedule.schema.ts` | Zod validation schema |
| `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue` | Read-only subscription detail page |
| `frontend/src/components/Hook0HealthBadge.vue` | Reusable health badge (pastille + percentage) |
| `frontend/src/utils/formatDuration.ts` | Shared duration formatting (seconds → human-readable) |

### Frontend — Modified Files

| File | Change |
|------|--------|
| `frontend/src/routes.ts` | Add retry schedule routes + subscription detail/edit route split |
| `frontend/src/queries/keys.ts` | Add `retryScheduleKeys` factory |
| `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue` | Add health column + "Edit" action link |
| `frontend/src/pages/organizations/applications/subscriptions/SubscriptionSectionAdvanced.vue` | Add retry schedule selector |
| `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue` | Pass retry schedule to Advanced section |
| `frontend/src/pages/organizations/applications/subscriptions/SubscriptionService.ts` | Re-export updated types (with `failure_percent`) |
| ~~`frontend/src/pages/organizations/applications/subscriptions/useSubscriptionQueries.ts`~~ | Not modified — retry schedule list is imported directly from `useRetryScheduleQueries.ts` into `SubscriptionsEdit.vue` |
| `frontend/src/composables/useRouteIds.ts` | Add `retryScheduleId` |
| `frontend/src/composables/useNavigationTabs.ts` | Add "Retry Schedules" org tab |
| `frontend/src/pages/OrganizationAndApplicationSelector.vue` | Add `SubscriptionsEdit` to fallback route map |
| `frontend/src/locales/en.json` | Add all i18n keys |

---

## Task 1: Merge `feat/auto-deactivation` + backend migration

**Files:**
- Create: `api/migrations/20260330120000_add_subscription_failure_percent.up.sql`
- Create: `api/migrations/20260330120000_add_subscription_failure_percent.down.sql`

- [ ] **Step 1: Merge auto-deactivation into retry-schedule**

```bash
git checkout feat/retry-schedule
git merge feat/auto-deactivation --ff-only
```

Expected: Fast-forward merge, no conflicts.

- [ ] **Step 2: Create up migration**

```sql
-- api/migrations/20260330120000_add_subscription_failure_percent.up.sql
ALTER TABLE webhook.subscription
    ADD COLUMN failure_percent double precision;
```

- [ ] **Step 3: Create down migration**

```sql
-- api/migrations/20260330120000_add_subscription_failure_percent.down.sql
ALTER TABLE webhook.subscription
    DROP COLUMN IF EXISTS failure_percent;
```

- [ ] **Step 4: Commit**

```bash
git add api/migrations/20260330120000_add_subscription_failure_percent.*
git commit -m "feat(db): add failure_percent column to webhook.subscription"
```

---

## Task 2: Backend — Persist `failure_percent` in health monitor

**Files:**
- Modify: `api/src/health_monitor.rs` — inside `process_subscription()`, before the `match` statement

- [ ] **Step 1: Add UPDATE query at top of `process_subscription`**

In `api/src/health_monitor.rs` (brought in by the merge from `feat/auto-deactivation`), inside `process_subscription()`, after `let mut actions = Vec::new();`, add:

```rust
    // Persist current failure_percent to subscription table for frontend display
    sqlx::query(
        "UPDATE webhook.subscription SET failure_percent = $1 WHERE subscription__id = $2",
    )
    .bind(ratio)
    .bind(sub.subscription__id)
    .execute(&mut **tx)
    .await?;
```

This updates the `failure_percent` column for every evaluated subscription on every health monitor tick (~30min), regardless of state machine branch.

- [ ] **Step 2: Verify it compiles**

```bash
cd api && cargo check 2>&1 | tail -5
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add api/src/health_monitor.rs
git commit -m "feat(api): persist failure_percent to subscription table in health monitor"
```

---

## Task 3: Backend — Add `failure_percent` to Subscription response

**Files:**
- Modify: `api/src/handlers/subscriptions.rs`

This task adds `failure_percent: Option<f64>` to the `Subscription` response struct and all SQL queries that build it (list, get, create, update).

- [ ] **Step 1: Add field to `Subscription` response struct**

In the `Subscription` struct (search for `pub struct Subscription`), add after `is_enabled`:

```rust
    pub failure_percent: Option<f64>,
```

- [ ] **Step 2: Add to list handler `RawSubscription` + SQL**

In the list handler's `RawSubscription` struct (search for `struct RawSubscription` inside the `list` function), add:

```rust
    failure_percent: Option<f64>,
```

In the list SQL inner CTE `subs` SELECT (search for `SELECT s.subscription__id, s.is_enabled`), add `s.failure_percent` to the selected columns.

In the list SQL outer SELECT (search for `SELECT subs.subscription__id, subs.is_enabled`), add `subs.failure_percent`.

In the list `Subscription` construction (search for `Subscription {` inside the `.map(|s|` closure), add:

```rust
    failure_percent: s.failure_percent,
```

- [ ] **Step 3: Add to get handler `RawSubscription` + SQL**

Same pattern as list: add `failure_percent: Option<f64>` to the get handler's `RawSubscription`, add `s.failure_percent` to inner CTE SELECT, add `subs.failure_percent` to outer SELECT, add `failure_percent: s.failure_percent` to the `Subscription` construction.

- [ ] **Step 4: Add to create handler response**

In the create handler's `Subscription` construction (search for `Subscription {` inside the `create` function), add:

```rust
    failure_percent: None,
```

Newly created subscriptions always have `NULL` failure_percent (never evaluated yet).

- [ ] **Step 5: Add to update handler response**

In the update handler's `Subscription` construction (search for `Subscription {` inside the `edit` function), same: add `failure_percent: None,` — the update handler doesn't SELECT failure_percent from DB (it returns the written data), so use `None`. The real value comes from the list/get queries.

- [ ] **Step 6: Verify compilation**

```bash
cd api && cargo check 2>&1 | tail -5
```

Expected: no errors. If there are other Subscription constructions (toggle enable, etc.), add `failure_percent: None` there too.

- [ ] **Step 7: Run existing tests**

```bash
cd api && cargo test 2>&1 | tail -20
```

Expected: all existing tests pass.

- [ ] **Step 8: Commit**

```bash
git add api/src/handlers/subscriptions.rs
git commit -m "feat(api): add failure_percent to Subscription response"
```

---

## Task 4: Regenerate OpenAPI types

**Files:**
- Modify: `frontend/src/types.ts` (auto-generated)

- [ ] **Step 1: Run the API locally**

```bash
cd api && cargo run -- [your usual args] &
```

Wait until the API is listening (check logs for "listening on").

- [ ] **Step 2: Regenerate types**

```bash
cd frontend && npm run generate:types
```

- [ ] **Step 3: Verify new types include `failure_percent` and retry schedule types**

Check `frontend/src/types.ts` for:
- `Subscription` schema has `failure_percent?: number`
- `RetrySchedule` schema exists with `retry_schedule_id`, `organization_id`, `name`, `strategy`, `max_retries`, `custom_intervals`, `linear_delay`, `created_at`, `updated_at`
- `RetrySchedulePost` and `RetrySchedulePut` schemas exist

- [ ] **Step 3b: Contingency — if type names differ**

If any expected type (`RetrySchedule`, `RetrySchedulePost`, `RetrySchedulePut`, `failure_percent`) is missing, grep `frontend/src/types.ts` for `retry_schedule` and `failure` to find actual generated names. Update type references in Tasks 9 and 17 accordingly. If `failure_percent` is entirely absent, verify the backend `Subscription` struct from Task 3 has the `Apiv2Schema` derive and the field is public.

- [ ] **Step 4: Stop the API, commit**

```bash
git add frontend/src/types.ts
git commit -m "chore(frontend): regenerate OpenAPI types with failure_percent and retry schedules"
```

---

## Task 5: i18n keys

**Files:**
- Modify: `frontend/src/locales/en.json`

- [ ] **Step 1: Add all i18n keys**

Add the following sections to `en.json`:

```json
"retrySchedules": {
  "title": "Retry Schedules",
  "create": "New Retry Schedule",
  "created": "Retry schedule created",
  "edit": "Edit Retry Schedule",
  "updated": "Retry schedule updated",
  "delete": "Delete Retry Schedule",
  "deleteConfirm": "Are you sure you want to delete this retry schedule? Subscriptions using it will revert to the default retry policy.",
  "deleted": "Retry schedule deleted",
  "nameColumn": "Name",
  "strategyColumn": "Strategy",
  "maxRetriesColumn": "Max Retries",
  "delayColumn": "Delay",
  "createdAtColumn": "Created",
  "strategyExponential": "Exponential",
  "strategyLinear": "Linear",
  "strategyCustom": "Custom",
  "delayExponential": "3s → 10h escalating",
  "delayLinear": "{delay} fixed",
  "delayCustom": "{count} custom intervals",
  "defaultSchedule": "Default",
  "fields": {
    "name": "Name",
    "namePlaceholder": "e.g. Aggressive then patient",
    "strategy": "Strategy",
    "strategyExponentialDesc": "Uses the built-in escalating schedule (3s, 10s, 3min, 30min, 1h, 3h, 5h, 10h…)",
    "strategyLinearDesc": "Fixed delay between each retry",
    "strategyCustomDesc": "Define a custom delay for each retry attempt",
    "maxRetries": "Max Retries",
    "linearDelay": "Delay (seconds)",
    "customIntervals": "Custom Intervals",
    "addInterval": "Add interval",
    "retryNumber": "Retry #{number}",
    "intervalSeconds": "Delay (seconds)"
  },
  "preview": {
    "title": "Retry Preview",
    "retryColumn": "Retry",
    "delayColumn": "Delay"
  },
  "empty": {
    "title": "No retry schedules",
    "description": "Create a retry schedule to customize how failed webhooks are retried.",
    "cta": "Create retry schedule"
  }
},
"subscriptionDetail": {
  "title": "Subscription Details",
  "editAction": "Edit",
  "sectionHealth": "Delivery Health",
  "sectionConfig": "Configuration",
  "sectionRetrySchedule": "Retry Schedule",
  "sectionRecentDeliveries": "Recent Deliveries",
  "targetUrl": "Target URL",
  "httpMethod": "HTTP Method",
  "eventTypes": "Event Types",
  "labels": "Labels",
  "headers": "Headers",
  "metadata": "Metadata",
  "retrySchedule": "Retry Schedule",
  "noDeliveries": "No recent deliveries",
  "statusColumn": "Status",
  "dateColumn": "Date",
  "responseCodeColumn": "Response Code",
  "retryCountColumn": "Retry"
},
"health": {
  "healthy": "Healthy",
  "warning": "Warning",
  "disabled": "Disabled",
  "noData": "N/A",
  "failureRate": "Failure rate",
  "healthColumn": "Health"
}
```

Also add to the `validation` section (create if it doesn't exist):

```json
"minLength": "{field} must be at least {min} characters",
"arrayLength": "{field} must have exactly {length} items"
```

Also add to the `nav` section:

```json
"retrySchedules": "Retry Schedules"
```

Also add to the `subscriptions` section:

```json
"retryScheduleLabel": "Retry Schedule",
"retryScheduleHint": "Choose a retry schedule or leave empty to use the default policy.",
"editAction": "Edit",
"detailAction": "View"
```

- [ ] **Step 2: Verify JSON is valid**

```bash
cd frontend && node -e "JSON.parse(require('fs').readFileSync('src/locales/en.json', 'utf8')); console.log('OK')"
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/locales/en.json
git commit -m "feat(i18n): add retry schedule, subscription detail, and health i18n keys"
```

---

## Task 6: Query keys + route IDs

**Files:**
- Modify: `frontend/src/queries/keys.ts`
- Modify: `frontend/src/composables/useRouteIds.ts`

- [ ] **Step 1: Add `retryScheduleKeys` to `keys.ts`**

Add after `serviceTokenKeys`:

```typescript
export const retryScheduleKeys = {
  all: ['retrySchedules'] as const,
  lists: () => [...retryScheduleKeys.all, 'list'] as const,
  list: (organizationId: string) =>
    [...retryScheduleKeys.lists(), organizationId] as const,
  details: () => [...retryScheduleKeys.all, 'detail'] as const,
  detail: (id: string, organizationId: string) =>
    [...retryScheduleKeys.details(), id, organizationId] as const,
};
```

- [ ] **Step 2: Add `retryScheduleId` to `useRouteIds.ts`**

Add to the return object:

```typescript
retryScheduleId: get('retry_schedule_id'),
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/queries/keys.ts frontend/src/composables/useRouteIds.ts
git commit -m "feat(frontend): add retry schedule query keys and route ID"
```

---

## Task 7: Routes

**Files:**
- Modify: `frontend/src/routes.ts`

- [ ] **Step 1: Add retry schedule route names to `routes` const**

Add to the `routes` object:

```typescript
RetrySchedulesList: 'RetrySchedulesList',
RetrySchedulesNew: 'RetrySchedulesNew',
RetrySchedulesEdit: 'RetrySchedulesEdit',
```

- [ ] **Step 2: Add subscription detail + edit route names**

The current `SubscriptionsDetail` route loads `SubscriptionsEdit.vue`. We need to split:
- `SubscriptionsDetail` → loads new `SubscriptionsDetail.vue` (read-only)
- `SubscriptionsEdit` → loads `SubscriptionsEdit.vue` (form)

Add to the `routes` object:

```typescript
SubscriptionsEdit: 'SubscriptionsEdit',
```

(`SubscriptionsDetail` already exists in the routes object.)

- [ ] **Step 3: Add route definitions**

Add retry schedule routes (in the route definitions array, near org-level routes like `ServicesTokenList`):

```typescript
{
  name: routes.RetrySchedulesList,
  path: '/organizations/:organization_id/retry_schedules',
  component: () =>
    import('./pages/organizations/retry_schedules/RetrySchedulesList.vue'),
  meta: { title: 'Retry Schedules' },
},
{
  name: routes.RetrySchedulesNew,
  path: '/organizations/:organization_id/retry_schedules/new',
  component: () =>
    import('./pages/organizations/retry_schedules/RetrySchedulesEdit.vue'),
  meta: { title: 'New Retry Schedule' },
},
{
  name: routes.RetrySchedulesEdit,
  path: '/organizations/:organization_id/retry_schedules/:retry_schedule_id',
  component: () =>
    import('./pages/organizations/retry_schedules/RetrySchedulesEdit.vue'),
  meta: { title: 'Edit Retry Schedule' },
},
```

Modify the existing subscription routes:

- Change `SubscriptionsDetail` to load `SubscriptionsDetail.vue` instead of `SubscriptionsEdit.vue`
- Add new `SubscriptionsEdit` route:

```typescript
{
  name: routes.SubscriptionsEdit,
  path: '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id/edit',
  component: () =>
    import('./pages/organizations/applications/subscriptions/SubscriptionsEdit.vue'),
  meta: { title: 'Edit Subscription' },
},
```

- [ ] **Step 4: Update `OrganizationAndApplicationSelector.vue` fallback route map**

Search for `SubscriptionsDetail` in the fallback map. Add an entry for the new edit route:

```typescript
[routes.SubscriptionsEdit]: routes.SubscriptionsList,
```

- [ ] **Step 5: Commit**

```bash
git add frontend/src/routes.ts frontend/src/pages/OrganizationAndApplicationSelector.vue
git commit -m "feat(frontend): add retry schedule and subscription detail/edit routes"
```

---

## Task 8: Navigation tab

**Files:**
- Modify: `frontend/src/composables/useNavigationTabs.ts` (or wherever org tabs are defined — confirmed in `useNavigationTabs.ts`)

- [ ] **Step 1: Add "Retry Schedules" tab to org-level tabs**

Import `Repeat` icon from `lucide-vue-next`.

Add a new tab entry in the org-level tabs array, between `service-tokens` and `team`:

```typescript
{
  id: 'retry-schedules',
  label: t('nav.retrySchedules'),
  icon: Repeat,
  to: {
    name: routes.RetrySchedulesList,
    params: { organization_id: contextStore.organizationId },
  },
  active: route.name === routes.RetrySchedulesList
    || route.name === routes.RetrySchedulesNew
    || route.name === routes.RetrySchedulesEdit,
},
```

- [ ] **Step 2: Update subscriptions tab active check**

Find the subscriptions tab active check (search for `SubscriptionsDetail`). Add `routes.SubscriptionsEdit` to the `active` condition:

```typescript
active:
  route.name === routes.SubscriptionsList ||
  route.name === routes.SubscriptionsNew ||
  route.name === routes.SubscriptionsDetail ||
  route.name === routes.SubscriptionsEdit,
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/composables/useNavigationTabs.ts
git commit -m "feat(frontend): add Retry Schedules tab and update subscriptions tab active check"
```

---

## Task 9: Retry Schedule Service + Queries

**Files:**
- Create: `frontend/src/pages/organizations/retry_schedules/RetryScheduleService.ts`
- Create: `frontend/src/pages/organizations/retry_schedules/useRetryScheduleQueries.ts`

- [ ] **Step 1: Create RetryScheduleService.ts**

```typescript
import http, { UUID } from '@/http.ts';
import type { components } from '@/types.ts';
import { unwrapResponse } from '@/utils/unwrapResponse';

type definitions = components['schemas'];
export type RetrySchedule = definitions['RetrySchedule'];
export type RetrySchedulePost = definitions['RetrySchedulePost'];
export type RetrySchedulePut = definitions['RetrySchedulePut'];

export function list(organization_id: UUID): Promise<Array<RetrySchedule>> {
  return unwrapResponse(
    http.get<Array<RetrySchedule>>('/retry_schedules', { params: { organization_id } }),
  );
}

export function get(
  retry_schedule_id: UUID,
  organization_id: UUID,
): Promise<RetrySchedule> {
  return unwrapResponse(
    http.get<RetrySchedule>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    }),
  );
}

export function create(
  schedule: RetrySchedulePost,
): Promise<RetrySchedule> {
  return unwrapResponse(http.post<RetrySchedule>('/retry_schedules', schedule));
}

export function update(
  retry_schedule_id: UUID,
  organization_id: UUID,
  schedule: RetrySchedulePut,
): Promise<RetrySchedule> {
  return unwrapResponse(
    http.put<RetrySchedule>(`/retry_schedules/${retry_schedule_id}`, schedule, {
      params: { organization_id },
    }),
  );
}

export function remove(
  retry_schedule_id: UUID,
  organization_id: UUID,
): Promise<void> {
  return unwrapResponse(
    http.delete<void>(`/retry_schedules/${retry_schedule_id}`, {
      params: { organization_id },
    }),
  );
}
```

- [ ] **Step 2: Create useRetryScheduleQueries.ts**

```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as RetryScheduleService from './RetryScheduleService';
import type {
  RetrySchedulePost,
  RetrySchedulePut,
} from './RetryScheduleService';
import { retryScheduleKeys, subscriptionKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';

export function useRetryScheduleList(organizationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.list(organizationId.value)),
    queryFn: () => RetryScheduleService.list(organizationId.value),
    enabled: computed(() => !!organizationId.value),
  });
}

export function useRetryScheduleDetail(
  id: Ref<string>,
  organizationId: Ref<string>,
) {
  return useQuery({
    queryKey: computed(() => retryScheduleKeys.detail(id.value, organizationId.value)),
    queryFn: () => RetryScheduleService.get(id.value, organizationId.value),
    enabled: computed(() => !!id.value && !!organizationId.value),
  });
}

export function useCreateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (schedule: RetrySchedulePost) =>
      RetryScheduleService.create(schedule),
    invalidateKeys: retryScheduleKeys.all,
  });
}

export function useUpdateRetrySchedule() {
  return useInvalidatingMutation({
    mutationFn: (params: {
      retryScheduleId: string;
      organizationId: string;
      schedule: RetrySchedulePut;
    }) =>
      RetryScheduleService.update(
        params.retryScheduleId,
        params.organizationId,
        params.schedule,
      ),
    invalidateKeys: retryScheduleKeys.all,
  });
}

// Uses raw useMutation (not useInvalidatingMutation) to invalidate both
// retryScheduleKeys AND subscriptionKeys — deleting a schedule sets
// retry_schedule_id = NULL on affected subscriptions (ON DELETE SET NULL).
export function useRemoveRetrySchedule() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      retryScheduleId: string;
      organizationId: string;
    }) =>
      RetryScheduleService.remove(
        params.retryScheduleId,
        params.organizationId,
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: retryScheduleKeys.all });
      void queryClient.invalidateQueries({ queryKey: subscriptionKeys.all });
    },
  });
}
```

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/organizations/retry_schedules/
git commit -m "feat(frontend): add retry schedule service and TanStack Query composables"
```

---

## Task 10: Retry Schedule Zod Schema

**Files:**
- Create: `frontend/src/pages/organizations/retry_schedules/retrySchedule.schema.ts`

- [ ] **Step 1: Create schema**

```typescript
import { z } from 'zod';
import i18n from '@/plugins/i18n';

const MAX_INTERVAL_SECONDS = 604800; // 1 week

export function createRetryScheduleSchema() {
  const t = i18n.global.t;

  return z
    .object({
      name: z
        .string()
        .min(2, t('validation.minLength', { field: t('retrySchedules.fields.name'), min: 2 }))
        .max(200),
      strategy: z.enum(['exponential', 'linear', 'custom']),
      max_retries: z.coerce
        .number()
        .int()
        .min(1)
        .max(100),
      linear_delay: z.coerce
        .number()
        .int()
        .min(1)
        .max(MAX_INTERVAL_SECONDS)
        .optional()
        .nullable(),
      custom_intervals: z
        .array(z.coerce.number().int().min(1).max(MAX_INTERVAL_SECONDS))
        .optional()
        .nullable(),
    })
    .superRefine((data, ctx) => {
      if (data.strategy === 'linear' && !data.linear_delay) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: t('validation.required', {
            field: t('retrySchedules.fields.linearDelay'),
          }),
          path: ['linear_delay'],
        });
      }
      if (data.strategy === 'custom') {
        if (
          !data.custom_intervals ||
          data.custom_intervals.length === 0
        ) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.required', {
              field: t('retrySchedules.fields.customIntervals'),
            }),
            path: ['custom_intervals'],
          });
        } else if (data.custom_intervals.length !== data.max_retries) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: t('validation.arrayLength', {
              field: t('retrySchedules.fields.customIntervals'),
              length: data.max_retries,
            }),
            path: ['custom_intervals'],
          });
        }
      }
    });
}

export type RetryScheduleFormValues = z.infer<
  ReturnType<typeof createRetryScheduleSchema>
>;
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/pages/organizations/retry_schedules/retrySchedule.schema.ts
git commit -m "feat(frontend): add retry schedule Zod validation schema"
```

---

## Task 11: Retry Schedules List Page

**Files:**
- Create: `frontend/src/pages/organizations/retry_schedules/RetrySchedulesList.vue`

- [ ] **Step 0: Create `frontend/src/utils/formatDuration.ts`**

```typescript
// Source of truth for exponential delays: output-worker/src/main.rs compute_next_retry_duration()
export const EXPONENTIAL_DELAYS = [3, 10, 180, 1800, 3600, 10800, 18000, 36000];

export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}min`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h`;
  return `${Math.floor(seconds / 86400)}d`;
}
```

- [ ] **Step 1: Create list page**

Follow the pattern from `SubscriptionsList.vue` and `ServicesTokenList.vue`. Key elements:

- `useRouteIds()` → `organizationId`
- `usePermissions()` for canCreate/canDelete
- `useRetryScheduleList(organizationId)` for data
- `useRemoveRetrySchedule()` for deletion
- TanStack Table with columns: Name (link to edit), Strategy (badge), Max Retries, Delay (human summary), Created (relative date)
- Delete action column using `useEntityDelete<RetrySchedule>` composable with `useRemoveRetrySchedule().mutateAsync`, following the pattern from `SubscriptionsList.vue`. Delete dialog variant="danger" with generic warning message from i18n.
- Empty state with CTA to create
- Page wrapped in `Hook0PageLayout` with breadcrumbs: `Organizations / [org] / Retry Schedules`
- Add `useTracking()` — call `trackEvent('retry-schedule', 'delete', 'success')` in delete success handler
- Add `data-test="retry-schedules-card"` on the main card element

**Conditional rendering order** (matches actual codebase, NOT CLAUDE.md example):

```vue
<Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />
<Hook0Card v-else-if="isLoading || !schedules" data-test="retry-schedules-card">
  <Hook0SkeletonGroup :count="5" />
</Hook0Card>
<Hook0Card v-else-if="schedules" data-test="retry-schedules-card">
  <!-- table or empty state -->
</Hook0Card>
```

**Column definitions:**

| Column | Accessor | Cell render |
|--------|----------|-------------|
| Name | `name` | `Hook0TableCellLink` → `routes.RetrySchedulesEdit` |
| Strategy | `strategy` | `Hook0Badge` — variant `info`, label from i18n (`strategyExponential`/`strategyLinear`/`strategyCustom`) |
| Max Retries | `max_retries` | Plain text |
| Delay | computed | `delayExponential` / `delayLinear` with `{delay}` = formatted seconds / `delayCustom` with `{count}` |
| Created | `created_at` | `Hook0DateTime` |
| Actions | — | Delete button (Trash2 icon) |

**Delay formatting helper** — extract to `frontend/src/utils/formatDuration.ts` (shared between list page, edit page preview, and detail page):

```typescript
// Import shared utility — do NOT redefine formatDuration locally
import { formatDuration } from '@/utils/formatDuration';
import i18n from '@/plugins/i18n';
const t = i18n.global.t;

function formatDelaySummary(schedule: RetrySchedule): string {
  switch (schedule.strategy) {
    case 'exponential':
      return t('retrySchedules.delayExponential');
    case 'linear':
      return t('retrySchedules.delayLinear', {
        delay: formatDuration(schedule.linear_delay!),
      });
    case 'custom':
      return t('retrySchedules.delayCustom', {
        count: schedule.custom_intervals!.length,
      });
    default:
      return '';
  }
}

```

- [ ] **Step 2: Verify it renders**

```bash
cd frontend && source .envrc && npm run dev
```

Navigate to `/organizations/<org_id>/retry_schedules`. Should show empty state or list.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/utils/formatDuration.ts frontend/src/pages/organizations/retry_schedules/RetrySchedulesList.vue
git commit -m "feat(frontend): add retry schedules list page"
```

---

## Task 12: Retry Schedule Edit/Create Page

**Files:**
- Create: `frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue`

- [ ] **Step 1: Create edit page**

Key behaviors:

- **Create vs Edit**: distinguished by presence of `retryScheduleId` from route params
- **Strategy selector**: 3 radio buttons (`exponential`, `linear`, `custom`) with description text
- **Conditional fields**:
  - `exponential`: only `name` + `max_retries`
  - `linear`: `name` + `max_retries` + `linear_delay` input
  - `custom`: `name` + `max_retries` (read-only, derived from array length) + custom intervals editor
- **Custom intervals editor**: array of `Hook0Input` fields. "Add interval" button appends a row. Each row has a delete button. `max_retries` is auto-synced to `custom_intervals.length`.
- **Preview panel**: `Hook0Card` at bottom showing a table of "Retry #N → delay" for all strategies:
  - `exponential`: David's table `[3, 10, 180, 1800, 3600, 10800, 18000, 36000]` repeated, truncated to `max_retries`. Extract as `EXPONENTIAL_DELAYS` constant in `formatDuration.ts` with a comment: `// Source of truth: output-worker/src/main.rs compute_next_retry_duration()`
  - `linear`: `max_retries` rows × `linear_delay`
  - `custom`: `custom_intervals` array directly

**Form uses VeeValidate + Zod** via `useForm` + `toTypedSchema(createRetryScheduleSchema())`. Import the custom adapter, NOT `@vee-validate/zod`:
```typescript
import { toTypedSchema } from '@/utils/zod-adapter';
```

**On submit**:
- Create: call `createMutation.mutate(cleanPayload(values, organizationId.value))`, on success → `void router.push({ name: routes.RetrySchedulesList, params: { organization_id: organizationId.value } })`
- Update: call `updateMutation.mutate({ retryScheduleId: retryScheduleId.value, organizationId: organizationId.value, schedule: cleanPayload(values, organizationId.value) })`, on success → toast
- Add `useTracking()` — call `trackEvent('retry-schedule', 'create'|'update', 'success')` in success handlers
- Add `data-test="retry-schedule-form"` on the form element, `data-test="retry-schedule-name-input"` on name field

**Breadcrumbs**: `Organizations / [org] / Retry Schedules / New` or `Organizations / [org] / Retry Schedules / [schedule name]`

**Strategy-specific payload cleanup** (before submit):
```typescript
function cleanPayload(values: RetryScheduleFormValues, organizationId: string) {
  const base = { organization_id: organizationId, name: values.name, strategy: values.strategy, max_retries: values.max_retries };
  switch (values.strategy) {
    case 'exponential':
      return { ...base, linear_delay: null, custom_intervals: null };
    case 'linear':
      return { ...base, linear_delay: values.linear_delay, custom_intervals: null };
    case 'custom':
      return { ...base, linear_delay: null, custom_intervals: values.custom_intervals, max_retries: values.custom_intervals!.length };
  }
}
```

- [ ] **Step 2: Verify create + edit flows work**

Navigate to `/organizations/<org_id>/retry_schedules/new`, create a schedule with each strategy. Navigate to edit page, verify form is pre-filled.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/organizations/retry_schedules/RetrySchedulesEdit.vue
git commit -m "feat(frontend): add retry schedule create/edit page with preview"
```

---

## Task 13: Health Badge Component

**Files:**
- Create: `frontend/src/components/Hook0HealthBadge.vue`

- [ ] **Step 1: Create component**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Hook0Badge from './Hook0Badge.vue';

const props = defineProps<{
  failurePercent: number | null | undefined;
}>();

const { t } = useI18n();

// Thresholds aligned with health monitor defaults:
// --health-monitor-warning-failure-percent=80 and --health-monitor-disable-failure-percent=95
const status = computed(() => {
  if (props.failurePercent == null) return 'noData';
  if (props.failurePercent >= 95) return 'disabled';
  if (props.failurePercent >= 80) return 'warning';
  return 'healthy';
});

const variant = computed(() => {
  switch (status.value) {
    case 'healthy':
      return 'success';
    case 'warning':
      return 'warning';
    case 'disabled':
      return 'danger';
    default:
      return 'default';
  }
});

const label = computed(() => {
  if (props.failurePercent == null) return t('health.noData');
  return `${Math.round(props.failurePercent)}%`;
});
</script>

<template>
  <Hook0Badge :variant="variant" size="sm">
    {{ label }}
  </Hook0Badge>
</template>

<style scoped></style>
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/components/Hook0HealthBadge.vue
git commit -m "feat(frontend): add Hook0HealthBadge component"
```

---

## Task 14: Subscription List — Add health column + Edit action

**Files:**
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue`

- [ ] **Step 1: Import Hook0HealthBadge**

Add to imports:

```typescript
import Hook0HealthBadge from '@/components/Hook0HealthBadge.vue';
```

- [ ] **Step 2: Add Health column to table columns**

Add a new column definition after the existing columns, before the actions column:

```typescript
{
  id: 'health',
  accessorKey: 'failure_percent',
  header: t('health.healthColumn'),
  cell: ({ row }) =>
    h(Hook0HealthBadge, {
      failurePercent: row.original.failure_percent,
    }),
},
```

- [ ] **Step 3: Change row click behavior**

Currently `SubscriptionsDetail` route loads the edit form. After Task 7, it loads the detail page. The table link in the Name column should point to `routes.SubscriptionsDetail` (the read-only view).

- [ ] **Step 4: Add "Edit" action link**

Add an "Edit" action in the actions column (alongside the existing delete), using `Pencil` icon from Lucide:

```typescript
import { Trash2, Link, Pencil } from 'lucide-vue-next';
```

Add a cell render in the actions column that includes a link/button to `routes.SubscriptionsEdit`.

- [ ] **Step 5: Verify list renders with health badge**

Navigate to subscription list. Each row should show a health badge (green with % or grey "N/A").

- [ ] **Step 6: Commit**

```bash
git add frontend/src/pages/organizations/applications/subscriptions/SubscriptionsList.vue
git commit -m "feat(frontend): add health column and edit action to subscription list"
```

---

## Task 15: Subscription Detail Page

**Files:**
- Create: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue`

- [ ] **Step 1: Create detail page**

Read-only page with 5 sections:

**Header**: Subscription description, enabled/disabled badge, "Edit" button → `routes.SubscriptionsEdit`

**Section 1 — Health** (`Hook0Card`):
- `Hook0HealthBadge` with `failurePercent`
- Text: "Failure rate: X%" or "No data available"

**Section 2 — Configuration** (`Hook0Card`):
- `Hook0CardContentLine` rows: Target URL, HTTP Method, Event Types, Labels, Headers, Metadata
- Use `type="split"` for key-value display

**Section 3 — Retry Schedule** (`Hook0Card`):
- If `retry_schedule_id`: fetch schedule details via `useRetryScheduleDetail(retryScheduleId, organizationId)`, show schedule name (link to `routes.RetrySchedulesEdit`), strategy badge, max_retries
- If null: show "Default" with description of David's escalating policy

**Section 4 — Recent Deliveries** (`Hook0Card` + `Hook0Table`):
- Use `useLogList(applicationId)` and filter client-side by `subscription_id`
- **Known limitation**: the log list returns all app deliveries; client-side filtering means only the last 7 days of deliveries are available, and performance may degrade with very large apps. A dedicated `subscription_id` query param on the logs endpoint would be better but is out of scope for this plan. Add a `// TODO: add subscription_id filter to request_attempts.list backend` comment.
- Table columns: Status (badge), Date, Response Code, Retry count
- Show max 10 entries

**Data fetching**:
- `useSubscriptionDetail(subscriptionId)` for subscription data
- Derive `retryScheduleId` as a computed ref from subscription data:
  ```typescript
  const retryScheduleId = computed(() => subscription.value?.retry_schedule_id ?? '');
  ```
- `useRetryScheduleDetail(retryScheduleId, organizationId)` for retry schedule name/details — the `enabled` guard in the composable (`!!id.value && !!organizationId.value`) ensures it only fires when `retry_schedule_id` is set
- `useLogList(applicationId)` for recent deliveries, filtered client-side:
  ```typescript
  const recentDeliveries = computed(() =>
    logs.value?.filter(l => l.subscription.subscription_id === subscriptionId.value).slice(0, 10) ?? []
  );
  ```

**Conditional rendering order** (error-first, matching actual codebase):
```vue
<Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />
<template v-else-if="isLoading || !subscription">
  <Hook0Card data-test="subscription-detail-card"><Hook0SkeletonGroup :count="5" /></Hook0Card>
</template>
<template v-else-if="subscription">
  <!-- sections -->
</template>
```

Add `data-test="subscription-detail-card"` on the main card.

**Breadcrumbs**: `Organizations / [org] / [app] / Subscriptions / [description]`

- [ ] **Step 2: Verify detail page renders**

Click on a subscription in the list. Should navigate to detail page with all sections.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/organizations/applications/subscriptions/SubscriptionsDetail.vue
git commit -m "feat(frontend): add subscription detail page"
```

---

## Task 16: Retry Schedule Selector in Subscription Edit

**Files:**
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionSectionAdvanced.vue`
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue`

- [ ] **Step 1: Add retry schedule props to SubscriptionSectionAdvanced**

**Replace** the existing bare `defineProps<Props>()` with the extended version below (the old `defineProps` call must be removed):

```typescript
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';

type Props = {
  headersKv: Hook0KeyValueKeyValuePair[];
  metadata: Hook0KeyValueKeyValuePair[];
  retryScheduleId?: string | null;
  retryScheduleOptions?: Hook0SelectSingleOption[];
};

const props = withDefaults(defineProps<Props>(), {
  retryScheduleId: null,
  retryScheduleOptions: () => [],
});
```

Add new emit to the existing `defineEmits`:

```typescript
'update:retryScheduleId': [value: string | null],
```

- [ ] **Step 2: Add retry schedule selector to template**

Add a new `.sub-row` before the Headers row:

```html
<div class="sub-row">
  <div class="sub-row__label">
    <span class="sub-row__title">{{ t('subscriptions.retryScheduleLabel') }}</span>
    <span class="sub-row__hint">{{ t('subscriptions.retryScheduleHint') }}</span>
  </div>
  <div class="sub-row__content">
    <Hook0Select
      :model-value="retryScheduleId"
      :options="retryScheduleOptions"
      @update:model-value="$emit('update:retryScheduleId', $event)"
    />
  </div>
</div>
```

- [ ] **Step 3: Wire up in SubscriptionsEdit.vue**

In `SubscriptionsEdit.vue`:

1. Import `useRetryScheduleList` from the retry schedule queries
2. Fetch the org's retry schedules: `const { data: retrySchedules } = useRetryScheduleList(organizationId)`
3. Compute options for the select, with "Default" as first option:

```typescript
const retryScheduleOptions = computed(() => {
  const options: Hook0SelectSingleOption[] = [
    { value: '', label: t('retrySchedules.defaultSchedule') },
  ];
  if (retrySchedules.value) {
    retrySchedules.value.forEach((s) => {
      options.push({ value: s.retry_schedule_id, label: s.name });
    });
  }
  return options;
});
```

4. Track the selected retry schedule ID with a ref, initialized from subscription data
5. Pass to `SubscriptionSectionAdvanced`:

```html
<SubscriptionSectionAdvanced
  :headers-kv="headersKv"
  :metadata="metadataKv"
  :retry-schedule-id="selectedRetryScheduleId"
  :retry-schedule-options="retryScheduleOptions"
  @update:headers="..."
  @update:metadata="..."
  @update:retry-schedule-id="selectedRetryScheduleId = $event"
/>
```

6. Include `retry_schedule_id` in the submit payload (map `''` to `null`):

```typescript
retry_schedule_id: selectedRetryScheduleId.value || null,
```

- [ ] **Step 4: Verify assignment works**

Create a retry schedule, then edit a subscription, select the retry schedule in Advanced section, save. Verify the subscription's `retry_schedule_id` is set.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/pages/organizations/applications/subscriptions/SubscriptionSectionAdvanced.vue
git add frontend/src/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue
git commit -m "feat(frontend): add retry schedule selector to subscription edit form"
```

---

## Task 17: Update SubscriptionService types

**Files:**
- Modify: `frontend/src/pages/organizations/applications/subscriptions/SubscriptionService.ts`

- [ ] **Step 1: Verify types are updated**

After `generate:types` (Task 4), the `Subscription` type should already include `failure_percent`. Check that `SubscriptionService.ts` re-exports the correct type. If the generated `Subscription` type has `failure_percent?: number`, no change needed here.

If `SubscriptionPost` doesn't include `retry_schedule_id` in the generated types, add it to the service:

```typescript
export type SubscriptionPostWithSchedule = SubscriptionPost & {
  retry_schedule_id?: string | null;
};
```

And use this type in the create/update functions.

- [ ] **Step 2: Commit if changes were needed**

```bash
git add frontend/src/pages/organizations/applications/subscriptions/SubscriptionService.ts
git commit -m "fix(frontend): align subscription service types with updated OpenAPI schema"
```

---

## Task 18: Final verification

- [ ] **Step 1: ESLint**

```bash
cd frontend && npx eslint --fix src/
```

- [ ] **Step 2: Type check**

```bash
cd frontend && npx vue-tsc --noEmit
```

- [ ] **Step 2b: Production build**

```bash
cd frontend && npm run build
```

Expected: build succeeds with no errors.

- [ ] **Step 3: Manual smoke test**

1. Navigate to org → "Retry Schedules" tab → empty state visible
2. Create exponential schedule → preview shows David's table → save → appears in list
3. Create linear schedule (delay 300s) → preview shows uniform rows → save
4. Create custom schedule (3 intervals) → max_retries auto-syncs → save
5. Edit a schedule → form pre-filled → change name → save
6. Delete a schedule → confirmation dialog → deleted
7. Navigate to app → Subscriptions → health column shows badges
8. Edit a subscription → Advanced section → select a retry schedule → save
9. Click subscription name → detail page with health, config, schedule, deliveries
10. Click "Edit" in detail page → navigates to edit form
11. Delete a retry schedule that is assigned to a subscription → verify subscription list still renders, schedule column shows "Default"

- [ ] **Step 4: Final commit (if any fixes)**

```bash
git add -u
git commit -m "fix(frontend): address lint and type errors"
```
