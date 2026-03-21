# Hook0 Frontend Complete Redesign - Specification

## 1. Context & Motivation

Hook0 is a webhook management platform (Webhook-as-a-Service). The frontend is a single-page application (SPA) dashboard that allows users to manage organizations, applications, event types, webhook subscriptions, inspect events, view delivery logs, and manage service tokens.

The current frontend was built with Vue 3, Tailwind CSS 3, AG-Grid, and a fully custom component library (35+ `Hook0*` components). It has near-zero test coverage, no centralized state management, no i18n, no dark mode, and manual loading/error handling in every component.

This specification defines a **complete redesign** of the frontend: new design system, modern libraries, improved UX, enhanced DX, and state-of-the-art architecture -- while keeping the existing backend API and route structure unchanged.

---

## 2. Goals

1. **Modern, clean SaaS aesthetic** -- Stripe/Clerk-inspired, light-first with full dark mode, whitespace-rich, professional
2. **State-of-the-art DX** -- TanStack Query, Pinia, VeeValidate+Zod, TypeScript strict, ESLint-enforced conventions
3. **Best-in-class UX** -- skeleton loading everywhere, contextual errors, cmd+k command palette, keyboard shortcuts, responsive (desktop + tablet + mobile)
4. **Accessibility** -- WCAG 2.1 AA compliance
5. **Performance** -- aggressive code splitting, <150KB main bundle gzip, lazy-loaded heavy deps
6. **i18n-ready** -- vue-i18n from day 1, English only initially, architecture supports adding locales
7. **RBAC-ready** -- `usePermissions()` composable (returns `true` for everything now, wired to backend later)
8. **Zero backend changes** -- pure frontend redesign, same API, same auth flow

---

## 3. Target Stack & Dependencies

### Core Framework

| Package | Version | Purpose |
|---|---|---|
| `vue` | `^3.5.26` | UI framework |
| `vue-router` | `^4.6.4` | Client-side routing |
| `vite` | `^7.3.1` | Build tool |
| `typescript` | `^5.6.0` | Type safety |
| `tailwindcss` | `^4.1.18` | Utility-first CSS (v4, CSS-first config) |

### State & Data

| Package | Version | Purpose |
|---|---|---|
| `@tanstack/vue-query` | `^5.92.6` | Server state management (caching, refetch, optimistic) |
| `pinia` | `^3.0.4` | Client state only (auth, UI prefs, org/app context) |
| `axios` | `^1.2.2` | HTTP client (keep existing, wrapped by TanStack Query) |

### Forms & Validation

| Package | Version | Purpose |
|---|---|---|
| `vee-validate` | `^4.15.1` | Form state management |
| `zod` | `^4.3.5` | Schema validation (type-safe) |

### Tables

| Package | Version | Purpose |
|---|---|---|
| `@tanstack/vue-table` | `^8.21.3` | Headless table engine (replaces AG-Grid) |

**Hard requirement**: TanStack Table must provide exact feature parity with current AG-Grid usage: sorting, filtering, pagination, custom cell renderers, row selection, resizable columns.

### i18n

| Package | Version | Purpose |
|---|---|---|
| `vue-i18n` | `^11.2.8` | Internationalization (English only at launch) |

### Icons

| Package | Version | Purpose |
|---|---|---|
| `lucide-vue-next` | `^0.562.0` | Icon library (replaces FontAwesome) |

### Code Editor

| Package | Version | Purpose |
|---|---|---|
| `codemirror` | `^6.0.1` | Code editor core |
| `@codemirror/lang-json` | `^6.0.1` | JSON language support |
| `@codemirror/theme-one-dark` | `^6.1.0` | Dark theme |
| `vue-codemirror` | `^6.1.1` | Vue wrapper |

Enhanced features: JSON validation, line numbers, search, copy button, collapsible sections.

### Charts (deferred but dependency pinned)

| Package | Version | Purpose |
|---|---|---|
| `echarts` | `^6.0.0` | Charting library |
| `vue-echarts` | `^8.0.1` | Vue wrapper |

### Animations

| Package | Version | Purpose |
|---|---|---|
| `@vueuse/motion` | `^3.0.3` | Spring physics, element animations |

Page transitions use Vue's built-in `<Transition>` + CSS. `@vueuse/motion` for interactive elements (hover, layout shifts, spring physics).

### Utilities

| Package | Version | Purpose |
|---|---|---|
| `@vueuse/core` | `^14.1.0` | Vue composition utilities |
| `date-fns` | `^4.1.0` | Date formatting |
| `notivue` | `^2.4.5` | Toast notifications |

### Auth & Security

| Package | Version | Purpose |
|---|---|---|
| `@biscuit-auth/biscuit-wasm` | `^0.6.0` | Biscuit token attenuation |
| `vue-turnstile` | `^1.0.11` | Cloudflare CAPTCHA |

### Analytics & Integrations

| Package | Version | Purpose |
|---|---|---|
| `vue-matomo` | `^4.2.0` | Matomo analytics |
| `@formbricks/js` | `^4.2.1` | User surveys |
| `crisp-sdk-web` | `^1.0.21` | Chat support |

### Dev Dependencies

