import { test, expect } from "@playwright/test";

/**
 * Extracts the token from the page URL hash.
 */
async function getToken(page: import("@playwright/test").Page): Promise<string> {
  return page.evaluate(() => location.hash.replace(/^#/, ""));
}

/**
 * Waits for the WebSocket connection to be established.
 */
async function waitForConnection(page: import("@playwright/test").Page): Promise<void> {
  await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
    timeout: 15000,
  });
}

test.describe("Play Webhook Flow", () => {
  test("sending a POST webhook makes it appear in the feed", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send a webhook via fetch from the browser context
    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ event: "test.created", id: 1 }),
    });
    expect(response.status()).toBeLessThan(400);

    // Verify it appears in the feed
    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });

    // Verify the method badge shows POST
    await expect(feedItem.locator(".method-badge")).toHaveText("POST");

    // Verify counter updated
    await expect(page.locator("#webhookCounter")).toHaveText(
      "1 webhook received",
    );
  });

  test("multiple webhooks appear in newest-first order", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send first webhook (GET)
    await page.request.get(webhookUrl);
    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    // Send second webhook (POST)
    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ order: "second" }),
    });
    await expect(page.locator(".feed-item")).toHaveCount(2, { timeout: 10000 });

    // Newest (POST) should be first in the list
    const firstBadge = page.locator(".feed-item").first().locator(".method-badge");
    await expect(firstBadge).toHaveText("POST");

    const secondBadge = page.locator(".feed-item").nth(1).locator(".method-badge");
    await expect(secondBadge).toHaveText("GET");
  });

  test("clicking a webhook entry shows detail panel with method, headers, body", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      headers: {
        "Content-Type": "application/json",
        "X-Test-Header": "play-test-value",
      },
      data: JSON.stringify({ action: "inspect.me" }),
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    // Detail panel should be visible with content
    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    // Method badge in detail
    await expect(detailContent.locator(".method-badge")).toHaveText("POST");

    // Headers table should contain our custom header
    await expect(detailContent.locator(".headers-table")).toContainText(
      "x-test-header",
    );

    // Body should contain the JSON we sent
    await expect(detailContent.locator(".body-display")).toContainText(
      "inspect.me",
    );
  });

  test("deleting a single webhook removes it from the feed", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      data: JSON.stringify({ will: "be deleted" }),
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    // Wait for detail content to render
    const deleteBtn = page.locator('#detailContent [data-action="delete"]');
    await expect(deleteBtn).toBeVisible({ timeout: 5000 });

    // Scroll to top of detail panel to ensure delete button is in view
    await deleteBtn.scrollIntoViewIfNeeded();

    // Click the delete button in the detail panel
    await deleteBtn.click({ force: true });

    // Feed should now be empty
    await expect(page.locator(".feed-item")).toHaveCount(0, { timeout: 10000 });
    await expect(page.locator("#webhookCounter")).toHaveText(
      "0 webhooks received",
    );
  });

  test("clear all webhooks empties the feed", async ({ page, baseURL }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send two webhooks
    await page.request.post(webhookUrl, { data: "first" });
    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    await page.request.post(webhookUrl, { data: "second" });
    await expect(page.locator(".feed-item")).toHaveCount(2, { timeout: 10000 });

    // Click "Clear all" button
    await page.locator("#btnClear").click();

    // Feed should be empty
    await expect(page.locator(".feed-item")).toHaveCount(0, { timeout: 5000 });
    await expect(page.locator("#webhookCounter")).toHaveText(
      "0 webhooks received",
    );
    // Empty state should reappear
    await expect(page.locator("#emptyState")).toBeVisible();
  });
});
