# Missing E2E Tests — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add 18 missing E2E test suites covering all untested features, bringing total coverage from 106 to ~130+ tests.

**Architecture:** Each test file follows the existing pattern: `setupTestEnvironment()` helper → data-test selectors → Three-Step Verification Pattern. Frontend data-test attrs are added first, then tests written against them.

**Tech Stack:** Playwright, TypeScript, pg (for DB setup), existing fixtures.

---

## Phase 1: Add data-test attributes to frontend (prerequisite)

### Task 1: Add data-test attrs for Error 404, Command Palette, Mobile Nav

**Files:**
- Modify: `frontend/src/pages/Error404.vue`
- Modify: `frontend/src/components/Hook0CommandPalette.vue`
- Modify: `frontend/src/composables/useCommandPalette.ts` (if needed for item IDs)
- Modify: `frontend/src/components/Hook0MobileTabBar.vue`
- Modify: `frontend/src/components/Hook0MobileDrawer.vue`

**Attrs to add:**

Error404:
- `data-test="error-404-page"` on root
- `data-test="error-404-title"` on title
- `data-test="error-404-dashboard-button"` on "Go to Dashboard"
- `data-test="error-404-back-button"` on "Go Back"

CommandPalette:
- `data-test="command-palette-overlay"` on overlay
- `data-test="command-palette-input"` on search input
- `data-test="command-palette-item"` on each result item
- `data-test="command-palette-empty"` on no-results

MobileTabBar:
- `data-test="mobile-tab-bar"` on nav
- `data-test="mobile-tab-{name}"` on each tab (events, types, webhooks, settings, more)

MobileDrawer:
- `data-test="mobile-drawer"` on drawer root
- `data-test="mobile-drawer-close"` on close button
- `data-test="mobile-drawer-nav-item"` on nav items
- `data-test="mobile-drawer-theme-toggle"` on theme toggle
- `data-test="mobile-drawer-logout"` on logout

**Commit:** `feat(frontend): add data-test attrs for 404, command palette, mobile nav`

---

### Task 2: Add data-test attrs for Tutorial steps 2-6, Service Token detail

**Files:**
- Modify: `frontend/src/pages/tutorial/TutorialWizardStepApplication.vue` or `TutorialWizardEntityStep.vue`
- Modify: `frontend/src/pages/tutorial/TutorialWizardStepForm.vue`
- Modify: `frontend/src/pages/tutorial/TutorialWizardStepSuccess.vue`
- Modify: `frontend/src/pages/tutorial/TutorialWizard.vue`
- Modify: `frontend/src/pages/organizations/services_token/ServiceTokenView.vue`

**Attrs to add:**

Tutorial Step 2 (Application):
- `data-test="tutorial-create-app-radio"` on create option
- `data-test="tutorial-select-app-option"` on select option

Tutorial Steps 3-5 (Form):
- `data-test="tutorial-step-form"` on form wrapper
- `data-test="tutorial-step-done"` on "Continue" button when form done

Tutorial Step 6 (Success):
- `data-test="tutorial-success-card"` on card
- `data-test="tutorial-success-dashboard-button"` on "Go to Dashboard"

Wizard shell:
- `data-test="tutorial-wizard-modal"` on modal

ServiceTokenView:
- `data-test="service-token-detail-card"` on card
- `data-test="service-token-detail-name"` on name field
- `data-test="service-token-detail-token"` on token display

**Commit:** `feat(frontend): add data-test attrs for tutorial steps 2-6, service token detail`

---

### Task 3: Add data-test attrs for remaining components

**Files:**
- Modify: `frontend/src/pages/api/documentation/ApiDocumentation.vue`
- Modify: `frontend/src/pages/CheckEmailPage.vue`
- Modify: `frontend/src/Home.vue`
- Modify: `frontend/src/components/Hook0Breadcrumbs.vue`
- Modify: `frontend/src/components/Hook0ShortcutsCheatSheet.vue`
- Modify: `frontend/src/pages/organizations/OrganizationsDashboard.vue`
- Modify: `frontend/src/pages/organizations/applications/ApplicationsDashboard.vue`

**Attrs to add:**

ApiDocumentation:
- `data-test="api-docs-page"` on root

CheckEmailPage:
- `data-test="check-email-page"` on root
- `data-test="check-email-title"` on title

Home:
- `data-test="home-page"` on root
- `data-test="home-banner"` on banner
- `data-test="home-org-card"` on org cards

Breadcrumbs:
- `data-test="breadcrumb-org"` on org segment
- `data-test="breadcrumb-app"` on app segment

ShortcutsCheatSheet:
- `data-test="shortcuts-dialog"` on dialog
- `data-test="shortcuts-close"` on close

