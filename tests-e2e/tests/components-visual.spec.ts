import { test, expect } from "@playwright/test";
import {
  verifyEmailViaMailpit,
  API_BASE_URL,
} from "../fixtures/email-verification";

/**
 * Visual regression tests for Hook0 UI components.
 *
 * These tests ensure that components render correctly and don't regress visually.
 */
test.describe("Component Visual Regression", () => {
  test.describe("Hook0Button", () => {
    test("buttons with icons should have icon and text on same baseline", async ({
      page,
      request,
    }) => {
      // Setup: Create test user and login
      const timestamp = Date.now();
      const email = `test-button-visual-${timestamp}@hook0.local`;
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

      // Navigate to settings page which has buttons with icons
      await page.goto("/settings");

      // Wait for the page to fully load
      await expect(page.locator('[data-test="change-password-card"]')).toBeVisible({
        timeout: 10000,
      });

      // Get the change password button
      const changePasswordButton = page.locator('[data-test="change-password-button"]');
      await expect(changePasswordButton).toBeVisible();

      // Take screenshot of the change password button - icon and text must be on same line
      await expect(changePasswordButton).toHaveScreenshot("button-change-password.png", {
        maxDiffPixels: 50,
      });

      // Get the delete account button
      const deleteButton = page.locator('[data-test="delete-account-button"]');
      await expect(deleteButton).toBeVisible();

      // Take screenshot of the delete button - icon and text must be on same line
      await expect(deleteButton).toHaveScreenshot("button-delete-account.png", {
        maxDiffPixels: 50,
      });

      // Verify button layout programmatically: icon and text should be on same Y position
      // This checks that the button is not wrapping content
      const changePasswordBox = await changePasswordButton.boundingBox();
      expect(changePasswordBox).not.toBeNull();
      if (changePasswordBox) {
        // A properly laid out button with icon + text should have height < 50px
        // If text wraps to second line, height would be much larger
        expect(changePasswordBox.height).toBeLessThan(50);
      }

      const deleteButtonBox = await deleteButton.boundingBox();
      expect(deleteButtonBox).not.toBeNull();
      if (deleteButtonBox) {
        // Same check for delete button
        expect(deleteButtonBox.height).toBeLessThan(50);
      }
    });

    test("button content should never wrap to multiple lines", async ({
      page,
      request,
    }) => {
      // Setup: Create test user and login
      const timestamp = Date.now();
      const email = `test-button-nowrap-${timestamp}@hook0.local`;
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

      // Wait for redirect
      await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
        timeout: 15000,
      });

      // Navigate to settings
      await page.goto("/settings");
      await expect(page.locator('[data-test="change-password-card"]')).toBeVisible({
        timeout: 10000,
      });

      // Check that buttons have flex-wrap: nowrap and white-space: nowrap
      const changePasswordButton = page.locator('[data-test="change-password-button"]');
      const deleteButton = page.locator('[data-test="delete-account-button"]');

      // Verify CSS properties that prevent wrapping
      const changePasswordStyles = await changePasswordButton.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          flexWrap: styles.flexWrap,
          whiteSpace: styles.whiteSpace,
          display: styles.display,
        };
      });

      expect(changePasswordStyles.flexWrap).toBe("nowrap");
      expect(changePasswordStyles.whiteSpace).toBe("nowrap");
      expect(changePasswordStyles.display).toContain("flex");

      const deleteStyles = await deleteButton.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          flexWrap: styles.flexWrap,
          whiteSpace: styles.whiteSpace,
          display: styles.display,
        };
      });

      expect(deleteStyles.flexWrap).toBe("nowrap");
      expect(deleteStyles.whiteSpace).toBe("nowrap");
      expect(deleteStyles.display).toContain("flex");
    });
  });
});