| Package | Version | Purpose |
|---|---|---|
| `@vitejs/plugin-vue` | `^5.0.2` | Vite Vue SFC support |
| `vite-plugin-wasm` | `^3.3.0` | WASM support (Biscuit) |
| `vite-plugin-top-level-await` | `^1.4.1` | Top-level await |
| `vite-plugin-checker` | `^0.9.1` | ESLint + TypeScript checking |
| `eslint` | `^8.4.1` | Linting |
| `prettier` | `^3.3.2` | Code formatting |
| `sass` | `^1.32.6` | SCSS support |
| `openapi-typescript` | `^7.0.0` | OpenAPI type generation |
| `jest` | `^30.1.3` | Unit tests (isolated logic only) |
| `ts-jest` | `^29.1.1` | TypeScript Jest support |
| `vue-tsc` | `^2.2.8` | Vue TypeScript checking |

### Removed Dependencies

| Package | Reason |
|---|---|
| `ag-grid-community` + `ag-grid-vue3` | Replaced by `@tanstack/vue-table` (-300KB) |
| `swagger-ui` | Replaced by external link to `https://documentation.hook0.com/api` (-1MB) |
| `@fortawesome/*` (3 packages) | Replaced by `lucide-vue-next` |
| `fp-ts` | Unnecessary FP abstraction, use native TS |
| `ramda` | Same as above |
| `vue-promised` | Replaced by TanStack Query |
| `vue-click-outside-element` | Replace with `@vueuse/core` `onClickOutside` |
| `party-js` | Confetti library, not needed in clean redesign |
| `lodash.debounce` | Replace with `@vueuse/core` `useDebounceFn` |
| `uuid` | Replace with `crypto.randomUUID()` (native) |

---

## 4. Design System

### 4.1 Visual Identity

- **Aesthetic**: Stripe/Clerk-inspired clean minimal SaaS
- **Mode**: Light-first with full dark mode (CSS custom properties on `:root`)
- **Typography**: Inter (variable font) for UI, JetBrains Mono for code
- **Icons**: Lucide icons (consistent stroke weight, clean lines)
- **Spacing**: 4px base grid (Tailwind v4 default scale)
- **Border radius**: `rounded-lg` (8px) for cards/panels, `rounded-md` (6px) for inputs/buttons
- **Shadows**: Subtle elevation system (sm/md/lg), no harsh drop shadows

### 4.2 Color System (CSS Custom Properties)

```css
:root {
  /* Neutral */
  --color-bg-primary: #ffffff;
  --color-bg-secondary: #f9fafb;
  --color-bg-tertiary: #f3f4f6;
  --color-bg-elevated: #ffffff;
  --color-border: #e5e7eb;
  --color-border-strong: #d1d5db;
  --color-text-primary: #111827;
  --color-text-secondary: #6b7280;
  --color-text-tertiary: #9ca3af;

  /* Brand / Primary */
  --color-primary: #4f46e5;       /* Indigo-600 */
  --color-primary-hover: #4338ca; /* Indigo-700 */
  --color-primary-light: #eef2ff; /* Indigo-50 */

  /* Semantic */
  --color-success: #059669;
  --color-warning: #d97706;
  --color-error: #dc2626;
  --color-info: #2563eb;
}

:root.dark {
  --color-bg-primary: #0f0f13;
  --color-bg-secondary: #18181f;
  --color-bg-tertiary: #1e1e28;
  --color-bg-elevated: #242432;
  --color-border: #2e2e3a;
  --color-border-strong: #3e3e4e;
  --color-text-primary: #f9fafb;
  --color-text-secondary: #9ca3af;
  --color-text-tertiary: #6b7280;
}
```

### 4.3 Dark Mode Implementation

- Toggle stored in `localStorage` via Pinia UI store
- Respects `prefers-color-scheme` on first visit
- Manual override persisted
- `.dark` class on `<html>` element
- All components use CSS custom properties (not hardcoded colors)
- CodeMirror switches between default theme and One Dark

### 4.4 Typography Scale

```
text-xs:   12px / 16px  -- Labels, metadata
text-sm:   14px / 20px  -- Body text, table cells
text-base: 16px / 24px  -- Primary body
text-lg:   18px / 28px  -- Section headers
text-xl:   20px / 28px  -- Page headers
text-2xl:  24px / 32px  -- Dashboard titles
```

Font stack:
```css
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
--font-mono: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
```

---

## 5. Component Library

All components keep the `Hook0*` prefix. Built from scratch (no headless library), but must be WCAG 2.1 AA compliant with proper aria attributes, keyboard navigation, and focus management.

### 5.1 Core Components

| Component | Description | Accessibility |
|---|---|---|
| `Hook0Button` | Primary/secondary/danger/ghost variants, loading state, icon support | `aria-disabled`, `aria-busy`, keyboard focusable |
| `Hook0Input` | Text input with label, error state, help text | `aria-describedby` for errors, `aria-invalid` |
| `Hook0Select` | Custom dropdown select | `role="listbox"`, arrow key navigation, type-ahead |
| `Hook0Textarea` | Multiline input | Same as Input |
| `Hook0Checkbox` | Checkbox with label | `role="checkbox"`, Space to toggle |
| `Hook0Switch` | Toggle switch | `role="switch"`, `aria-checked` |
| `Hook0Dialog` | Modal dialog with focus trap | `role="dialog"`, `aria-modal`, Escape to close, focus trap |
| `Hook0Dropdown` | Dropdown menu | `role="menu"`, `aria-expanded`, arrow key navigation |
| `Hook0Popover` | Popover with arrow | Focus management, Escape to close |
| `Hook0Tooltip` | Tooltip on hover/focus | `role="tooltip"`, `aria-describedby` |
| `Hook0Tabs` | Tabbed interface | `role="tablist"`, arrow key navigation |
| `Hook0Badge` | Status badge | Semantic colors for status |
| `Hook0Avatar` | User/org avatar | `alt` text, fallback initials |

