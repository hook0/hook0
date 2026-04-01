import { test, expect } from "@playwright/test";
import { loginAndCreateApp } from "../fixtures/test-setup";

/**
 * Log Detail E2E tests for Hook0.
 *
 * Tests for the split-panel detail view and LogDetail full page flows.
 */
test.describe("Log Detail", () => {
  /**
   * Helper to set up an environment with an event type, subscription, and sent event.
   * Navigates to the logs page and waits for at least one log row to appear.
   * Uses the same proven setup pattern as logs.spec.ts.
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

    // Add labels (required for event matching)
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

    // Select event type
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 15000 });
    await eventTypeCheckbox.click();

    const createSubResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="subscription-submit-button"]').click();
    await createSubResponse;
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Send an event via UI (same pattern as logs.spec.ts)
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

    // Add event labels matching the subscription
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

    // Wait for navigation to event detail before navigating away
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });

    // Navigate to logs and wait for data
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    return env;
  }

  async function waitForLogRow(page: import("@playwright/test").Page) {
    await expect(async () => {
      await page.reload();
      await expect(
        page.locator('[data-test="logs-table"] [row-id]').first()
      ).toBeVisible({ timeout: 10000 });
    }).toPass({ timeout: 60000, intervals: [2000] });
  }

  test("should show delivery detail in split panel when clicking a log row", async ({
    page,
    request,
  }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "drawer-open");
    await waitForLogRow(page);

    // Click on the first log row
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    await firstRow.click();

    // Verify the detail panel shows content (scoped to the detail side of the split)
    const detail = page.locator('.log-split__detail');
    await expect(detail.getByText("log.test.created")).toBeVisible({ timeout: 10000 });
    await expect(detail.getByText("Request")).toBeVisible();
    await expect(detail.getByText("Payload")).toBeVisible();
    await expect(detail.getByText("Lifecycle")).toBeVisible();

    // Verify the URL has a delivery query param
    await expect(page).toHaveURL(/delivery=/, { timeout: 5000 });
  });

  test("should navigate to LogDetail full page", async ({
    page,
    request,
  }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "full-page");
    await waitForLogRow(page);

    // Get the first row's ID
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    const rowId = await firstRow.getAttribute("row-id");

    // Navigate directly to the full page detail
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/${rowId}`
    );

    // Verify the log-detail-page test element is visible
    await expect(page.locator('[data-test="log-detail-page"]')).toBeVisible({ timeout: 10000 });

    // Verify it shows the same content sections
    const detailPage = page.locator('[data-test="log-detail-page"]');
    await expect(detailPage).toContainText("log.test.created");
    await expect(detailPage).toContainText("Payload");
    await expect(detailPage).toContainText("Lifecycle");
  });

  test("should show error for non-existent request attempt", async ({ page, request }) => {
    test.slow();
    const env = await loginAndCreateApp(page, request, "log-not-found");

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/00000000-0000-0000-0000-000000000000`
    );

    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });
  });

  test("should redirect old response detail URLs to logs", async ({ page, request }) => {
    test.slow();
    const env = await loginAndCreateApp(page, request, "log-redirect");

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/responses/00000000-0000-0000-0000-000000000000`
    );

    await expect(page).toHaveURL(
      new RegExp(
        `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
      ),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should navigate back from LogDetail to logs", async ({ page, request }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "back-nav");
    await waitForLogRow(page);

    // Get the first row's ID and navigate to full page
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    const rowId = await firstRow.getAttribute("row-id");

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs/${rowId}`
    );

    await expect(page.locator('[data-test="log-detail-page"]')).toBeVisible({ timeout: 10000 });

    // Go back
    await page.goBack();

    await expect(page).toHaveURL(
      new RegExp(
        `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
      ),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });
});
