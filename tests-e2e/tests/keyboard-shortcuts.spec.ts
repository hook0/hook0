import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Keyboard Shortcuts E2E tests for Hook0.
 *
 * Tests the keyboard shortcuts cheat sheet dialog.
 */
test.describe("Keyboard Shortcuts", () => {
  test("should open shortcuts cheat sheet with ? key", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-shortcuts-open-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    await verifyEmailViaMailpit(request, email);

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Press ? to open shortcuts dialog
    await page.keyboard.press("?");

    await expect(page.locator('[data-test="shortcuts-dialog"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should close shortcuts dialog with Escape", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-shortcuts-close-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    await verifyEmailViaMailpit(request, email);

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

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
