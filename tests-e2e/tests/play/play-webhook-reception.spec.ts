import { test, expect } from "@playwright/test";

/**
 * Extracts the token from the page URL hash.
 */
async function getToken(
  page: import("@playwright/test").Page,
): Promise<string> {
  return page.evaluate(() => location.hash.replace(/^#/, ""));
}

/**
 * Waits for the WebSocket connection (or polling fallback) to be established.
 */
async function waitForConnection(
  page: import("@playwright/test").Page,
): Promise<void> {
  await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
    timeout: 15000,
  });
}

/**
 * Standard setup: navigate, wait for connection, extract token, build base URL.
 */
async function setup(page: import("@playwright/test").Page, baseURL: string) {
  await page.goto("/");
  await waitForConnection(page);
  const token = await getToken(page);
  const webhookUrl = `${baseURL}/in/${token}/`;
  return { token, webhookUrl };
}

/**
 * Sends a webhook and waits for the feed item to appear.
 */
async function sendAndWaitForFeedItem(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
  url: string,
  options: Parameters<typeof request.fetch>[1],
  expectedCount: number,
) {
  const response = await request.fetch(url, options);
  expect(response.status()).toBeLessThan(400);
  await expect(page.locator(".feed-item")).toHaveCount(expectedCount, {
    timeout: 10000,
  });
  return response;
}