### 5.2 Data Display Components

| Component | Description |
|---|---|
| `Hook0Table` | TanStack Table wrapper with sorting, filtering, pagination, virtual scroll, resizable columns |
| `Hook0TableSkeleton` | Matching skeleton for table loading state |
| `Hook0Card` | Content container with header/content/footer slots |
| `Hook0CardSkeleton` | Matching skeleton for card loading state |
| `Hook0KeyValue` | Key-value pair display (label + value) |
| `Hook0Code` | Enhanced CodeMirror wrapper (JSON validation, copy, search, line numbers) |
| `Hook0DateTime` | Formatted date/time display with relative time tooltip |
| `Hook0EmptyState` | Illustrated empty state with CTA + optional code example |
| `Hook0ErrorCard` | Error state with retry button (for partial failures) |
| `Hook0SidePanel` | Slide-out right panel for detail views |

### 5.3 Layout Components

| Component | Description |
|---|---|
| `Hook0Sidebar` | Persistent left navigation with org/app dropdown |
| `Hook0Header` | Top bar with breadcrumbs, search trigger, user menu |
| `Hook0Breadcrumb` | Route-based breadcrumb trail |
| `Hook0CommandPalette` | cmd+k search overlay (navigation + actions) |
| `Hook0MobileTabBar` | Bottom tab bar for mobile |
| `Hook0MobileDrawer` | Full-height slide-out nav for mobile |
| `Hook0PageLayout` | Standard page wrapper (title, actions, content area) |

### 5.4 Feedback Components

| Component | Description |
|---|---|
| `Hook0Alert` | Inline alert (info/warning/error/success) -- persistent within page |
| `Hook0Toast` | Transient notification (via Notivue) |
| `Hook0Skeleton` | Generic skeleton loader (text, circle, rect variants) |
| `Hook0ProgressBar` | Determinate/indeterminate progress |
| `Hook0Spinner` | Loading spinner for buttons and small areas |

### 5.5 Form Components

| Component | Description |
|---|---|
| `Hook0Form` | VeeValidate form wrapper with Zod schema |
| `Hook0FormField` | Field wrapper with label, error display, help text |
| `Hook0FormActions` | Form footer with submit/cancel buttons |

---

## 6. Navigation & Information Architecture

### 6.1 Persistent Sidebar

The sidebar is always visible on desktop (240px width). It contains:

1. **Hook0 Logo** (top)
2. **Org/App Switcher** -- dropdown at top showing current org name + current app name. Click to switch via searchable dropdown.
3. **Navigation sections**:
   - If an app is selected:
     - Dashboard
     - Events
     - Event Types
     - Subscriptions
     - Logs
     - API Keys
     - Settings
   - Organization section:
     - Applications
     - Service Tokens
     - Organization Settings
   - Global:
     - API Documentation (external link to `https://documentation.hook0.com/api`)
4. **User menu** (bottom) -- avatar, name, settings, logout
5. **Dark mode toggle** (bottom)

### 6.2 Header Bar

Top bar (56px height) containing:
- **Breadcrumbs** (left) -- auto-generated from route, clickable segments
- **Command palette trigger** (center) -- search icon + "Search or jump to..." + `cmd+K` shortcut hint
- **User avatar + dropdown** (right) -- settings, logout

### 6.3 Command Palette (cmd+K)

Scope: **Navigation + Actions**

- Search pages by name (fuzzy match)
- Switch organizations
- Switch applications
- Quick actions: "Create subscription", "Create event type", "Send test event", "Copy API key"
- Recent pages (last 5 visited)
- Keyboard navigable (arrow keys, Enter to select, Escape to close)

### 6.4 Mobile Navigation

- **Bottom tab bar** (4 tabs): Dashboard, Events, Subscriptions, More
- **"More" tab** opens a full-height drawer with complete navigation
- Sidebar hidden on screens < 768px
- All features accessible on all screen sizes

### 6.5 Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `cmd+K` / `ctrl+K` | Open command palette |
| `Escape` | Close modals, panels, command palette |
| `?` | Show keyboard shortcuts cheat sheet |
| Arrow keys | Navigate lists and tables |
| `Enter` | Confirm selection / submit form |
| `Tab` / `Shift+Tab` | Focus navigation |

---

## 7. Screen-by-Screen Specification

### 7.1 Authentication Screens (fullscreen, no sidebar)

#### Login (`/login`)
- Hook0 logo centered
- Email + password fields
- "Forgot password?" link
- "Sign up" link
- Cloudflare Turnstile (if configured)
- Clean, centered card layout with subtle background

#### Register (`/register`)
- Same layout as login
- Name, email, password fields
- Cloudflare Turnstile
- Link to login

#### Forgot Password (`/begin-reset-password`)
- Email field only
- Back to login link

#### Reset Password (`/reset-password`)
- New password + confirm fields
- Token from URL query param

#### Verify Email (`/verify-email`)
- Status message (verifying... / verified / error)
- Auto-redirect to dashboard on success

#### Check Email (`/check-email`)
- Informational page: "Check your inbox"

### 7.2 Tutorial / Onboarding (improved wizard)

