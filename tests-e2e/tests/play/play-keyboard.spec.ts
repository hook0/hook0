import { test, expect } from "@playwright/test";

test.describe("Play Keyboard Shortcuts", () => {
  test("pressing c copies the webhook URL to clipboard", async ({
    page,
    context,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    await page.goto("/");

    const urlText = page.locator("#webhookUrl");
    await expect(urlText).toBeVisible({ timeout: 10000 });

    const webhookUrl = await urlText.textContent();

    // Press 'c' key
    await page.keyboard.press("c");

    // Verify copy feedback
    await expect(page.locator("#copyFeedback")).toHaveClass(/show/);

    // Verify clipboard
    const clipboardContent = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboardContent).toBe(webhookUrl);
  });

  test("pressing j/k navigates the webhook list", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");

    await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
      timeout: 15000,
    });

    const token = await page.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );
    const webhookUrl = `${baseURL}/in/${token}/`;

    // Send 3 webhooks to have items to navigate
    await page.request.get(webhookUrl);
    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    await page.request.post(webhookUrl, { data: "second" });
    await expect(page.locator(".feed-item")).toHaveCount(2, { timeout: 10000 });

    await page.request.put(webhookUrl, { data: "third" });
    await expect(page.locator(".feed-item")).toHaveCount(3, { timeout: 10000 });

    // Press 'j' to move down to the first item (index 0)
    await page.keyboard.press("j");

    // The first feed item should be selected
    await expect(page.locator(".feed-item").first()).toHaveClass(/selected/);

    // Press 'j' again to move to the second item
    await page.keyboard.press("j");
    await expect(page.locator(".feed-item").nth(1)).toHaveClass(/selected/);

    // Press 'k' to move back to the first item
    await page.keyboard.press("k");
    await expect(page.locator(".feed-item").first()).toHaveClass(/selected/);
  });

  test("pressing Enter expands the selected webhook detail", async ({
    page,
    baseURL,
  }) => {
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
      data: JSON.stringify({ keyboard: "enter-test" }),
    });

    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    // Navigate to first item with 'j'
    await page.keyboard.press("j");
    await expect(page.locator(".feed-item").first()).toHaveClass(/selected/);

    // Press Enter to expand detail
    await page.keyboard.press("Enter");

    // Detail content should be visible
    await expect(page.locator("#detailContent")).toBeVisible();
    await expect(
      page.locator("#detailContent .method-badge"),
    ).toHaveText("POST");
    await expect(page.locator("#detailContent .body-display")).toContainText(
      "enter-test",
    );
  });

  test("pressing Escape closes the detail panel", async ({
    page,
    baseURL,
  }) => {
    await page.goto("/");

    await expect(page.locator("#connLabel")).toHaveText(/(Connected|Polling)/, {
      timeout: 15000,
    });

    const token = await page.evaluate(() =>
      location.hash.replace(/^#/, ""),
    );
    const webhookUrl = `${baseURL}/in/${token}/`;

    await page.request.post(webhookUrl, { data: "escape-test" });

    await expect(page.locator(".feed-item")).toHaveCount(1, { timeout: 10000 });

    // Select and open the webhook
    await page.locator(".feed-item").first().click();
    await expect(page.locator("#detailContent")).toBeVisible();

    // Press Escape to close
    await page.keyboard.press("Escape");

    // Detail content should be hidden
    await expect(page.locator("#detailContent")).not.toBeVisible();
  });
});
