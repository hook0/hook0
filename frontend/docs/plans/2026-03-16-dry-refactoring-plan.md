# DRY Refactoring Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate all DRY violations across the frontend codebase using composables for shared logic and components for identical UI patterns.

**Architecture:** Mix approach — composables absorb duplicated logic (delete flow, form flow, error handling), components replace visually identical UI (DangerZoneCard, AuthTrustBadges), utilities centralize repeated helpers (unwrapResponse, toMap/fromMap).

**Tech Stack:** Vue 3.5, vue-i18n 11, TanStack Query, VeeValidate + Zod, Notivue, Lucide icons

---

### Task 1: Replace local `displayError` copies with shared import

**Files:**
- Modify: `src/pages/organizations/OrganizationsRemove.vue`
- Modify: `src/pages/organizations/applications/ApplicationsRemove.vue`
- Modify: `src/pages/organizations/applications/subscriptions/SubscriptionsRemove.vue`
- Modify: `src/pages/LoginPage.vue`
- Modify: `src/pages/RegisterPage.vue`
- Modify: `src/pages/BeginResetPassword.vue`

**Step 1: In each of the 6 files above:**

Remove the local `displayError` function (typically ~8 lines):
```ts
// DELETE THIS:
function displayError(err: Problem) {
  console.error(err);
  const options = { ... };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
```

Add import at the top (alongside existing imports):
```ts
import { displayError } from '@/utils/displayError';
```

Also remove the now-unused `push` import from `notivue` IF it's no longer used elsewhere in the file. Check each file:
- `OrganizationsRemove.vue`: `push` IS still used in `confirmRemove()` for success toast → KEEP `push` import
- `ApplicationsRemove.vue`: `push` is NOT used elsewhere → REMOVE `push` import
- `SubscriptionsRemove.vue`: `push` is NOT used elsewhere → REMOVE `push` import
- `LoginPage.vue`: `push` IS used for success toast → KEEP `push` import
- `RegisterPage.vue`: `push` is NOT used elsewhere → REMOVE `push` import
- `BeginResetPassword.vue`: `push` IS used for success toast → KEEP `push` import

Also remove the unused `Problem` import IF it's no longer used elsewhere (check each file — the Remove pages use `Problem` only in `displayError` signature).

**Step 2: Run type check**

Run: `npx vue-tsc --noEmit`
Expected: No errors

**Step 3: Run ESLint**

Run: `npx eslint --fix src/pages/organizations/OrganizationsRemove.vue src/pages/organizations/applications/ApplicationsRemove.vue src/pages/organizations/applications/subscriptions/SubscriptionsRemove.vue src/pages/LoginPage.vue src/pages/RegisterPage.vue src/pages/BeginResetPassword.vue`
Expected: No issues

**Step 4: Commit**

```bash
git add -A && git commit -m "refactor(frontend): replace local displayError copies with shared import"
```

---

### Task 2: Centralize `toMap` / `fromMap` in Hook0KeyValue.ts

**Files:**
- Modify: `src/components/Hook0KeyValue.ts`
- Modify: `src/pages/organizations/applications/events/EventsList.vue`
- Modify: `src/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue`

**Step 1: Add utilities to `src/components/Hook0KeyValue.ts`**

Current content is just the type. Add the two functions:

```ts
export type Hook0KeyValueKeyValuePair = {
  key: string;
  value: string;
};

export function kvPairsToRecord(pairs: Hook0KeyValueKeyValuePair[]): Record<string, string> {
  return pairs.reduce<Record<string, string>>((m, { key, value }) => {
    m[key] = value;
    return m;
  }, {});
}

export function recordToKvPairs(record: Record<string, unknown>): Hook0KeyValueKeyValuePair[] {
  return Object.entries(record).map(([key, value]) => ({
    key,
    value: String(value),
  }));
}
```

Note: using `kvPairsToRecord` / `recordToKvPairs` names instead of `toMap` / `fromMap` to be more descriptive.

**Step 2: Replace in EventsList.vue**

Find and remove the local `toMap` function. Replace usage with:
```ts
import { kvPairsToRecord } from '@/components/Hook0KeyValue';
```

Replace all `toMap(...)` calls with `kvPairsToRecord(...)`.

**Step 3: Replace in SubscriptionsEdit.vue**

Find and remove the local `toMap` and `fromMap` functions. Replace with:
```ts
import { kvPairsToRecord, recordToKvPairs } from '@/components/Hook0KeyValue';
```

