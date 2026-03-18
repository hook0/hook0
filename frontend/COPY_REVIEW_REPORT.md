# Hook0 Frontend — Copy Review Report (Aggregated)

> **Date**: 2026-03-18
> **Method**: 10 independent AI reviewers examined all frontend text simultaneously
> **Scope**: All `.vue`, `.ts` files in `frontend/src/`, locale file `en.json`, Zod schemas
> **Consensus scoring**: Number in parentheses = how many of 10 agents flagged the issue

---

## Executive Summary

The copy quality is **solid overall**. The i18n architecture is well-implemented (nearly all strings go through `vue-i18n`). Tone is professional and developer-appropriate. Auth pages use proven conversion patterns. The main issues fall into 4 categories:

1. **Grammar errors** in high-visibility strings (footer, events page, tutorial)
2. **Terminology inconsistencies** ("Log out"/"Logout", "Request Attempts"/"Request Logs")
3. **i18n gaps** — Zod validation schemas and `http.ts` contain hardcoded English strings
4. **Verbose copy** — several help texts and tutorial descriptions exceed 50 words

---

## P0 — Critical (fix immediately)

### 1. Grammar: "Open-source always win" (10/10 agents)
- **Key**: `footer.openSourceTagline`
- **Current**: `"Open-source always win in the end."`
- **Fix**: `"Open source always wins in the end."`
- Subject-verb disagreement. Visible on every page (footer).

### 2. Grammar: "Events that Hook0 receive" (10/10)
- **Key**: `events.subtitle`
- **Current**: `"Events that Hook0 receive from your application and that Hook0 forwarded to subscriptions (webhooks)."`
- **Fix**: `"Events received from your application and forwarded to subscriptions as webhooks."`
- Subject-verb disagreement + mixed tenses.

### 3. French punctuation in tutorial success (10/10)
- **Key**: `tutorial.step5.eventSentMessage`
- **Current**: `"Wow ! You just sent an event to your webhook !"`
- **Fix**: `"You just sent your first webhook event!"`
- Spaces before `!` are French convention. "Wow" is overly casual for a developer tool.

### 4. "Are you sure to..." pattern — ungrammatical (10/10)
Six confirmation dialogs use "Are you sure to..." instead of "Are you sure you want to...":

| Key | Current | Fix |
|-----|---------|-----|
| `members.revokeConfirm` | "Are you sure to revoke access of {email}..." | "Are you sure you want to revoke {email}'s access to this organization?" |
| `members.roleChangeConfirm` | "Are you sure to change..." | "Are you sure you want to change..." |
| `eventTypes.confirmDeactivate` | "Are you sure to deactivate..." | "Are you sure you want to deactivate..." |
| `userSettings.deleteAccountConfirm` | "Are you sure to delete..." | "Are you sure you want to delete..." |
| `remove.confirmDeleteOrganization` | "Are you sure to delete..." | "Are you sure you want to delete..." |
| `remove.confirmDeleteApplication` | "Are you sure to delete..." | "Are you sure you want to delete..." |
| `remove.confirmDeleteSubscription` | "Are you sure to delete..." | "Are you sure you want to delete..." |

### 5. Zod validation messages bypass i18n (6/10)
Every `.schema.ts` file contains hardcoded English strings. ~18 strings across 7 files:

| File | Hardcoded strings |
|------|------------------|
| `login.schema.ts` | "Please enter a valid email address", "Password is required" |
| `register.schema.ts` | "Please enter a valid email address", "First name is required", "Last name is required", "Password must be at least 8 characters" |
| `organization.schema.ts` | "Organization name is required" |
| `application.schema.ts` | "Application name is required" |
| `subscription.schema.ts` | "Subscription description is required", "HTTP method is required", "URL is required", "A valid URL is required" |
| `eventType.schema.ts` | "Service is required", "Resource type is required", "Verb is required" |
| `passwordChange.schema.ts` | "Password must be at least 8 characters", "Please confirm your password", "Passwords do not match" |

**Recommendation**: Move to `en.json` under `validation.*` namespace.

### 6. Hardcoded error strings in `http.ts` (1/10 — unique find by agent 5)
Exact mapping of hardcoded strings to existing i18n keys:

| Line | Hardcoded | Replace with i18n key |
|------|-----------|----------------------|
| 48 | `'Error'` | `t('common.error')` = "Error" |
| 49 | `'Your session has expired. Please log in again.'` | `t('common.sessionExpiredMessage')` = "Your session has expired. Please log in again." |
| 148 | `'Timeout Exceeded'` | Add `errors.timeoutExceeded`: `"Timeout Exceeded"` |
| 167 | `'Unknown Error'` | `t('errors.unknownError')` needs split → add `errors.unknownErrorTitle`: `"Unknown Error"` |
| 169 | `` `An unknown error occurred: ${err.message}` `` | `t('errors.unknownErrorDetail', { message: err.message })` — add key: `"An unknown error occurred: {message}"` |

