import { test, expect } from "@playwright/test";

test.describe("Play Home", () => {
  test("page loads with correct title", async ({ page }) => {
    await page.goto("/");

    await expect(page).toHaveTitle(/Hook0 Play/);
  });

  test("webhook URL is displayed and contains the token", async ({ page }) => {
    await page.goto("/");

    const urlText = page.locator("#webhookUrl");
    await expect(urlText).toBeVisible({ timeout: 10000 });

    const urlValue = await urlText.textContent();
    // URL must match pattern: http(s)://host/in/c_<27 base62 chars>/
    expect(urlValue).toMatch(/\/in\/c_[0-9A-Za-z]{27}\//);
  });

  test("copy button copies URL to clipboard", async ({ page, context }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.goto("/");

    const urlText = page.locator("#webhookUrl");
    await expect(urlText).toBeVisible({ timeout: 10000 });

    const webhookUrl = await urlText.textContent();

    await page.locator("#btnCopy").click();

    // Verify "Copied!" feedback appears
    await expect(page.locator("#copyFeedback")).toHaveClass(/show/);

    // Verify clipboard content
    const clipboardContent = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardContent).toBe(webhookUrl);
  });

  test("New URL button generates a different token", async ({ page }) => {
    await page.goto("/");

    const urlText = page.locator("#webhookUrl");
    await expect(urlText).toBeVisible({ timeout: 10000 });

    const firstUrl = await urlText.textContent();

    await page.locator("#btnNew").click();

    // Wait for hash to change and new URL to render
    await page.waitForFunction(
      (oldUrl: string) =>
        document.getElementById("webhookUrl")!.textContent !== oldUrl,
      firstUrl,
    );

    const secondUrl = await urlText.textContent();
    expect(secondUrl).not.toBe(firstUrl);
    expect(secondUrl).toMatch(/\/in\/c_[0-9A-Za-z]{27}\//);
  });

  test("connection status shows Connected with green indicator", async ({
    page,
  }) => {
    await page.goto("/");

    // Wait for WebSocket to establish connection
    await expect(page.locator("#connLabel")).toHaveText("Connected", {
      timeout: 15000,
    });
    await expect(page.locator("#connDot")).toHaveClass(/green/);
  });

  test("webhook counter shows 0 webhooks received initially", async ({
    page,
  }) => {
    await page.goto("/");

    await expect(page.locator("#webhookCounter")).toHaveText(
      "0 webhooks received",
      { timeout: 10000 },
    );
  });
});