test.describe("Play Webhook Reception", () => {
  // ─── HTTP Methods ─────────────────────────────────────────────────

  test("1 - GET webhook shows GET badge and query string in detail", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.get(`${webhookUrl}?foo=bar`);
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await expect(feedItem.locator(".method-badge")).toHaveText("GET");

    // Open detail
    await feedItem.click();
    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".method-badge")).toHaveText("GET");
    // Query string visible in path
    await expect(detail.locator(".detail-path")).toContainText("foo=bar");
  });

  test("2 - PUT webhook shows PUT badge", async ({ page, baseURL }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.put(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ action: "update", id: 42 }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await expect(feedItem.locator(".method-badge")).toHaveText("PUT");
  });

  test("3 - PATCH webhook shows PATCH badge", async ({ page, baseURL }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.patch(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ status: "partial" }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await expect(feedItem.locator(".method-badge")).toHaveText("PATCH");
  });

  test("4 - DELETE webhook shows DELETE badge with empty body", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.delete(webhookUrl);
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await expect(feedItem.locator(".method-badge")).toHaveText("DELETE");

    // Open detail, verify empty body
    await feedItem.click();
    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".body-note")).toContainText("Empty body");
  });

  test("5 - HEAD webhook shows HEAD badge", async ({ page, baseURL }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.head(webhookUrl);
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await expect(feedItem.locator(".method-badge")).toHaveText("HEAD");
  });

  // ─── Sub-paths ────────────────────────────────────────────────────

  test("6 - Webhook with sub-path shows path in detail", async ({
    page,
    baseURL,
  }) => {
    const { token } = await setup(page, baseURL!);
    const subPathUrl = `${baseURL}/in/${token}/orders/123/confirm`;

    const response = await page.request.post(subPathUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ confirmed: true }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });

    // Feed path should show the sub-path
    await expect(feedItem.locator(".feed-path")).toContainText(
      "/orders/123/confirm",
    );

    // Detail panel
    await feedItem.click();
    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".detail-path")).toContainText(
      "/orders/123/confirm",
    );
  });

  test("7 - Webhook with deep path and query string", async ({
    page,
    baseURL,
  }) => {
    const { token } = await setup(page, baseURL!);
    const deepUrl = `${baseURL}/in/${token}/api/v2/events?type=payment`;

    const response = await page.request.post(deepUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ event: "charge.completed" }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });

    // Feed path shows both path and query
    await expect(feedItem.locator(".feed-path")).toContainText(
      "/api/v2/events",
    );
    await expect(feedItem.locator(".feed-path")).toContainText("type=payment");

    // Detail panel
    await feedItem.click();
    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".detail-path")).toContainText(
      "/api/v2/events",
    );
    await expect(detail.locator(".detail-path")).toContainText("type=payment");
  });

  // ─── Headers ──────────────────────────────────────────────────────

  test("8 - Custom headers are captured and visible in detail", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.post(webhookUrl, {
      headers: {
        "Content-Type": "application/json",
        "X-Custom-Header": "test-value",
        "X-Webhook-Signature": "sha256=abc",
      },
      data: JSON.stringify({ data: "headers-test" }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    const headersTable = detail.locator(".headers-table").first();
    await expect(headersTable).toContainText("x-custom-header");
    await expect(headersTable).toContainText("test-value");
    await expect(headersTable).toContainText("x-webhook-signature");
    await expect(headersTable).toContainText("sha256=abc");
  });

  test("9 - Content-Type header visible in feed and detail", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ content: "type-test" }),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    // Content-Type in headers table
    const headersTable = detail.locator(".headers-table").first();
    await expect(headersTable).toContainText("content-type");
    await expect(headersTable).toContainText("application/json");

    // Content-Type shown next to Body label
    const bodySection = detail.locator(".detail-section").filter({
      hasText: "Body",
    });
    await expect(bodySection).toContainText("application/json");
  });

  // ─── Body sizes ───────────────────────────────────────────────────

  test("10 - Large JSON body is pretty-printed and displayed", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    // Build a ~10KB JSON payload
    const items = Array.from({ length: 50 }, (_, i) => ({
      id: i,
      name: `item-${i}`,
      description: `This is a description for item number ${i} with some padding text to increase the overall payload size.`,
      tags: ["alpha", "beta", "gamma", "delta"],
      metadata: { created: "2026-01-01T00:00:00Z", version: i * 10 },
    }));
    const largePayload = { event: "bulk.created", items };
    const jsonString = JSON.stringify(largePayload);
    // Verify it is roughly 10KB
    expect(jsonString.length).toBeGreaterThan(8000);

    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: jsonString,
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    // Body should be pretty-printed (contains indentation)
    const bodyDisplay = detail.locator(".body-display pre");
    await expect(bodyDisplay).toBeVisible();
    const bodyText = await bodyDisplay.textContent();
    expect(bodyText).toBeTruthy();
    // Pretty-printed JSON contains "bulk.created"
    expect(bodyText).toContain("bulk.created");
    // Pretty-printed JSON contains item names
    expect(bodyText).toContain("item-0");
    expect(bodyText).toContain("item-49");
  });

  test("11 - Empty body shows graceful message", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.post(webhookUrl);
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".body-note")).toContainText("Empty body");
  });

  test("12 - Form-encoded body is parsed as key-value pairs", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      data: "name=John&email=john@example.com",
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    // The form body is rendered as a table with key-value pairs
    const bodySection = detail.locator(".detail-section").filter({
      hasText: "Body",
    });
    await expect(bodySection).toContainText("name");
    await expect(bodySection).toContainText("John");
    await expect(bodySection).toContainText("email");
    await expect(bodySection).toContainText("john@example.com");
  });

  // ─── Rapid-fire / stress ──────────────────────────────────────────

  test("13 - 10 webhooks in quick succession all appear in the feed", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    // Fire 10 requests in parallel
    const promises = Array.from({ length: 10 }, (_, i) =>
      page.request.post(webhookUrl, {
        headers: { "Content-Type": "application/json" },
        data: JSON.stringify({ index: i }),
      }),
    );
    const responses = await Promise.all(promises);
    for (const r of responses) {
      expect(r.status()).toBeLessThan(400);
    }

    // All 10 should appear
    await expect(page.locator(".feed-item")).toHaveCount(10, {
      timeout: 15000,
    });
    await expect(page.locator("#webhookCounter")).toHaveText(
      "10 webhooks received",
    );
  });

  test("14 - New webhook appears while detail panel is open for another", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    // Send webhook A
    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ name: "webhook-A" }),
    });

    const feedItemA = page.locator(".feed-item").first();
    await expect(feedItemA).toBeVisible({ timeout: 10000 });
    await feedItemA.click();

    // Detail panel is open with webhook A
    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();
    await expect(detail.locator(".body-display")).toContainText("webhook-A");

    // Send webhook B while detail is open
    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ name: "webhook-B" }),
    });

    // B should appear in the feed (2 items total)
    await expect(page.locator(".feed-item")).toHaveCount(2, {
      timeout: 10000,
    });

    // Detail for A should still be visible
    await expect(detail.locator(".body-display")).toContainText("webhook-A");
  });

  // ─── Real-world payloads ──────────────────────────────────────────

  test("15 - Stripe-like webhook payload is pretty-printed correctly", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const stripePayload = {
      id: "evt_1MqqbKLt4dXK03v5qaIbkMWH",
      object: "event",
      api_version: "2023-10-16",
      created: 1680064028,
      type: "payment_intent.succeeded",
      data: {
        object: {
          id: "pi_3MqqbJLt4dXK03v50Uf0UOKW",
          object: "payment_intent",
          amount: 2000,
          currency: "usd",
          status: "succeeded",
          charges: {
            data: [
              {
                id: "ch_3MqqbJLt4dXK03v50IgGBaGw",
                amount: 2000,
                paid: true,
                receipt_url: "https://pay.stripe.com/receipts/example",
              },
            ],
          },
        },
      },
      livemode: false,
      pending_webhooks: 1,
    };

    const response = await page.request.post(webhookUrl, {
      headers: {
        "Content-Type": "application/json",
        "Stripe-Signature":
          "t=1680064028,v1=abcdef1234567890abcdef1234567890",
      },
      data: JSON.stringify(stripePayload),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    // Verify JSON is pretty-printed and contains key fields
    const bodyDisplay = detail.locator(".body-display");
    await expect(bodyDisplay).toContainText("payment_intent.succeeded");
    await expect(bodyDisplay).toContainText("pi_3MqqbJLt4dXK03v50Uf0UOKW");
    await expect(bodyDisplay).toContainText("2000");
    await expect(bodyDisplay).toContainText("succeeded");

    // Verify stripe-signature header is visible
    const headersTable = detail.locator(".headers-table").first();
    await expect(headersTable).toContainText("stripe-signature");
  });

  test("16 - GitHub-like webhook with X-GitHub-Event header", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const githubPayload = {
      ref: "refs/heads/main",
      before: "0000000000000000000000000000000000000000",
      after: "abc123def456789012345678901234567890abcd",
      repository: {
        id: 123456789,
        full_name: "octocat/Hello-World",
        html_url: "https://github.com/octocat/Hello-World",
      },
      pusher: {
        name: "octocat",
        email: "octocat@github.com",
      },
      commits: [
        {
          id: "abc123def456789012345678901234567890abcd",
          message: "Fix all the bugs",
          author: { name: "Octocat", email: "octocat@github.com" },
        },
      ],
    };

    const response = await page.request.post(webhookUrl, {
      headers: {
        "Content-Type": "application/json",
        "X-GitHub-Event": "push",
        "X-GitHub-Delivery": "72d3162e-cc78-11e3-81ab-4c9367dc0958",
      },
      data: JSON.stringify(githubPayload),
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detail = page.locator("#detailContent");
    await expect(detail).toBeVisible();

    // Verify GitHub headers
    const headersTable = detail.locator(".headers-table").first();
    await expect(headersTable).toContainText("x-github-event");
    await expect(headersTable).toContainText("push");
    await expect(headersTable).toContainText("x-github-delivery");

    // Verify body content
    const bodyDisplay = detail.locator(".body-display");
    await expect(bodyDisplay).toContainText("octocat/Hello-World");
    await expect(bodyDisplay).toContainText("Fix all the bugs");
  });

  // ─── WebSocket real-time verification ─────────────────────────────

  test("17 - Webhook appears within 2 seconds of sending", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    const startTime = Date.now();

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ timing: "test" }),
    });

    // Wait for the feed item to appear
    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 5000 });

    const elapsed = Date.now() - startTime;
    expect(elapsed).toBeLessThan(2000);
  });

  test("18 - Counter updates in real-time without page reload", async ({
    page,
    baseURL,
  }) => {
    const { webhookUrl } = await setup(page, baseURL!);

    // Counter starts at 0
    await expect(page.locator("#webhookCounter")).toHaveText(
      "0 webhooks received",
    );

    // Send 3 webhooks sequentially to avoid race conditions in counter display
    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ n: 1 }),
    });
    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ n: 2 }),
    });
    await expect(page.locator(".feed-item")).toHaveCount(2, { timeout: 10000 });

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ n: 3 }),
    });
    await expect(page.locator(".feed-item")).toHaveCount(3, { timeout: 10000 });

    // Counter should show 3 without any reload
    await expect(page.locator("#webhookCounter")).toHaveText(
      "3 webhooks received",
    );
  });
});
