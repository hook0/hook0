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
      organizationId: organizationId!,
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

  /**
   * Helper to send a test event: navigates, opens form, fills labels, submits.
   * Returns the API response promise.
   */
  async function sendTestEvent(
    page: import("@playwright/test").Page,
    env: { organizationId: string; applicationId: string; eventTypeName: string },
  ) {
    // Navigate to events
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/events`);
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="events-send-button"]').click();
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });

    // Select event type
    await page.locator('[data-test="send-event-type-select"]').selectOption(env.eventTypeName);

    // Fill labels
    const labelKey = page.locator('[data-test="send-event-labels"] [data-test="kv-key-input-0"]');
    const labelValue = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKey).toBeVisible({ timeout: 5000 });
    await labelKey.clear();
    await labelKey.fill("all");
    await labelKey.blur();
    await labelValue.clear();
    await labelValue.fill("yes");
    await labelValue.blur();
    await expect(labelKey).toHaveValue("all");
    await expect(labelValue).toHaveValue("yes");

    // Set occurred_at
    const now = new Date();
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(now.toISOString().slice(0, 16));

    // Submit
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") && response.request().method() === "POST" && !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );
    await page.locator('[data-test="send-event-submit-button"]').click();
    return responsePromise;
  }

  test("should send test event and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "send");

    const response = await sendTestEvent(page, env);

    // Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown using data-test selector
    await expect(page.locator('[data-sonner-toast]').first()).toBeVisible({
      timeout: 10000,
    });

    // Verify form is closed and events list is shown
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should display events list with sent event", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list-with-event");

    const response = await sendTestEvent(page, env);
    expect(response.status()).toBeLessThan(400);

    // Wait for events list to refresh
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Verify events table has at least 1 row (wait for table data to load)
    const rows = page.locator('[data-test="events-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

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

  test("should add and remove labels when sending event", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "kv-labels");

    // Navigate to events page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );

    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });

    // Open send event form
    await page.locator('[data-test="events-send-button"]').click();
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });

    // Row 0 already exists (KV component starts with one empty pair).
    // The remove button is disabled when there's only 1 row.
    // Add a second row by clicking the add button on row 0.
    const addButton = page.locator('[data-test="kv-add-button-0"]');
    await expect(addButton).toBeVisible({ timeout: 5000 });
    await addButton.click();

    // Fill the new row (index 1)
    const keyInput1 = page.locator('[data-test="kv-key-input-1"]');
    const valueInput1 = page.locator('[data-test="kv-value-input-1"]');
    await expect(keyInput1).toBeVisible({ timeout: 5000 });
    await keyInput1.fill("test-key");
    await keyInput1.blur();
    await valueInput1.fill("test-value");
    await valueInput1.blur();

    // Verify inputs have values
    await expect(keyInput1).toHaveValue("test-key");
    await expect(valueInput1).toHaveValue("test-value");

    // Now remove row 1 (enabled because there are 2 rows)
    const removeButton = page.locator('[data-test="kv-remove-button-1"]');
    await expect(removeButton).toBeVisible({ timeout: 5000 });
    await expect(removeButton).toBeEnabled();
    await removeButton.click();

    // Verify the second row is removed
    await expect(keyInput1).not.toBeVisible({ timeout: 5000 });
  });

  test("should open side panel when clicking event row", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "side-panel");

    const response = await sendTestEvent(page, env);
    expect(response.status()).toBeLessThan(400);

    // Wait for events list
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Wait for event row to appear
    const rows = page.locator('[data-test="events-table"] [row-id]');
    await expect(rows.first()).toBeVisible({ timeout: 10000 });

    // Click on the event row to open side panel
    await rows.first().click();

    // Verify side panel is visible
    await expect(page.locator('[data-test="side-panel"]')).toBeVisible({ timeout: 10000 });

    // Close side panel
    await page.locator('[data-test="side-panel-close"]').click();

    // Verify side panel is hidden
    await expect(page.locator('[data-test="side-panel"]')).not.toBeVisible({ timeout: 10000 });
  });

  test("should navigate to event detail page", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "detail");

    const response = await sendTestEvent(page, env);
    expect(response.status()).toBeLessThan(400);

    // Wait for events list to show
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Click on the event ID link in the first row
    const rows = page.locator('[data-test="events-table"] [row-id]');
    await expect(rows.first()).toBeVisible();
    const eventLink = rows.first().locator('a').first();
    await expect(eventLink).toBeVisible({ timeout: 5000 });
    await eventLink.click();

    // Verify we're on the event detail page
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-page"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-type"]')).toContainText(env.eventTypeName);
  });

  test("should display event detail page with payload and metadata", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "detail-full");

    const response = await sendTestEvent(page, env);
    expect(response.status()).toBeLessThan(400);

    // Wait for events list
    await expect(page.locator('[data-test="events-card"]')).toBeVisible({ timeout: 10000 });

    // Click on the event ID link to navigate to the detail page
    const rows = page.locator('[data-test="events-table"] [row-id]');
    await expect(rows.first()).toBeVisible({ timeout: 10000 });
    const eventLink = rows.first().locator('a').first();
    await expect(eventLink).toBeVisible({ timeout: 5000 });
    await eventLink.click();

    // Verify we're on the event detail page
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-page"]')).toBeVisible({ timeout: 10000 });

    // Verify event type is displayed
    await expect(page.locator('[data-test="event-detail-type"]')).toContainText(env.eventTypeName);

    // Verify the detail card contains event metadata (occurred_at, received_at, source IP)
    const detailCard = page.locator('[data-test="event-detail-card"]');
    await expect(detailCard).toBeVisible({ timeout: 10000 });
    await expect(detailCard).toContainText(env.eventTypeName);

    // Verify payload section is visible (the page has 4 cards: detail, metadata, labels, payload)
    const detailPage = page.locator('[data-test="event-detail-page"]');
    await expect(detailPage).toContainText('application/json', { timeout: 30000 });

    // Verify labels section displays the label we sent
    await expect(detailPage).toContainText("all", { timeout: 15000 });
    await expect(detailPage).toContainText("yes", { timeout: 15000 });

    // Verify payload content is displayed (we sent the default '{"test": true}')
    await expect(detailPage).toContainText("test", { timeout: 15000 });
  });
});
