# Hook0 Analytics Configuration

This document describes the Matomo analytics configuration for the Hook0 project.

## Overview

Hook0 uses Matomo for web analytics across three properties:
- **Website** (hook0.com) - Marketing site
- **Frontend** (app.hook0.com) - Application dashboard
- **Documentation** (documentation.hook0.com) - Docusaurus docs

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MATOMO_URL` | Matomo instance URL (e.g., `https://matomo.hook0.com/`) |
| `MATOMO_SITE_ID` | Site ID in Matomo (each property has its own ID; the app site is `2`) |
| `MATOMO_TOKEN_AUTH` | Server-side Tracking API `token_auth` secret. Enables the server-side activation event (see North-Star Metric). Dark by default: unset means no server-side emission. |
| `MATOMO_ACTIVATION_PERIOD_IN_S` | Seconds between server-side activation-event scan passes (default `300`, `0` disables) |
| `MATOMO_ACTIVATION_SCAN_LIMIT` | Max organizations claimed per activation scan pass (default `500`, range `[1, 10000]`) |

## Matomo Goals Configuration

Goals are configured in the Matomo admin interface. Below are the Goals and the events that trigger them.

### Website Goals (hook0.com)

| Goal ID | Goal Name | Match Type | Event Name | Code Location |
|---------|-----------|------------|------------|---------------|
| 1 | Registration Click | Event Name | `Register` | `website/src/includes/_head.ejs` (inline onclick) |
| 2 | Login Click | Event Name (exact) | `Login` | `website/src/includes/_head.ejs:138` |
| 3 | Contact Support | Event Name (exact) | `Contact Support` | `website/src/includes/_head.ejs:142` |
| 4 | Search Result Click | Event Name (exact) | `Search Result Click` | `website/src/includes/_head.ejs:146` |
| 5 | Full Page Read | Event Name (exact) | `100%` | `website/src/includes/_head.ejs:156` |

### Frontend Goals (app.hook0.com)

| Goal ID | Goal Name | Match Type | Event Name Pattern | Code Location |
|---------|-----------|------------|-------------------|---------------|
| 1 | Registration | Event Name | `complete` | `RegisterPage.vue` |
| 2 | Login | Event Name | `complete` | `LoginPage.vue` |
| 3 | Tutorial Complete | Event Name | `tutorial-complete` | `TutorialSuccess.vue` |
| 4 | First Application | Event Name | `step-complete` + `application` | `TutorialCreateApplication.vue` |
| 5 | First Event Sent | Event Name | `step-complete` + `send-event` | `TutorialSendEvent.vue` |

## North-Star Metric — First Webhook Delivered

The north-star activation metric is **time-to-first-webhook-received**: how long it takes a new
organization to go from sign-up to actually **receiving** its first webhook at an endpoint. This is
strictly deeper in the funnel than "First Event Sent" (Frontend Goal 5): sending an event only proves
the org called our ingestion API, while a *delivered* webhook proves their integration works end to
end — a request attempt that completed with `succeeded_at` set.

### Server-side event (`first_webhook_delivered`)

Delivery happens inside the webhook-delivery worker, not in the browser, so this signal is emitted
**server-side** by a periodic background job (`api/src/google_ads/first_webhook_delivered_conversion.rs`),
never on the delivery hot path. Per pass it finds gclid-attributed organizations that have at least one
successful request attempt and whose conversion is still pending, uploads a Google Ads click conversion,
and marks the org idempotently (`iam.signup_attribution.first_webhook_delivered_uploaded_at`).

| Aspect | Value |
|--------|-------|
| Event / conversion name | `first_webhook_delivered` |
| Emission | Server-side periodic job (background scan; never the delivery hot path) |
| OpenTelemetry metric | `conversions.uploaded{kind="first_webhook_delivered", outcome="success\|partial_failure\|failed"}` |
| Enable (dark by default) | Set `GOOGLE_ADS_FIRST_WEBHOOK_DELIVERED_CONVERSION_ACTION_ID` |
| Scan period | `GOOGLE_ADS_FIRST_WEBHOOK_DELIVERED_CONVERSION_PERIOD_IN_S` (default `300`, `0` disables) |

