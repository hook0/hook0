import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Dark Mode E2E tests for Hook0.
 *
 * Tests toggling dark mode via user settings, persistence across reload,
 * and persistence after logout.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Dark Mode", () => {
  /**
   * Helper to register, verify, and login a test user.
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ): Promise<{
    email: string;
    password: string;
    organizationId: string;
    timestamp: number;
  }> {
    const timestamp = Date.now();
    const email = `test-dark-${testId}-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email and get organization ID
    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();
    if (!organizationId) {
      throw new Error("Organization ID is required");
    }

    // Login via UI
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

    return {
      email,
      password,
      organizationId,
      timestamp,
    };
  }

  test("should toggle dark mode from user settings", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "toggle");

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
    await setupTestEnvironment(page, request, "persist-reload");

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
    await setupTestEnvironment(page, request, "persist-logout");

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
    await page.locator('.hook0-topnav__user-trigger').click();
    const logoutButton = page.locator('.hook0-topnav__dropdown-item--danger');
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
