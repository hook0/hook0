import { test, expect } from "@playwright/test";

test.describe("Play Responsive Layout", () => {
  test("desktop viewport shows two-panel layout", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({
      viewport: { width: 1280, height: 800 },
      baseURL: baseURL!,
    });
    const page = await context.newPage();
    await page.goto("/");

    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Feed panel and detail panel should both exist in the DOM
    const feedPanel = page.locator("#feedPanel");
    const detailPanel = page.locator("#detailPanel");

    await expect(feedPanel).toBeVisible();
    // At desktop, detail panel has display:block via media query (min-width:768px)
    await expect(detailPanel).toBeVisible();

    // Verify side-by-side: feed panel has width ~40%, detail panel takes the rest
    const feedBox = await feedPanel.boundingBox();
    const detailBox = await detailPanel.boundingBox();
    expect(feedBox).not.toBeNull();
    expect(detailBox).not.toBeNull();

    // They should be horizontally adjacent (detail starts where feed ends, roughly)
    expect(detailBox!.x).toBeGreaterThanOrEqual(feedBox!.x + feedBox!.width - 2);

    await context.close();
  });

  test("mobile viewport shows single-column layout", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({
      viewport: { width: 375, height: 667 },
      baseURL: baseURL!,
    });
    const page = await context.newPage();
    await page.goto("/");

    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Feed panel should be visible and full-width
    const feedPanel = page.locator("#feedPanel");
    await expect(feedPanel).toBeVisible();

    const feedBox = await feedPanel.boundingBox();
    expect(feedBox).not.toBeNull();
    // Feed panel should span the full viewport width (accounting for border)
    expect(feedBox!.width).toBeGreaterThanOrEqual(370);

    // Detail panel should be hidden at mobile (display:none in CSS below 768px)
    const detailPanel = page.locator("#detailPanel");
    await expect(detailPanel).not.toBeVisible();

    await context.close();
  });

  test("mobile: tapping a webhook shows full-screen detail view with close button", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({
      viewport: { width: 375, height: 667 },
      baseURL: baseURL!,
    });
    const page = await context.newPage();
    await page.goto("/");

    await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
      timeout: 15000,
    });

    const token = await page.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ mobile: true }),
    });

    const feedItem = page.locator(".feed-item").first();
    await expect(feedItem).toBeVisible({ timeout: 10000 });
    await feedItem.click();

    // Detail panel should now be visible as a full-screen overlay
    const detailPanel = page.locator("#detailPanel");
    await expect(detailPanel).toBeVisible();

    // Detail content should be visible with the webhook data
    const detailContent = page.locator("#detailContent");
    await expect(detailContent).toBeVisible();
    await expect(detailContent.locator(".method-badge")).toHaveText("POST");

    // Close button should be present on mobile
    const closeBtn = detailContent.locator('[data-action="close"]');
    await expect(closeBtn).toBeVisible();
    await closeBtn.scrollIntoViewIfNeeded();
    await closeBtn.click({ force: true });

    // Detail panel should be hidden again
    await expect(detailPanel).not.toBeVisible();

    await context.close();
  });

  test("mobile: URL card and copy button are visible", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({
      viewport: { width: 375, height: 667 },
      baseURL: baseURL!,
    });
    const page = await context.newPage();
    await page.goto("/");

    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("#btnCopy")).toBeVisible();

    const urlText = await page.locator("#webhookUrl").textContent();
    expect(urlText).toMatch(/\/in\/c_[0-9A-Za-z]{27}\//);

    await context.close();
  });
});