**Note**: `http.ts` uses dynamic import for auth store. Same pattern needed for i18n: `import('@/i18n').then(({ i18n }) => i18n.global.t(...))` or import at module level if the i18n instance is available.

---

## P1 — Important (fix in next sprint)

### 7. "Log out" vs "Logout" inconsistency (10/10)

| Key | Current | Fix |
|-----|---------|-----|
| `auth.logout` | "Log out" | keep |
| `nav.logout` | "Log out" | keep |
| `sidebar.logout` | "Logout" | "Log out" |
| `commandPalette.logout` | "Logout" | "Log out" |
| `loginMenu.logout` | "Logout" | "Log out" |

"Log out" (two words) is the verb form. "Logout" is a noun/adjective.

### 8. "Request Attempts" vs "Request Logs" (9/10)

| Key | Current | Fix |
|-----|---------|-----|
| `nav.logs` | "Request Attempts" | "Request Logs" |
| `breadcrumbs.logs` | "Request Logs" | keep |
| `logs.title` | "Request Attempts" | "Request Logs" |
| `logs.subtitle` | "Last webhooks sent by Hook0." | "Recent webhook delivery logs." |

**Decision**: Standardize on **"Request Logs"** everywhere. "Attempts" is internal jargon; "Logs" is what developers expect.

### 9. ~~Duplicate plan labels~~ — RESOLVED
- `planFree` no longer exists in codebase (confirmed via grep). Only `organizations.planDeveloper` and `orgAppSelector.planDeveloper` remain. No action needed.

### 10. Verbose help texts (8/10)

| Key | Word count | Suggested rewrite |
|-----|-----------|-------------------|
| `events.eventLabelsHelp` | ~55 words | "Events match subscriptions whose labels are a subset of the event's labels. Events with extra labels still trigger matching subscriptions." |
| `subscriptions.subscriptionLabelsHelp` | ~50 words | "Only events with at least these labels will trigger this subscription. Events can have additional labels." |
| `tutorial.step3.subtitle` | ~68 words | "Event types categorize your events. Each subscription filters by event type. Hook0 includes the event type as a header in each webhook." |
| `tutorial.step1Description` | ~55 words | "An organization groups your team and billing plan. Create one to get started." |

### 11. Subscription description placeholder reuses app placeholder (7/10)
- **Key**: `subscriptions.descriptionPlaceholder`
- **Current**: `"my awesome api - production"` (same as application name placeholder)
- **Fix**: `"e.g., Production webhook for order notifications"`

### 12. Application name help text is awkward (7/10)
- **Key**: `applications.nameHelpText`
- **Current**: `"Name of your company's product or API. Don't forget also to specify the environment, for example: \"facebook-production\""`
- **Fix**: `"Your product or API name. Include the environment, e.g. 'acme-api-production'."`

### 13. "Coming soon" copy undermines confidence (8/10)
- **Key**: `comingSoon.description`
- **Current**: `"Hook0 API is complete and production ready but the UI is still a work in progress..."`
- **Fix**: `"This feature is coming soon. The API already supports it — check the docs to get started."`

### 14. API key creation toast shows button label (4/10)
- **File**: `ApplicationSecretsList.vue` line 81
- Success toast uses `t('apiKeys.create')` = "Create API Key" (a button label, not a confirmation)
- **Fix**: Add `apiKeys.created`: `"API key created"`

### 17. Hardcoded example lists in EventTypesNew.vue (6/10)
- Lines 194-236: example service names, resource types, and verbs are hardcoded in the template as `<li>` elements
- These are domain terms (`billing`, `chat`, `project`, `created`, `updated`...) unlikely to be translated
- **Fix**: Extract to a `const` array in `<script setup>` and `v-for` over it. This respects the CLAUDE.md rule "NEVER write hardcoded lists" while keeping them out of i18n (they are technical identifiers, not prose).

### 18. Typo "linkedin" in console.error (8/10)
- **File**: `OrganizationAndApplicationSelector.vue` lines 73, 84
- **Current**: `"application is linkedin to unknown organization"`
- **Fix**: `"application is linked to unknown organization"`

### 19. Grammar: "be able choose" in tutorial (2/10)
- **Key**: `tutorial.step3.subtitle`
- **Current**: `"...you will then be able choose among..."`
- **Fix**: `"...you will then be able to choose among..."`

### 20. "Edit" used as submit button text on update forms (3/10)
- 3 forms affected: `ApplicationsEdit.vue:194`, `OrganizationsEdit.vue:189`, `SubscriptionsEdit.vue:599`
- All use `isNew ? t('common.create') : t('common.edit')` — "Edit" is shown on submit for existing entities
- `common.save` already exists in `en.json` (line 8) = `"Save"`
- **Fix**: Replace `t('common.edit')` with `t('common.save')` in the ternary of these 3 files.

---

## P2 — Nice-to-have (backlog)

### Terminology & Consistency

