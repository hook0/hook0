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

  test("should display events per day chart on organization dashboard", async ({
    page,
    request,
  }) => {
    const timestamp = Date.now();
    const email = `test-org-chart-${timestamp}@hook0.local`;
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
    await expect(page.locator('[data-test="org-dashboard-page"]')).toBeVisible({ timeout: 15000 });

    // Verify the events per day chart section renders with KPI stats
    // The chart component renders .chart__stat-label spans with "Total events", "Avg / day", "Peak day"
    await expect(page.locator('.chart__stats')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.chart__stat-label').filter({ hasText: "Total events" })).toBeVisible();
    await expect(page.locator('.chart__stat-label').filter({ hasText: "Avg / day" })).toBeVisible();
    await expect(page.locator('.chart__stat-label').filter({ hasText: "Peak day" })).toBeVisible();

    // Verify day preset buttons are rendered (7d, 30d, 90d)
    await expect(page.locator('.chart__toolbar')).toBeVisible();
    await expect(page.locator('.chart__toolbar button').first()).toBeVisible();
  });

  test("should display organization dashboard card with org info", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-org-card-${timestamp}@hook0.local`;
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
    await expect(page.locator('[data-test="org-dashboard-page"]')).toBeVisible({ timeout: 15000 });

    // Verify the organization dashboard card renders
    await expect(page.locator('[data-test="organization-dashboard-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify the card contains the "Organization" label text
    await expect(page.locator('[data-test="organization-dashboard-card"]')).toContainText(
      "Organization"
    );

    // Verify applications list is embedded in the org dashboard
    await expect(page.locator('[data-test="applications-card"]')).toBeVisible({ timeout: 10000 });
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