> **Dark by default:** when `GOOGLE_ADS_FIRST_WEBHOOK_DELIVERED_CONVERSION_ACTION_ID` is unset (the
> typical self-hosting case), the job is not spawned and nothing is emitted — zero impact on
> deliverability. Only the pseudonymous `gclid` (already issued by Google at ad click) is sent back; no
> Hook0 PII leaves the system.

### Server-side Matomo activation event (`activation` / `first-webhook-delivered`)

The **product-activation Goal** (distinct from the Google Ads conversion above, and not gated on any ad
attribution) is fed by a Matomo event emitted **server-side** — activation happens in the delivery
worker, so it cannot be a browser `trackEvent`. A periodic background job (`api/src/matomo.rs`) sends the
event through Matomo's **HTTP Tracking API** (`POST {MATOMO_URL}matomo.php`) for **every** organization on
its genuine first successful webhook delivery, once each. The per-org one-shot claim lives in
`iam.organization.matomo_activation_emitted_at` (stamped before sending, released back to `NULL` if the
send fails so the org is retried next pass). The event uses category `activation`, action
`first-webhook-delivered`, and dates the event at the org's first delivery (`cdt`).

| Aspect | Value |
|--------|-------|
| Event Category / Action | `activation` / `first-webhook-delivered` |
| Target site | `MATOMO_SITE_ID` (the app site, `2`) |
| Emission | Server-side periodic job (background scan; never the delivery hot path) |
| Visitor id (`_id`) | 16-hex pseudonymous value derived from the organization id — **no PII**, no email, no user id |
| Enable (dark by default) | Set `MATOMO_URL` + `MATOMO_SITE_ID` + `MATOMO_TOKEN_AUTH` |
| Scan period | `MATOMO_ACTIVATION_PERIOD_IN_S` (default `300`, `0` disables) |
| Scan limit | `MATOMO_ACTIVATION_SCAN_LIMIT` (default `500`) |

> **Dark by default:** when any of `MATOMO_URL` / `MATOMO_SITE_ID` / `MATOMO_TOKEN_AUTH` is unset, the job
> is not spawned and nothing is emitted. `MATOMO_TOKEN_AUTH` is the server-side switch, so front-end
> Matomo tracking (URL + site id alone) is unaffected. No PII is sent to Matomo: the visitor id is a
> pseudonymous value derived from the (already opaque) organization id — no email, name, or user id.

### Matomo Goal to create (recipe, no code)

The dedicated Matomo Goal is configured **in the Matomo admin UI**, not in code. On the **Frontend
property (app.hook0.com)**, create:

| Field | Value |
|-------|-------|
| Goal name | `First Webhook Delivered` |
| Trigger | Manually (via the tracking API) |
| Match | Event — Event Category `activation`, Event Action `first-webhook-delivered` |
| Allow multiple conversions per visit | No (first delivery only) |
| Revenue | none |

Steps: Matomo → **Administration → Websites → Manage Goals** (app.hook0.com) → **Add a new goal** →
fill the table above → **Add Goal**. The event that feeds this Goal (category `activation`, action
`first-webhook-delivered`) is emitted **server-side** by the job described above (see *Server-side Matomo
activation event*); once the Goal exists it records every emission automatically.

## Event Naming Conventions

### Frontend (app.hook0.com)

Uses **kebab-case** for all event parameters:

```typescript
trackEvent(category: string, action: string, name?: string, value?: number)
```

**Categories:**
- `auth` - Authentication events (login, registration)
- `signup` - Registration flow events
- `tutorial` - Onboarding tutorial events
- `organization` - Organization CRUD operations
- `application` - Application CRUD operations
- `subscription` - Subscription CRUD operations
- `service-token` - Service token management
- `event-type` - Event type management
- `app-secret` - API key management
- `api-docs` - API documentation interactions

**Actions:**
- `login`, `logout` - Authentication actions
- `form-start`, `form-submit` - Form interactions
- `create`, `update`, `delete` - CRUD operations
- `step-complete` - Tutorial step completions
- `page-view` - Page views
- `email-verified` - Email verification
- `attenuate` - Token attenuation

