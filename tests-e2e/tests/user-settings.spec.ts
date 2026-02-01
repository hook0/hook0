import { test, expect } from "@playwright/test";
import {
  verifyEmailViaMailpit,
  API_BASE_URL,
  getPasswordResetTokenFromMailpit,
} from "../fixtures/email-verification";

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
        response.url().includes("/api/v1/auth/password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="change-password-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown
    await expect(
      page.locator('[data-test="toast-notification"]').first()
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
      page.locator('[data-test="toast-notification"]').first()
    ).toBeVisible({
      timeout: 10000,
    });
  });

  test("should show not implemented error when trying to delete account", async ({ page, request }) => {
    // Note: The delete account feature is not implemented yet.
    // The frontend shows "Not implemented yet" error when clicking delete.
    // This test verifies the error notification appears.

    // Setup - create a test user
    const timestamp = Date.now();
    const email = `test-delete-account-${timestamp}@hook0.local`;
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

    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler for delete confirmation
    page.on("dialog", (dialog) => {
      dialog.accept();
    });

    // Click delete - this will show "Not implemented yet" error
    await page.locator('[data-test="delete-account-button"]').click();

    // Verify error notification is shown (not implemented feature)
    await expect(
      page.locator('[data-test="toast-notification"]').first()
    ).toBeVisible({
      timeout: 10000,
    });

    // User should still be on settings page (not logged out)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });
  });

  test("should cancel account deletion when dialog is dismissed", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-cancel-delete-${timestamp}@hook0.local`;
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

    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click delete button
    await page.locator('[data-test="delete-account-button"]').click();

    // Should still be on settings page (not logged out)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });

    // Delete account card should still be visible
    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();
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
      page.locator('[data-test="toast-notification"]').first()
    ).toBeVisible({
      timeout: 10000,
    });
  });

  test("should complete password reset flow with valid token and login with new password", async ({
    page,
    request,
  }) => {
    // Step 1: Create a verified user first
    const timestamp = Date.now();
    const email = `test-reset-complete-${timestamp}@hook0.local`;
    const originalPassword = `OriginalPass123!${timestamp}`;
    const newPassword = `NewSecurePass456!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Reset",
        last_name: "Tester",
        password: originalPassword,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Step 2: Initiate password reset via API
    const beginResetResponse = await request.post(
      `${API_BASE_URL}/auth/begin-reset-password`,
      {
        data: { email },
      }
    );
    expect(beginResetResponse.status()).toBeLessThan(400);

    // Step 3: Get the password reset token from Mailpit
    const resetToken = await getPasswordResetTokenFromMailpit(request, email, 20000);
    expect(resetToken).toBeTruthy();

    // Step 4: Navigate to the reset password page with token
    await page.goto(`/reset-password?token=${resetToken}`);

    // Verify form is visible
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 5: Fill in new password
    await page.locator('[data-test="reset-password-new-password-input"]').fill(newPassword);
    await page
      .locator('[data-test="reset-password-confirm-password-input"]')
      .fill(newPassword);

    // Step 6: Submit and wait for API response
    const resetResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="reset-password-submit-button"]').click();

    const resetResponse = await resetResponsePromise;
    expect(resetResponse.status()).toBeLessThan(400);

    // Step 7: Should redirect to login page
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Step 8: Login with new password
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(newPassword);

    const loginResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/login") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();

    const loginResponse = await loginResponsePromise;
    expect(loginResponse.status()).toBeLessThan(400);

    // Step 9: Verify login succeeded - should redirect to dashboard/home
    await expect(page).toHaveURL(/\/(organizations|tutorial)/, { timeout: 15000 });
  });
});
