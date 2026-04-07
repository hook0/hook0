import { test, expect } from "@playwright/test";
import { loginAndCreateApp } from "../fixtures/test-setup";

/**
 * Logs (Request Attempts) E2E tests for Hook0.
 *
 * Tests for viewing the request attempts list.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Logs", () => {
  test("should display logs page with empty state", async ({ page, request }) => {
    const env = await loginAndCreateApp(page, request, "empty");

    // Navigate to logs page
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);

    // Verify logs card is visible
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // Empty state should be shown (no request attempts yet)
    // The card should have content indicating no requests
    const cardContent = page.locator('[data-test="logs-card"]');
    await expect(cardContent).toContainText(/Delivery Logs|did not send any requests/);
  });

  test("should display logs card header correctly", async ({ page, request }) => {
    const env = await loginAndCreateApp(page, request, "header");

    // Navigate to logs page
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);

    // Verify logs card is visible and has correct header
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // Check the header contains "Request Attempts"
    const cardHeader = page.locator('[data-test="logs-card"]');
    await expect(cardHeader).toContainText("Delivery Logs");
    await expect(cardHeader).toContainText("Every webhook delivery attempt");
  });

  test("should display log rows with event links that navigate to event detail", async ({
    page,
    request,
  }) => {
    test.slow(); // This test creates event type + subscription + sends event + navigates multiple pages
    const env = await loginAndCreateApp(page, request, "event-link");

    // Create an event type
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="event-type-service-input"]').fill("link");
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
      .fill(`Link Test Sub ${env.timestamp}`);
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");
    await page.locator('[data-test="subscription-url-input"]').fill("https://webhook.site/test");

    // Add labels
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
    await page.waitForURL('**/events/send');
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="send-event-type-select"]')
      .selectOption("link.test.created");

    // Add event labels
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

    // After send, page navigates to event detail — wait for it before navigating away
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });

    // Navigate to logs and wait for data
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/logs`
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="logs-table"] [row-id]').first()).toBeVisible({
      timeout: 15000,
    });

    // Verify the event name link is displayed in the log row
    const eventLink = page.locator('[data-test="log-event-link"]').first();
    await expect(eventLink).toBeVisible();
    await expect(eventLink).toContainText("link.test.created");

    // Click the event link to navigate to event detail
    await eventLink.click();

    // Verify event detail page renders
    await expect(page).toHaveURL(/\/events\//, { timeout: 10000 });
    await expect(page.locator('[data-test="event-detail-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should display logs table after sending event with subscription", async ({
    page,
    request,
  }) => {
    test.slow(); // This test creates event type + subscription + sends event + navigates multiple pages
    const env = await loginAndCreateApp(page, request, "with-logs");

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
    const eventTypeResponse = await createEventTypeResponse;
    expect(eventTypeResponse.status()).toBeLessThan(400);

    // Wait for navigation after event type creation
    await expect(page).toHaveURL(/\/event_types$/, { timeout: 10000 });

    // Verify event type appears in the list (confirms data is persisted)
    await expect(page.locator('[data-test="event-types-table"]')).toBeVisible({ timeout: 10000 });

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

    // Add a label using data-test selectors (scoped to subscription-labels container)
    const labelKeyInput = page.locator('[data-test="subscription-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="subscription-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await labelKeyInput.clear();
    await labelKeyInput.fill("all");
    await labelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await labelValueInput.clear();
    await labelValueInput.fill("yes");
    await labelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(labelKeyInput).toHaveValue("all");
    await expect(labelValueInput).toHaveValue("yes");

    // Select event type using data-test selector
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

    // Wait for navigation after subscription creation (router.back() is called)
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Send an event
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="events-send-button"]').click();
    await page.waitForURL('**/events/send');

    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="send-event-type-select"]')
      .selectOption("billing.invoice.created");

    // Add labels (required for event submission, and must match subscription labels)
    // Use data-test selectors (scoped to send-event-labels container)
    const eventLabelKeyInput = page.locator('[data-test="send-event-labels"] [data-test="kv-key-input-0"]');
    const eventLabelValueInput = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    await expect(eventLabelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await eventLabelKeyInput.clear();
    await eventLabelKeyInput.fill("all");
    await eventLabelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await eventLabelValueInput.clear();
    await eventLabelValueInput.fill("yes");
    await eventLabelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(eventLabelKeyInput).toHaveValue("all");
    await expect(eventLabelValueInput).toHaveValue("yes");

    const now = new Date();
    const dateTimeValue = now.toISOString().slice(0, 16);
    await page.locator('[data-test="send-event-occurred-at-input"]').fill(dateTimeValue);

    const sendEventResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event") && response.request().method() === "POST" && !response.url().includes("/api/v1/event_types"),
      { timeout: 15000 }
    );
    await page.locator('[data-test="send-event-submit-button"]').click();
    await sendEventResponse;

    // After send, page navigates to event detail — wait for it before navigating away
    await expect(page).toHaveURL(/\/events\/[^/]+$/, { timeout: 10000 });

    // Navigate to logs page and poll for data to appear (webhook processing takes time)
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    // Poll for at least one row to appear (webhook might take a moment to be processed)
    // Use expect with longer timeout instead of arbitrary waitForTimeout
    await expect(page.locator('[data-test="logs-table"] [row-id]').first()).toBeVisible({ timeout: 15000 });

    // The logs table should have at least 1 row (the webhook request)
    const rows = page.locator('[data-test="logs-table"] [row-id]');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);
  });
});