**Names:**
- Use descriptive labels (e.g., `organization-name`, `success`)
- Never include UUIDs or sensitive data
- For CRUD operations, use `success` as the name

**Examples:**
```typescript
// Authentication
trackEvent('auth', 'login', 'complete');
trackEvent('signup', 'form-start');
trackEvent('signup', 'email-verified');

// CRUD operations
trackEvent('organization', 'create', 'success');
trackEvent('application', 'delete', 'success');
trackEvent('subscription', 'update', 'success');

// Tutorial
trackEvent('tutorial', 'step-complete', 'organization');
trackEvent('tutorial', 'complete');
```

### Website (hook0.com)

Uses **PascalCase/Title Case** for compatibility with existing Matomo Goals:

```javascript
trackEvent(category, action, name, value)
```

**Examples:**
```javascript
// CTA clicks
trackEvent('CTA', 'Click', 'Register');
trackEvent('CTA', 'Click', 'Login');
trackEvent('CTA', 'Click', 'Contact Support');

// Content engagement
trackEvent('Scroll', 'Depth', '100%');
trackEvent('Documentation', 'Click', 'Search Result Click');
```

> **Note:** Website events use PascalCase because the Matomo Goals are configured with exact match on these specific event names. Changing them would require updating the Goals in Matomo.

## Implementation Details

### Frontend (Vue 3)

The frontend uses the `useTracking` composable located at `frontend/src/composables/useTracking.ts`.

```typescript
import { useTracking } from '@/composables/useTracking';

// In component setup
const { trackEvent, trackPageView } = useTracking();

// Track events
trackEvent('category', 'action', 'name');
```

### Website (EJS/Vanilla JS)

The website tracking is implemented in `website/src/includes/_head.ejs` with GDPR consent management.

```javascript
// Safe tracking function (handles blocked Matomo)
window.trackEvent = function(c, a, n, v) {
  try {
    _paq.push(['trackEvent', c, a, n, v]);
  } catch(e) {}
};

// Usage
trackEvent('CTA', 'Click', 'Register');
```

### Documentation (Docusaurus)

The documentation site uses `documentation/src/theme/Root.tsx` for tracking.

## GDPR Compliance

All properties implement GDPR-compliant tracking:

1. **Cookie consent banner** - Shown on first visit
2. **Consent required** - Tracking disabled until user accepts
3. **Consent stored** - In localStorage as `hook0_cookie_consent`
4. **Reset capability** - `window.hook0Consent.reset()` to re-show banner

```javascript
// Consent API
window.hook0Consent.accept();  // Grant consent
window.hook0Consent.deny();    // Deny consent
window.hook0Consent.reset();   // Reset (re-show banner)
window.hook0Consent.getStatus(); // Get current status
```

### Server-side activation event

The server-side activation event (see *North-Star Metric*) is sent to the self-hosted Matomo instance
with a pseudonymous visitor id derived from the organization id — no email, name, user id, or other PII.
It carries no new personal-data flow beyond the existing first-party Matomo analytics.

## Adding New Events

When adding new tracked events:

1. **Follow naming conventions** for the property (kebab-case for frontend, PascalCase for website)
2. **Never include sensitive data** (UUIDs, emails, tokens) in event names
3. **Use `success` as name** for CRUD operation completions
4. **Document the event** in this file if it relates to a Goal
5. **Consider creating a Goal** in Matomo for important conversion events

## Verifying Tracking

### Development

1. Open browser DevTools > Network tab
2. Filter by `piwik.php` or `matomo`
3. Perform the action you want to track
4. Verify the request is sent with correct parameters

### Matomo Real-time

1. Log into Matomo admin
2. Go to Visitors > Real-time
3. Perform actions on the site
4. Verify events appear in real-time view

## Files Reference

| Property | File | Description |
|----------|------|-------------|
| Frontend | `frontend/src/composables/useTracking.ts` | Tracking composable |
| Frontend | `frontend/src/pages/**/*.vue` | Vue components with tracking |
| Website | `website/src/includes/_head.ejs` | Matomo script + consent |
| Documentation | `documentation/src/theme/Root.tsx` | Docusaurus tracking wrapper |
