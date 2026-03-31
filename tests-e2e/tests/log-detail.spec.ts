import { test, expect } from "@playwright/test";
import { loginAndCreateApp } from "../fixtures/test-setup";

/**
 * Log Detail E2E tests for Hook0.
 *
 * Tests for the drawer (LogSidePanel) and LogDetail full page flows.
 */
test.describe("Log Detail", () => {
  /**
   * Helper to set up an environment with an event type, subscription, and sent event.
   * Navigates to the logs page and waits for at least one log row to appear.
   */
  async function setupLogsWithDelivery(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const env = await loginAndCreateApp(page, request, `log-${testId}`);

    // Create an event type
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="event-type-service-input"]').fill("log");
    await page.locator('[data-test="event-type-resource-input"]').fill("test");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");

    const createETResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createETResponse;
    await expect(page).toHaveURL(/\/event_types$/, { timeout: 10000 });

    // Create a subscription
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="subscription-description-input"]')
      .fill(`Log Test Sub ${env.timestamp}`);
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");
    await page.locator('[data-test="subscription-url-input"]').fill("https://webhook.site/test");

    const labelKeyInput = page.locator(
      '[data-test="subscription-labels"] [data-test="kv-key-input-0"]'
    );
    const labelValueInput = page.locator(
      '[data-test="subscription-labels"] [data-test="kv-value-input-0"]'
    );
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });
    await labelKeyInput.clear();
    await labelKeyInput.fill("all");
    await labelKeyInput.blur();
    await labelValueInput.clear();
    await labelValueInput.fill("yes");
    await labelValueInput.blur();
    await expect(labelKeyInput).toHaveValue("all");
    await expect(labelValueInput).toHaveValue("yes");

    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 15000 });
    await eventTypeCheckbox.click();

    const createSubResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="subscription-submit-button"]').click();
    await createSubResponse;
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Send an event
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="events-send-button"]').click();
    await page.waitForURL("**/events/send");
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="send-event-type-select"]')
      .selectOption("log.test.created");

    const eventLabelKey = page.locator(
      '[data-test="send-event-labels"] [data-test="kv-key-input-0"]'
    );
    const eventLabelValue = page.locator(
      '[data-test="send-event-labels"] [data-test="kv-value-input-0"]'
    );
    await expect(eventLabelKey).toBeVisible({ timeout: 5000 });
    await eventLabelKey.clear();
    await eventLabelKey.fill("all");
    await eventLabelKey.blur();
    await eventLabelValue.clear();
    await eventLabelValue.fill("yes");
    await eventLabelValue.blur();
    await expect(eventLabelKey).toHaveValue("all");
    await expect(eventLabelValue).toHaveValue("yes");

    const now = new Date();
    await page
      .locator('[data-test="send-event-occurred-at-input"]')
      .fill(now.toISOString().slice(0, 16));

    const sendEventResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") &&
        response.request().method() === "POST" &&
        !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );
    await page.locator('[data-test="send-event-submit-button"]').click();
    await sendEventResponse;

    // After send, page navigates to event detail
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });

    // Navigate to logs and wait for at least one row
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="logs-table"] [row-id]').first()).toBeVisible({
      timeout: 15000,
    });

    return env;
  }

  /**
   * Helper to poll until a log row has response data available (webhook processing takes time).
   * Reloads the page until at least one row is visible with data.
   */
  async function waitForResponseInRow(page: import("@playwright/test").Page) {
    await expect(async () => {
      await page.reload();
      await expect(page.locator('[data-test="logs-table"]')).toBeVisible({ timeout: 10000 });
      await expect(
        page.locator('[data-test="logs-table"] [row-id]').first()
      ).toBeVisible({ timeout: 5000 });
    }).toPass({ timeout: 30000 });
  }

  test("should open drawer with event and response when clicking a log row", async ({
    page,
    request,
  }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "drawer-open");
    await waitForResponseInRow(page);

    // Click on the first log row
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    await firstRow.click();

    // Verify the drawer (side panel) opens
    const sidePanel = page.locator('[data-test="side-panel"]');
    await expect(sidePanel).toBeVisible({ timeout: 10000 });

    // Verify the drawer contains event metadata (event_type)
    await expect(sidePanel).toContainText("log.test.created");

    // Verify the drawer contains event payload section
    await expect(sidePanel).toContainText("Payload");

    // Verify the drawer contains response summary with HTTP status code
    await expect(sidePanel).toContainText("Response Summary");
    await expect(sidePanel).toContainText("HTTP Status Code");

    // Verify the drawer contains response headers section
    await expect(sidePanel).toContainText("Response Headers");

    // Verify the drawer contains response body section
    await expect(sidePanel).toContainText("Response Body");
  });

  test("should navigate to LogDetail page via Open full page button", async ({
    page,
    request,
  }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "full-page");
    await waitForResponseInRow(page);

    // Click on the first log row to open the drawer
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    await firstRow.click();

    // Verify the drawer opens
    const sidePanel = page.locator('[data-test="side-panel"]');
    await expect(sidePanel).toBeVisible({ timeout: 10000 });

    // Click the "Open full page" button
    await page.locator('[data-test="log-panel-full-page"]').click();

    // Verify URL changed to /logs/[request_attempt_id] (UUID pattern)
    await expect(page).toHaveURL(
      /\/logs\/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/,
      { timeout: 10000 }
    );

    // Verify the log-detail-page test element is visible
    await expect(page.locator('[data-test="log-detail-page"]')).toBeVisible({ timeout: 10000 });

    // Verify it shows the same content sections as the drawer
    const detailPage = page.locator('[data-test="log-detail-page"]');
    await expect(detailPage).toContainText("log.test.created");
    await expect(detailPage).toContainText("Payload");
    await expect(detailPage).toContainText("Response Summary");
  });

  test("should show error for non-existent request attempt", async ({ page, request }) => {
    test.slow();
    const env = await loginAndCreateApp(page, request, "log-not-found");

    // Navigate directly to a non-existent request attempt
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/00000000-0000-0000-0000-000000000000`
    );

    // Verify error card appears (Hook0ErrorCard renders when the query fails)
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });
  });

  test("should redirect old response detail URLs to logs", async ({ page, request }) => {
    test.slow();
    const env = await loginAndCreateApp(page, request, "log-redirect");

    // Navigate to the old URL pattern
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/responses/00000000-0000-0000-0000-000000000000`
    );

    // Verify the page redirected to the logs list
    await expect(page).toHaveURL(
      new RegExp(
        `/organizations/${env.organizationId}/applications/${env.applicationId}/logs$`
      ),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should navigate back from LogDetail to logs", async ({ page, request }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "back-nav");
    await waitForResponseInRow(page);

    // Click on the first log row to open the drawer
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    await firstRow.click();

    // Open full page
    const sidePanel = page.locator('[data-test="side-panel"]');
    await expect(sidePanel).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="log-panel-full-page"]').click();

    // Verify we're on the LogDetail page
    await expect(page.locator('[data-test="log-detail-page"]')).toBeVisible({ timeout: 10000 });

    // Go back
    await page.goBack();

    // Verify we're back on the logs page
    await expect(page).toHaveURL(
      new RegExp(
        `/organizations/${env.organizationId}/applications/${env.applicationId}/logs$`
      ),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });
});
