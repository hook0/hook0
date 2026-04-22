import { test, expect } from "@playwright/test";

test.describe("Play SEO", () => {
  test("page has correct title tag", async ({ page }) => {
    await page.goto("/");

    await expect(page).toHaveTitle(
      "Hook0 Play - Free Webhook Tester & Inspector Online",
    );
  });

  test("page has meta description with expected content", async ({ page }) => {
    await page.goto("/");

    const description = page.locator('meta[name="description"]');
    await expect(description).toHaveAttribute(
      "content",
      /free webhook tester/i,
    );
  });

  test("page has canonical link", async ({ page }) => {
    await page.goto("/");

    const canonical = page.locator('link[rel="canonical"]');
    await expect(canonical).toHaveAttribute("href", "https://www.hook0.com/webhook-playground");
  });

  test("page has FAQPage JSON-LD script", async ({ page }) => {
    await page.goto("/");

    const jsonLd = page.locator('script[type="application/ld+json"]');
    await expect(jsonLd).toHaveCount(1);

    const content = await jsonLd.textContent();
    const parsed = JSON.parse(content!);

    expect(parsed["@context"]).toBe("https://schema.org");
    expect(parsed["@type"]).toBe("FAQPage");
    expect(Array.isArray(parsed.mainEntity)).toBe(true);
    expect(parsed.mainEntity.length).toBeGreaterThanOrEqual(7);

    // Verify first question structure
    expect(parsed.mainEntity[0]["@type"]).toBe("Question");
    expect(parsed.mainEntity[0].name).toBeTruthy();
    expect(parsed.mainEntity[0].acceptedAnswer["@type"]).toBe("Answer");
    expect(parsed.mainEntity[0].acceptedAnswer.text).toBeTruthy();
  });

  test("noscript block contains SEO content", async ({ page }) => {
    await page.goto("/");

    const noscriptContent = await page.evaluate(() => {
      const noscript = document.querySelector("noscript");
      return noscript ? noscript.innerHTML : "";
    });

    expect(noscriptContent).toContain("Hook0 Play");
    expect(noscriptContent).toContain("Free Webhook Tester");
    expect(noscriptContent).toContain("requires JavaScript");
  });

  test("all 7 H2 sections are present in the SEO content below the fold", async ({
    page,
  }) => {
    await page.goto("/");

    const seoSection = page.locator(".seo-content");
    await expect(seoSection).toBeVisible();

    const h2Elements = seoSection.locator("h2");
    await expect(h2Elements).toHaveCount(7);

    const expectedHeadings = [
      "What is a Webhook Tester?",
      "How to Test Webhooks Online in 3 Steps",
      "Why Hook0 Play? Free Webhook Testing with No Limits",
      "Test Webhooks Locally with the Hook0 CLI",
      "Supported Integrations",
      "Webhook Testing Questions",
      "Go Further with Hook0",
    ];

    for (let i = 0; i < expectedHeadings.length; i++) {
      await expect(h2Elements.nth(i)).toHaveText(expectedHeadings[i]);
    }
  });
});
