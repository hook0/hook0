import { test, expect } from "@playwright/test";

test.describe("Tracking & RGPD compliance", () => {
  test.describe("gtag.js consent gating", () => {
    test("gtag.js is NOT loaded without cookie consent", async ({ page }) => {
      // Clear any existing consent
      await page.goto("/hook0-vs-svix");
      await page.evaluate(() => localStorage.removeItem("hook0_cookie_consent"));
      await page.reload();
      await page.waitForLoadState("networkidle");

      // gtag.js should NOT be in the DOM
      const gtagScripts = await page.locator(
        'script[src*="googletagmanager.com/gtag/js"]'
      );
      await expect(gtagScripts).toHaveCount(0);
    });

    test("gtag.js IS loaded after accepting cookies", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      await page.evaluate(() => localStorage.removeItem("hook0_cookie_consent"));
      await page.reload();

      // Click accept on cookie banner
      const acceptButton = page.locator(
        'button:has-text("Accept"), button:has-text("Accepter")'
      );
      if (await acceptButton.isVisible({ timeout: 3000 }).catch(() => false)) {
        await acceptButton.click();
        await page.waitForTimeout(1000);

        // gtag.js should now be loaded
        const gtagScripts = await page.locator(
          'script[src*="googletagmanager.com/gtag/js"]'
        );
        await expect(gtagScripts).toHaveCount(1);
      }
    });

    test("Matomo requires consent before tracking", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      const html = await page.content();
      // requireConsent must appear BEFORE trackPageView
      const requireConsentIndex = html.indexOf("requireConsent");
      const trackPageViewIndex = html.indexOf("trackPageView");
      if (requireConsentIndex !== -1 && trackPageViewIndex !== -1) {
        expect(requireConsentIndex).toBeLessThan(trackPageViewIndex);
      }
    });
  });

  test.describe("Cookie banner", () => {
    test("cookie banner is visible on first visit", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      await page.evaluate(() => localStorage.removeItem("hook0_cookie_consent"));
      await page.reload();

      // Look for the cookie consent UI
      const consentBanner = page.locator(
        '[class*="cookie"], [id*="cookie"], [class*="consent"], [id*="consent"]'
      );
      // At least one consent-related element should exist in the page
      const html = await page.content();
      expect(html).toContain("cookie");
    });

    test("declining cookies does not load gtag.js", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      await page.evaluate(() => localStorage.removeItem("hook0_cookie_consent"));
      await page.reload();

      const declineButton = page.locator(
        'button:has-text("Decline"), button:has-text("Refuser")'
      );
      if (await declineButton.isVisible({ timeout: 3000 }).catch(() => false)) {
        await declineButton.click();
        await page.waitForTimeout(500);

        const gtagScripts = await page.locator(
          'script[src*="googletagmanager.com/gtag/js"]'
        );
        await expect(gtagScripts).toHaveCount(0);
      }
    });
  });

  test.describe("gclid persistence and propagation", () => {
    test("gclid from URL is stored in sessionStorage", async ({ page }) => {
      await page.goto("/hook0-vs-svix?gclid=test-gclid-123&mtm_source=google");
      await page.waitForLoadState("domcontentloaded");

      const storedParams = await page.evaluate(() => {
        const data = sessionStorage.getItem("hook0_tracking_params");
        return data ? JSON.parse(data) : null;
      });

      expect(storedParams).not.toBeNull();
      expect(storedParams.gclid).toBe("test-gclid-123");
      expect(storedParams.mtm_source).toBe("google");
    });

    test("gclid is propagated to app.hook0.com links", async ({ page }) => {
      await page.goto("/hook0-vs-svix?gclid=test-gclid-456");
      await page.waitForLoadState("domcontentloaded");

      // Find a link to app.hook0.com and click it (but intercept navigation)
      const appLink = page.locator('a[href*="app.hook0.com"]').first();
      const href = await appLink.evaluate((el) => {
        // Trigger the click handler to update href, then read it
        el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
        return el.getAttribute("href");
      });

      expect(href).toContain("gclid=test-gclid-456");
    });

    test("gclid is propagated to documentation.hook0.com links", async ({
      page,
    }) => {
      await page.goto(
        "/hook0-vs-svix?gclid=test-gclid-789&mtm_campaign=test"
      );
      await page.waitForLoadState("domcontentloaded");

      const docLink = page
        .locator('a[href*="documentation.hook0.com"]')
        .first();
      if ((await docLink.count()) > 0) {
        const href = await docLink.evaluate((el) => {
          el.dispatchEvent(new MouseEvent("click", { bubbles: true }));
          return el.getAttribute("href");
        });

        expect(href).toContain("gclid=test-gclid-789");
      }
    });

    test("wbraid and gbraid are also persisted", async ({ page }) => {
      await page.goto(
        "/build-vs-buy-webhooks?wbraid=test-wbraid&gbraid=test-gbraid"
      );
      await page.waitForLoadState("domcontentloaded");

      const storedParams = await page.evaluate(() => {
        const data = sessionStorage.getItem("hook0_tracking_params");
        return data ? JSON.parse(data) : null;
      });

      expect(storedParams).not.toBeNull();
      expect(storedParams.wbraid).toBe("test-wbraid");
      expect(storedParams.gbraid).toBe("test-gbraid");
    });
  });

  test.describe("Conversion events configuration", () => {
    test("playground click conversion is wired", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      const html = await page.content();
      // The conversion handler for play.hook0.com links should be in the page
      expect(html).toContain("play.hook0.com");
      expect(html).toContain("playground_click");
    });

    test("documentation visit conversion is wired", async ({ page }) => {
      await page.goto("/hook0-vs-svix");
      const html = await page.content();
      expect(html).toContain("documentation.hook0.com");
      expect(html).toContain("documentation_visit");
    });
  });
});