**Current**: 7 separate fullscreen pages.
**Redesign**: Modal/overlay wizard within the main UI layout.

- Step indicator at top (1/7 progress)
- Each step is a focused form within a centered modal
- Steps:
  1. Welcome / introduction
  2. Create organization (name)
  3. Create application (name)
  4. Create event type (name, schema)
  5. Create subscription (URL, event type)
  6. Send test event (pre-filled payload)
  7. Success (confetti optional, CTA to dashboard)
- Can be dismissed and resumed later
- Progress stored in Pinia (not URL)
- **Routes preserved**: all `/tutorial/*` routes still work, but they render the wizard overlay on top of the main layout

### 7.3 Home / Dashboard (`/`)

- If no org/app selected: Org/App selector (cards grid)
- If org+app selected: redirect to app dashboard

### 7.4 Organization Dashboard (`/organizations/:org_id/dashboard`)

- Organization name + settings link
- Applications list (card grid, each showing name + event count)
- Quick stats: total events, total subscriptions, total apps
- Empty state if no apps: "Create your first application" with illustration + CTA

### 7.5 Application Dashboard (`/organizations/:org_id/applications/:app_id/dashboard`)

- App name + settings link
- Stats cards: Events (24h), Active Subscriptions, Delivery Success Rate
- Recent events table (last 10, click to expand side panel)
- Quick actions: "Create Subscription", "View Logs"
- Empty states with code examples when relevant

### 7.6 Events List (`/organizations/:org_id/applications/:app_id/events`)

- TanStack Table with columns: Event ID, Event Type, Occurred At, Payload preview
- Filters: event type, date range
- Click row: opens **side panel** with event detail
- Side panel shows: full JSON payload (CodeMirror read-only), metadata, delivery attempts summary
- "Open full page" button in side panel header

### 7.7 Event Detail (`/organizations/:org_id/applications/:app_id/events/:event_id`)

- Full-page view
- Event metadata (ID, type, timestamp)
- JSON payload (CodeMirror, read-only, copy button)
- Delivery timeline: for each subscription that received this event:
  - Attempt 1: timestamp, status code, latency, response preview
  - Attempt 2: (if retried) same
  - Visual timeline with success/failure indicators
- Back button returns to events list

### 7.8 Event Types (`/organizations/:org_id/applications/:app_id/event_types`)

- TanStack Table: Type name, Created At, Event count
- "New Event Type" button
- Delete action (with confirmation dialog)
- Empty state: "Define your first event type" + code example showing API call

### 7.9 New Event Type (`/organizations/:org_id/applications/:app_id/event_types/new`)

- VeeValidate form with Zod schema
- Fields: name (dot-notation convention hint), description (optional), JSON schema (CodeMirror editor)
- Inline validation
- Submit creates and redirects to list

### 7.10 Subscriptions List (`/organizations/:org_id/applications/:app_id/subscriptions`)

- TanStack Table: Label, URL, Event Types, Status (active/paused), Created At
- Status badge (green/yellow/red)
- Actions: Edit, Delete, Test
- "New Subscription" button
- Empty state: "Create your first webhook subscription" + illustration

### 7.11 Subscription Edit (`/organizations/:org_id/applications/:app_id/subscriptions/:subscription_id` and `/new`)

- VeeValidate form:
  - Label
  - Target URL
  - Event types filter (multi-select)
  - HTTP headers (key-value editor)
  - Secret (for signature verification)
- **Test endpoint button**: sends test payload to URL, shows response inline (status, latency, body preview)
- Create/Update button with loading state

### 7.12 Logs (`/organizations/:org_id/applications/:app_id/logs`)

**Structured log viewer**:
- Filter bar: status code range, subscription, time range, search
- TanStack Table: Timestamp, Subscription URL, Event Type, Status Code, Latency
- Status code badges (2xx green, 4xx yellow, 5xx red)
- Expandable rows: full request/response (headers + body, CodeMirror syntax highlighted)
- Search within payloads

### 7.13 API Keys (`/organizations/:org_id/applications/:app_id/application_secrets`)

- Table: Key (masked), Created At, Last Used
- "Create API Key" button
- Copy key action (shown only once on creation)
- Delete with confirmation

### 7.14 Service Tokens (`/organizations/:org_id/services_tokens`)

- Table: Token name, Scope, Expires At, Created At
- "Create Token" button

### 7.15 Service Token Creation / View (`/organizations/:org_id/services_tokens/:service_token_id`)

**Progressive disclosure with two modes**:

**Simple mode** (default):
- Select application scope (dropdown)
- Set expiry (preset options: 7d, 30d, 90d, 1y, custom)
- Generate token

**Advanced mode** (toggle):
- Granular permission selection
- Custom Biscuit claims
- Time-bound with custom datetime picker
- Token preview (decoded view + raw)

Both modes: one-click copy, token displayed only once.

### 7.16 Organization Settings (`/organizations/:org_id/settings`)

- VeeValidate form: Organization name
- Members list (table with role display)
- Danger zone: Delete organization (confirmation)

### 7.17 Application Settings (`/organizations/:org_id/applications/:app_id/settings`)

- VeeValidate form: Application name
- Danger zone: Delete application (confirmation)

### 7.18 User Settings (`/settings`)

- Profile: name, email (read-only or editable depending on backend)
- Password change
- Dark mode preference
- Language preference (English only initially, dropdown ready for more)

### 7.19 API Documentation

