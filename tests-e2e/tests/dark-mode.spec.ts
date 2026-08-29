import { test, expect } from "@playwright/test";
import { loginAsNewUser } from "../fixtures/test-setup";

/**
 * Dark Mode E2E tests for Hook0.
 *
 * Tests toggling dark mode via user settings, persistence across reload,
 * and persistence after logout.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Dark Mode", () => {
  test("should toggle dark mode from user settings", async ({ page, request }) => {
    await loginAsNewUser(page, request, "dark-toggle");

    // Step 1: Navigate to user settings
    await page.goto("/settings");
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 2: Find theme select and choose "dark"
    const themeSelect = page.locator('[data-test="theme-select"]');
    await expect(themeSelect).toBeVisible({ timeout: 5000 });
    await themeSelect.selectOption("dark");

    // Step 3: Verify document.documentElement has .dark class
    const hasDarkClass = await page.evaluate(() =>
      document.documentElement.classList.contains("dark")
    );
    expect(hasDarkClass).toBe(true);
  });

  test("should persist dark mode across page reload", async ({ page, request }) => {
    await loginAsNewUser(page, request, "dark-persist-reload");

    // Step 1: Enable dark mode
    await page.goto("/settings");
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    const themeSelect = page.locator('[data-test="theme-select"]');
    await expect(themeSelect).toBeVisible({ timeout: 5000 });
    await themeSelect.selectOption("dark");

    // Verify dark mode is active
    const hasDarkBefore = await page.evaluate(() =>
      document.documentElement.classList.contains("dark")
    );
    expect(hasDarkBefore).toBe(true);

    // Step 2: Reload page
    await page.reload();

    // Wait for page to be fully loaded
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 3: Verify dark mode persisted
    const hasDarkAfter = await page.evaluate(() =>
      document.documentElement.classList.contains("dark")
    );
    expect(hasDarkAfter).toBe(true);
  });

  test("should persist dark mode after logout", async ({ page, request }) => {
    await loginAsNewUser(page, request, "dark-persist-logout");

    // Step 1: Enable dark mode
    await page.goto("/settings");
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    const themeSelect = page.locator('[data-test="theme-select"]');
    await expect(themeSelect).toBeVisible({ timeout: 5000 });
    await themeSelect.selectOption("dark");

    // Verify dark mode is active
    const hasDarkBefore = await page.evaluate(() =>
      document.documentElement.classList.contains("dark")
    );
    expect(hasDarkBefore).toBe(true);

    // Step 2: Logout via navigation to login (clearing auth triggers redirect)
    // Use the user dropdown menu to logout
    await page.locator(".hook0-topnav__user-trigger").click();
    const logoutButton = page.locator(".hook0-topnav__dropdown-item--danger");
    await expect(logoutButton).toBeVisible({ timeout: 5000 });
    await logoutButton.click();

    // Wait for redirect to login page
    await expect(page).toHaveURL(/\/login/, { timeout: 15000 });
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 3: Verify dark mode persists on login page
    const hasDarkAfterLogout = await page.evaluate(() =>
      document.documentElement.classList.contains("dark")
    );
    expect(hasDarkAfterLogout).toBe(true);
  });
});