Replace all `toMap(...)` → `kvPairsToRecord(...)` and `fromMap(...)` → `recordToKvPairs(...)`.

**Step 4: Run type check + ESLint**

Run: `npx vue-tsc --noEmit && npx eslint --fix src/components/Hook0KeyValue.ts src/pages/organizations/applications/events/EventsList.vue src/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue`
Expected: No errors

**Step 5: Commit**

```bash
git add -A && git commit -m "refactor(frontend): centralize toMap/fromMap in Hook0KeyValue.ts"
```

---

### Task 3: Create `unwrapResponse` utility and migrate services

**Files:**
- Create: `src/utils/unwrapResponse.ts`
- Modify: `src/pages/organizations/OrganizationService.ts`
- Modify: `src/pages/organizations/applications/ApplicationService.ts`
- Modify: `src/pages/organizations/applications/events/EventsService.ts`
- Modify: `src/pages/organizations/applications/subscriptions/SubscriptionService.ts`
- Modify: any other `*Service.ts` files that use the pattern

**Step 1: Create `src/utils/unwrapResponse.ts`**

```ts
import type { AxiosError, AxiosResponse } from 'axios';
import { handleError } from '@/http';
import type { Problem } from '@/http';

/**
 * Unwrap an Axios response promise: extract .data on success, handleError on failure.
 * Replaces the repeated .then(res => res.data, err => Promise.reject(handleError(err))) pattern.
 */
export function unwrapResponse<T>(promise: Promise<AxiosResponse<T>>): Promise<T> {
  return promise.then(
    (res: AxiosResponse<T>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
```

**Step 2: Migrate OrganizationService.ts**

Replace each occurrence of:
```ts
http.get('/organizations', {}).then(
  (res: AxiosResponse<Array<Organization>>) => res.data,
  (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
)
```

With:
```ts
unwrapResponse(http.get<Array<Organization>>('/organizations', {}))
```

Import at top:
```ts
import { unwrapResponse } from '@/utils/unwrapResponse';
```

Remove now-unused imports: `AxiosError`, `AxiosResponse`, `handleError`, `Problem` — but only if ALL usages in the file are replaced. In OrganizationService, `handleError` is used only in the unwrap pattern, so it can be removed.

For functions that chain `.then((entity) => useAuthStore().refresh().then(() => entity))` after the unwrap, keep that chain:
```ts
export function create(organization: OrganizationPost): Promise<OrganizationInfo> {
  return unwrapResponse(http.post<OrganizationInfo>('/organizations', organization)).then(
    (organization) => useAuthStore().refresh().then(() => organization)
  );
}
```

**Step 3: Migrate ApplicationService.ts** — same pattern as Step 2.

**Step 4: Migrate EventsService.ts** — same pattern.

**Step 5: Migrate SubscriptionService.ts** — same pattern.

**Step 6: Search for any other Service files with the pattern**

Run: `grep -rl "Promise.reject(handleError" src/` to find remaining files. Migrate them.

**Step 7: Run type check + ESLint**

Run: `npx vue-tsc --noEmit && npx eslint --fix src/utils/unwrapResponse.ts src/pages/organizations/OrganizationService.ts src/pages/organizations/applications/ApplicationService.ts src/pages/organizations/applications/events/EventsService.ts src/pages/organizations/applications/subscriptions/SubscriptionService.ts`
Expected: No errors

**Step 8: Commit**

```bash
git add -A && git commit -m "refactor(frontend): create unwrapResponse utility and migrate all services"
```

---

### Task 4: Create `Hook0DangerZoneCard` component

**Files:**
- Create: `src/components/Hook0DangerZoneCard.vue`
- Modify: `src/pages/organizations/OrganizationsRemove.vue`
- Modify: `src/pages/organizations/applications/ApplicationsRemove.vue`
- Modify: `src/pages/organizations/applications/subscriptions/SubscriptionsRemove.vue`

