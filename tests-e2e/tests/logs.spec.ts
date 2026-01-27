import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Logs (Request Attempts) E2E tests for Hook0.
 *
 * Tests for viewing the request attempts list.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Logs", () => {
  /**
   * Helper to setup test environment: user, organization, and application
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-logs-${testId}-${timestamp}@hook0.local`;
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

    // If applicationId wasn't captured, extract from URL after navigation
    if (!applicationId) {
      await expect(page).toHaveURL(/\/applications\/[^/]+/, { timeout: 10000 });
      const url = page.url();
      const match = url.match(/\/applications\/([^/]+)/);
      if (match) {
        applicationId = match[1];
      }
    }
    expect(applicationId).toBeTruthy();

    return {
      email,
      password,
      organizationId,
      applicationId,
      timestamp,
    };
  }

  test("should display logs page with empty state", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "empty");

    // Navigate to logs page
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);

    // Verify logs card is visible
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // Empty state should be shown (no request attempts yet)
    // The card should have content indicating no requests
    const cardContent = page.locator('[data-test="logs-card"]');
    await expect(cardContent).toContainText(/Request Attempts|did not send any requests/);
  });

  test("should display logs card header correctly", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "header");

    // Navigate to logs page
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);

    // Verify logs card is visible and has correct header
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // Check the header contains "Request Attempts"
    const cardHeader = page.locator('[data-test="logs-card"]');
    await expect(cardHeader).toContainText("Request Attempts");
    await expect(cardHeader).toContainText("Last webhooks sent by Hook0");
  });

  test("should display logs table after sending event with subscription", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "with-logs");

    // Create an event type
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("billing");
    await page.locator('[data-test="event-type-resource-input"]').fill("invoice");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createEventTypeResponse;

    // Create a subscription
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page
      .locator('[data-test="subscription-description-input"]')
      .fill(`Test Subscription ${env.timestamp}`);
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");
    await page.locator('[data-test="subscription-url-input"]').fill("https://webhook.site/test");

    // Add a label
    const labelKeyInput = page.locator('input[placeholder="Label key"]').first();
    const labelValueInput = page.locator('input[placeholder="Label value"]').first();
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });
    await labelKeyInput.fill("all");
    await labelValueInput.fill("yes");

    // Select event type
    const eventTypeCheckbox = page.locator('input[type="checkbox"]').first();
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 5000 });
    await eventTypeCheckbox.click();

    const createSubResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="subscription-submit-button"]').click();
    await createSubResponse;

    // Wait for navigation after subscription creation (router.back() is called)
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Send an event
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="events-send-button"]').click();

    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="send-event-type-select"]')
      .selectOption("billing.invoice.created");

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    const sendEventResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/events") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="send-event-submit-button"]').click();
    await sendEventResponse;

    // Wait a bit for the webhook to be sent
    await page.waitForTimeout(2000);

    // Navigate to logs page
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);

    // Verify logs card is visible
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // The logs table should have at least 1 row (the webhook request)
    const rows = page.locator('[data-test="logs-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);
  });
});
