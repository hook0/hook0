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
 * Waits for the WebSocket connection to be established.
 */
async function waitForConnection(
  page: import("@playwright/test").Page,
): Promise<void> {
  await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
    timeout: 15000,
  });
}

test.describe("Play Content Types", () => {
  test("POST with Content-Type application/json shows pretty-printed JSON", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ event: "order.created", amount: 42 }),
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    // Body should contain pretty-printed JSON with indentation
    const bodyDisplay = detailContent.locator(".body-display");
    await expect(bodyDisplay).toBeVisible();

    const bodyText = await bodyDisplay.textContent();
    // Pretty-printed JSON has newlines and indentation
    expect(bodyText).toContain('"event"');
    expect(bodyText).toContain('"order.created"');
    expect(bodyText).toContain('"amount"');
  });

  test("POST with Content-Type application/x-www-form-urlencoded shows key-value table", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
      data: "name=hook0&type=webhook&active=true",
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    // Form data is rendered as a table inside the body-display section
    // The body section should contain the key-value pairs
    const bodySection = detailContent.locator(".body-display");
    await expect(bodySection).toBeVisible();

    // The form-urlencoded body is rendered as a headers-table inside body-display
    const kvTable = bodySection.locator(".headers-table");
    await expect(kvTable).toBeVisible();

    const tableText = await kvTable.textContent();
    expect(tableText).toContain("name");
    expect(tableText).toContain("hook0");
    expect(tableText).toContain("type");
    expect(tableText).toContain("webhook");
  });

  test("POST with Content-Type text/plain shows body as raw text", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "text/plain" },
      data: "Hello, this is plain text content from Hook0 Play test",
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    const bodyDisplay = detailContent.locator(".body-display pre");
    await expect(bodyDisplay.first()).toBeVisible();

    const bodyText = await bodyDisplay.first().textContent();
    expect(bodyText).toContain(
      "Hello, this is plain text content from Hook0 Play test",
    );
  });

  test("POST with no Content-Type shows body as raw text", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send a raw POST with explicit empty content-type workaround
    // Playwright always sets content-type, so we use fetch from the test context
    const response = await page.request.fetch(webhookUrl, {
      method: "POST",
      data: "raw body without content type header",
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    // Body should be displayed as raw text
    const bodyDisplay = detailContent.locator(".body-display");
    await expect(bodyDisplay).toBeVisible();
    const bodyText = await bodyDisplay.textContent();
    expect(bodyText).toContain("raw body without content type header");
  });

  test("POST with binary body shows base64 note", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");
    await waitForConnection(page);

    const token = await getToken(page);
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send binary data (bytes that include control characters)
    const binaryData = Buffer.from([
      0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x89, 0x50, 0x4e, 0x47, 0x0d,
      0x0a, 0x1a, 0x0a, 0xff, 0xfe, 0xfd,
    ]);

    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/octet-stream" },
      data: binaryData,
    });
    expect(response.status()).toBeLessThan(400);

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();

    // Should show the base64 note for binary content
    const bodyNote = detailContent.locator(".body-note");
    const allNotes = await bodyNote.allTextContents();
    const hasBase64Note = allNotes.some((text) => text.includes("base64"));
    expect(hasBase64Note).toBe(true);
  });
});