- `/api/documentation` route: redirect to `https://documentation.hook0.com/api` (external)
- `/organizations/:org_id/applications/:app_id/documentation`: same redirect
- Preserves backward compatibility for bookmarked URLs

### 7.20 Error 404

- Fullscreen, clean design
- "Page not found" message
- "Go to dashboard" CTA
- Hook0 logo

---

## 8. Architecture & File Structure

### 8.1 Directory Structure

```
src/
  main.ts                          # App entry, plugin registration
  App.vue                          # Root component (Suspense boundary, error boundary)
  router.ts                        # Router instance
  routes.ts                        # Route definitions (preserved)

  assets/
    fonts/                         # Inter, JetBrains Mono (self-hosted)
    styles/
      tailwind.css                 # Tailwind v4 entry (CSS-first config)
      transitions.css              # Page & element transition definitions

  components/                      # Shared UI components (Hook0* prefixed)
    Hook0Button.vue
    Hook0Input.vue
    Hook0Select.vue
    Hook0Dialog.vue
    Hook0Table.vue
    Hook0TableSkeleton.vue
    Hook0Card.vue
    Hook0CardSkeleton.vue
    Hook0Code.vue
    Hook0CommandPalette.vue
    Hook0Sidebar.vue
    Hook0Header.vue
    Hook0Breadcrumb.vue
    Hook0SidePanel.vue
    Hook0EmptyState.vue
    Hook0ErrorCard.vue
    Hook0Alert.vue
    Hook0Skeleton.vue
    Hook0Form.vue
    Hook0FormField.vue
    Hook0MobileTabBar.vue
    Hook0MobileDrawer.vue
    Hook0PageLayout.vue
    Hook0Badge.vue
    Hook0Tooltip.vue
    Hook0Tabs.vue
    Hook0Avatar.vue
    Hook0Switch.vue
    Hook0KeyValue.vue
    Hook0DateTime.vue
    Hook0Logo.vue
    Hook0Footer.vue
    Hook0LoginMenu.vue
    Hook0ProgressBar.vue

  composables/                     # Shared composables
    usePermissions.ts              # RBAC composable (returns true for now)
    useTracking.ts                 # Matomo analytics (improved)
    useColorMode.ts                # Dark mode toggle
    useCommandPalette.ts           # cmd+k state & actions
    useKeyboardShortcuts.ts        # Global keyboard shortcuts
    usePageQuery.ts                # Standardized page data fetching
    usePageMutation.ts             # Standardized mutations
    usePageForm.ts                 # Standardized form handling

  stores/                          # Pinia stores (client state ONLY)
    auth.ts                        # Authentication (tokens, user, refresh)
    ui.ts                          # UI preferences (dark mode, sidebar collapsed)
    context.ts                     # Current org/app selection

  features/                        # Feature modules (colocated)
    auth/
      pages/
        LoginPage.vue
        RegisterPage.vue
        CheckEmailPage.vue
        BeginResetPassword.vue
        ResetPassword.vue
        VerifyEmail.vue

    tutorial/
      pages/
        TutorialWizard.vue         # Single wizard component for all steps
      components/
        TutorialStep.vue
        TutorialProgress.vue

    user/
      pages/
        UserSettings.vue
      services/
        UserService.ts
      queries/
        useUserQueries.ts

    organizations/
      pages/
        OrganizationsDashboard.vue
        OrganizationsEdit.vue
      services/
        OrganizationService.ts
      queries/
        useOrganizationQueries.ts
      schemas/
        organizationSchema.ts      # Zod schemas
      components/
        MembersList.vue
        OrgSwitcher.vue

    applications/
      pages/
        ApplicationsList.vue
        ApplicationsDashboard.vue
        ApplicationsEdit.vue
      services/
        ApplicationService.ts
      queries/
        useApplicationQueries.ts
      schemas/
        applicationSchema.ts
      components/
        AppSwitcher.vue

    event-types/
      pages/
        EventTypesList.vue
        EventTypesNew.vue
      services/
        EventTypeService.ts
      queries/
        useEventTypeQueries.ts
      schemas/
        eventTypeSchema.ts

    events/
      pages/
        EventsList.vue
        EventsDetail.vue
      services/
        EventsService.ts
      queries/
        useEventQueries.ts
      components/
        EventSidePanel.vue
        EventTimeline.vue

    subscriptions/
      pages/
        SubscriptionsList.vue
        SubscriptionsEdit.vue
      services/
        SubscriptionService.ts
      queries/
        useSubscriptionQueries.ts
      schemas/
        subscriptionSchema.ts
      components/
        TestEndpointButton.vue

    logs/
      pages/
        LogList.vue
      services/
        LogService.ts
      queries/
        useLogQueries.ts
      components/
        LogFilterBar.vue
        LogExpandedRow.vue

    service-tokens/
      pages/
        ServicesTokenList.vue
        ServiceTokenView.vue
      services/
        ServicesTokenService.ts
      queries/
        useServiceTokenQueries.ts
      schemas/
        serviceTokenSchema.ts
      components/
        TokenSimpleMode.vue
        TokenAdvancedMode.vue

  plugins/
    i18n.ts                        # vue-i18n setup
    query.ts                       # TanStack Query client config
    matomo.ts                      # Matomo plugin
    formbricks.ts                  # Formbricks plugin

  locales/
    en.json                        # English strings (all UI text)

  utils/
    biscuit_auth.ts                # Biscuit token attenuation
    problemFactory.ts              # API error handling (RFC 7807)
    instance-config.ts             # Instance configuration

  types.ts                         # OpenAPI-generated types (keep)
  http.ts                          # Axios HTTP client (keep, wrapped by services)
```

