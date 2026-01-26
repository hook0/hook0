import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Events E2E tests for Hook0.
 *
 * Tests for viewing events list, sending test events, and viewing event details.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Events", () => {
  /**
   * Helper to setup test environment: user, organization, application, and event type
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-events-${testId}-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

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
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Create an application
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(`Test App ${timestamp}`);

    const createAppResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);
    const app = await appResponse.json();

    // Create an event type
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createEventTypeResponse;

    return {
      email,
      password,
      organizationId,
      applicationId: app.application_id,
      timestamp,
      eventTypeName: "test.entity.created",
    };
  }

  test("should display events list page with send event button", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list-display");

    // Navigate to events page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    // Verify events card is visible
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Verify send event button is present
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible();
  });

  test("should display send event form when clicking send button", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "form-display");

    // Navigate to events page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Click send event button
    await page.locator('[data-test="events-send-button"]').click();

    // Verify send event form is visible
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="send-event-type-select"]')).toBeVisible();
    await expect(page.locator('[data-test="send-event-occurred-at-input"]')).toBeVisible();
    await expect(page.locator('[data-test="send-event-submit-button"]')).toBeVisible();
    await expect(page.locator('[data-test="send-event-cancel-button"]')).toBeVisible();
  });

  test("should send test event and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "send");

    // Navigate to events page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Click send event button to open form
    await page.locator('[data-test="events-send-button"]').click();

    // Wait for form
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Fill form
    await page.locator('[data-test="send-event-type-select"]').selectOption(env.eventTypeName);

    // Set occurred_at to current date/time
    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16); // Format: YYYY-MM-DDTHH:MM
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/events") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown
    await expect(
      page.locator('[class*="Notivue"], [class*="notivue"], [role="alert"]').first()
    ).toBeVisible({
      timeout: 10000,
    });

    // Verify form is closed and events list is shown
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should display events list with sent event", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list-with-event");

    // Navigate to events page and send an event first
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Click send event button to open form
    await page.locator('[data-test="events-send-button"]').click();

    // Wait for form and fill it
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="send-event-type-select"]').selectOption(env.eventTypeName);

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    // Submit
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/events") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();
    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    // Wait for events list to refresh
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Verify events table has at least 1 row
    const rows = page.locator('[data-test="events-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Verify first row contains the event type name
    const firstRow = rows.first();
    await expect(firstRow).toContainText(env.eventTypeName);
  });

  test("should cancel send event form when clicking cancel", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "cancel");

    // Navigate to events page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Click send event button to open form
    await page.locator('[data-test="events-send-button"]').click();

    // Wait for form
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });

    // Click cancel
    await page.locator('[data-test="send-event-cancel-button"]').click();

    // Verify form is closed and events list is shown
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="send-event-form"]')).not.toBeVisible();
  });

  test("should navigate to event detail page", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "detail");

    // Navigate to events page and send an event first
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Send an event
    await page.locator('[data-test="events-send-button"]').click();
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="send-event-type-select"]').selectOption(env.eventTypeName);

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/events") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();
    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);
    const eventData = await response.json();

    // Wait for events list to show
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Click on the event ID link in the first row
    const rows = page.locator('[data-test="events-table"] .ag-row');
    await expect(rows.first()).toBeVisible();
    await rows.first().locator("a").first().click();

    // Verify we're on the event detail page
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-page"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-type"]')).toContainText(env.eventTypeName);
  });
});
