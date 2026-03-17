import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Navigation E2E tests for Hook0.
 *
 * Tests navigation between pages with STRICT content assertions.
 * Verifies that pages render correctly after navigation (no blank pages).
 * Uses explicit text content checks, not just visibility.
 */
test.describe("Navigation", () => {
  test.describe("Authenticated Navigation", () => {
    test("should navigate between all main application pages and verify content renders", async ({
      page,
      request,
    }) => {
      // Setup: Create test user with application
      const timestamp = Date.now();
      const email = `test-nav-main-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `Nav Test App ${timestamp}`;

      // Register via API
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: {
          email,
          first_name: "Test",
          last_name: "User",
          password,
        },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      // Verify email and get organization ID
      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login via UI
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();

      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create an application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);

      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      // Create an event type for complete navigation testing
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/event_types/new`
      );
      await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="event-type-service-input"]').fill("test");
      await page.locator('[data-test="event-type-resource-input"]').fill("resource");
      await page.locator('[data-test="event-type-verb-input"]').fill("created");
      await page.locator('[data-test="event-type-submit-button"]').click();
      await expect(page).toHaveURL(/\/event_types$/, { timeout: 15000 });

      // =================================================================
      // TEST 1: Navigate to Event Types List - verify specific content
      // =================================================================
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/event_types`
      );

      // STRICT assertion: Page title must contain "Event Types"
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });
      const eventTypesHeader = page.locator('[data-test="event-types-card"] h2, [data-test="event-types-card"] h3').first();
      await expect(eventTypesHeader).toContainText("Event Types");

      // STRICT assertion: Table should have our event type
      await expect(page.locator('[data-test="event-types-table"]')).toBeVisible();
      await expect(page.locator('[data-test="event-types-table"]')).toContainText("test.resource.created");

      // =================================================================
      // TEST 2: Navigate to Events List - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/applications/${applicationId}/events`);

      // STRICT assertion: Card must be visible with correct header
      await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });
      const eventsHeader = page.locator('[data-test="events-card"] h2, [data-test="events-card"] h3').first();
      await expect(eventsHeader).toContainText("Events");

      // STRICT assertion: Either table or empty state must be present
      const eventsTable = page.locator('[data-test="events-table"]');
      const sendButton = page.locator('[data-test="events-send-button"]');

      // At least one should be visible (empty state has send button, populated has table)
      const tableOrButton = await Promise.race([
        eventsTable.waitFor({ state: "visible", timeout: 5000 }).then(() => "table"),
        sendButton.waitFor({ state: "visible", timeout: 5000 }).then(() => "button"),
      ]).catch(() => null);

      expect(tableOrButton).not.toBeNull();

      // =================================================================
      // TEST 3: Navigate to Subscriptions List - verify specific content
      // =================================================================
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/subscriptions`
      );

      // STRICT assertion: Card must be visible
      await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });
      const subscriptionsHeader = page.locator('[data-test="subscriptions-card"] h2, [data-test="subscriptions-card"] h3').first();
      await expect(subscriptionsHeader).toContainText("Subscriptions");

      // STRICT assertion: Create button must be accessible
      await expect(page.locator('[data-test="subscriptions-create-button"]')).toBeVisible();

      // =================================================================
      // TEST 4: Navigate to Logs List - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/applications/${applicationId}/logs`);

      // STRICT assertion: Card must be visible
      await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
      const logsHeader = page.locator('[data-test="logs-card"] h2, [data-test="logs-card"] h3').first();
      await expect(logsHeader).toContainText("Request Attempts");

      // =================================================================
      // TEST 5: Navigate to Application Settings - verify specific content
      // =================================================================
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/settings`
      );

      // STRICT assertion: Form must be visible with application name
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      const nameInput = page.locator('[data-test="application-name-input"]');
      await expect(nameInput).toBeVisible();
      await expect(nameInput).toHaveValue(appName);

      // STRICT assertion: Delete card must be present
      await expect(page.locator('[data-test="application-delete-card"]')).toBeVisible();

      // =================================================================
      // TEST 6: Navigate to Organization Dashboard - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/dashboard`);

      // Wait for page to load (either dashboard card or applications card will be visible)
      // The org dashboard includes an ApplicationsList component that has data-test="applications-card"
      await expect(
        page.locator('[data-test="organization-dashboard-card"], [data-test="applications-card"]').first()
      ).toBeVisible({ timeout: 15000 });

      // =================================================================
      // TEST 7: Navigate to Organization Settings - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/settings`);

      // STRICT assertion: Form must be visible
      await expect(page.locator('[data-test="organization-form"]')).toBeVisible({ timeout: 10000 });
      await expect(page.locator('[data-test="organization-name-input"]')).toBeVisible();

      // =================================================================
      // TEST 8: Navigate to Service Tokens - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/services_tokens`);

      // STRICT assertion: Service tokens card must be visible
      await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });
      const serviceTokensHeader = page.locator('[data-test="service-tokens-card"] h2, [data-test="service-tokens-card"] h3').first();
      await expect(serviceTokensHeader).toContainText("Service Tokens");

      // =================================================================
      // TEST 9: Navigate to Applications List - verify specific content
      // =================================================================
      await page.goto(`/organizations/${organizationId}/applications`);

      // STRICT assertion: Applications card must be visible
      await expect(page.locator('[data-test="applications-card"]')).toBeVisible({ timeout: 10000 });

      // STRICT assertion: Our created app must be in the list
      await expect(page.locator('[data-test="applications-table"]')).toContainText(appName);
    });

    test("should navigate via sidebar links and verify each page renders content", async ({
      page,
      request,
    }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-sidebar-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `Sidebar Nav App ${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);
      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      // Start from application dashboard
      await page.goto(`/organizations/${organizationId}/applications/${applicationId}/dashboard`);
      // Wait for dashboard to load - it includes EventTypesList which has data-test="event-types-card"
      await expect(
        page.locator('[data-test="application-dashboard-card"], [data-test="event-types-card"]').first()
      ).toBeVisible({ timeout: 15000 });

      // Click on Events in navigation
      const eventsNavLink = page.locator('nav a[href*="/events"]').first();
      if (await eventsNavLink.isVisible()) {
        await eventsNavLink.click();
        await expect(page).toHaveURL(/\/events/, { timeout: 10000 });
        // STRICT assertion: Events card must render
        await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });
      }

      // Click on Event Types in navigation
      const eventTypesNavLink = page.locator('nav a[href*="/event_types"]').first();
      if (await eventTypesNavLink.isVisible()) {
        await eventTypesNavLink.click();
        await expect(page).toHaveURL(/\/event_types/, { timeout: 10000 });
        // STRICT assertion: Event types card must render
        await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });
      }

      // Click on Subscriptions in navigation
      const subscriptionsNavLink = page.locator('nav a[href*="/subscriptions"]').first();
      if (await subscriptionsNavLink.isVisible()) {
        await subscriptionsNavLink.click();
        await expect(page).toHaveURL(/\/subscriptions/, { timeout: 10000 });
        // STRICT assertion: Subscriptions card must render
        await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });
      }

      // Click on Logs in navigation
      const logsNavLink = page.locator('nav a[href*="/logs"]').first();
      if (await logsNavLink.isVisible()) {
        await logsNavLink.click();
        await expect(page).toHaveURL(/\/logs/, { timeout: 10000 });
        // STRICT assertion: Logs card must render
        await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
      }
    });

    test("should navigate from list to create form and back without blank pages", async ({
      page,
      request,
    }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-create-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `Create Flow App ${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);
      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      // =================================================================
      // TEST: Event Types - list → create → list
      // =================================================================
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/event_types`
      );
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

      // Click create button
      await page.locator('[data-test="event-types-create-button"]').click();
      await expect(page).toHaveURL(/\/event_types\/new/, { timeout: 10000 });

      // STRICT assertion: Form must render with all fields
      await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
      await expect(page.locator('[data-test="event-type-service-input"]')).toBeVisible();
      await expect(page.locator('[data-test="event-type-resource-input"]')).toBeVisible();
      await expect(page.locator('[data-test="event-type-verb-input"]')).toBeVisible();

      // Cancel and go back
      await page.locator('[data-test="event-type-cancel-button"]').click();
      await expect(page).toHaveURL(/\/event_types$/, { timeout: 10000 });

      // STRICT assertion: List must render again (no blank page)
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

      // =================================================================
      // TEST: Subscriptions - list → create → list
      // =================================================================
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/subscriptions`
      );
      await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

      // Click create button
      await page.locator('[data-test="subscriptions-create-button"]').click();
      await expect(page).toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

      // STRICT assertion: Form must render
      await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({ timeout: 10000 });
      await expect(page.locator('[data-test="subscription-url-input"]')).toBeVisible();

      // Cancel and go back
      await page.locator('[data-test="subscription-cancel-button"]').click();
      await expect(page).toHaveURL(/\/subscriptions$/, { timeout: 10000 });

      // STRICT assertion: List must render again
      await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });
    });

    test("should handle rapid navigation without blank pages", async ({ page, request }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-rapid-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `Rapid Nav App ${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);
      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      // Rapid navigation sequence
      const baseUrl = `/organizations/${organizationId}/applications/${applicationId}`;

      // Navigate rapidly between pages
      await page.goto(`${baseUrl}/event_types`);
      await page.goto(`${baseUrl}/events`);
      await page.goto(`${baseUrl}/subscriptions`);
      await page.goto(`${baseUrl}/logs`);

      // Final page should render correctly
      await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
      const logsHeader = page.locator('[data-test="logs-card"] h2, [data-test="logs-card"] h3').first();
      await expect(logsHeader).toContainText("Request Attempts");

      // Navigate back to event types
      await page.goto(`${baseUrl}/event_types`);
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });
    });

    test("should navigate via breadcrumb segments", async ({ page, request }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-breadcrumb-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `Breadcrumb Nav App ${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);
      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      // Navigate to an app-level page
      await page.goto(
        `/organizations/${organizationId}/applications/${applicationId}/event_types`
      );
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

      // Verify breadcrumb nav is visible
      await expect(page.locator('[data-test="breadcrumb-nav"]')).toBeVisible({ timeout: 10000 });

      // Click the org-level breadcrumb segment to open the dropdown
      await page.locator('[data-test="breadcrumb-org"] button').first().click();

      // Click the current org in the dropdown to navigate to org level
      const orgDropdownItem = page.locator('[data-test="breadcrumb-org"] [role="option"]').first();
      await expect(orgDropdownItem).toBeVisible({ timeout: 5000 });
      await orgDropdownItem.click();

      // Verify navigation to org level
      await expect(page).toHaveURL(new RegExp(`/organizations/${organizationId}`), {
        timeout: 10000,
      });
      // Should no longer be on the app-level event_types page
      await expect(page).not.toHaveURL(/\/event_types/, { timeout: 5000 });
    });

    test("should verify page content after browser back/forward navigation", async ({
      page,
      request,
    }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-history-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;
      const appName = `History Nav App ${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      const verificationResult = await verifyEmailViaMailpit(request, email);
      const organizationId = verificationResult.organizationId;
      expect(organizationId).toBeTruthy();

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Create application
      await page.goto(`/organizations/${organizationId}/applications/new`);
      await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="application-name-input"]').fill(appName);
      const createAppResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/applications") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="application-submit-button"]').click();
      const appResponse = await createAppResponse;
      expect(appResponse.status()).toBeLessThan(400);
      const app = await appResponse.json();
      const applicationId = app.application_id;

      const baseUrl = `/organizations/${organizationId}/applications/${applicationId}`;

      // Visit Event Types
      await page.goto(`${baseUrl}/event_types`);
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

      // Visit Events (adds to history)
      await page.goto(`${baseUrl}/events`);
      await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

      // Visit Subscriptions (adds to history)
      await page.goto(`${baseUrl}/subscriptions`);
      await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

      // Go back to Events
      await page.goBack();
      await expect(page).toHaveURL(/\/events/, { timeout: 10000 });
      // STRICT assertion: Events card must render after back navigation
      await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

      // Go back to Event Types
      await page.goBack();
      await expect(page).toHaveURL(/\/event_types/, { timeout: 10000 });
      // STRICT assertion: Event types card must render after back navigation
      await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

      // Go forward to Events
      await page.goForward();
      await expect(page).toHaveURL(/\/events/, { timeout: 10000 });
      // STRICT assertion: Events card must render after forward navigation
      await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

      // Go forward to Subscriptions
      await page.goForward();
      await expect(page).toHaveURL(/\/subscriptions/, { timeout: 10000 });
      // STRICT assertion: Subscriptions card must render after forward navigation
      await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });
    });
  });

  test.describe("Unauthenticated Navigation", () => {
    test("should display login page with all required elements", async ({ page }) => {
      await page.goto("/login");

      // STRICT assertions: Verify actual content, not just visibility
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });

      // Check for page title content
      const pageTitle = page.locator('h1, h2').first();
      await expect(pageTitle).toContainText(/Welcome|Sign in|Log in/i);

      // Check email input has correct placeholder or label
      const emailInput = page.locator('[data-test="login-email-input"]');
      await expect(emailInput).toBeVisible();

      // Check password input exists
      const passwordInput = page.locator('[data-test="login-password-input"]');
      await expect(passwordInput).toBeVisible();

      // Check submit button has correct text
      const submitButton = page.locator('[data-test="login-submit-button"]');
      await expect(submitButton).toContainText(/Sign in/i);

      // Check forgot password link exists
      const forgotPasswordLink = page.locator('a[href*="reset-password"], a[href*="forgot"]');
      await expect(forgotPasswordLink).toBeVisible();

      // Check register link exists
      const registerLink = page.locator('a[href*="register"]');
      await expect(registerLink).toBeVisible();
    });

    test("should display register page with all required elements", async ({ page }) => {
      await page.goto("/register");

      // STRICT assertions: Verify actual content
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({ timeout: 10000 });

      // Check page title (may be in h1, h2, h3, or a div within the card header)
      const pageTitle = page.locator('h1, h2, h3, .hook0-card-header').first();
      await expect(pageTitle).toContainText(/Free Trial|Create|Sign up|Register/i);

      // Check all form fields exist
      await expect(page.locator('[data-test="register-firstname-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-lastname-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-email-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-password-input"]')).toBeVisible();

      // Check submit button
      const submitButton = page.locator('[data-test="register-submit-button"]');
      await expect(submitButton).toContainText(/Create|Sign up|Register/i);

      // Check login link exists
      const loginLink = page.locator('a[href*="login"]');
      await expect(loginLink).toBeVisible();
    });

    test("should navigate between login and register pages", async ({ page }) => {
      // Start at login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });

      // Click register link
      const registerLink = page.locator('a[href*="register"]');
      await registerLink.click();
      await expect(page).toHaveURL(/\/register/, { timeout: 10000 });

      // STRICT assertion: Register form must render
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({ timeout: 10000 });

      // Click login link
      const loginLink = page.locator('a[href*="login"]');
      await loginLink.click();
      await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

      // STRICT assertion: Login form must render
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    });

    test("should display forgot password page correctly", async ({ page }) => {
      await page.goto("/begin-reset-password");

      // STRICT assertions: Verify form renders with correct content
      await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });

      // Check email input
      await expect(page.locator('[data-test="reset-password-email-input"]')).toBeVisible();

      // Check submit button
      const submitButton = page.locator('[data-test="reset-password-submit-button"]');
      await expect(submitButton).toContainText(/Send|Reset|Submit/i);

      // Check back to login link
      const loginLink = page.locator('a[href*="login"]');
      await expect(loginLink).toBeVisible();
    });
  });

  test.describe("User Settings Navigation", () => {
    test("should navigate to user settings and verify all sections render", async ({
      page,
      request,
    }) => {
      // Setup
      const timestamp = Date.now();
      const email = `test-nav-settings-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;

      // Register and verify
      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: { email, first_name: "Test", last_name: "User", password },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      await verifyEmailViaMailpit(request, email);

      // Login
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);
      await page.locator('[data-test="login-submit-button"]').click();
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

      // Navigate to user settings
      await page.goto("/settings");

      // STRICT assertions: Verify all sections render
      await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({ timeout: 10000 });

      // Personal info section - email should be in the input
      await expect(page.locator('[data-test="user-email-input"]')).toHaveValue(email);

      // Password change section
      await expect(page.locator('[data-test="change-password-card"]')).toBeVisible();
      await expect(page.locator('[data-test="new-password-input"]')).toBeVisible();
      await expect(page.locator('[data-test="confirm-password-input"]')).toBeVisible();

      // Delete account section
      await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();
      await expect(page.locator('[data-test="delete-account-button"]')).toBeVisible();
    });
  });
});