### 8.2 State Management Rules

**Pinia** (3 stores maximum):
- `stores/auth.ts` -- tokens, user info, login/logout, token refresh
- `stores/ui.ts` -- dark mode, sidebar collapsed, recent pages
- `stores/context.ts` -- selected org ID, selected app ID

**TanStack Query** for everything else:
- All API data fetched via `useQuery` / `useMutation`
- Queries defined in feature `queries/` files
- Query keys follow factory pattern per domain
- Existing `*Service.ts` files kept as the fetch layer, wrapped by query composables

**Rule**: Pinia stores NEVER cache API responses. If data comes from the API, it lives in TanStack Query cache.

### 8.3 Query Pattern (Service Layer Adapters)

```typescript
// features/subscriptions/services/SubscriptionService.ts
// (existing service, mostly unchanged)
export const SubscriptionService = {
  list(orgId: string, appId: string): Promise<Subscription[]> { ... },
  get(orgId: string, appId: string, id: string): Promise<Subscription> { ... },
  create(orgId: string, appId: string, data: CreateSubscription): Promise<Subscription> { ... },
  // ...
}

// features/subscriptions/queries/useSubscriptionQueries.ts
export const subscriptionKeys = {
  all: ['subscriptions'] as const,
  lists: () => [...subscriptionKeys.all, 'list'] as const,
  list: (orgId: string, appId: string) => [...subscriptionKeys.lists(), orgId, appId] as const,
  details: () => [...subscriptionKeys.all, 'detail'] as const,
  detail: (orgId: string, appId: string, id: string) => [...subscriptionKeys.details(), orgId, appId, id] as const,
}

export function useSubscriptionList(orgId: string, appId: string) {
  return useQuery({
    queryKey: subscriptionKeys.list(orgId, appId),
    queryFn: () => SubscriptionService.list(orgId, appId),
  })
}
```

### 8.4 Page Component Convention (enforced via ESLint)

Every page component follows this structure:

```vue
<script setup lang="ts">
// 1. Imports
// 2. Route params
// 3. Permissions check
const { canEdit } = usePermissions()

// 4. Data queries (via feature composables)
const { data, isLoading, isError, refetch } = useSubscriptionList(orgId, appId)

// 5. Local state & computed
// 6. Event handlers
</script>

<template>
  <Hook0PageLayout title="..." :actions="...">
    <!-- Skeleton while loading -->
    <Hook0TableSkeleton v-if="isLoading" />

    <!-- Error with retry -->
    <Hook0ErrorCard v-else-if="isError" @retry="refetch" />

    <!-- Content -->
    <template v-else>
      <!-- ... -->
    </template>
  </Hook0PageLayout>
</template>
```

Custom ESLint rules enforce:
- Pages use `usePageQuery` or TanStack Query hooks (not raw Axios)
- Pages have skeleton and error states
- Pages use `Hook0PageLayout` wrapper

### 8.5 Error Handling Strategy

**Error Boundaries** (safety net):
- Vue `errorCaptured` hook at layout level catches unexpected render errors
- Displays "Something went wrong" with retry

**Contextual Inline Errors** (UX):
- Each TanStack Query's `isError` state renders a `Hook0ErrorCard` in place of the expected content
- Retry button triggers `refetch()`
- Other panels on the same page continue working

**Mutation Errors**:
- Validation errors (422): mapped to form field errors (inline under the field)
- Server errors (500): toast notification with error message
- Network errors: toast with "Connection error" + retry suggestion

**Session Expiry**:
- TanStack Query handles token refresh transparently via auth interceptor
- If refresh fails: modal "Session expired, please log in again"
- Redirect to login with return URL (user returns to same page after re-login)
- Never lose user context silently

### 8.6 Permission System (RBAC-ready)

```typescript
// composables/usePermissions.ts
export function usePermissions() {
  // For now, always returns true.
  // When backend RBAC ships, this will call an API endpoint.
  return {
    canView: (_resource: string) => true,
    canCreate: (_resource: string) => true,
    canEdit: (_resource: string) => true,
    canDelete: (_resource: string) => true,
  }
}
```

Components use these to conditionally render actions:
```vue
<Hook0Button v-if="canDelete('subscription')" variant="danger" @click="handleDelete">
  Delete
</Hook0Button>
```

---

## 9. Loading States

**Skeleton screens everywhere**. Every data-dependent component has a matching skeleton variant.

- `Hook0TableSkeleton` -- animated rows matching table column layout
- `Hook0CardSkeleton` -- animated card placeholder
- `Hook0Skeleton` -- generic (text lines, circles, rects)
- TanStack Query's `isLoading` / `isFetching` drive skeleton display
- First paint looks like real content (matching heights, widths)
- Background refetches show a subtle top progress bar (not skeleton again)

---

## 10. Empty States

Empty states combine **illustrations + actionable CTAs + code examples** (when developer-facing).

Template:
```
[Illustration/Icon]
[Title]
[Description]
[CTA Button]
[Code example - optional, for developer-facing screens]
```