**Step 1: Create `src/components/Hook0DangerZoneCard.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { AlertTriangle, Trash2 } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';

const { t } = useI18n();

interface Props {
  title: string;
  subtitle: string;
  warningMessage: string;
  confirmMessage: string;
  entityName: string;
  loading?: boolean;
  dataTest?: string;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  dataTest: undefined,
});

const emit = defineEmits<{
  confirm: [];
}>();

const showDeleteDialog = ref(false);

function requestDelete(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();
  showDeleteDialog.value = true;
}

function confirmDelete() {
  showDeleteDialog.value = false;
  emit('confirm');
}
</script>

<template>
  <Hook0Card :data-test="dataTest">
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="danger">
            <AlertTriangle :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <span class="danger-zone__title">{{ title }}</span>
        </Hook0Stack>
      </template>
      <template #subtitle>
        {{ subtitle }}
        <span class="danger-zone__name">{{ entityName }}</span>
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0Alert type="alert">
        <template #description>
          {{ warningMessage }}
        </template>
      </Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        variant="danger"
        type="button"
        :loading="loading"
        :data-test="dataTest ? `${dataTest}-button` : undefined"
        @click="requestDelete($event)"
      >
        <Trash2 :size="16" aria-hidden="true" />
        {{ t('common.delete') }}
      </Hook0Button>
    </Hook0CardFooter>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="title"
      @close="showDeleteDialog = false"
      @confirm="confirmDelete()"
    >
      <p>{{ confirmMessage }}</p>
    </Hook0Dialog>
  </Hook0Card>
</template>

<style scoped>
.danger-zone__title {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}

.danger-zone__name {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
```

**Step 2: Rewrite `OrganizationsRemove.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { push } from 'notivue';
import { useI18n } from 'vue-i18n';

import * as OrganizationService from './OrganizationService';
import { displayError } from '@/utils/displayError';
import router from '@/router';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import Hook0DangerZoneCard from '@/components/Hook0DangerZoneCard.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();
const { canDelete } = usePermissions();

interface Props {
  organizationId: string;
  organizationName: string;
}

const props = defineProps<Props>();
const loading = ref(false);

function confirmRemove() {
  loading.value = true;
  OrganizationService.remove(props.organizationId)
    .then(() => {
      trackEvent('organization', 'delete', 'success');
      push.success({
        title: t('remove.organizationDeleted'),
        message: t('remove.organizationDeletedMessage', { name: props.organizationName }),
        duration: 5000,
      });
      return router.push({ name: routes.Home });
    })
    .catch(displayError)
    .finally(() => (loading.value = false));
}
</script>

<template>
  <Hook0DangerZoneCard
    v-if="canDelete('organization')"
    :title="t('remove.deleteOrganization')"
    :subtitle="t('remove.deleteOrganizationWarning', { name: organizationName })"
    :warning-message="t('remove.irreversibleWarning')"
    :confirm-message="t('remove.confirmDeleteOrganization', { name: organizationName })"
    :entity-name="organizationName"
    :loading="loading"
    data-test="organization-delete-card"
    @confirm="confirmRemove"
  />
</template>
```

**Step 3: Rewrite `ApplicationsRemove.vue`** — same pattern, using `ApplicationsService.remove`, navigating to `routes.OrganizationsDashboard`.

**Step 4: Rewrite `SubscriptionsRemove.vue`** — same pattern, using `SubscriptionsService.remove`, navigating to `routes.SubscriptionsList`.

**Step 5: Verify data-test attributes are preserved**

Check that:
- `data-test="organization-delete-card"` → on the card, `data-test="organization-delete-card-button"` on the button
- `data-test="application-delete-card"` → same pattern
- `data-test="subscription-delete-card"` → same pattern

Note: the button data-test changes from `organization-delete-button` to `organization-delete-card-button`. Search E2E tests for these selectors:
```bash
grep -r "organization-delete-button\|application-delete-button\|subscription-delete-button" tests-e2e/
```
If found, update the selectors in the E2E tests OR adjust the `Hook0DangerZoneCard` to accept a `dataTestButton` prop.

**Step 6: Run type check + ESLint**

Run: `npx vue-tsc --noEmit && npx eslint --fix src/components/Hook0DangerZoneCard.vue src/pages/organizations/OrganizationsRemove.vue src/pages/organizations/applications/ApplicationsRemove.vue src/pages/organizations/applications/subscriptions/SubscriptionsRemove.vue`

**Step 7: Commit**

```bash
git add -A && git commit -m "refactor(frontend): extract Hook0DangerZoneCard, simplify 3 Remove pages"
```

---

### Task 5: Create `Hook0AuthTrustBadges` component

**Files:**
- Create: `src/components/Hook0AuthTrustBadges.vue`
- Modify: `src/pages/LoginPage.vue`
- Modify: `src/pages/RegisterPage.vue`

**Step 1: Create `src/components/Hook0AuthTrustBadges.vue`**

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Shield, CheckCircle } from 'lucide-vue-next';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();
</script>

