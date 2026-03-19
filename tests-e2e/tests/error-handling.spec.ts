import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Error Handling E2E tests for Hook0.
 *
 * Tests error states when navigating to non-existent resources
 * (valid routes but invalid IDs that trigger API errors).
 * Distinct from error-404.spec.ts which tests unknown routes.
 */
test.describe("Error Handling", () => {
  /**
   * Helper to setup authenticated test environment
   */
  async function setupAuthenticatedUser(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-err-${testId}-${timestamp}@hook0.local`;
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

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    return { email, password, organizationId: organizationId!, timestamp };
  }

  test("should show error for non-existent organization", async ({ page, request }) => {
    await setupAuthenticatedUser(page, request, "bad-org");

    // Navigate to a non-existent organization UUID
    const fakeOrgId = "00000000-0000-0000-0000-000000000000";
    await page.goto(`/organizations/${fakeOrgId}/dashboard`);

    // The page should show an error card (Hook0ErrorCard renders .hook0-error-card)
    // OrganizationsDashboard.vue shows Hook0ErrorCard on orgError or when !organization
    await expect(page.locator('.hook0-error-card')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('.hook0-error-card-title')).toBeVisible();
    await expect(page.locator('.hook0-error-card-detail')).toBeVisible();
  });

  test("should show error for non-existent application in valid organization", async ({
    page,
    request,
  }) => {
    const env = await setupAuthenticatedUser(page, request, "bad-app");

    // Navigate to a valid org but fake application UUID
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/dashboard`
    );

    // The page should show an error card
    // ApplicationsDashboard.vue shows Hook0ErrorCard on appError
    await expect(page.locator('.hook0-error-card')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('.hook0-error-card-title')).toBeVisible();
    await expect(page.locator('.hook0-error-card-detail')).toBeVisible();
  });

  test("should show error for non-existent application sub-page", async ({ page, request }) => {
    const env = await setupAuthenticatedUser(page, request, "bad-app-sub");

    // Navigate to a valid org but fake app UUID on a sub-page (event_types)
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/event_types`
    );

    // Should show an error card or error state
    await expect(page.locator('.hook0-error-card')).toBeVisible({ timeout: 15000 });
  });
});
