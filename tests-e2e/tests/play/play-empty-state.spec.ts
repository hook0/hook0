import { test, expect } from "@playwright/test";

test.describe("Play Empty State", () => {
  test("empty state shows webhook URL", async ({ page }) => {
    await page.goto("/");

    const emptyState = page.locator("#emptyState");
    await expect(emptyState).toBeVisible({ timeout: 10000 });

    // Empty state heading
    await expect(emptyState.locator("h3")).toHaveText("Waiting for webhooks");

    // The webhook URL should be visible in the header area
    const webhookUrl = page.locator("#webhookUrl");
    await expect(webhookUrl).toBeVisible();
    const urlText = await webhookUrl.textContent();
    expect(urlText).toMatch(/\/in\/c_[0-9A-Za-z]{27}\//);
  });

  test("empty state shows curl example with the correct token", async ({
    page,
  }) => {
    await page.goto("/");

    const emptyState = page.locator("#emptyState");
    await expect(emptyState).toBeVisible({ timeout: 10000 });

    const curlExample = page.locator("#curlExample");
    await expect(curlExample).toBeVisible();

    const curlText = await curlExample.textContent();

    // Extract token from the URL hash
    const token = await page.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );

    // Curl example should contain the token
    expect(curlText).toContain(token);
    // Should be a curl command
    expect(curlText).toContain("curl");
  });

  test("empty state disappears after first webhook arrives", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");

    // Wait for connection
    await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
      timeout: 15000,
    });

    const emptyState = page.locator("#emptyState");
    await expect(emptyState).toBeVisible({ timeout: 10000 });

    // Extract token and send a webhook
    const token = await page.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );
    const webhookUrl = `${baseURL}/in/${token}/`;

    const response = await page.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ test: "empty-state-gone" }),
    });
    expect(response.status()).toBeLessThan(400);

    // Wait for the feed item to appear
    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    // Empty state should be hidden
    await expect(emptyState).not.toBeVisible();
  });
});
