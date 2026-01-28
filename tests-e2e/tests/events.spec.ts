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

    // Capture response body inside the predicate to avoid race condition with navigation
    let applicationId: string = "";
    const createAppResponse = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/applications") && response.request().method() === "POST") {
          if (response.status() < 400) {
            try {
              const app = await response.json();
              applicationId = app.application_id;
            } catch {
              // Response body may be unavailable due to navigation
            }
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);

    // Wait for redirect to complete - URL should contain a UUID application ID, not "new"
    // UUID pattern: 8-4-4-4-12 hex characters
    const uuidPattern = /\/applications\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;
    await expect(page).toHaveURL(uuidPattern, { timeout: 15000 });
    const url = page.url();
    const match = url.match(uuidPattern);
    expect(match, "Failed to extract application ID (UUID) from URL").toBeTruthy();
    applicationId = match![1];

    // Create an event type
    await page.goto(
      `/organizations/${organizationId}/applications/${applicationId}/event_types/new`
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
    const eventTypeResponse = await createEventTypeResponse;
    expect(eventTypeResponse.status()).toBeLessThan(400);

    // Wait for navigation after event type creation
    await expect(page).toHaveURL(/\/event_types$/, { timeout: 10000 });

    // Verify event type appears in the list (confirms data is persisted)
    await expect(page.locator('[data-test="event-types-table"]')).toBeVisible({ timeout: 10000 });

    return {
      email,
      password,
      organizationId,
      applicationId,
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

    // Add labels (required for event submission) using data-test selectors
    const labelKeyInput = page.locator('[data-test="send-event-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await labelKeyInput.clear();
    await labelKeyInput.fill("env");
    await labelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await labelValueInput.clear();
    await labelValueInput.fill("test");
    await labelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(labelKeyInput).toHaveValue("env");
    await expect(labelValueInput).toHaveValue("test");

    // Set occurred_at to current date/time
    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16); // Format: YYYY-MM-DDTHH:MM
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") && response.request().method() === "POST" && !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown using data-test selector
    await expect(page.locator('[data-test="toast-notification"]').first()).toBeVisible({
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

    // Add labels (required for event submission) using data-test selectors
    const labelKeyInput = page.locator('[data-test="send-event-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await labelKeyInput.clear();
    await labelKeyInput.fill("env");
    await labelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await labelValueInput.clear();
    await labelValueInput.fill("test");
    await labelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(labelKeyInput).toHaveValue("env");
    await expect(labelValueInput).toHaveValue("test");

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    // Submit
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") && response.request().method() === "POST" && !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();
    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    // Wait for events list to refresh
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Verify events table has at least 1 row
    const rows = page.locator('[data-test="events-table"] [row-id]');
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

    // Add labels (required for event submission) using data-test selectors
    const labelKeyInput = page.locator('[data-test="send-event-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await labelKeyInput.clear();
    await labelKeyInput.fill("env");
    await labelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await labelValueInput.clear();
    await labelValueInput.fill("test");
    await labelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(labelKeyInput).toHaveValue("env");
    await expect(labelValueInput).toHaveValue("test");

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") && response.request().method() === "POST" && !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );

    await page.locator('[data-test="send-event-submit-button"]').click();
    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    // Wait for events list to show
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Click on the event ID link in the first row
    const rows = page.locator('[data-test="events-table"] [row-id]');
    await expect(rows.first()).toBeVisible();
    await rows.first().locator('[data-test="event-id-link"]').click();

    // Verify we're on the event detail page
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-page"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-type"]')).toContainText(env.eventTypeName);
  });
});
