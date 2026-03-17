import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Mobile Navigation E2E tests for Hook0.
 *
 * Tests mobile-specific navigation components (tab bar, drawer)
 * using a mobile viewport (iPhone-like 375x812).
 */
test.describe("Mobile Navigation", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("should display mobile tab bar on mobile viewport", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-tabbar-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    expect(verificationResult.organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Verify mobile tab bar is visible
    await expect(page.locator('[data-test="mobile-tab-bar"]')).toBeVisible({ timeout: 10000 });
  });

  test("should navigate via tab bar", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-tabnav-${timestamp}@hook0.local`;
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

    // Navigate to org level to get multiple tabs (applications, service-tokens, settings)
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="mobile-tab-bar"]')).toBeVisible({ timeout: 10000 });

    // Capture current URL
    const initialUrl = page.url();

    // Tap the service-tokens tab (second tab at org level)
    const serviceTokensTab = page.locator('[data-test="mobile-tab-service-tokens"]');
    await serviceTokensTab.click();

    // Verify URL changed
    await expect(page).not.toHaveURL(initialUrl, { timeout: 10000 });
  });

  test("should open drawer via More tab", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-drawer-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Drawer Test App ${timestamp}`;

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

    // Create an application to get app-level tabs (>4 tabs triggers the More button)
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

    // Navigate to app-level page (6 tabs: events, subscriptions, event-types, logs, api-keys, settings)
    await page.goto(`/organizations/${organizationId}/applications/${applicationId}/events`);
    await expect(page.locator('[data-test="mobile-tab-bar"]')).toBeVisible({ timeout: 10000 });

    // Tap the More button (last button in the tab bar that is not a router-link)
    const moreButton = page.locator('[data-test="mobile-tab-more"], [data-test="mobile-tab-bar"] button');
    await expect(moreButton.first()).toBeVisible({ timeout: 10000 });
    await moreButton.first().click();

    // Verify drawer is visible
    await expect(page.locator('[data-test="mobile-drawer"]')).toBeVisible({ timeout: 10000 });
  });

  test("should close drawer", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-close-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Close Drawer App ${timestamp}`;

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

    // Create an application for app-level context (>4 tabs for More button)
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

    // Navigate to app-level page
    await page.goto(`/organizations/${organizationId}/applications/${applicationId}/events`);
    await expect(page.locator('[data-test="mobile-tab-bar"]')).toBeVisible({ timeout: 10000 });

    // Open drawer via More button
    const moreButton = page.locator('[data-test="mobile-tab-more"], [data-test="mobile-tab-bar"] button');
    await expect(moreButton.first()).toBeVisible({ timeout: 10000 });
    await moreButton.first().click();
    await expect(page.locator('[data-test="mobile-drawer"]')).toBeVisible({ timeout: 10000 });

    // Close drawer
    await page.locator('[data-test="mobile-drawer-close"]').click();

    // Verify drawer is hidden
    await expect(page.locator('[data-test="mobile-drawer"]')).toBeHidden({ timeout: 10000 });
  });
});
