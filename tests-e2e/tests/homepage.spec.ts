import { test, expect } from "@playwright/test";

/**
 * Homepage E2E tests.
 *
 * Basic smoke tests to verify the application loads correctly.
 */
test.describe("Homepage", () => {
  test("should load the homepage", async ({ page }) => {
    await page.goto("/");

    await expect(page).toHaveTitle(/Hook0/);
    await expect(page.locator('[data-test="login-link"], [data-test="signup-link"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate to login page", async ({ page }) => {
    await page.goto("/");

    await page.locator('[data-test="login-link"]').click();
    await expect(page).toHaveURL(/\/login/);
    await expect(page.locator('[data-test="login-form"]')).toBeVisible();
  });

  test("should navigate to signup page", async ({ page }) => {
    await page.goto("/");

    await page.locator('[data-test="signup-link"]').click();
    await expect(page).toHaveURL(/\/register|\/signup/);
    await expect(page.locator('[data-test="signup-form"]')).toBeVisible();
  });
});