Examples:
- **No subscriptions**: Webhook icon, "No subscriptions yet", "Create a webhook subscription to start receiving events", [Create Subscription] button
- **No events**: Code icon, "No events received", "Send your first event using the API:", `curl -X POST ...` code block, [View API Docs] link
- **No apps**: Folder icon, "No applications", "Applications group your webhooks and events", [Create Application] button

---

## 11. Responsive Design

### Breakpoints

| Breakpoint | Width | Layout |
|---|---|---|
| Desktop | >= 1280px | Full sidebar (240px) + content |
| Tablet | 768px - 1279px | Collapsed sidebar (64px icons) + content |
| Mobile | < 768px | No sidebar, bottom tab bar + drawer |

### Mobile-specific behaviors

- Tables: horizontal scroll with sticky first column
- Side panels: full-screen overlay on mobile
- Command palette: full-screen on mobile
- Forms: single-column layout
- Cards: full-width, stacked

### E2E tests required

Playwright E2E tests must cover:
- Mobile viewport navigation (bottom tab bar, drawer open/close)
- Tablet viewport (collapsed sidebar icon navigation)
- Responsive table scroll behavior
- Side panel to full-screen transition on mobile

---

## 12. Animations & Motion

### Page Transitions
- Vue `<Transition>` with CSS
- Enter: `opacity 0 -> 1` + `translateY(4px) -> 0` (150ms ease-out)
- Leave: `opacity 1 -> 0` (100ms ease-in)

### Micro-interactions
- Button press: `scale(0.98)` on `mousedown` (50ms)
- Hover: subtle `translateY(-1px)` on cards
- Focus: `ring-2 ring-primary ring-offset-2`
- Loading: skeleton shimmer animation (CSS keyframes)
- Toast: slide-in from right (200ms spring)

### Layout Animations
- Side panel: slide-in from right (250ms ease-out)
- Dropdown: `scaleY(0) -> scaleY(1)` from top (150ms)
- Dialog: `scale(0.95) opacity(0) -> scale(1) opacity(1)` (200ms)
- Command palette: `translateY(-10px) opacity(0) -> translateY(0) opacity(1)` (150ms)

### Rich Motion (via @vueuse/motion)
- Org/app switcher dropdown items: staggered entrance
- Table row enter/exit: fade + slide
- Card hover glow effect (mouse tracking)

---

## 13. i18n Architecture

### Setup
- `vue-i18n` v11 in composition API mode
- All UI strings in `locales/en.json`
- Lazy-loaded locale files (when more languages added)
- `useI18n()` composable in every component

### String Organization
```json
{
  "common": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete",
    "create": "Create",
    "edit": "Edit",
    "loading": "Loading...",
    "retry": "Retry",
    "noResults": "No results found"
  },
  "auth": {
    "login": { "title": "Sign in to Hook0", "email": "Email", ... },
    "register": { ... }
  },
  "subscriptions": {
    "title": "Subscriptions",
    "empty": { "title": "No subscriptions yet", "description": "...", "cta": "Create Subscription" },
    "form": { "url": "Endpoint URL", "urlHelp": "The URL that will receive webhook payloads" },
    ...
  }
}
```

### Rules
- **No hardcoded strings in templates** -- everything goes through `$t()` or `t()`
- Date/time formatting via `vue-i18n` number/datetime formats
- Pluralization rules defined per locale

---

## 14. Analytics & Tracking (improved)

### Event Taxonomy

Standardized naming: `{category}.{action}` format.

| Event | When |
|---|---|
| `auth.login` | User logs in |
| `auth.register` | User registers |
| `auth.logout` | User logs out |
| `org.create` | Organization created |
| `org.switch` | User switches org |
| `app.create` | Application created |
| `app.switch` | User switches app |
| `subscription.create` | Subscription created |
| `subscription.test` | Test endpoint triggered |
| `event_type.create` | Event type created |
| `navigation.command_palette` | cmd+k opened |
| `navigation.shortcut` | Keyboard shortcut used |

### Custom Dimensions

- `organization_id` -- current org context
- `application_id` -- current app context
- `color_mode` -- light/dark
- `viewport` -- desktop/tablet/mobile

### `useTracking` composable (improved)

```typescript
const { trackEvent, trackPageView } = useTracking()
trackEvent('subscription.create', { eventTypes: 3 })
```

---

## 15. Bundle Optimization

### Code Splitting Strategy

| Chunk | Contents | Max Size (gzip) |
|---|---|---|
| `main` | Vue, Vue Router, Pinia, TanStack Query core, Tailwind | <150KB |
| `vendor-biscuit` | @biscuit-auth/biscuit-wasm | Lazy (token pages only) |
| `vendor-codemirror` | CodeMirror + JSON lang | Lazy (code editor pages only) |
| `vendor-charts` | ECharts + vue-echarts | Lazy (dashboard only) |
| Per-route chunks | Each page/feature | <30KB each |

### Removed from bundle
- **AG-Grid** (~300KB) -- replaced by TanStack Table (~30KB)
- **Swagger UI** (~1MB) -- external link
- **FontAwesome** (~200KB with tree-shaking) -- replaced by Lucide (tree-shakeable, ~2KB per icon)
- **fp-ts + Ramda** (~100KB) -- native TypeScript

**Estimated total bundle reduction**: ~1.5MB uncompressed

### Build Configuration

Vite manual chunks:
```typescript
manualChunks: {
  'vendor-core': ['vue', 'vue-router', 'pinia', '@tanstack/vue-query'],
  'vendor-biscuit': ['@biscuit-auth/biscuit-wasm'],
  'vendor-codemirror': ['codemirror', '@codemirror/lang-json', 'vue-codemirror'],
}
```

