import { test, expect } from "@playwright/test";
import {
  loginAndCreateApp,
  selectEventType,
  submitEventWithLabels,
  submitSubscriptionWithLabels,
} from "../fixtures/test-setup";
import { API_BASE_URL } from "../fixtures/email-verification";

/**
 * Log Detail E2E tests for Hook0.
 *
 * Tests for the split-panel detail view and LogDetail full page flows.
 */
test.describe("Log Detail", () => {
  /**
   * Set a key/value field and leave it holding that value. Filling once is not
   * enough here: the form re-renders while it is being built, and a field that
   * held the right thing a moment ago is not the same claim as a field that
   * holds it when the form is submitted.
   */
  async function setKeyValue(input: import("@playwright/test").Locator, value: string) {
    if ((await input.inputValue()) !== value) {
      await input.clear();
      await input.fill(value);
      await input.blur();
    }
    await expect(input).toHaveValue(value);
  }

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
    await setKeyValue(labelKeyInput, "all");
    await setKeyValue(labelValueInput, "yes");

    // Select event type
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await selectEventType(eventTypeCheckbox);

    // Ticking the box re-renders the form, and these two fields sit above it.
    // The editor rebuilds a row from what the form holds, so a field settled
    // before the tick can come back holding the form's default instead.
    await setKeyValue(labelKeyInput, "all");
    await setKeyValue(labelValueInput, "yes");

    await submitSubscriptionWithLabels(page, { all: "yes" });
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Send an event via UI (same pattern as logs.spec.ts)
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events`
    );
    await expect(page.locator('[data-test="events-send-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="events-send-button"]').click();
    await page.waitForURL("**/events/send");
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="send-event-type-select"]').selectOption("log.test.created");

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

    await submitEventWithLabels(page, { all: "yes" });

    // Wait for navigation to event detail before navigating away. The send form
    // itself lives at /events/send, which a looser pattern also matches, so the
    // check would hold on the very page it is meant to prove we left.
    await expect(page).toHaveURL(/\/events\/(?!send$)[^/]+$/, { timeout: 10000 });

    // Navigate to logs and wait for data
    await page.goto(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`);
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });

    return env;
  }

  async function waitForLogRow(page: import("@playwright/test").Page, applicationId: string) {
    const accessToken = await page.evaluate(() => {
      const stored = window.localStorage.getItem("auth");
      if (stored === null) {
        return null;
      }
      return (JSON.parse(stored) as { accessToken: string }).accessToken;
    });
    expect(accessToken, "the browser holds no session, so the API cannot be asked").not.toBeNull();

    // A database trigger writes the delivery attempt in the same transaction as
    // the event, once per enabled subscription whose label pair the event
    // carries. The row is therefore already there when the event POST answers,
    // or it is never coming. Reloading the page for a minute, which is what this
    // used to do, could only ever turn that difference into "element not found".
    let attemptCount = 0;
    await expect(async () => {
      const response = await page.request.get(
        `${API_BASE_URL}/request_attempts?application_id=${applicationId}`,
        { headers: { Authorization: `Bearer ${accessToken!}` } }
      );
      expect(
        response.status(),
        `listing delivery attempts answered ${response.status()}: ${await response.text()}`
      ).toBe(200);
      const attempts = (await response.json()) as unknown[];
      attemptCount = attempts.length;
      expect(
        attemptCount,
        "the API lists no delivery attempt for this application"
      ).toBeGreaterThan(0);
    })
      .toPass({ timeout: 45000, intervals: [1500] })
      .catch(async (error: Error) => {
        // An empty list says nothing about which side of the match came up
        // short, and the event POST answering 2xx does not say what the event
        // was stored as carrying. Both sides are read back, so the run names
        // the side instead of leaving it to be guessed from outside.
        const readBack = (collection: string) =>
          page.request
            .get(`${API_BASE_URL}/${collection}?application_id=${applicationId}`, {
              headers: { Authorization: `Bearer ${accessToken!}` },
            })
            .then((response) => response.text())
            .catch(() => "(could not be read)");
        const [subscriptions, events] = await Promise.all([
          readBack("subscriptions"),
          readBack("events"),
        ]);
        throw new Error(
          `${error.message}\nsubscriptions on this application: ${subscriptions}` +
            `\nevents on this application: ${events}`
        );
      });

    await page.reload();
    await expect(
      page.locator('[data-test="logs-table"] [row-id]').first(),
      `the API reports ${attemptCount} delivery attempt(s), so the table should show at least one`
    ).toBeVisible({ timeout: 15000 });
  }

  test("should show delivery detail in split panel when clicking a log row", async ({
    page,
    request,
  }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "drawer-open");
    await waitForLogRow(page, env.applicationId);

    // Click on the status column of the first row (not the event link which navigates away)
    const firstRow = page.locator('[data-test="logs-table"] [row-id]').first();
    await firstRow.locator(".log-status").click();

    // Verify the detail panel shows content (scoped to the detail side of the split)
    const detail = page.locator(".hook0-split-layout__detail");
    await expect(detail.getByText("log.test.created")).toBeVisible({ timeout: 10000 });
    await expect(detail.getByText("Request")).toBeVisible();
    await expect(detail.getByText("Payload")).toBeVisible();
    await expect(detail.getByText("Lifecycle")).toBeVisible();

    // Verify the URL has a delivery query param
    await expect(page).toHaveURL(/delivery=/, { timeout: 5000 });
  });

  test("should navigate to LogDetail full page", async ({ page, request }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "full-page");
    await waitForLogRow(page, env.applicationId);

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
      new RegExp(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });

  test("should navigate back from LogDetail to logs", async ({ page, request }) => {
    test.slow();
    const env = await setupLogsWithDelivery(page, request, "back-nav");
    await waitForLogRow(page, env.applicationId);

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
      new RegExp(`/organizations/${env.organizationId}/applications/${env.applicationId}/logs`),
      { timeout: 10000 }
    );
    await expect(page.locator('[data-test="logs-card"]')).toBeVisible({ timeout: 10000 });
  });
});
