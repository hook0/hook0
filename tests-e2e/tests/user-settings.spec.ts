import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * User Settings E2E tests for Hook0.
 *
 * Tests for viewing and updating user settings including password change.
 * Following the Three-Step Verification Pattern.
 */
test.describe("User Settings", () => {
  test("should display user settings page with all sections", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-settings-display-${timestamp}@hook0.local`;
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

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to settings page
    await page.goto("/settings");

    // Verify user info card is visible
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="user-email-input"]')).toBeVisible();

    // Verify change password card is visible
    await expect(page.locator('[data-test="change-password-card"]')).toBeVisible();
    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible();
    await expect(page.locator('[data-test="new-password-input"]')).toBeVisible();
    await expect(page.locator('[data-test="confirm-password-input"]')).toBeVisible();
    await expect(page.locator('[data-test="change-password-button"]')).toBeVisible();

    // Verify delete account card is visible
    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();
    await expect(page.locator('[data-test="delete-account-form"]')).toBeVisible();
    await expect(page.locator('[data-test="delete-account-button"]')).toBeVisible();
  });

  test("should display user email in personal information section", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-email-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify email is displayed and disabled (read-only)
    await expect(page.locator('[data-test="user-email-input"]')).toHaveValue(email);
    await expect(page.locator('[data-test="user-email-input"]')).toBeDisabled();
  });

  test("should change password successfully and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-password-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const newPassword = `NewPassword456!${timestamp}`;

    // Register and verify
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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill password change form
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/change-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="change-password-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown
    await expect(
      page.locator('[class*="Notivue"], [class*="notivue"], [role="alert"]').first()
    ).toBeVisible({
      timeout: 10000,
    });
  });

  test("should show error when passwords do not match", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-mismatch-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill mismatching passwords
    await page.locator('[data-test="new-password-input"]').fill("NewPassword123!");
    await page.locator('[data-test="confirm-password-input"]').fill("DifferentPassword456!");

    // Submit
    await page.locator('[data-test="change-password-button"]').click();

    // Verify error notification is shown
    await expect(
      page.locator('[class*="Notivue"], [class*="notivue"], [role="alert"]').first()
    ).toBeVisible({
      timeout: 10000,
    });
  });
});

test.describe("Password Reset Flow", () => {
  test("should display reset password form", async ({ page }) => {
    await page.goto("/begin-reset-password");

    // Verify form is visible
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="reset-password-email-input"]')).toBeVisible();
    await expect(page.locator('[data-test="reset-password-submit-button"]')).toBeVisible();
    await expect(page.locator('[data-test="reset-password-back-link"]')).toBeVisible();
  });

  test("should navigate back to login when clicking back link", async ({ page }) => {
    await page.goto("/begin-reset-password");

    await expect(page.locator('[data-test="reset-password-back-link"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="reset-password-back-link"]').click();

    await expect(page).toHaveURL(/\/login/);
    await expect(page.locator('[data-test="login-form"]')).toBeVisible();
  });

  test("should submit reset password request and verify API response", async ({ page }) => {
    const timestamp = Date.now();
    const email = `test-reset-${timestamp}@hook0.local`;

    await page.goto("/begin-reset-password");

    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill email
    await page.locator('[data-test="reset-password-email-input"]').fill(email);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="reset-password-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response (should succeed even for non-existent emails for security)
    expect(response.status()).toBeLessThan(500);

    // Verify success notification is shown
    await expect(
      page.locator('[class*="Notivue"], [class*="notivue"], [role="alert"]').first()
    ).toBeVisible({
      timeout: 10000,
    });
  });
});