| # | Issue | Agents |
|---|-------|--------|
| 21 | `applications.namePlaceholder`: `"my awesome api - production"` (lowercase) → `"My Product API - Production"` | 5/10 |
| 22 | `common.getStarted`: `"Get started by creating your first item."` — used as fallback in `Hook0EmptyState.vue:19`. → `"No data yet. Create your first entry to get started."` | 6/10 |
| 23 | `error404.description`: `"The endpoint you're looking for swam away..."` → `"The page you're looking for doesn't exist or has been moved."` (reuse `common.pageNotFoundDescription` wording) | 7/10 |
| 24 | `common.consumption`: `"Here is the consumption of your {entityType}."` — "Here is" is filler → `"Usage for your {entityType}"` | 3/10 |
| 25 | `eventTypes.helpDescription`: `"An event is something that has happened in your application. In the past."` — fragment "In the past." adds nothing | 2/10 |

### Duplicate/Redundant Keys

| # | Issue | Agents |
|---|-------|--------|
| 26 | `showPassword`/`hidePassword` defined in `common.*`, `auth.login.*`, `auth.register.*`, `auth.resetPassword.*` — 4 copies of 2 strings. **Fix**: Delete `auth.login.showPassword`, `auth.login.hidePassword`, `auth.register.showPassword`, `auth.register.hidePassword`, `auth.resetPassword.showPassword`, `auth.resetPassword.hidePassword`. Keep only `common.showPassword` / `common.hidePassword`. Verify `Hook0Input` uses the `common.*` keys (it does). | 5/10 |
| 27 | `common.pageNotFound` + `error404.title` — two parallel 404 text sets. **Fix**: Keep both — `common.*` is for generic inline 404 states, `error404.*` is for the dedicated page with personality. No duplication issue. | 3/10 |
| 28 | Delete confirmations duplicated between domain namespaces (`organizations.deleteConfirm`) and `remove.*` namespace. **Fix**: Audit which set is actually rendered. Keep the one used in components, delete the other. | 4/10 |

### Missing Microcopy

| # | Gap | Agents |
|---|-----|--------|
| 33 | `UserSettings.vue` delete button uses `t('common.delete')` = "Delete". → Use a dedicated key `userSettings.deleteMyAccount`: `"Delete my account"` for this destructive action | 3/10 |
| 35 | `logs.subtitleRetention`: `"Items older than 7 days are not shown."` — "7 days" should be parameterized → `"Items older than {days} days are not shown."` with `{ days: retentionDays }` from org plan | 2/10 |

### Button Casing

| # | Issue | Agents |
|---|-------|--------|
| 36 | Mixed Title Case and sentence case in CTAs. Fix: standardize to **sentence case** everywhere. Affected keys: `tutorial.step5.backToApplication` → `"Back to application"`, `tutorial.congrats.goToDashboard` → `"Go to your application dashboard"`, `error404.goBack` → `"Go back"` | 4/10 |

### Tone

| # | Issue | Agents |
|---|-------|--------|
| 37 | `events.sendTestEventSubtitle`: `"For sending a test event, you need to create an event type first. After that you can create a subscription for this event type. Finally you can send a test event."` → `"To send a test event: first create an event type, then a subscription, then send the event."` | 5/10 |
| 38 | `serviceTokens.createPrompt`: `"Create a new secret key, name?"` → `"Name your new service token"` | 3/10 |
| 39 | `members.invite`: `"Invite a user"` → `"Invite member"` | 1/10 |

---

## Systemic Recommendations

1. **Create a `validation.*` namespace in `en.json`** for all Zod schema messages — single highest-impact change for i18n completeness
2. **Establish a copy style guide** defining: sentence case for buttons, "Log out" (not "Logout"), max 25 words for wizard step descriptions
3. **Audit for subject-verb agreement** across all strings using "Hook0" as subject — currently treated as both singular and plural
4. **Consolidate duplicate keys** — `showPassword`/`hidePassword`, delete confirmations, 404 text
5. **Replace hardcoded strings in `http.ts`** with existing i18n keys that already exist but are unused

---

## Files Requiring Changes

| File | Change type |
|------|------------|
| `frontend/src/locales/en.json` | Grammar fixes, consistency fixes, trim verbose text, add validation keys |
| `frontend/src/http.ts` | Replace hardcoded error strings with i18n |
| `frontend/src/pages/login.schema.ts` | Move validation messages to i18n |
| `frontend/src/pages/register.schema.ts` | Same |
| `frontend/src/pages/organizations/organization.schema.ts` | Same |
| `frontend/src/pages/organizations/applications/application.schema.ts` | Same |
| `frontend/src/pages/organizations/applications/subscriptions/subscription.schema.ts` | Same |
| `frontend/src/pages/organizations/applications/event_types/eventType.schema.ts` | Same |
| `frontend/src/pages/user/passwordChange.schema.ts` | Same |
| `frontend/src/pages/OrganizationAndApplicationSelector.vue` | Fix "linkedin" typo |
| `frontend/src/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue` | Fix success toast key |
| `frontend/src/pages/organizations/applications/event_types/EventTypesNew.vue` | Extract hardcoded examples |
