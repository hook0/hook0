import { test, expect } from "@playwright/test";

test.describe("Play Polling Fallback", () => {
  test("second tab falls back to polling when token is already in use", async ({
    browser,
    baseURL,
  }) => {
    // Tab 1: open page, gets WebSocket
    const context1 = await browser.newContext();
    const tab1 = await context1.newPage();
    await tab1.goto(baseURL + "/");

    await expect(tab1.locator("#connLabel")).toHaveText("Connected", {
      timeout: 15000,
    });
    await expect(tab1.locator("#connDot")).toHaveClass(/green/);

    // Extract the token from tab 1
    const token = await tab1.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );

    // Tab 2: open the same token URL -- should get token_in_use -> polling fallback
    const context2 = await browser.newContext();
    const tab2 = await context2.newPage();
    await tab2.goto(baseURL + "/#" + token);

    // Tab 2 should fall back to polling mode
    await expect(tab2.locator("#connLabel")).toHaveText(/Polling/, {
      timeout: 15000,
    });
    await expect(tab2.locator("#connDot")).toHaveClass(/yellow/);

    // Verify tab2 displays the same token URL as tab1
    const tab1Url = await tab1.locator("#webhookUrl").textContent();
    const tab2Url = await tab2.locator("#webhookUrl").textContent();
    expect(tab2Url).toBe(tab1Url);

    await context1.close();
    await context2.close();
  });

  test("tab1 receives webhooks via WebSocket while tab2 is in polling mode", async ({
    browser,
    baseURL,
  }) => {
    // Tab 1: open page, gets WebSocket
    const context1 = await browser.newContext();
    const tab1 = await context1.newPage();
    await tab1.goto(baseURL + "/");

    await expect(tab1.locator("#connLabel")).toHaveText("Connected", {
      timeout: 15000,
    });

    const token = await tab1.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );

    // Tab 2: open same token -> polling
    const context2 = await browser.newContext();
    const tab2 = await context2.newPage();
    await tab2.goto(baseURL + "/#" + token);

    await expect(tab2.locator("#connLabel")).toHaveText(/Polling/, {
      timeout: 15000,
    });

    // Send a webhook
    const webhookUrl = `${baseURL}/in/${token}/`;
    const response = await tab1.request.post(webhookUrl, {
      headers: { "Content-Type": "application/json" },
      data: JSON.stringify({ shared: true }),
    });
    expect(response.status()).toBeLessThan(400);

    // Tab 1 receives it via WebSocket (fast)
    await expect(tab1.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });
    await expect(
      tab1.locator(".feed-item").first().locator(".method-badge"),
    ).toHaveText("POST");

    await context1.close();
    await context2.close();
  });
});