<template>
  <Hook0Stack align="center" justify="center" gap="lg" wrap responsive>
    <Hook0Badge display="trust" variant="success">
      <template #icon>
        <Shield :size="20" aria-hidden="true" />
      </template>
      {{ t('auth.trust.openSource') }}
    </Hook0Badge>
    <Hook0Badge display="trust" variant="success">
      <template #icon>
        <CheckCircle :size="20" aria-hidden="true" />
      </template>
      {{ t('auth.trust.uptime') }}
    </Hook0Badge>
    <Hook0Badge display="trust" variant="success">
      <template #icon>
        <CheckCircle :size="20" aria-hidden="true" />
      </template>
      {{ t('auth.trust.gdpr') }}
    </Hook0Badge>
  </Hook0Stack>
</template>
```

Note: LoginPage shows `openSource + uptime + gdpr`, RegisterPage shows `openSource + noCreditCard + gdpr`. They are NOT identical. Two options:
- Accept a `badges` prop array to configure which badges to show
- Use slots

Simpler: accept a prop `variant: 'login' | 'register'` that picks the right middle badge. Or even simpler: make the middle badge configurable via a `badges` array prop:

```vue
<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Shield, CheckCircle } from 'lucide-vue-next';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();

interface Props {
  badges?: string[];
}

withDefaults(defineProps<Props>(), {
  badges: () => ['auth.trust.openSource', 'auth.trust.uptime', 'auth.trust.gdpr'],
});
</script>

<template>
  <Hook0Stack align="center" justify="center" gap="lg" wrap responsive>
    <Hook0Badge v-for="badge in badges" :key="badge" display="trust" variant="success">
      <template #icon>
        <Shield v-if="badge === 'auth.trust.openSource'" :size="20" aria-hidden="true" />
        <CheckCircle v-else :size="20" aria-hidden="true" />
      </template>
      {{ t(badge) }}
    </Hook0Badge>
  </Hook0Stack>
</template>
```

**Step 2: Replace in LoginPage.vue**

Replace the trust badges `<Hook0Stack>` block in `<template #footer>` with:
```vue
<Hook0AuthTrustBadges />
```

Remove unused imports: `Shield`, `CheckCircle` (only if not used elsewhere in the file), `Hook0Badge`, `Hook0Stack` (only if not used elsewhere — check carefully, `Hook0Stack` IS used elsewhere in LoginPage).

**Step 3: Replace in RegisterPage.vue**

Replace the trust badges `<Hook0Stack>` block with:
```vue
<Hook0AuthTrustBadges :badges="['auth.trust.openSource', 'auth.trust.noCreditCard', 'auth.trust.gdpr']" />
```

Remove unused imports (`Shield`, `CheckCircle` — but `CheckCircle` may be unused, `Shield` too; `Hook0Badge` IS still used for the subtitle badge; `Hook0Stack` IS still used).

**Step 4: Run type check + ESLint**

Run: `npx vue-tsc --noEmit && npx eslint --fix src/components/Hook0AuthTrustBadges.vue src/pages/LoginPage.vue src/pages/RegisterPage.vue`

**Step 5: Commit**

```bash
git add -A && git commit -m "refactor(frontend): extract Hook0AuthTrustBadges component"
```

---

### Task 6: Create `useAuthErrorHandler` composable

**Files:**
- Create: `src/composables/useAuthErrorHandler.ts`
- Modify: `src/pages/LoginPage.vue`
- Modify: `src/pages/RegisterPage.vue`
- Modify: `src/pages/BeginResetPassword.vue`

**Step 1: Create `src/composables/useAuthErrorHandler.ts`**

```ts
import type { AxiosError, AxiosResponse } from 'axios';
import { handleError } from '@/http';
import type { Problem } from '@/http';
import { displayError } from '@/utils/displayError';

/**
 * Composable for auth pages that combines handleError + displayError.
 * Replaces the duplicated .catch(err => { handleError(err); displayError(problem); }) pattern.
 */
export function useAuthErrorHandler() {
  function handleAuthError(err: unknown): void {
    const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
    displayError(problem);
  }

  return { handleAuthError };
}
```

**Step 2: Migrate LoginPage.vue**

Replace:
```ts
.catch((err) => {
  const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
  trackEvent('auth', 'login', 'error');
  displayError(problem);
})
```

With:
```ts
.catch((err) => {
  trackEvent('auth', 'login', 'error');
  handleAuthError(err);
})
```

Add at top:
```ts
import { useAuthErrorHandler } from '@/composables/useAuthErrorHandler';
const { handleAuthError } = useAuthErrorHandler();
```

