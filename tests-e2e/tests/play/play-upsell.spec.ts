import { test, expect } from "@playwright/test";

test.describe("Play Upsell Banner", () => {
  test("upsell banner is visible on first visit", async ({
    browser,
    baseURL,
  }) => {
    // Fresh context with no localStorage
    const context = await browser.newContext({ baseURL: baseURL! });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Upsell banner should be visible
    const upsell = page.locator("#upsell");
    await expect(upsell).toBeVisible();

    await context.close();
  });

  test("dismiss button hides the upsell banner", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({ baseURL: baseURL! });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    const upsell = page.locator("#upsell");
    await expect(upsell).toBeVisible();

    // Click dismiss
    await page.locator("#btnDismissUpsell").click();

    // Banner should be hidden
    await expect(upsell).not.toBeVisible();

    await context.close();
  });

  test("upsell banner stays hidden after page reload", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({ baseURL: baseURL! });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Dismiss the banner
    await expect(page.locator("#upsell")).toBeVisible();
    await page.locator("#btnDismissUpsell").click();
    await expect(page.locator("#upsell")).not.toBeVisible();

    // Verify localStorage flag is set
    const dismissed = await page.evaluate(() =>
      localStorage.getItem("hook0play_upsell_dismissed"),
    );
    expect(dismissed).toBe("1");

    // Reload
    await page.reload();
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Banner should still be hidden
    await expect(page.locator("#upsell")).not.toBeVisible();

    await context.close();
  });

  test("upsell banner links to hook0.com", async ({
    browser,
    baseURL,
  }) => {
    const context = await browser.newContext({ baseURL: baseURL! });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    const upsellLink = page.locator("#upsell a");
    await expect(upsellLink).toHaveAttribute("href", "https://www.hook0.com");
    await expect(upsellLink).toHaveText("Try Hook0 free");

    await context.close();
  });
});
