import { test, expect } from "@playwright/test";

/**
 * Visual regression tests for Hook0Button component variants.
 *
 * Uses the /__dev/components showcase page (dev-only route) to render
 * all button variants in isolation and verify they are visually correct.
 *
 * These tests verify:
 * - All variants are visible (primary, secondary, danger, ghost, link, icon)
 * - Buttons render as the correct HTML element (<button> vs <a>)
 * - Disabled and loading states display correctly
 * - Buttons with icons have proper inline layout
 * - Full-width variant works
 */

const SHOWCASE_URL = "/__dev/components";

// Use port 3000 for the dev server in local testing
const BASE_URL = process.env.BASE_URL || "http://localhost:3000";

test.describe("Hook0Button Visual Regression", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(`${BASE_URL}${SHOWCASE_URL}`);
    await expect(page.locator('[data-test="component-showcase"]')).toBeVisible({
      timeout: 15000,
    });
  });

  test("all button variants are visible and correctly styled", async ({ page }) => {
    const variantsSection = page.locator('[data-test="button-variants"]');
    await expect(variantsSection).toBeVisible();

    // Verify each variant is visible
    const variants = ["primary", "secondary", "danger", "ghost", "link", "icon"];
    for (const variant of variants) {
      const btn = page.locator(`[data-test="btn-${variant}"]`);
      await expect(btn).toBeVisible();
    }

    // Take a screenshot of the variants section
    await expect(variantsSection).toHaveScreenshot("button-variants.png", {
      maxDiffPixels: 100,
    });
  });

  test("primary button has correct background color", async ({ page }) => {
    const btn = page.locator('[data-test="btn-primary"]');
    const styles = await btn.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        backgroundColor: computed.backgroundColor,
        color: computed.color,
        cursor: computed.cursor,
        display: computed.display,
      };
    });

    // Primary should NOT have transparent background
    expect(styles.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");
    expect(styles.backgroundColor).not.toBe("transparent");
    expect(styles.color).toBeDefined();
    expect(styles.cursor).toBe("pointer");
    expect(styles.display).toContain("flex");
  });

  test("secondary button has correct border and background", async ({ page }) => {
    const btn = page.locator('[data-test="btn-secondary"]');
    const styles = await btn.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        backgroundColor: computed.backgroundColor,
        borderWidth: computed.borderWidth,
        display: computed.display,
      };
    });

    // Secondary should have visible background and border
    expect(styles.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");
    expect(styles.borderWidth).not.toBe("0px");
    expect(styles.display).toContain("flex");
  });

  test("danger button has distinct styling", async ({ page }) => {
    const btn = page.locator('[data-test="btn-danger"]');
    const styles = await btn.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        backgroundColor: computed.backgroundColor,
        color: computed.color,
        display: computed.display,
      };
    });

    // Danger should NOT have transparent background
    expect(styles.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");
    expect(styles.backgroundColor).not.toBe("transparent");
    expect(styles.display).toContain("flex");
  });

  test("ghost button has transparent background", async ({ page }) => {
    const btn = page.locator('[data-test="btn-ghost"]');
    const styles = await btn.evaluate((el) => {
      const computed = window.getComputedStyle(el);
      return {
        backgroundColor: computed.backgroundColor,
        display: computed.display,
      };
    });

    // Ghost should have transparent background
    expect(styles.backgroundColor).toBe("rgba(0, 0, 0, 0)");
    expect(styles.display).toContain("flex");
  });

  test("all button sizes are visible and have correct relative sizing", async ({ page }) => {
    const sizesSection = page.locator('[data-test="button-sizes"]');
    await expect(sizesSection).toBeVisible();

    const smBtn = page.locator('[data-test="btn-size-sm"]');
    const mdBtn = page.locator('[data-test="btn-size-md"]');
    const lgBtn = page.locator('[data-test="btn-size-lg"]');

    const smBox = await smBtn.boundingBox();
    const mdBox = await mdBtn.boundingBox();
    const lgBox = await lgBtn.boundingBox();

    expect(smBox).not.toBeNull();
    expect(mdBox).not.toBeNull();
    expect(lgBox).not.toBeNull();

    if (smBox && mdBox && lgBox) {
      // sm < md < lg in height
      expect(smBox.height).toBeLessThan(mdBox.height);
      expect(mdBox.height).toBeLessThan(lgBox.height);
    }

    await expect(sizesSection).toHaveScreenshot("button-sizes.png", {
      maxDiffPixels: 100,
    });
  });

  test("buttons with icons have icon and text on same baseline", async ({ page }) => {
    const iconsSection = page.locator('[data-test="button-icons"]');
    await expect(iconsSection).toBeVisible();

    // Verify each icon button is visible
    const iconButtons = ["btn-icon-right", "btn-icon-left", "btn-icon-both"];
    for (const testId of iconButtons) {
      const btn = page.locator(`[data-test="${testId}"]`);
      await expect(btn).toBeVisible();

      // Check that button height stays reasonable (no wrapping)
      const box = await btn.boundingBox();
      expect(box).not.toBeNull();
      if (box) {
        expect(box.height).toBeLessThan(50);
      }
    }

    await expect(iconsSection).toHaveScreenshot("button-icons.png", {
      maxDiffPixels: 100,
    });
  });

  test("disabled buttons are visually distinct", async ({ page }) => {
    const disabledSection = page.locator('[data-test="button-disabled"]');
    await expect(disabledSection).toBeVisible();

    const disabledBtns = [
      "btn-disabled-primary",
      "btn-disabled-secondary",
      "btn-disabled-danger",
    ];

    for (const testId of disabledBtns) {
      const btn = page.locator(`[data-test="${testId}"]`);
      await expect(btn).toBeVisible();

      const styles = await btn.evaluate((el) => {
        const computed = window.getComputedStyle(el);
        return {
          opacity: computed.opacity,
          cursor: computed.cursor,
          pointerEvents: computed.pointerEvents,
        };
      });

      // Disabled buttons should have reduced opacity
      expect(parseFloat(styles.opacity)).toBeLessThan(1);
      // cursor should be not-allowed (or pointer-events: none prevents click)
      expect(
        styles.cursor === "not-allowed" || styles.pointerEvents === "none"
      ).toBeTruthy();
    }

    await expect(disabledSection).toHaveScreenshot("button-disabled.png", {
      maxDiffPixels: 100,
    });
  });

  test("loading buttons show spinner", async ({ page }) => {
    const loadingSection = page.locator('[data-test="button-loading"]');
    await expect(loadingSection).toBeVisible();

    const loadingBtns = [
      "btn-loading-primary",
      "btn-loading-secondary",
      "btn-loading-danger",
    ];

    for (const testId of loadingBtns) {
      const btn = page.locator(`[data-test="${testId}"]`);
      await expect(btn).toBeVisible();

      // Loading button should contain a spinner (SVG or animation element)
      const hasSpinner = await btn.evaluate((el) => {
        return el.querySelector("svg") !== null || el.querySelector(".hook0-spinner") !== null;
      });
      expect(hasSpinner).toBeTruthy();
    }

    await expect(loadingSection).toHaveScreenshot("button-loading.png", {
      maxDiffPixels: 100,
    });
  });

  test("full-width buttons span container width", async ({ page }) => {
    const fullWidthSection = page.locator('[data-test="button-fullwidth"]');
    await expect(fullWidthSection).toBeVisible();

    const primaryFull = page.locator('[data-test="btn-fullwidth-primary"]');
    const secondaryFull = page.locator('[data-test="btn-fullwidth-secondary"]');

    const primaryBox = await primaryFull.boundingBox();
    const secondaryBox = await secondaryFull.boundingBox();

    expect(primaryBox).not.toBeNull();
    expect(secondaryBox).not.toBeNull();

    if (primaryBox && secondaryBox) {
      // Full-width buttons should be wider than 300px (they should fill container)
      expect(primaryBox.width).toBeGreaterThan(300);
      expect(secondaryBox.width).toBeGreaterThan(300);
      // Both should have similar width (same container)
      expect(Math.abs(primaryBox.width - secondaryBox.width)).toBeLessThan(10);
    }

    await expect(fullWidthSection).toHaveScreenshot("button-fullwidth.png", {
      maxDiffPixels: 100,
    });
  });

  test("action buttons render as <button> elements", async ({ page }) => {
    const btn = page.locator('[data-test="btn-element-button"]');
    await expect(btn).toBeVisible();

    const tagName = await btn.evaluate((el) => el.tagName.toLowerCase());
    expect(tagName).toBe("button");

    // Action buttons should have type="button" by default
    const buttonType = await btn.evaluate((el) => el.getAttribute("type"));
    expect(buttonType).toBe("button");
  });

  test("navigation buttons render as <a> elements", async ({ page }) => {
    const btn = page.locator('[data-test="btn-element-anchor"]');
    await expect(btn).toBeVisible();

    const tagName = await btn.evaluate((el) => el.tagName.toLowerCase());
    expect(tagName).toBe("a");

    // Anchor should have href
    const href = await btn.evaluate((el) => el.getAttribute("href"));
    expect(href).toBe("https://example.com");
  });

  test("submit buttons render as <button type='submit'>", async ({ page }) => {
    const btn = page.locator('[data-test="btn-submit-primary"]');
    await expect(btn).toBeVisible();

    const tagName = await btn.evaluate((el) => el.tagName.toLowerCase());
    expect(tagName).toBe("button");

    const buttonType = await btn.evaluate((el) => el.getAttribute("type"));
    expect(buttonType).toBe("submit");
  });

  test("all variants have consistent flex layout", async ({ page }) => {
    const allButtons = [
      "btn-primary",
      "btn-secondary",
      "btn-danger",
      "btn-ghost",
      "btn-link",
    ];

    for (const testId of allButtons) {
      const btn = page.locator(`[data-test="${testId}"]`);
      const styles = await btn.evaluate((el) => {
        const computed = window.getComputedStyle(el);
        return {
          display: computed.display,
          flexWrap: computed.flexWrap,
          whiteSpace: computed.whiteSpace,
          alignItems: computed.alignItems,
        };
      });

      expect(styles.display).toContain("flex");
      expect(styles.flexWrap).toBe("nowrap");
      expect(styles.whiteSpace).toBe("nowrap");
      expect(styles.alignItems).toBe("center");
    }
  });

  test("full page screenshot of showcase", async ({ page }) => {
    await expect(page.locator('[data-test="component-showcase"]')).toHaveScreenshot(
      "button-showcase-full.png",
      {
        maxDiffPixels: 200,
      }
    );
  });
});