Remove now-unused imports: `handleError`, `Problem` (if not used elsewhere), `displayError` (already removed in Task 1), `AxiosError`, `AxiosResponse` (if not used elsewhere).

**Step 3: Migrate RegisterPage.vue** — same pattern. The `.catch` also has tracking:
```ts
.catch((err) => {
  trackEvent('signup', 'form-error', (handleError(err as ...) as Problem).title || 'unknown');
  handleAuthError(err);
})
```
Note: the tracking needs the problem title. Adjust:
```ts
.catch((err) => {
  const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
  trackEvent('signup', 'form-error', problem.title || 'unknown');
  displayError(problem);
})
```
Actually in this case, keep `handleError` + `displayError` separate because tracking needs the `problem` object. Or extend `handleAuthError` to return the problem:

```ts
export function useAuthErrorHandler() {
  function handleAuthError(err: unknown): Problem {
    const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
    displayError(problem);
    return problem;
  }
  return { handleAuthError };
}
```

Then in RegisterPage:
```ts
.catch((err) => {
  const problem = handleAuthError(err);
  trackEvent('signup', 'form-error', problem.title || 'unknown');
})
```

**Step 4: Migrate BeginResetPassword.vue** — straightforward, no tracking on error:
```ts
.catch((err) => handleAuthError(err))
```

**Step 5: Run type check + ESLint**

**Step 6: Commit**

```bash
git add -A && git commit -m "refactor(frontend): extract useAuthErrorHandler composable"
```

---

### Task 7: Create `useEntityDelete` composable

**Files:**
- Create: `src/composables/useEntityDelete.ts`
- Modify: `src/pages/organizations/applications/ApplicationsList.vue`
- Modify: `src/pages/organizations/MembersList.vue`
- Modify: `src/pages/organizations/applications/event_types/EventTypesList.vue`
- Modify: `src/pages/organizations/applications/subscriptions/SubscriptionsList.vue`
- Modify: `src/pages/organizations/services_token/ServicesTokenList.vue`
- Modify: `src/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue`

**Step 1: Create `src/composables/useEntityDelete.ts`**

```ts
import { ref, type Ref } from 'vue';
import { push } from 'notivue';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';

interface UseEntityDeleteOptions<T> {
  deleteFn: (entity: T) => Promise<unknown>;
  successTitle: string;
  successMessage: string | ((entity: T) => string);
}

interface UseEntityDeleteReturn<T> {
  showDeleteDialog: Ref<boolean>;
  entityToDelete: Ref<T | null>;
  deleteLoading: Ref<boolean>;
  requestDelete: (entity: T) => void;
  confirmDelete: () => void;
}

export function useEntityDelete<T>(options: UseEntityDeleteOptions<T>): UseEntityDeleteReturn<T> {
  const showDeleteDialog = ref(false) as Ref<boolean>;
  const entityToDelete = ref<T | null>(null) as Ref<T | null>;
  const deleteLoading = ref(false);

  function requestDelete(entity: T) {
    entityToDelete.value = entity;
    showDeleteDialog.value = true;
  }

  function confirmDelete() {
    const entity = entityToDelete.value;
    if (!entity) return;

    showDeleteDialog.value = false;
    deleteLoading.value = true;

    options
      .deleteFn(entity)
      .then(() => {
        const message =
          typeof options.successMessage === 'function'
            ? options.successMessage(entity)
            : options.successMessage;
        push.success({
          title: options.successTitle,
          message,
          duration: 5000,
        });
      })
      .catch((err: Problem) => displayError(err))
      .finally(() => {
        deleteLoading.value = false;
        entityToDelete.value = null;
      });
  }

  return {
    showDeleteDialog,
    entityToDelete,
    deleteLoading,
    requestDelete,
    confirmDelete,
  };
}
```

**Step 2: Migrate ApplicationsList.vue as the first example**

Find the current delete-related code (typically):
```ts
const showDeleteDialog = ref(false);
const entityToDelete = ref<Application | null>(null);
// + handleDelete function
// + confirmDelete function
```

Replace with:
```ts
import { useEntityDelete } from '@/composables/useEntityDelete';

const removeMutation = useRemoveApplication();
const { showDeleteDialog, entityToDelete, requestDelete, confirmDelete } = useEntityDelete<Application>({
  deleteFn: (app) => removeMutation.mutateAsync(app.application_id),
  successTitle: t('applications.deleted'),
  successMessage: (app) => t('applications.deletedMessage', { name: app.name }),
});
```