OrgDashboard:
- `data-test="org-dashboard-page"` on root

AppDashboard:
- `data-test="app-dashboard-page"` on root
- `data-test="app-dashboard-tutorial-widget"` on tutorial widget section

**Commit:** `feat(frontend): add data-test attrs for docs, email, home, breadcrumbs, shortcuts, dashboards`

---

## Phase 2: P0 Tests (Critical)

### Task 4: Tutorial wizard full flow test (steps 1-6)

**File:** Create `tests-e2e/tests/tutorial-wizard-flow.spec.ts`

**Tests:**
1. `should complete full tutorial wizard flow from intro to success` — The big integration test:
   - Login with fresh user
   - Navigate to /tutorial
   - Click Start → step 1 (org)
   - Select "Create new" → fill org name → submit → wait for API response
   - Step 2 (app): Select "Create new" → fill app name → submit
   - Step 3 (event type): Fill event type segments → submit
   - Step 4 (subscription): Fill URL → submit
   - Step 5 (send event): Send test event → submit
   - Step 6: Verify success page → click "Go to Dashboard"
   - Verify redirect to app dashboard

2. `should allow selecting existing organization in step 1` — Create org via API first, then select it in dropdown

3. `should allow selecting existing application in step 2` — Create org+app via API, start at step 2

4. `should dismiss wizard and resume later` — Start wizard, dismiss at step 2, navigate back to /tutorial, verify it returns to intro (or step 1)

**Commit:** `test(e2e): add tutorial wizard full flow tests`

---

### Task 5: Error 404 page test

**File:** Create `tests-e2e/tests/error-404.spec.ts`

**Tests:**
1. `should display 404 page for non-existent route` — Navigate to `/this-does-not-exist`, verify `error-404-page` visible, title text, illustration
2. `should navigate to dashboard from 404 page` — Click "Go to Dashboard" → verify redirect to `/`
3. `should go back from 404 page` — Navigate to valid page, then to 404, click "Go Back" → verify back navigation

**Commit:** `test(e2e): add error 404 page tests`

---

## Phase 3: P1 Tests (High)

### Task 6: Command palette test

**File:** Create `tests-e2e/tests/command-palette.spec.ts`

**Tests:**
1. `should open command palette via Search button` — Click Search → verify overlay
2. `should display search results when typing` — Type "event" → verify filtered results
3. `should navigate to selected result` — Type "event types" → press Enter → verify URL
4. `should close palette with Escape` — Open → Escape → verify closed
5. `should show organization switching options` — Open → verify org names in results

**Commit:** `test(e2e): add command palette tests`

---

### Task 7: Dark mode toggle test

**File:** Create `tests-e2e/tests/dark-mode.spec.ts`

**Tests:**
1. `should toggle dark mode from user settings` — Login → /settings → find `theme-select` → select "Dark" → verify `html.dark` class
2. `should persist dark mode across page reload` — Enable dark → reload → verify still dark
3. `should persist dark mode after logout and login` — Enable dark → logout → verify login page is dark → login → verify still dark

**Commit:** `test(e2e): add dark mode tests`

---

### Task 8: Service token detail view test

**File:** Create `tests-e2e/tests/service-token-detail.spec.ts`

**Tests:**
1. `should navigate to service token detail page` — Create token → click to view → verify detail card
2. `should display token info and biscuit content` — Verify token name, created date, token preview

**Commit:** `test(e2e): add service token detail tests`

---

### Task 9: Subscription test endpoint

**File:** Add to existing `tests-e2e/tests/subscriptions.spec.ts`

**Tests:**
1. `should test subscription endpoint and display results` — Create subscription with real URL (webhook.site or httpbin) → click test endpoint → verify status, latency

Note: This test requires a reachable endpoint. Use `https://httpbin.org/post` or skip if unreachable.

**Commit:** `test(e2e): add subscription endpoint test`

---

## Phase 4: P2 Tests (Medium)

### Task 10: Mobile navigation tests

**File:** Create `tests-e2e/tests/mobile-navigation.spec.ts`

Use mobile viewport: `test.use({ viewport: { width: 375, height: 812 } })`

**Tests:**
1. `should display mobile tab bar on mobile viewport` — Verify `mobile-tab-bar` visible
2. `should navigate via tab bar tabs` — Tap Events tab → verify events page
3. `should open drawer via More tab` — Tap More → verify `mobile-drawer` visible
4. `should navigate via drawer links` — Open drawer → tap a link → verify navigation → verify drawer closed
5. `should close drawer via close button` — Open → close → verify hidden

**Commit:** `test(e2e): add mobile navigation tests`

---

