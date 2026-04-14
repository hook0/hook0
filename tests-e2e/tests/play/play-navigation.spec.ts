import { test, expect } from "@playwright/test";

test.describe("Play Navigation - Header Links (desktop)", () => {
  test.use({ viewport: { width: 1280, height: 800 } });

  test("header has Docs link pointing to documentation.hook0.com", async ({
    page,
  }) => {
    await page.goto("/");

    const nav = page.locator('[data-test="header-nav"]');
    await expect(nav).toBeVisible({ timeout: 10000 });

    const docsLink = nav.locator('a[href="https://documentation.hook0.com"]');
    await expect(docsLink).toBeVisible();
    await expect(docsLink).toHaveText("Docs");
  });

  test("header has Pricing link pointing to hook0.com/#pricing", async ({
    page,
  }) => {
    await page.goto("/");

    const nav = page.locator('[data-test="header-nav"]');
    await expect(nav).toBeVisible({ timeout: 10000 });

    const pricingLink = nav.locator(
      'a[href="https://www.hook0.com/#pricing"]',
    );
    await expect(pricingLink).toBeVisible();
    await expect(pricingLink).toHaveText("Pricing");
  });

  test("header has Login link pointing to app.hook0.com", async ({ page }) => {
    await page.goto("/");

    const loginLink = page.locator('[data-test="login-link"]');
    await expect(loginLink).toBeVisible({ timeout: 10000 });
    await expect(loginLink).toHaveAttribute("href", "https://app.hook0.com/");
    await expect(loginLink).toHaveText("Login");
  });

  test("header has Get Started button pointing to app.hook0.com/register", async ({
    page,
  }) => {
    await page.goto("/");

    const registerBtn = page.locator('[data-test="register-btn"]');
    await expect(registerBtn).toBeVisible({ timeout: 10000 });
    await expect(registerBtn).toHaveAttribute(
      "href",
      "https://app.hook0.com/register",
    );
    await expect(registerBtn).toContainText("Get Started");
  });

  test("logo links to hook0.com", async ({ page }) => {
    await page.goto("/");

    const logo = page.locator("header .logo");
    await expect(logo).toBeVisible({ timeout: 10000 });
    await expect(logo).toHaveAttribute("href", "https://www.hook0.com");
  });
});

test.describe("Play Navigation - Mobile Menu", () => {
  test.use({ viewport: { width: 375, height: 667 } });

  test("hamburger button is visible at 375px viewport", async ({ page }) => {
    await page.goto("/");

    const hamburger = page.locator('[data-test="mobile-menu-btn"]');
    await expect(hamburger).toBeVisible({ timeout: 10000 });

    // Desktop nav and auth should be hidden at mobile
    await expect(page.locator('[data-test="header-nav"]')).not.toBeVisible();
    await expect(page.locator('[data-test="header-auth"]')).not.toBeVisible();
  });

  test("clicking hamburger opens menu with all links", async ({ page }) => {
    await page.goto("/");

    const hamburger = page.locator('[data-test="mobile-menu-btn"]');
    await expect(hamburger).toBeVisible({ timeout: 10000 });

    // Menu should be hidden initially
    const mobileMenu = page.locator('[data-test="mobile-menu"]');
    await expect(mobileMenu).not.toBeVisible();

    // Click hamburger to open
    await hamburger.click();
    await expect(mobileMenu).toBeVisible();

    // Check all expected links are present
    await expect(
      mobileMenu.locator('a[href="https://documentation.hook0.com"]'),
    ).toBeVisible();
    await expect(
      mobileMenu.locator('a[href="https://www.hook0.com/#pricing"]'),
    ).toBeVisible();
    await expect(
      mobileMenu.locator('[data-test="mobile-login-link"]'),
    ).toBeVisible();
    await expect(
      mobileMenu.locator('[data-test="mobile-register-btn"]'),
    ).toBeVisible();

    // Visual proof: screenshot with menu open
    await page.screenshot({
      path: "test-results/mobile-menu-open.png",
      fullPage: true,
    });
  });

  test("pressing Escape closes the mobile menu", async ({ page }) => {
    await page.goto("/");

    const hamburger = page.locator('[data-test="mobile-menu-btn"]');
    await expect(hamburger).toBeVisible({ timeout: 10000 });

    const mobileMenu = page.locator('[data-test="mobile-menu"]');

    // Open the menu
    await hamburger.click();
    await expect(mobileMenu).toBeVisible();

    // Press Escape
    await page.keyboard.press("Escape");
    await expect(mobileMenu).not.toBeVisible();
  });
});

