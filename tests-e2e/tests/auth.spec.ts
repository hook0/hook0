import { test, expect } from "@playwright/test";

/**
 * Authentication tests for Hook0.
 * Tests the login and registration flows.
 */
test.describe("Authentication", () => {
  test.describe("Login Page", () => {
    test("should display login form", async ({ page }) => {
      await page.goto("/login");

      // Check the page has a login form with email and password fields
      await expect(page.locator("#email")).toBeVisible({ timeout: 10000 });
      await expect(page.locator("#password")).toBeVisible();

      // Check the submit button is visible
      await expect(
        page.getByRole("button", { name: /sign in/i })
      ).toBeVisible();
    });

    test("should have link to register page", async ({ page }) => {
      await page.goto("/login");

      // Check link to register page
      const registerLink = page.getByRole("link", {
        name: /create an account/i,
      });
      await expect(registerLink).toBeVisible();

      // Click and verify navigation
      await registerLink.click();
      await expect(page).toHaveURL(/\/register/);
    });

    test("should have link to forgot password", async ({ page }) => {
      await page.goto("/login");

      // Check forgot password link exists
      const forgotLink = page.getByRole("link", { name: /forgot password/i });
      await expect(forgotLink).toBeVisible();
    });

    test("should show error for invalid credentials", async ({ page }) => {
      await page.goto("/login");

      // Fill in invalid credentials
      await page.locator("#email").fill("invalid@example.com");
      await page.locator("#password").fill("wrongpassword");

      // Submit the form
      await page.getByRole("button", { name: /sign in/i }).click();

      // Should show an error - either as a toast notification or inline error message
      // Wait for either the toast container or an error state on the form
      await expect(
        page.locator("[class*='Notivue'], [class*='notivue'], [role='alert'], .error-message, [class*='error']").first()
      ).toBeVisible({
        timeout: 10000,
      });
    });
  });

  test.describe("Register Page", () => {
    test("should display registration form", async ({ page }) => {
      await page.goto("/register");

      // Check the form fields are visible
      await expect(page.locator("#email")).toBeVisible({ timeout: 10000 });
      await expect(page.locator("#firstName")).toBeVisible();
      await expect(page.locator("#lastName")).toBeVisible();
      await expect(page.locator("#password")).toBeVisible();

      // Check submit button
      await expect(
        page.getByRole("button", { name: /create account/i })
      ).toBeVisible();
    });

    test("should have link back to login page", async ({ page }) => {
      await page.goto("/register");

      // Check link to login page
      const loginLink = page.getByRole("link", { name: /sign in/i });
      await expect(loginLink).toBeVisible();

      // Click and verify navigation
      await loginLink.click();
      await expect(page).toHaveURL(/\/login/);
    });

    test("should display benefits and trust indicators", async ({ page }) => {
      await page.goto("/register");

      // Check that the benefits are displayed (use .first() since text appears multiple times)
      await expect(page.getByText(/no credit card required/i).first()).toBeVisible({
        timeout: 10000,
      });
      await expect(page.getByText(/100 free events/i).first()).toBeVisible();

      // Check trust indicators
      await expect(page.getByText(/gdpr compliant/i).first()).toBeVisible();
    });
  });
});
