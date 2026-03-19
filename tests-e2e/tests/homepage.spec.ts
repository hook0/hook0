import { test, expect } from "@playwright/test";
import { loginAsNewUser } from "../fixtures/test-setup";

/**
 * Homepage E2E tests for Hook0.
 *
 * Tests the homepage behavior including authentication redirects
 * and basic navigation.
 */
test.describe("Homepage", () => {
  test("should redirect to login when not authenticated", async ({ page }) => {
    // Navigate to homepage
    await page.goto("/");

    // Should redirect to login page when not authenticated
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Verify login form is displayed
    await expect(page.locator('[data-test="login-form"]')).toBeVisible();
  });

  test("should display Hook0 branding on login page", async ({ page }) => {
    await page.goto("/login");

    // Check the page loads with Hook0 logo
    await expect(page.locator('[data-test="login-logo"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify logo has correct alt text
    await expect(page.locator('[data-test="login-logo"]')).toHaveAttribute("alt", "Hook0");
  });

  test("should have complete navigation flow from login to register and back", async ({ page }) => {
    // Start at login page
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Navigate to register page
    await page.locator('[data-test="login-register-link"]').click();
    await expect(page).toHaveURL(/\/register/);
    await expect(page.locator('[data-test="register-form"]')).toBeVisible();

    // Navigate back to login page
    await page.locator('[data-test="register-login-link"]').click();
    await expect(page).toHaveURL(/\/login/);
    await expect(page.locator('[data-test="login-form"]')).toBeVisible();
  });

  test("should display organization cards on home page", async ({ page, request }) => {
    await loginAsNewUser(page, request, "home-cards");

    // Navigate to home page
    await page.goto("/");

    // Verify home page and banner are visible
    await expect(page.locator('[data-test="home-page"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="home-banner"]')).toBeVisible({ timeout: 10000 });
  });

  test("should preserve redirect after authentication", async ({ page, request }) => {
    await loginAsNewUser(page, request, "redirect");

    // Should be in an authenticated area (not on login)
    await expect(page).not.toHaveURL(/\/login/, {
      timeout: 15000,
    });
  });
});