---

## 16. Route Changes

### Preserved Routes (NO changes)

All existing routes from `routes.ts` are preserved with identical paths, parameter names, and route names. The only changes:

### Modified Routes

| Route | Change |
|---|---|
| `/api/documentation` | Component replaced: instead of Swagger UI embed, redirects to `https://documentation.hook0.com/api` |
| `/organizations/:org_id/applications/:app_id/documentation` | Same redirect |

These routes perform a client-side redirect (not 404) for backward compatibility with bookmarks.

### data-test-* Attributes

**Hard requirement**: All `data-test-*` attributes from current components MUST be preserved on their equivalent elements in redesigned components. This ensures existing E2E tests in `tests-e2e/` continue working without modification.

If a `data-test-*` attribute cannot be placed on an equivalent element (because the page structure changed fundamentally), explicit permission must be requested before modifying any E2E test.

---

## 17. Testing Strategy

### Primary: Playwright E2E (black-box)

- Tests in `tests-e2e/` (project root, existing)
- Tests use `data-test-*` selectors exclusively
- Run against a real backend
- **No mocks, ever**
- New E2E tests required for:
  - Mobile viewport navigation (bottom tab bar, drawer)
  - Tablet viewport (collapsed sidebar)
  - Responsive table behavior
  - Dark mode toggle
  - Command palette (cmd+k open, search, navigate)

### Secondary: Component tests (complex isolated logic only)

- Jest for pure logic (Zod schemas, utility functions, query key factories)
- Not for component rendering (that's E2E's job)

### E2E Test Preservation Rule

When redesigning a page, the developer MUST:
1. Identify all `data-test-*` attributes used by existing E2E tests
2. Place those exact attributes on the corresponding elements in the new design
3. Run existing E2E tests to verify they pass
4. Only if a test CANNOT pass due to fundamental structural changes: request explicit permission before modifying the test

---

## 18. Migration Strategy

### Approach: Big Bang Rewrite

The entire frontend is rewritten in a single branch. No hybrid state, no feature flags, no incremental migration.

### Execution Order

1. **Foundation** (must be first):
   - Tailwind v4 setup
   - CSS custom properties (light + dark)
   - Font loading (Inter + JetBrains Mono)
   - Pinia stores (auth, ui, context)
   - TanStack Query client setup
   - vue-i18n setup
   - Router + route definitions (preserved)
   - ESLint rules for conventions

2. **Component Library**:
   - Core: Button, Input, Select, Dialog, Tooltip, Badge
   - Layout: Sidebar, Header, Breadcrumb, PageLayout, MobileTabBar, MobileDrawer
   - Data: Table, Card, Code, DateTime, KeyValue, EmptyState, ErrorCard
   - Form: Form, FormField, FormActions
   - Feedback: Alert, Skeleton, Toast config
   - Special: CommandPalette, SidePanel

3. **Auth pages** (no sidebar, simpler):
   - Login, Register, ForgotPassword, ResetPassword, VerifyEmail, CheckEmail

4. **Main layout** (sidebar + header + breadcrumb):
   - Root.vue / App.vue layout
   - Sidebar with org/app switcher
   - Header with breadcrumbs + command palette trigger
   - Mobile responsive layout

5. **Feature pages** (one feature at a time):
   - Organizations (dashboard, settings, members)
   - Applications (list, dashboard, settings)
   - Event Types (list, create)
   - Subscriptions (list, edit with test button)
   - Events (list with side panel, detail with timeline)
   - Logs (structured viewer)
   - API Keys
   - Service Tokens (simple + advanced modes)
   - User Settings
   - Tutorial (wizard overlay)
   - Error 404
   - API Documentation (redirect)

6. **Polish**:
   - Animations & transitions
   - Dark mode fine-tuning
   - Responsive breakpoint testing
   - Accessibility audit (WCAG 2.1 AA)
   - Bundle size verification

---

## 19. Out of Scope

| Feature | Reason |
|---|---|
| Real-time event streaming (WebSocket/SSE) | Deferred to future iteration (see TODO.md) |
| Dashboard charts & graphs | Implemented with ECharts + vue-echarts |
| Event Composer / Playground | Deferred to future iteration |
| Billing / payments UI | Not applicable currently |
| Backend API changes | Pure frontend redesign |
| Notification center (bell inbox) | Toast + inline alerts sufficient for now |
| Vim-like keyboard shortcuts | Essential shortcuts only for v1 |
| Multiple languages | i18n architecture ready, English only at launch |

---

## 20. Open Questions

1. **Org/App dashboard stats**: Which backend API endpoints provide the aggregate stats (event count, delivery rate, active subscriptions)? If they don't exist, the dashboard will show the entities list without aggregate numbers.
2. **Event delivery timeline**: Does the current API expose per-subscription delivery attempts for an event? If not, the event detail page will show the payload only (no delivery timeline until the API supports it).
3. **Members list write operations**: Can members be invited/removed via the current API? If not, the members list is read-only.

---

## 21. Dependency Upgrade Strategy

- **Pin all dependencies** in `package.json` (exact versions for core deps, caret for patches)
- **Quarterly review cycle**: check for updates, evaluate breaking changes, upgrade in a dedicated branch
- Use Renovate or Dependabot for automated PR creation
- CI validates build + E2E tests before merging any dependency update
