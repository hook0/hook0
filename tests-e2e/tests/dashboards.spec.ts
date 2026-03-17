import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Dashboard E2E tests for Hook0.
 *
 * Tests organization and application dashboard pages render correctly
 * with their expected widgets and content.
 */
test.describe("Dashboards", () => {
  test("should display organization dashboard", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-org-dash-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Navigate to organization dashboard
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Verify organization dashboard page renders
    await expect(page.locator('[data-test="org-dashboard-page"]')).toBeVisible({ timeout: 15000 });
  });

  test("should display application dashboard with tutorial widget", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-app-dash-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Dashboard Test App ${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Create an application
    await page.goto(`/organizations/${organizationId}/applications/new`);
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createAppResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);
    const app = await appResponse.json();
    const applicationId = app.application_id;

    // Navigate to application dashboard
    await page.goto(`/organizations/${organizationId}/applications/${applicationId}/dashboard`);

    // Verify application dashboard page renders
    await expect(page.locator('[data-test="app-dashboard-page"]')).toBeVisible({ timeout: 15000 });

    // Verify tutorial widget section is visible
    const tutorialWidget = page.locator(
      '[data-test="app-dashboard-tutorial-widget"], [data-test="event-types-card"]'
    ).first();
    await expect(tutorialWidget).toBeVisible({ timeout: 10000 });
  });
});
