import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Error 404 Page E2E tests for Hook0.
 *
 * Tests the 404 error page behavior for non-existent routes.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Error 404 Page", () => {
  /**
   * Helper to setup test environment with authenticated user
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-404-${testId}-${timestamp}@hook0.local`;
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

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect to any authenticated area
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    return { email, password, timestamp };
  }

  test("should display 404 page for non-existent route", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "display");

    // Navigate to a non-existent route
    await page.goto("/this-page-does-not-exist");

    // Verify 404 page elements
    await expect(page.locator('[data-test="error-404-page"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="error-404-title"]')).toBeVisible();
  });

  test("should navigate to dashboard from 404 page", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "dashboard");

    // Navigate to a non-existent route
    await page.goto("/this-page-does-not-exist");

    // Verify 404 page is displayed
    await expect(page.locator('[data-test="error-404-page"]')).toBeVisible({
      timeout: 10000,
    });

    // Click dashboard button
    await expect(page.locator('[data-test="error-404-home-button"]')).toBeVisible();
    await page.locator('[data-test="error-404-home-button"]').click();

    // Verify redirect to home/dashboard
    await expect(page).toHaveURL(/\/$|\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });
  });

  test("should go back from 404 page", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "back");

    // First, navigate to home to establish history
    await page.goto("/");
    await expect(page).toHaveURL(/\/$|\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to a non-existent route
    await page.goto("/this-page-does-not-exist");

    // Verify 404 page is displayed
    await expect(page.locator('[data-test="error-404-page"]')).toBeVisible({
      timeout: 10000,
    });

    // Click back button
    await expect(page.locator('[data-test="error-404-back-button"]')).toBeVisible();
    await page.locator('[data-test="error-404-back-button"]').click();

    // Verify navigation back to previous page (home)
    await expect(page).toHaveURL(/\/$|\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });
  });
});
