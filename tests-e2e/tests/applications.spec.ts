import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Application management E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting applications.
 * Uses UI-based login to avoid auth state conflicts.
 */
test.describe("Applications", () => {
  test("should display applications list after navigating", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-apps-list-${timestamp}@hook0.local`;
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

    // Verify email before login (required by API)
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect to organizations page (which contains the org ID in URL)
    await expect(page).toHaveURL(/\/organizations\/[a-f0-9-]+/, {
      timeout: 15000,
    });

    // Extract organization ID from URL
    const url = page.url();
    const match = url.match(/\/organizations\/([a-f0-9-]+)/);
    const organizationId = match ? match[1] : "";
    expect(organizationId).toBeTruthy();

    // Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    // Verify applications card is visible
    await expect(page.locator('[data-test="applications-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible();
  });

  test("should create new application and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-create-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Test App ${timestamp}`;

    // Register and verify email
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email before login (required by API)
    await verifyEmailViaMailpit(request, email);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect to organizations page (which contains the org ID in URL)
    await expect(page).toHaveURL(/\/organizations\/[a-f0-9-]+/, {
      timeout: 15000,
    });

    // Extract organization ID from URL
    const url = page.url();
    const match = url.match(/\/organizations\/([a-f0-9-]+)/);
    const organizationId = match ? match[1] : "";
    expect(organizationId).toBeTruthy();

    // Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Click create button
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill form
    await page.locator('[data-test="application-name-input"]').fill(appName);

    // Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Verify API response
    expect(response.status()).toBeLessThan(400);
    const responseBody = await response.json();
    expect(responseBody).toHaveProperty("application_id");
    expect(responseBody.name).toBe(appName);
  });

  test("should update application name and verify persistence", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-update-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const originalName = `Original App ${timestamp}`;
    const updatedName = `Updated App ${timestamp}`;

    // Register
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email before login (required by API)
    await verifyEmailViaMailpit(request, email);

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect to organizations page (which contains the org ID in URL)
    await expect(page).toHaveURL(/\/organizations\/[a-f0-9-]+/, {
      timeout: 15000,
    });

    // Extract organization ID from URL
    const url = page.url();
    const match = url.match(/\/organizations\/([a-f0-9-]+)/);
    const organizationId = match ? match[1] : "";
    expect(organizationId).toBeTruthy();

    // Navigate to applications list and create application via UI
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form and create application
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(originalName);

    // Submit and wait for API response to get the application ID
    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Navigate to settings
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Update name
    await page.locator('[data-test="application-name-input"]').clear();
    await page.locator('[data-test="application-name-input"]').fill(updatedName);

    // Submit and wait for response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Verify
    expect(response.status()).toBeLessThan(400);
    const responseBody = await response.json();
    expect(responseBody.name).toBe(updatedName);
  });

  test("should show disabled submit when name is empty", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-validation-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email before login (required by API)
    await verifyEmailViaMailpit(request, email);

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    // Wait for redirect to organizations page (which contains the org ID in URL)
    await expect(page).toHaveURL(/\/organizations\/[a-f0-9-]+/, {
      timeout: 15000,
    });

    // Extract organization ID from URL
    const url = page.url();
    const match = url.match(/\/organizations\/([a-f0-9-]+)/);
    const organizationId = match ? match[1] : "";
    expect(organizationId).toBeTruthy();

    // Navigate to create page
    await page.goto(`/organizations/${organizationId}/applications/new`);

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled when empty
    await expect(page.locator('[data-test="application-submit-button"]')).toBeDisabled();

    // Clear if any value
    await page.locator('[data-test="application-name-input"]').clear();

    // Still disabled
    await expect(page.locator('[data-test="application-submit-button"]')).toBeDisabled();
  });
});
