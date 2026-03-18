# Navigation Redesign — Clerk-style 2-Level Context Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the current logo + breadcrumbs + flat nav with a Clerk-style 2-level navigation: context bar (org/app switchers) on top, contextual tabs below.

**Architecture:** The `Hook0TopNav` is rewritten to have 2 rows: a context bar (org selector, app selector with dropdowns) and a tab bar (changes based on org vs app level). `Hook0Breadcrumbs` is removed entirely — the context bar replaces it. The `useNavigationTabs` composable stays mostly unchanged. A new `useOrgAppSwitcher` composable provides the dropdown data (org list, app list).

**Tech Stack:** Vue 3 Composition API, vue-router, Pinia (context store), TanStack Query (org/app lists), vue-i18n, Lucide icons.

---

## Task 1: Create `useOrgAppSwitcher` composable

**Files:**
- Create: `frontend/src/composables/useOrgAppSwitcher.ts`

**What it does:**
- Exposes reactive data for org switcher dropdown: list of orgs (from existing `useOrganizationList` query), current org name/id, loading state
- Exposes reactive data for app switcher dropdown: list of apps for current org (from existing `useApplicationList` query), current app name/id, loading state
- Provides `switchOrg(orgId)` → navigates to org dashboard
- Provides `switchApp(appId)` → navigates to app dashboard
- Provides `goToOrgSettings()` → navigates to org settings
- Provides `goToCreateOrg()` → navigates to org creation
- Provides `goToCreateApp()` → navigates to app creation
- Reads org/app names from `useContextStore`

**Step 1: Create the composable**

```typescript
// frontend/src/composables/useOrgAppSwitcher.ts
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { routes } from '@/routes';
import { useContextStore } from '@/stores/context';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';

export function useOrgAppSwitcher() {
  const router = useRouter();
  const ctx = useContextStore();

  const currentOrgId = computed(() => ctx.organizationId);
  const currentOrgName = computed(() => ctx.organizationName);
  const currentAppId = computed(() => ctx.applicationId);
  const currentAppName = computed(() => ctx.applicationName);

  const { data: orgs, isLoading: orgsLoading } = useOrganizationList();
  const orgIdForApps = computed(() => currentOrgId.value || '');
  const { data: apps, isLoading: appsLoading } = useApplicationList(orgIdForApps);

  const isAppLevel = computed(() => !!currentOrgId.value && !!currentAppId.value);
  const isOrgLevel = computed(() => !!currentOrgId.value && !currentAppId.value);

  function switchOrg(orgId: string) {
    void router.push({ name: routes.ApplicationsList, params: { organization_id: orgId } });
  }
  function switchApp(orgId: string, appId: string) {
    void router.push({ name: routes.ApplicationsDashboard, params: { organization_id: orgId, application_id: appId } });
  }
  function goToOrgSettings() {
    if (currentOrgId.value) {
      void router.push({ name: routes.OrganizationsDetail, params: { organization_id: currentOrgId.value } });
    }
  }
  function goToCreateOrg() {
    void router.push({ name: routes.OrganizationsNew });
  }
  function goToCreateApp() {
    if (currentOrgId.value) {
      void router.push({ name: routes.ApplicationsNew, params: { organization_id: currentOrgId.value } });
    }
  }
  function goToOrgDashboard() {
    if (currentOrgId.value) {
      void router.push({ name: routes.ApplicationsList, params: { organization_id: currentOrgId.value } });
    }
  }

  return {
    currentOrgId, currentOrgName, currentAppId, currentAppName,
    orgs, orgsLoading, apps, appsLoading,
    isAppLevel, isOrgLevel,
    switchOrg, switchApp, goToOrgSettings, goToCreateOrg, goToCreateApp, goToOrgDashboard,
  };
}
```

**Step 2: Verify compilation**

Run: `npx vue-tsc --noEmit`

**Step 3: Commit**

```
feat(frontend): add useOrgAppSwitcher composable
```

---

