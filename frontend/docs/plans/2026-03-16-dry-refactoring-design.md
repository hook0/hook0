# DRY Refactoring Design

**Date:** 2026-03-16
**Approach:** Mix — composables for shared logic, components for identical UI patterns

## Section 1: Quick Wins (Pure Utilities)

### 1a. Remove local `displayError` copies

6 files re-declare `displayError()` locally instead of importing `@/utils/displayError`.

**Files:** `OrganizationsRemove`, `ApplicationsRemove`, `SubscriptionsRemove`, `LoginPage`, `RegisterPage`, `BeginResetPassword`.

**Action:** Delete local function, add `import { displayError } from '@/utils/displayError'`.

### 1b. Centralize `toMap` / `fromMap`

Move `toMap()` and `fromMap()` into `@/components/Hook0KeyValue.ts` (types file already exists). Import from `SubscriptionsEdit` and `EventsList`.

### 1c. `unwrapResponse<T>` utility

Create `@/utils/unwrapResponse.ts`:

```ts
export function unwrapResponse<T>(promise: Promise<AxiosResponse<T>>): Promise<T> {
  return promise.then(
    (res) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
```

Replace ~20 occurrences across `OrganizationService.ts`, `ApplicationService.ts`, `EventsService.ts`, `SubscriptionService.ts`, and other service files.

---

## Section 2: Reusable UI Components

### 2a. `Hook0DangerZoneCard`

Replaces `OrganizationsRemove.vue`, `ApplicationsRemove.vue`, `SubscriptionsRemove.vue`.

**Props:**
- `entityName: string`
- `warningMessageKey: string`
- `deleteButtonKey: string`
- `confirmMessageKey: string`
- `loading?: boolean`
- `disabled?: boolean`

**Emit:** `@confirm` — parent handles service call + navigation.

Encapsulates: Card + AlertTriangle header + Alert content + Trash2 button + confirmation Dialog.

Each Remove page becomes ~15 lines: import, mutation, `<Hook0DangerZoneCard @confirm="doDelete" />`.

### 2b. `Hook0AuthTrustBadges`

Stateless component rendering the 4 trust badges (open source, uptime, GDPR, no credit card). Used in `LoginPage` and `RegisterPage`. No props, everything from i18n.

---

## Section 3: Shared Logic Composables

### 3a. `useEntityDelete`

Extracted from the 6 list pages' identical delete flow.

```ts
// @/composables/useEntityDelete.ts
function useEntityDelete<T>(options: {
  deleteFn: (entity: T) => Promise<void>;
  onSuccess?: () => void;
  successMessageKey: string;
}) => {
  showDeleteDialog: Ref<boolean>;
  entityToDelete: Ref<T | null>;
  requestDelete: (entity: T) => void;
  confirmDelete: () => void;
}
```

List pages keep their template, replace ~20 lines of state management with destructured return values.

### 3b. `useEntityForm`

Extracted from `OrganizationsEdit` and `ApplicationsEdit` (~85% identical).

```ts
// @/composables/useEntityForm.ts
function useEntityForm<T>(options: {
  queryResult: { data: Ref<T | undefined>; isLoading: Ref<boolean>; error: Ref<Error | null> };
  schema: ZodSchema;
  createFn: (values: any) => Promise<T>;
  updateFn: (values: any) => Promise<T>;
  isNew: Ref<boolean>;
  onCreated?: (entity: T) => void;
  onUpdated?: (entity: T) => void;
}) => {
  form: ReturnType<typeof useForm>;
  handleSubmit: () => void;
  isSubmitting: Ref<boolean>;
}
```

Edit pages keep their template (differs for consumption, cancel button, etc.) but share VeeValidate+Zod logic.

### 3c. `useAuthErrorHandler`

Combines `handleError` + `displayError` for auth pages.

```ts
// @/composables/useAuthErrorHandler.ts
function useAuthErrorHandler() {
  return {
    handleAuthError: (err: unknown) => {
      const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
      displayError(problem);
    }
  };
}
```

Replaces duplicated `.catch()` in `LoginPage`, `RegisterPage`, `BeginResetPassword`.

---

## Section 4: Structural Refactoring

### 4a. `TutorialWizardEntityStep`

Replaces `TutorialWizardStepOrganization` and `TutorialWizardStepApplication`.

**Props:**
- `entityType: 'organization' | 'application'`
- `stepNumber: number`
- `entities: Ref<Entity[]>`
- `editComponent: Component`

Internalizes: `entityId` ref, `selectedEntityId` ref, `entitySection` ref (Create/SelectExisting), auto-select watch, `handleCreated`, `handleAdvance`.

### 4b. `createEntityQueries` factory

```ts
// @/composables/createEntityQueries.ts
function createEntityQueries<TList, TDetail, TCreate, TUpdate>(config: {
  keys: { all: string[]; list: (parentId: string) => string[]; detail: (id: string) => string[] };
  listFn: (parentId: string) => Promise<TList[]>;
  detailFn: (id: string) => Promise<TDetail>;
  createFn?: (data: TCreate) => Promise<TDetail>;
  updateFn?: (data: TUpdate) => Promise<TDetail>;
  removeFn?: (id: string) => Promise<void>;
})
```

Each `use*Queries.ts` file becomes ~15 lines: key definitions + factory call.

### 4c. CSS typography duplication — no action

Inherent to scoped CSS (mandated by CLAUDE.md). Abstraction cost exceeds the gain for 4-line blocks. Left as-is.

---

## Execution Order

1. **1a** displayError imports (zero risk)
2. **1b** toMap/fromMap centralization
3. **1c** unwrapResponse utility + service migration
4. **2a** Hook0DangerZoneCard + Remove pages refactor
5. **2b** Hook0AuthTrustBadges
6. **3c** useAuthErrorHandler
7. **3a** useEntityDelete + list pages refactor
8. **3b** useEntityForm + edit pages refactor
9. **4a** TutorialWizardEntityStep
10. **4b** createEntityQueries factory

Order rationale: utilities first (no dependencies), then UI components, then composables (which may use the new utilities), then structural refactoring (largest scope, highest risk).
