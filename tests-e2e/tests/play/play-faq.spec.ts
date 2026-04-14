import { test, expect } from "@playwright/test";

test.describe("Play FAQ", () => {
  test("FAQ accordion items can be expanded by clicking", async ({ page }) => {
    await page.goto("/");

    // Scroll to the FAQ section
    const faqDetails = page.locator(".seo-content details");
    const firstFaq = faqDetails.first();
    await firstFaq.scrollIntoViewIfNeeded();

    // Initially, the details element should not be open
    await expect(firstFaq).not.toHaveAttribute("open", /.*/);

    // Click the summary to expand
    await firstFaq.locator("summary").click();

    // Now it should be open
    await expect(firstFaq).toHaveAttribute("open", /.*/);

    // The content div inside should be visible
    const content = firstFaq.locator("div");
    await expect(content).toBeVisible();
  });

  test("FAQ accordion items can be collapsed by clicking again", async ({
    page,
  }) => {
    await page.goto("/");

    const faqDetails = page.locator(".seo-content details");
    const firstFaq = faqDetails.first();
    await firstFaq.scrollIntoViewIfNeeded();

    // Open it
    await firstFaq.locator("summary").click();
    await expect(firstFaq).toHaveAttribute("open", /.*/);

    // Close it
    await firstFaq.locator("summary").click();
    await expect(firstFaq).not.toHaveAttribute("open", /.*/);
  });

  test("each FAQ has a summary and a content div", async ({ page }) => {
    await page.goto("/");

    const faqDetails = page.locator(".seo-content details");
    const count = await faqDetails.count();

    // There should be 7 FAQ items (matching the JSON-LD)
    expect(count).toBe(7);

    // Each details element should have a summary and a div child
    for (let i = 0; i < count; i++) {
      const detail = faqDetails.nth(i);
      const summary = detail.locator("summary");
      const contentDiv = detail.locator("> div");

      // Summary should exist and have text
      await expect(summary).toHaveCount(1);
      const summaryText = await summary.textContent();
      expect(summaryText!.length).toBeGreaterThan(0);

      // Content div should exist
      await expect(contentDiv).toHaveCount(1);
    }
  });
});
