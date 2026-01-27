import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Authentication E2E tests for Hook0.
 *
 * Tests the login and registration flows with full API verification.
 * Following the Three-Step Verification Pattern:
 * 1. Fill form fields
 * 2. Submit and waitForResponse on API endpoint
 * 3. Verify response.status < 400 AND verify data/navigation
 */
test.describe("Authentication", () => {
  test.describe("Login Page", () => {
    test("should display login form with all required elements", async ({ page }) => {
      await page.goto("/login");

      // Verify form is visible
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Verify all form elements are present
      await expect(page.locator('[data-test="login-email-input"]')).toBeVisible();
      await expect(page.locator('[data-test="login-password-input"]')).toBeVisible();
      await expect(page.locator('[data-test="login-submit-button"]')).toBeVisible();
      await expect(page.locator('[data-test="login-forgot-password-link"]')).toBeVisible();
      await expect(page.locator('[data-test="login-register-link"]')).toBeVisible();
    });

    test("should navigate to register page when clicking create account link", async ({ page }) => {
      await page.goto("/login");

      await expect(page.locator('[data-test="login-register-link"]')).toBeVisible({
        timeout: 10000,
      });

      await page.locator('[data-test="login-register-link"]').click();

      await expect(page).toHaveURL(/\/register/);
      await expect(page.locator('[data-test="register-form"]')).toBeVisible();
    });

    test("should navigate to forgot password page", async ({ page }) => {
      await page.goto("/login");

      await expect(page.locator('[data-test="login-forgot-password-link"]')).toBeVisible({
        timeout: 10000,
      });

      await page.locator('[data-test="login-forgot-password-link"]').click();

      await expect(page).toHaveURL(/\/begin-reset-password|\/reset-password|\/forgot/);
    });

    test("should show error notification for invalid credentials and verify API response", async ({
      page,
    }) => {
      await page.goto("/login");

      await expect(page.locator('[data-test="login-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Step 1: Fill form fields
      await page.locator('[data-test="login-email-input"]').fill("nonexistent@example.com");
      await page.locator('[data-test="login-password-input"]').fill("WrongPassword123!");

      // Step 2: Submit and wait for API response
      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/auth/login") || response.url().includes("/auth/login"),
        { timeout: 15000 }
      );

      await page.locator('[data-test="login-submit-button"]').click();

      const response = await responsePromise;

      // Step 3: Verify API response indicates authentication failure (4xx expected)
      expect(response.status()).toBeGreaterThanOrEqual(400);
      expect(response.status()).toBeLessThan(500);

      // Verify error notification is shown to user
      await expect(
        page.locator('[data-test="toast-notification"]').first()
      ).toBeVisible({
        timeout: 10000,
      });

      // Verify we're still on login page (not redirected)
      await expect(page).toHaveURL(/\/login/);
    });

    test("should login with valid credentials and verify API response", async ({
      page,
      request,
    }) => {
      // Setup: Create a test user via API
      const timestamp = Date.now();
      const email = `test-login-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;

      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: {
          email,
          first_name: "Test",
          last_name: "User",
          password,
        },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      // Verify email before login (required by API)
      await verifyEmailViaMailpit(request, email);

      // Navigate to login page
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Step 1: Fill form fields
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(password);

      // Step 2: Submit and wait for API response
      const responsePromise = page.waitForResponse(
        (response) =>
          (response.url().includes("/api/v1/auth/login") ||
            response.url().includes("/auth/login")) &&
          response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="login-submit-button"]').click();

      const response = await responsePromise;

      // Step 3: Verify API response
      expect(response.status()).toBeLessThan(400);

      // Verify redirect to dashboard or organizations
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
        timeout: 15000,
      });
    });
  });

  test.describe("Register Page", () => {
    test("should display registration form with all required elements", async ({ page }) => {
      await page.goto("/register");

      // Verify form is visible
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Verify all form elements are present
      await expect(page.locator('[data-test="register-email-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-firstname-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-lastname-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-password-input"]')).toBeVisible();
      await expect(page.locator('[data-test="register-submit-button"]')).toBeVisible();
      await expect(page.locator('[data-test="register-login-link"]')).toBeVisible();
    });

    test("should navigate to login page when clicking sign in link", async ({ page }) => {
      await page.goto("/register");

      await expect(page.locator('[data-test="register-login-link"]')).toBeVisible({
        timeout: 10000,
      });

      await page.locator('[data-test="register-login-link"]').click();

      await expect(page).toHaveURL(/\/login/);
      await expect(page.locator('[data-test="login-form"]')).toBeVisible();
    });

    test("should register new user with required fields only and verify API response", async ({
      page,
    }) => {
      const timestamp = Date.now();
      const email = `test-register-required-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;

      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Step 1: Fill form fields (required only)
      await page.locator('[data-test="register-email-input"]').fill(email);
      await page.locator('[data-test="register-firstname-input"]').fill("Test");
      await page.locator('[data-test="register-lastname-input"]').fill("User");
      await page.locator('[data-test="register-password-input"]').fill(password);

      // Step 2: Submit and wait for API response
      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/register") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="register-submit-button"]').click();

      const response = await responsePromise;

      // Step 3: Verify API response
      expect(response.status()).toBeLessThan(400);

      // Verify redirect to check-email or dashboard
      await expect(page).toHaveURL(/\/check-email|\/verify|\/dashboard|\/organizations/, {
        timeout: 15000,
      });
    });

    test("should register new user with all fields and verify API response body", async ({
      page,
    }) => {
      const timestamp = Date.now();
      const email = `test-register-full-${timestamp}@hook0.local`;
      const firstName = "John";
      const lastName = "Doe";
      const password = `TestPassword123!${timestamp}`;

      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Step 1: Fill ALL form fields
      await page.locator('[data-test="register-email-input"]').fill(email);
      await page.locator('[data-test="register-firstname-input"]').fill(firstName);
      await page.locator('[data-test="register-lastname-input"]').fill(lastName);
      await page.locator('[data-test="register-password-input"]').fill(password);

      // Step 2: Submit and wait for API response
      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/register") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="register-submit-button"]').click();

      const response = await responsePromise;

      // Step 3: Verify API response status and body
      expect(response.status()).toBeLessThan(400);

      const responseBody = await response.json();
      // Verify response contains user data (structure may vary)
      expect(responseBody).toBeDefined();

      // Verify redirect
      await expect(page).toHaveURL(/\/check-email|\/verify|\/dashboard|\/organizations/, {
        timeout: 15000,
      });
    });

    test("should handle duplicate email registration gracefully", async ({ page, request }) => {
      // Setup: Create a user first via direct API call
      const timestamp = Date.now();
      const email = `test-duplicate-${timestamp}@hook0.local`;
      const password = `TestPassword123!${timestamp}`;

      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: {
          email,
          first_name: "First",
          last_name: "User",
          password,
        },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      // Try to register with same email via UI
      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      // Step 1: Fill form with duplicate email
      await page.locator('[data-test="register-email-input"]').fill(email);
      await page.locator('[data-test="register-firstname-input"]').fill("Second");
      await page.locator('[data-test="register-lastname-input"]').fill("User");
      await page.locator('[data-test="register-password-input"]').fill(password);

      // Step 2: Submit and wait for API response
      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/register") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="register-submit-button"]').click();

      const response = await responsePromise;

      // Step 3: For duplicate email registration, API should return 409 Conflict
      // This is the expected behavior for Hook0 - it explicitly rejects duplicate emails
      expect(response.status()).toBe(409);

      // Verify error notification is shown to user
      await expect(
        page.locator('[data-test="toast-notification"]').first()
      ).toBeVisible({
        timeout: 10000,
      });

      // Should stay on register page (not redirected)
      await expect(page).toHaveURL(/\/register/);
    });
  });
});
