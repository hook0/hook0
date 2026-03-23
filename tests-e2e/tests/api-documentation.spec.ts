import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * API Documentation E2E tests for Hook0.
 *
 * Tests the API documentation page (may be Swagger UI or an iframe).
 */
test.describe("API Documentation", () => {
  test("should display API documentation page", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-apidocs-${timestamp}@hook0.local`;
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

    // Navigate to API documentation
    await page.goto("/api/documentation");

    // Verify API docs page is visible (could be a dedicated page, Swagger iframe, or redirect)
    await expect(page.locator('[data-test="api-docs-page"]')).toBeVisible({
      timeout: 15000,
    });
  });
});