### Task 11: Dashboard tests (org + app)

**File:** Create `tests-e2e/tests/dashboards.spec.ts`

**Tests:**
1. `should display organization dashboard with apps list` — Navigate to org dashboard → verify `org-dashboard-page`, apps section
2. `should display application dashboard with overview sections` — Navigate to app dashboard → verify `app-dashboard-page`, event types / events / subscriptions overview sections
3. `should display tutorial widget on app dashboard` — Verify `app-dashboard-tutorial-widget` visible with progress

**Commit:** `test(e2e): add dashboard tests`

---

### Task 12: Breadcrumb navigation test

**File:** Add to existing `tests-e2e/tests/navigation.spec.ts`

**Tests:**
1. `should navigate via breadcrumb org segment` — On app page → click `breadcrumb-org` → verify navigation to org dashboard
2. `should navigate via breadcrumb app dropdown` — On event types page → click `breadcrumb-app` dropdown → select different app

**Commit:** `test(e2e): add breadcrumb navigation tests`

---

### Task 13: Home page org/app selector test

**File:** Add to existing `tests-e2e/tests/homepage.spec.ts`

**Tests:**
1. `should display org cards on home page` — Login → verify `home-org-card` visible
2. `should navigate to org dashboard when clicking org card` — Click org card → verify redirect

**Commit:** `test(e2e): add home page org selector tests`

---

### Task 14: Subscription event type checkboxes

**File:** Add to existing `tests-e2e/tests/subscriptions.spec.ts`

**Tests:**
1. `should select and deselect event types when creating subscription` — Open create form → verify `event-types-list` → check `event-type-checkbox-0` → verify checked → uncheck → verify unchecked

**Commit:** `test(e2e): add subscription event type checkbox test`

---

## Phase 5: P3 Tests (Low)

### Task 15: Keyboard shortcuts cheat sheet test

**File:** Create `tests-e2e/tests/keyboard-shortcuts.spec.ts`

**Tests:**
1. `should open shortcuts cheat sheet with ? key` — Press ? → verify `shortcuts-dialog` visible
2. `should close shortcuts with Escape` — Open → Escape → verify hidden

**Commit:** `test(e2e): add keyboard shortcuts tests`

---

### Task 16: API Documentation page test

**File:** Create `tests-e2e/tests/api-documentation.spec.ts`

**Tests:**
1. `should display API documentation page` — Navigate to `/api/documentation` → verify `api-docs-page`
2. `should display app-scoped documentation` — Navigate to app-level docs → verify renders

**Commit:** `test(e2e): add API documentation tests`

---

### Task 17: Check email / verify email pages test

**File:** Add to existing `tests-e2e/tests/auth.spec.ts`

**Tests:**
1. `should display check email page after registration` — Register → verify redirect to `/check-email` → verify `check-email-page`

**Commit:** `test(e2e): add check email page test`

---

### Task 18: Key-Value input component test

**File:** Add to existing `tests-e2e/tests/events.spec.ts`

**Tests:**
1. `should add and remove labels when sending event` — Open send event → click `kv-add-button-0` → fill `kv-key-input-0` + `kv-value-input-0` → click `kv-remove-button-0`

**Commit:** `test(e2e): add key-value input test`

---

### Task 19: Side panel (event detail) test

**File:** Add to existing `tests-e2e/tests/events.spec.ts`

**Tests:**
1. `should open side panel when clicking event row` — Send event → click row → verify `side-panel` visible
2. `should close side panel` — Open → click `side-panel-close` → verify hidden
3. `should navigate to full page event detail` — Open panel → click `event-panel-full-page` → verify `event-detail-page`

**Commit:** `test(e2e): add side panel event detail tests`

---

### Task 20: Language selector test

**File:** Add to existing `tests-e2e/tests/user-settings.spec.ts`

**Tests:**
1. `should display language selector with English selected` — Login → /settings → verify `language-select` → verify value is "en"

**Commit:** `test(e2e): add language selector test`

---

## Execution Summary

| Phase | Tasks | New test files | Tests added | data-test attrs |
|-------|-------|---------------|-------------|-----------------|
| 1. Frontend attrs | 3 | 0 | 0 | ~42 |
| 2. P0 Critical | 2 | 2 | ~7 | 0 |
| 3. P1 High | 4 | 3 (+1 existing) | ~12 | 0 |
| 4. P2 Medium | 5 | 2 (+3 existing) | ~10 | 0 |
| 5. P3 Low | 6 | 2 (+4 existing) | ~8 | 0 |
| **Total** | **20** | **9 new + 8 existing** | **~37** | **~42** |

Final target: **~143 tests** (106 existing + ~37 new)
