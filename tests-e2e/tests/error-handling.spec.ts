import { test, expect } from "@playwright/test";
import { loginAsNewUser } from "../fixtures/test-setup";

/**
 * Error Handling E2E tests for Hook0.
 *
 * Tests error states when navigating to non-existent resources
 * (valid routes but invalid IDs that trigger API errors).
 * Distinct from error-404.spec.ts which tests unknown routes.
 */
test.describe("Error Handling", () => {
  test("should show error for non-existent organization", async ({ page, request }) => {
    await loginAsNewUser(page, request, "bad-org");

    // Navigate to a non-existent organization UUID
    const fakeOrgId = "00000000-0000-0000-0000-000000000000";
    await page.goto(`/organizations/${fakeOrgId}/dashboard`);

    // The page should show an error card (Hook0ErrorCard renders .hook0-error-card)
    // OrganizationsDashboard.vue shows Hook0ErrorCard on orgError or when !organization
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('[data-test="error-card-title"]')).toBeVisible();
    await expect(page.locator('[data-test="error-card-detail"]')).toBeVisible();
  });

  test("should show error for non-existent application in valid organization", async ({
    page,
    request,
  }) => {
    const env = await loginAsNewUser(page, request, "bad-app");

    // Navigate to a valid org but fake application UUID
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/dashboard`
    );

    // The page should show an error card
    // ApplicationsDashboard.vue shows Hook0ErrorCard on appError
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('[data-test="error-card-title"]')).toBeVisible();
    await expect(page.locator('[data-test="error-card-detail"]')).toBeVisible();
  });

  test("should show error for non-existent application sub-page", async ({ page, request }) => {
    const env = await loginAsNewUser(page, request, "bad-app-sub");

    // Navigate to a valid org but fake app UUID on a sub-page (event_types)
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/event_types`
    );

    // Should show an error card or error state
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });
  });
});