## Task 2: Rewrite `Hook0TopNav` — context bar + tabs (2 rows)

**Files:**
- Modify: `frontend/src/components/Hook0TopNav.vue`

**What changes:**
- Row 1 (context bar): `[avatar] Org Name` (clic → org dashboard) + `[<>]` (clic → org dropdown) + optionally `/ [icon] App Name [<>]` + right side (Search, Docs, API, User)
- Row 2 (tabs): same tabs as before from `useNavigationTabs`, rendered below the context bar
- Remove the old logo, the old nav tab rendering from row 1
- The org dropdown shows: list of orgs with "Manage" button, separator, "Create organization"
- The app dropdown shows: list of apps for current org, separator, "Create application"
- Dropdowns use click-outside to close (existing pattern)

**Key template structure:**
```vue
<header class="hook0-topnav">
  <!-- Row 1: Context Bar -->
  <div class="hook0-topnav__context-bar">
    <!-- Org selector -->
    <Hook0Avatar :name="currentOrgName" size="sm" variant="square" />
    <button class="hook0-topnav__org-name" @click="goToOrgDashboard">{{ currentOrgName }}</button>
    <button class="hook0-topnav__switcher" @click="toggleOrgDropdown">
      <ChevronsUpDown :size="14" />
    </button>

    <!-- App selector (when in app context) -->
    <template v-if="isAppLevel">
      <span class="hook0-topnav__separator">/</span>
      <Box :size="14" />
      <span class="hook0-topnav__app-name">{{ currentAppName }}</span>
      <button class="hook0-topnav__switcher" @click="toggleAppDropdown">
        <ChevronsUpDown :size="14" />
      </button>
    </template>

    <!-- Right side -->
    <div class="hook0-topnav__right">
      <button @click="openSearch">Search ⌘K</button>
      <a href="..." target="_blank">Docs</a>
      <a href="..." target="_blank">API</a>
      <button @click="toggleUserMenu">{{ userInitial }}</button>
    </div>
  </div>

  <!-- Org dropdown -->
  <div v-if="orgDropdownOpen" class="hook0-topnav__dropdown">
    <div v-for="org in orgs" @click="switchOrg(org.id)">...</div>
    <hr />
    <div @click="goToCreateOrg">+ Create organization</div>
  </div>

  <!-- App dropdown -->
  <div v-if="appDropdownOpen" class="hook0-topnav__dropdown">
    <div v-for="app in apps" @click="switchApp(orgId, app.id)">...</div>
    <hr />
    <div @click="goToCreateApp">+ Create application</div>
  </div>

  <!-- Row 2: Tabs -->
  <nav class="hook0-topnav__tabs">
    <router-link v-for="tab in navTabs" :to="tab.to" :class="{ active: tab.active }">
      <component :is="tab.icon" :size="14" /> {{ tab.label }}
    </router-link>
  </nav>
</header>
```

**CSS structure:**
- `.hook0-topnav` — sticky top, white bg, border-bottom
- `.hook0-topnav__context-bar` — flex row, padding 0.75rem 1rem, items centered
- `.hook0-topnav__tabs` — flex row, border-bottom, tabs with active underline
- `.hook0-topnav__dropdown` — absolute positioned, z-50, shadow, border-radius
- `.hook0-topnav__switcher` — small button with border, 1.5rem square
- `.hook0-topnav__separator` — `/` divider, muted color
- `.hook0-topnav__org-name` — clickable, bold, hover: primary color
- `.hook0-topnav__app-name` — secondary color
- Responsive: on mobile, context bar collapses to hamburger + drawer

**Step 1: Rewrite the component**

Full rewrite of Hook0TopNav.vue preserving:
- `data-test` attributes
- Mobile hamburger menu (toggle drawer)
- User dropdown menu with settings/logout
- Search button opening command palette
- External links (Docs, API Reference)
- Dark mode toggle in user menu
- Icon-only tabs at tablet (< 1280px)

**Step 2: Verify compilation**

Run: `npx vue-tsc --noEmit`