Update template references: `handleDelete(entity)` → `requestDelete(entity)`.

**Step 3: Migrate remaining 5 list pages** — same pattern, adapting `deleteFn`, `successTitle`, `successMessage` for each entity type.

**Step 4: Run type check + ESLint on all modified files**

**Step 5: Commit**

```bash
git add -A && git commit -m "refactor(frontend): extract useEntityDelete composable, migrate 6 list pages"
```

---

### Task 8: Create `useEntityForm` composable

**Files:**
- Create: `src/composables/useEntityForm.ts`
- Modify: `src/pages/organizations/OrganizationsEdit.vue`
- Modify: `src/pages/organizations/applications/ApplicationsEdit.vue`

**Step 1: Create `src/composables/useEntityForm.ts`**

```ts
import { computed, watch, type Ref } from 'vue';
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@/utils/zod-adapter';
import type { ZodSchema } from 'zod';
import { push } from 'notivue';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';

interface UseEntityFormOptions<TEntity, TValues> {
  schema: ZodSchema;
  existingData: Ref<TEntity | undefined>;
  isNew: Ref<boolean>;
  mapToFormValues: (entity: TEntity) => TValues;
  createFn: (values: TValues) => Promise<unknown>;
  updateFn: (values: TValues) => Promise<unknown>;
  successCreateTitle: string;
  successCreateMessage: string | ((values: TValues) => string);
  successUpdateTitle: string;
  successUpdateMessage: string | ((values: TValues) => string);
  onCreated?: (values: TValues) => void;
  onUpdated?: (values: TValues) => void;
}

export function useEntityForm<TEntity, TValues extends Record<string, unknown>>(
  options: UseEntityFormOptions<TEntity, TValues>
) {
  const form = useForm({
    validationSchema: toTypedSchema(options.schema),
  });

  // Reset form when existing data loads
  watch(
    () => options.existingData.value,
    (entity) => {
      if (entity) {
        form.resetForm({ values: options.mapToFormValues(entity) });
      }
    },
    { immediate: true }
  );

  const isSubmitting = computed(() => form.isSubmitting.value);

  const onSubmit = form.handleSubmit((values: TValues) => {
    const fn = options.isNew.value ? options.createFn : options.updateFn;
    const titleKey = options.isNew.value ? options.successCreateTitle : options.successUpdateTitle;
    const msgKey = options.isNew.value ? options.successCreateMessage : options.successUpdateMessage;
    const callback = options.isNew.value ? options.onCreated : options.onUpdated;

    return fn(values)
      .then(() => {
        const message = typeof msgKey === 'function' ? msgKey(values) : msgKey;
        push.success({ title: titleKey, message, duration: 5000 });
        callback?.(values);
      })
      .catch((err: Problem) => displayError(err));
  });

  return {
    form,
    errors: form.errors,
    defineField: form.defineField,
    isSubmitting,
    onSubmit,
  };
}
```

**Step 2: Migrate OrganizationsEdit.vue**

Replace the ~30 lines of form setup (useForm, defineField, watch resetForm, handleSubmit with create/update branching) with:
```ts
import { useEntityForm } from '@/composables/useEntityForm';
import { organizationSchema } from './organization.schema';

const { errors, defineField, isSubmitting, onSubmit } = useEntityForm<OrganizationInfo, { name: string }>({
  schema: organizationSchema,
  existingData: data,
  isNew,
  mapToFormValues: (org) => ({ name: org.name }),
  createFn: (values) => createMutation.mutateAsync(values),
  updateFn: (values) => updateMutation.mutateAsync({ organizationId: organizationId.value!, organization: values }),
  successCreateTitle: t('organizations.created'),
  successCreateMessage: (v) => t('organizations.createdMessage', { name: v.name }),
  successUpdateTitle: t('organizations.updated'),
  successUpdateMessage: (v) => t('organizations.updatedMessage', { name: v.name }),
  onCreated: (values) => { /* navigate + emit tutorial event */ },
});
```

**Step 3: Migrate ApplicationsEdit.vue** — same pattern with application-specific fields.

**Step 4: Run type check + ESLint**

**Step 5: Commit**

```bash
git add -A && git commit -m "refactor(frontend): extract useEntityForm composable, migrate edit pages"
```

---

### Task 9: Merge tutorial wizard steps into `TutorialWizardEntityStep`

**Files:**
- Create: `src/pages/tutorial/TutorialWizardEntityStep.vue`
- Modify: `src/pages/tutorial/TutorialWizardStepOrganization.vue` (thin wrapper)
- Modify: `src/pages/tutorial/TutorialWizardStepApplication.vue` (thin wrapper)

