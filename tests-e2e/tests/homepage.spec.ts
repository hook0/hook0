import { test, expect } from "@playwright/test";

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
    await expect(page.locator('[data-test="login-logo"]')).toHaveAttribute(
      "alt",
      "Hook0"
    );
  });

  test("should have complete navigation flow from login to register and back", async ({
    page,
  }) => {
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

  test("should preserve redirect after authentication", async ({
    page,
    request,
  }) => {
    // Setup: Create a test user
    const timestamp = Date.now();
    const email = `test-redirect-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post("/api/v1/register", {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Try to access protected route (should redirect to login)
    await page.goto("/");
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Login with valid credentials
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);

    const responsePromise = page.waitForResponse(
      (response) =>
        (response.url().includes("/api/v1/login") ||
          response.url().includes("/iam/login")) &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();

    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    // Should be redirected to dashboard/organizations/tutorial after login
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });
  });
});