**Step 3: Verify visual**

Check http://localhost:3000 at org level and app level.

**Step 4: Commit**

```
feat(frontend): rewrite TopNav with Clerk-style context bar + tabs
```

---

## Task 3: Remove `Hook0Breadcrumbs` from layout

**Files:**
- Modify: `frontend/src/App.vue` — remove `<Hook0Breadcrumbs />` and its import
- Modify: `frontend/src/components/Hook0PageLayout.vue` — remove breadcrumb rendering if it delegates to Hook0Breadcrumbs
- Keep: `frontend/src/components/Hook0Breadcrumbs.vue` — don't delete yet (might be needed for reference)

**Step 1: Remove from App.vue**

Remove the `<Hook0Breadcrumbs />` component and its import. The context bar in TopNav now handles the org/app hierarchy display.

**Step 2: Check if Hook0PageLayout renders breadcrumbs**

If Hook0PageLayout has breadcrumb logic, remove it too. The page title should remain.

**Step 3: Verify compilation**

Run: `npx vue-tsc --noEmit`

**Step 4: Verify no broken pages**

Navigate through all major pages and confirm no breadcrumb remnants.

**Step 5: Commit**

```
refactor(frontend): remove Hook0Breadcrumbs from layout (replaced by context bar)
```

---

## Task 4: Update `Hook0MobileTabBar` and `Hook0MobileDrawer`

**Files:**
- Modify: `frontend/src/components/Hook0MobileTabBar.vue`
- Modify: `frontend/src/components/Hook0MobileDrawer.vue`

**What changes:**
- MobileTabBar: stays the same (app-level tabs at bottom)
- MobileDrawer: update to show org switcher at the top (like the desktop dropdown), then app-level nav items. Remove any breadcrumb-like rendering.
- Drawer should show: current org (with switch option), current app if applicable, nav items, theme toggle, settings, logout.

**Step 1: Update drawer**

Add org switcher section at top of drawer content.

**Step 2: Verify on mobile viewport**

**Step 3: Commit**

```
feat(frontend): update mobile drawer with org/app switcher
```

---

## Task 5: Add i18n keys

**Files:**
- Modify: `frontend/src/locales/en.json`

**Keys to add:**
```json
{
  "nav": {
    "orgSettings": "Organization Settings",
    "createOrganization": "Create organization",
    "createApplication": "Create application",
    "switchOrganization": "Switch organization",
    "manage": "Manage"
  }
}
```

**Step 1: Add keys**

**Step 2: Commit**

```
feat(frontend): add i18n keys for nav redesign
```

---

## Task 6: Update E2E tests

**Files:**
- Modify: `tests-e2e/tests/navigation.spec.ts` — update selectors for new nav structure
- Modify: any test that uses breadcrumb selectors

**What changes:**
- Replace breadcrumb selectors with context bar selectors
- Update navigation assertions (org/app switching now via dropdown)
- Add data-test attributes to new nav elements

**Step 1: Add data-test attributes to TopNav context bar**

```
data-test="context-bar-org-name"
data-test="context-bar-org-switcher"
data-test="context-bar-app-name"
data-test="context-bar-app-switcher"
data-test="context-bar-org-dropdown"
data-test="context-bar-app-dropdown"
```

**Step 2: Update navigation tests**

**Step 3: Run full test suite**

Run: `npx playwright test --project=chromium --workers=2`

**Step 4: Commit**

```
test(e2e): update navigation tests for context bar
```

---

## Execution Summary

| Task | What | Files | Estimated |
|------|------|-------|-----------|
| 1 | Composable useOrgAppSwitcher | 1 new | Small |
| 2 | Rewrite Hook0TopNav (context bar + tabs) | 1 modify | Large |
| 3 | Remove Hook0Breadcrumbs from layout | 2 modify | Small |
| 4 | Update mobile drawer | 2 modify | Medium |
| 5 | i18n keys | 1 modify | Small |
| 6 | E2E tests | 2+ modify | Medium |
