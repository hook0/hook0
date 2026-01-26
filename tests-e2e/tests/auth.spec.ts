import { test, expect } from "@playwright/test";

/**
 * Authentication E2E tests.
 *
 * Tests for registration, login, and logout flows.
 */
test.describe("Authentication", () => {
  test.describe("Registration", () => {
    test("should register new user with required fields only and verify API response", async ({
      page,
    }) => {
      const timestamp = Date.now();
      const email = `test-required-${timestamp}@hook0.local`;
      const password = `TestPass123!${timestamp}`;

      await page.goto("/register");
      await expect(page.locator('[data-test="signup-form"]')).toBeVisible();

      await page.locator('[data-test="email-input"]').fill(email);
      await page.locator('[data-test="password-input"]').fill(password);
      await page.locator('[data-test="password-confirm-input"]').fill(password);

      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/register") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="signup-button"]').click();

      const response = await responsePromise;
      expect(response.status()).toBeLessThan(400);

      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/verify/, { timeout: 15000 });
    });

    test("should show validation errors for invalid email", async ({ page }) => {
      await page.goto("/register");
      await expect(page.locator('[data-test="signup-form"]')).toBeVisible();

      await page.locator('[data-test="email-input"]').fill("invalid-email");
      await page.locator('[data-test="password-input"]').fill("TestPass123!");
      await page.locator('[data-test="password-confirm-input"]').fill("TestPass123!");

      await page.locator('[data-test="signup-button"]').click();

      await expect(page.locator('[data-test="email-error"]')).toBeVisible({ timeout: 5000 });
    });
  });

  test.describe("Login", () => {
    test("should login with valid credentials and verify redirect", async ({ page, request }) => {
      const timestamp = Date.now();
      const email = `test-login-${timestamp}@hook0.local`;
      const password = `TestPass123!${timestamp}`;

      const registerResponse = await request.post("/api/v1/register", {
        data: {
          email,
          password,
          password_confirmation: password,
        },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible();

      await page.locator('[data-test="email-input"]').fill(email);
      await page.locator('[data-test="password-input"]').fill(password);

      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/login") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="login-button"]').click();

      const response = await responsePromise;
      expect(response.status()).toBeLessThan(400);

      await expect(page).toHaveURL(/\/dashboard|\/organizations/, { timeout: 15000 });
    });

    test("should show error for invalid credentials", async ({ page }) => {
      await page.goto("/login");
      await expect(page.locator('[data-test="login-form"]')).toBeVisible();

      await page.locator('[data-test="email-input"]').fill("nonexistent@hook0.local");
      await page.locator('[data-test="password-input"]').fill("WrongPassword123!");

      await page.locator('[data-test="login-button"]').click();

      await expect(page.locator('[data-test="login-error"]')).toBeVisible({ timeout: 10000 });
    });
  });

  test.describe("Logout", () => {
    test("should logout and redirect to login page", async ({ page, request }) => {
      const timestamp = Date.now();
      const email = `test-logout-${timestamp}@hook0.local`;
      const password = `TestPass123!${timestamp}`;

      await request.post("/api/v1/register", {
        data: {
          email,
          password,
          password_confirmation: password,
        },
      });

      await page.goto("/login");
      await page.locator('[data-test="email-input"]').fill(email);
      await page.locator('[data-test="password-input"]').fill(password);
      await page.locator('[data-test="login-button"]').click();

      await expect(page).toHaveURL(/\/dashboard|\/organizations/, { timeout: 15000 });

      await page.locator('[data-test="user-menu"]').click();
      await page.locator('[data-test="logout-button"]').click();

      await expect(page).toHaveURL(/\/login|\//, { timeout: 10000 });
    });
  });
});
