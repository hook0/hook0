import { test, expect } from "@playwright/test";
import { loginAsNewUser } from "../fixtures/test-setup";

/**
 * Keyboard Shortcuts E2E tests for Hook0.
 *
 * Tests the keyboard shortcuts cheat sheet dialog.
 */
test.describe("Keyboard Shortcuts", () => {
  test("should open shortcuts cheat sheet with ? key", async ({ page, request }) => {
    await loginAsNewUser(page, request, "shortcuts-open");

    // Press ? to open shortcuts dialog
    await page.keyboard.press("?");

    await expect(page.locator('[data-test="shortcuts-dialog"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should close shortcuts dialog with Escape", async ({ page, request }) => {
    await loginAsNewUser(page, request, "shortcuts-close");

    // Open shortcuts dialog
    await page.keyboard.press("?");
    await expect(page.locator('[data-test="shortcuts-dialog"]')).toBeVisible({
      timeout: 10000,
    });

    // Close with Escape
    await page.keyboard.press("Escape");

    await expect(page.locator('[data-test="shortcuts-dialog"]')).not.toBeVisible({
      timeout: 10000,
    });
  });
});
