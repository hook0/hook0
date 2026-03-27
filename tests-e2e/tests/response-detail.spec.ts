import { test, expect } from "@playwright/test";
import { loginAndCreateApp } from "../fixtures/test-setup";

/**
 * Response Detail E2E tests for Hook0.
 *
 * Tests for viewing webhook delivery response details from the logs table.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Response Detail", () => {
  /**
   * Helper to set up an environment with an event type, subscription, and sent event.
   * Navigates to the logs page and waits for at least one log row to appear.
   */
  async function setupLogsWithDelivery(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const env = await loginAndCreateApp(page, request, `resp-${testId}`);

    // Create an event type
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="event-type-service-input"]').fill("resp");
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
      .fill(`Resp Test Sub ${env.timestamp}`);
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
      .selectOption("resp.test.created");

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
   * Helper to poll until a response link appears in the logs table.
   * Reloads the page until the response_id is populated (webhook processing takes time).
   */
  async function waitForResponseLink(page: import("@playwright/test").Page) {
    const responseLink = page.locator('[data-test="log-response-link"]').first();
    await expect(async () => {
      await page.reload();
      await expect(page.locator('[data-test="logs-table"]')).toBeVisible({ timeout: 10000 });
      await expect(responseLink).toBeVisible({ timeout: 5000 });
    }).toPass({ timeout: 30000 });
    return responseLink;
  }

  test("should display Response column header in logs table", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "col-header");

    // Verify the Response column header is visible in the table
    const table = page.locator('[data-test="logs-table"]');
    await expect(table).toContainText("Response");
  });

  test("should show truncated response ID link when response exists", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "resp-link");

    // Poll until a response link appears
    const responseLink = await waitForResponseLink(page);

    // The link should render a truncated UUID (contains the ellipsis character from Hook0Uuid)
    const linkText = await responseLink.textContent();
    expect(linkText).toBeTruthy();
    expect(linkText!.length).toBeGreaterThan(0);
    // Hook0Uuid with truncated=true renders middle-truncated text with \u2026 (…)
    expect(linkText).toContain("\u2026");
  });

  test("should navigate to response detail page when clicking response ID", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "nav-detail");

    // Poll until a response link appears
    const responseLink = await waitForResponseLink(page);

    // Click the response ID link
    await responseLink.click();

    // Verify URL changed to the response detail page
    await expect(page).toHaveURL(/\/logs\/responses\/[0-9a-f-]+/, { timeout: 10000 });

    // Verify response detail card is visible
    await expect(page.locator('[data-test="response-detail-card"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display response detail with all sections", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "detail-sections");

    const responseLink = await waitForResponseLink(page);

    // Navigate to response detail
    await responseLink.click();
    await expect(page).toHaveURL(/\/logs\/responses\/[0-9a-f-]+/, { timeout: 10000 });

    const detailPage = page.locator('[data-test="response-detail-page"]');
    await expect(detailPage).toBeVisible({ timeout: 10000 });

    // Verify the summary card is present with expected fields
    const summaryCard = page.locator('[data-test="response-detail-card"]');
    await expect(summaryCard).toBeVisible();
    await expect(summaryCard).toContainText("Response Summary");
    await expect(summaryCard).toContainText("Response ID");
    await expect(summaryCard).toContainText("HTTP Status Code");
    await expect(summaryCard).toContainText("Elapsed Time");

    // Verify headers section is present
    const headersCard = page.locator('[data-test="response-headers-card"]');
    await expect(headersCard).toBeVisible();
    await expect(headersCard).toContainText("Response Headers");

    // Verify body section is present
    const bodyCard = page.locator('[data-test="response-body-card"]');
    await expect(bodyCard).toBeVisible();
    await expect(bodyCard).toContainText("Response Body");
  });

  test("should not display sensitive headers in response detail", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "no-sensitive");

    const responseLink = await waitForResponseLink(page);

    // Navigate to response detail
    await responseLink.click();
    await expect(page).toHaveURL(/\/logs\/responses\/[0-9a-f-]+/, { timeout: 10000 });

    const headersCard = page.locator('[data-test="response-headers-card"]');
    await expect(headersCard).toBeVisible({ timeout: 10000 });

    // Sensitive headers should never be displayed
    const headersText = await headersCard.textContent();
    const lower = headersText!.toLowerCase();
    expect(lower).not.toContain("set-cookie");
    expect(lower).not.toContain("authorization");
    expect(lower).not.toContain("www-authenticate");
    expect(lower).not.toContain("proxy-authorization");
    expect(lower).not.toContain("proxy-authenticate");
  });

  test("should show event link button on response detail page", async ({ page, request }) => {
    test.slow();
    await setupLogsWithDelivery(page, request, "event-link");

    const responseLink = await waitForResponseLink(page);

    // Navigate to response detail
    await responseLink.click();
    await expect(page).toHaveURL(/\/logs\/responses\/[0-9a-f-]+/, { timeout: 10000 });
    await expect(page.locator('[data-test="response-detail-card"]')).toBeVisible({ timeout: 10000 });

    // The event link button should be visible (event_id is passed via query param)
    const eventLink = page.locator('[data-test="response-event-link"]');
    await expect(eventLink).toBeVisible();

    // Clicking it should navigate to the event detail page
    await eventLink.click();
    await expect(page).toHaveURL(/\/events\/[0-9a-f-]+$/, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-page"]')).toBeVisible({ timeout: 10000 });
  });

  test("should navigate back to logs from response detail", async ({ page, request }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "back-nav");

    const responseLink = await waitForResponseLink(page);

    // Navigate to response detail
    await responseLink.click();
    await expect(page.locator('[data-test="response-detail-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Go back
    await page.goBack();

    // Verify we're back on the logs page
    await expect(page).toHaveURL(
      new RegExp(
        `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
      ),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });
});