test.describe("Play Navigation - Footer", () => {
  test("footer has Product column with 5 links", async ({ page }) => {
    await page.goto("/");

    const footer = page.locator("footer.site-footer");
    await expect(footer).toBeVisible({ timeout: 10000 });

    const productCol = footer.locator(".footer-col").nth(0);
    await expect(productCol.locator("h4")).toHaveText("Product");

    const productLinks = productCol.locator("a");
    await expect(productLinks).toHaveCount(5);

    await expect(productLinks.nth(0)).toHaveAttribute(
      "href",
      "https://www.hook0.com",
    );
    await expect(productLinks.nth(1)).toHaveAttribute(
      "href",
      "https://www.hook0.com/#pricing",
    );
    await expect(productLinks.nth(2)).toHaveAttribute(
      "href",
      "https://documentation.hook0.com",
    );
    await expect(productLinks.nth(3)).toHaveAttribute(
      "href",
      "https://documentation.hook0.com/api",
    );
    await expect(productLinks.nth(4)).toHaveAttribute(
      "href",
      "https://status.hook0.com",
    );
  });

  test("footer has Community column with 4 links", async ({ page }) => {
    await page.goto("/");

    const footer = page.locator("footer.site-footer");
    await expect(footer).toBeVisible({ timeout: 10000 });

    const communityCol = footer.locator(".footer-col").nth(1);
    await expect(communityCol.locator("h4")).toHaveText("Community");

    const communityLinks = communityCol.locator("a");
    await expect(communityLinks).toHaveCount(4);

    await expect(communityLinks.nth(0)).toHaveText("GitHub");
    await expect(communityLinks.nth(1)).toHaveText("Discord");
    await expect(communityLinks.nth(2)).toHaveText("Twitter/X");
    await expect(communityLinks.nth(3)).toHaveText("LinkedIn");
  });

  test("footer has Legal column with 3 links", async ({ page }) => {
    await page.goto("/");

    const footer = page.locator("footer.site-footer");
    await expect(footer).toBeVisible({ timeout: 10000 });

    const legalCol = footer.locator(".footer-col").nth(2);
    await expect(legalCol.locator("h4")).toHaveText("Legal");

    const legalLinks = legalCol.locator("a");
    await expect(legalLinks).toHaveCount(3);

    await expect(legalLinks.nth(0)).toHaveText("Privacy Policy");
    await expect(legalLinks.nth(1)).toHaveText("Terms of Service");
    await expect(legalLinks.nth(2)).toHaveText("Contact");
  });

  test("all footer external links are reachable", async ({ page }) => {
    await page.goto("/");

    const footer = page.locator("footer.site-footer");
    await expect(footer).toBeVisible({ timeout: 10000 });

    const links = footer.locator("a[href^='http']");
    const count = await links.count();
    expect(count).toBeGreaterThan(0);

    // Twitter/X blocks HEAD requests from automated clients (returns 403).
    // Use GET as fallback when HEAD fails.
    const skipHeadDomains = ["twitter.com", "x.com"];

    const results: { url: string; status: number }[] = [];

    for (let i = 0; i < count; i++) {
      const href = await links.nth(i).getAttribute("href");
      if (!href) continue;

      const useGet = skipHeadDomains.some((d) => href.includes(d));
      const response = useGet
        ? await page.request.get(href, { timeout: 15000 })
        : await page.request.head(href, { timeout: 15000 });
      results.push({ url: href, status: response.status() });
    }

    const failures = results.filter((r) => r.status >= 400);
    expect(
      failures,
      `Unreachable footer links: ${JSON.stringify(failures)}`,
    ).toHaveLength(0);
  });
});