**Step 1: Create `src/pages/tutorial/TutorialWizardEntityStep.vue`**

Extract the shared logic from both steps. The component accepts:

```ts
interface Props {
  entityType: 'organization' | 'application';
  stepNumber: number;
  stepTitle: string;
  stepDescription: string;
  chooseLabel: string;
  createLabel: string;
  selectExistingLabel: string;
  selectLabel: string;
  continueLabel: string;
  progressSteps: ProgressStep[];
  entityIcon: Component;
  entities: { label: string; value: string }[];
  entitiesLoading: boolean;
  entitiesError: Error | null;
  editComponent: Component;
  organizationId?: string; // only for application step
}
```

Emits: `advance(entityId)`, `skip()`, `entity-created(entityId)`.

The template is the merged version of both steps: header (Badge + title + close), content (description + TutorialStepProgress + SelectableCard grid + inline edit or select dropdown), footer (skip + advance button).

**Step 2: Rewrite `TutorialWizardStepOrganization.vue` as a thin wrapper**

```vue
<script setup lang="ts">
import { computed } from 'vue';
import type { Component } from 'vue';
import { useI18n } from 'vue-i18n';
import { Building2 } from 'lucide-vue-next';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import TutorialWizardEntityStep from './TutorialWizardEntityStep.vue';

type ProgressStep = { icon: Component; label: string };
type Props = { progressSteps: ProgressStep[] };
defineProps<Props>();

const emit = defineEmits<{ advance: [id: string]; skip: [] }>();
const { t } = useI18n();
const { data, isLoading, error, refetch } = useOrganizationList();

const options = computed(() => [
  { label: t('tutorial.selectOrganization'), value: '' },
  ...(data.value ?? []).map((o) => ({ label: o.name, value: o.organization_id })),
]);
</script>

<template>
  <TutorialWizardEntityStep
    entity-type="organization"
    :step-number="1"
    :step-title="t('tutorial.step1Title')"
    :step-description="t('tutorial.step1Description')"
    :choose-label="t('tutorial.chooseOrganization')"
    :create-label="t('tutorial.createNewOrganization')"
    :select-existing-label="t('tutorial.selectExistingOrganization')"
    :select-label="t('tutorial.selectOrganization')"
    :continue-label="t('tutorial.continueStep2')"
    :progress-steps="progressSteps"
    :entity-icon="Building2"
    :entities="options"
    :entities-loading="isLoading"
    :entities-error="error"
    :edit-component="OrganizationsEdit"
    @advance="emit('advance', $event)"
    @skip="emit('skip')"
  />
</template>
```

**Step 3: Rewrite `TutorialWizardStepApplication.vue`** — same thin wrapper pattern with application-specific data.

**Step 4: Run type check + ESLint**

**Step 5: Commit**

```bash
git add -A && git commit -m "refactor(frontend): merge tutorial wizard steps into TutorialWizardEntityStep"
```

---

### Task 10: Create `createEntityQueries` factory

**Files:**
- Create: `src/composables/createEntityQueries.ts`
- Modify: `src/pages/organizations/useOrganizationQueries.ts`
- Modify: `src/pages/organizations/applications/useApplicationQueries.ts`
- Modify: `src/pages/organizations/applications/event_types/useEventTypeQueries.ts`
- Modify: `src/pages/organizations/applications/subscriptions/useSubscriptionQueries.ts`

**Step 1: Create `src/composables/createEntityQueries.ts`**

```ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';

interface EntityQueryKeys {
  all: readonly string[];
  lists: () => readonly string[];
  list?: (...args: string[]) => readonly string[];
  details?: () => readonly string[];
  detail?: (...args: string[]) => readonly string[];
}

interface CreateEntityQueriesConfig<TList, TDetail, TCreate, TUpdate> {
  keys: EntityQueryKeys;
  listFn?: (...args: string[]) => Promise<TList[]>;
  detailFn?: (...args: string[]) => Promise<TDetail>;
  createFn?: (data: TCreate) => Promise<TDetail>;
  updateFn?: (data: TUpdate) => Promise<TDetail>;
  removeFn?: (...args: string[]) => Promise<void>;
}

export function createEntityQueries<TList, TDetail, TCreate = never, TUpdate = never>(
  config: CreateEntityQueriesConfig<TList, TDetail, TCreate, TUpdate>
) {
  function useList(parentId?: Ref<string>) {
    const queryKey = parentId
      ? computed(() => (config.keys.list ? config.keys.list(parentId.value) : config.keys.lists()))
      : config.keys.lists();

    return useQuery({
      queryKey,
      queryFn: () => config.listFn!(parentId?.value ?? ''),
      enabled: parentId ? computed(() => !!parentId.value) : true,
    });
  }

  function useDetail(id: Ref<string>, ...extraArgs: Ref<string>[]) {
    return useQuery({
      queryKey: computed(() =>
        config.keys.detail
          ? config.keys.detail(id.value, ...extraArgs.map((a) => a.value))
          : [...config.keys.all, 'detail', id.value]
      ),
      queryFn: () => config.detailFn!(id.value, ...extraArgs.map((a) => a.value)),
      enabled: computed(() => !!id.value),
    });
  }

  function useCreate() {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: (data: TCreate) => config.createFn!(data),
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: config.keys.all });
      },
    });
  }

  function useUpdate() {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: (data: TUpdate) => config.updateFn!(data),
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: config.keys.all });
      },
    });
  }

  function useRemove() {
    const queryClient = useQueryClient();
    return useMutation({
      mutationFn: (...args: string[]) => config.removeFn!(...args),
      onSuccess: () => {
        void queryClient.invalidateQueries({ queryKey: config.keys.all });
      },
    });
  }

  return {
    useList: config.listFn ? useList : undefined,
    useDetail: config.detailFn ? useDetail : undefined,
    useCreate: config.createFn ? useCreate : undefined,
    useUpdate: config.updateFn ? useUpdate : undefined,
    useRemove: config.removeFn ? useRemove : undefined,
  };
}
```

**Step 2: Migrate `useOrganizationQueries.ts`**

Note: the migration needs to preserve the exact same public API (function names and signatures) that the rest of the codebase imports. The factory generates the internals, but we re-export with the original names:

```ts
import { createEntityQueries } from '@/composables/createEntityQueries';
import * as OrganizationService from './OrganizationService';
import type { OrganizationPost, OrganizationInfo, Organization } from './OrganizationService';
import { organizationKeys } from '@/queries/keys';

const queries = createEntityQueries<Organization, OrganizationInfo, OrganizationPost, { organizationId: string; organization: OrganizationPost }>({
  keys: organizationKeys,
  listFn: () => OrganizationService.list(),
  detailFn: (id) => OrganizationService.get(id),
  createFn: (data) => OrganizationService.create(data),
  updateFn: (data) => OrganizationService.update(data.organizationId, data.organization),
  removeFn: (id) => OrganizationService.remove(id),
});

export const useOrganizationList = queries.useList!;
export const useOrganizationDetail = queries.useDetail!;
export const useCreateOrganization = queries.useCreate!;
export const useUpdateOrganization = queries.useUpdate!;
export const useRemoveOrganization = queries.useRemove!;
```

Important: verify that the `useOrganizationList()` signature (no params) still works. The factory's `useList` takes an optional `parentId` — calling with no args should work.

**Step 3: Migrate `useApplicationQueries.ts`** — `useApplicationList(organizationId)` takes a `Ref<string>`, so it maps to `useList(parentId)`.

**Step 4: Migrate `useEventTypeQueries.ts`** and `useSubscriptionQueries.ts` — same pattern.

**Step 5: Run type check + ESLint**

This task has the highest risk of type mismatches. Run `npx vue-tsc --noEmit` carefully and fix any issues. The factory's generic types need to match each domain's specific types.

**Step 6: Commit**

```bash
git add -A && git commit -m "refactor(frontend): create query factory, migrate 4 query files"
```

---

## Execution Checklist

| # | Task | Risk | Files |
|---|------|------|-------|
| 1 | displayError imports | Zero | 6 |
| 2 | toMap/fromMap centralize | Zero | 3 |
| 3 | unwrapResponse + services | Low | 5+ |
| 4 | Hook0DangerZoneCard | Low | 4 |
| 5 | Hook0AuthTrustBadges | Low | 3 |
| 6 | useAuthErrorHandler | Low | 4 |
| 7 | useEntityDelete | Medium | 7 |
| 8 | useEntityForm | Medium | 3 |
| 9 | TutorialWizardEntityStep | Medium | 3 |
| 10 | createEntityQueries | High | 5 |

**After each task:** run `npx vue-tsc --noEmit` + `npx eslint --fix` on modified files. If E2E tests exist, run them after tasks 4, 7, 8, 9.
