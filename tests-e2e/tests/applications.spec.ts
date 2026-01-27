import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Application management E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting applications.
 * Uses UI-based login to avoid auth state conflicts.
 * Gets organization ID from database during email verification.
 */
test.describe("Applications", () => {
  test("should display applications list with created application", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-apps-list-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `List Test App ${timestamp}`;

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

    // Verify email and get organization ID from database
    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect to authenticated area (could be tutorial, dashboard, or organizations)
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Step 1: CREATE at least one application via UI before verifying list
    await page.goto(`/organizations/${organizationId}/applications/new`);
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const createdApp = await createResponse.json();

    // Step 2: Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    // Verify applications card is visible
    await expect(page.locator('[data-test="applications-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible();

    // Step 3: Verify list has at least 1 row (AG Grid uses .ag-row class)
    const rows = page.locator('[data-test="applications-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 4: Verify first row contains expected application data
    const firstRow = rows.first();
    await expect(firstRow).toContainText(appName);
  });

  test("should create new application and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-create-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Test App ${timestamp}`;

    // Register and verify email, get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

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

    // Navigate to applications list using the organization ID from database
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
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { application_id?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/applications") && response.request().method() === "POST") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Verify API response
    expect(response.status()).toBeLessThan(400);
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

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
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
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes(`/api/v1/applications/${app.application_id}`) && response.request().method() === "PUT") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Verify
    expect(response.status()).toBeLessThan(400);
    expect(responseBody.name).toBe(updatedName);
  });

  test("should show disabled submit when name is empty", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-validation-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
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

    // Navigate to create page using the organization ID from database
    await page.goto(`/organizations/${organizationId}/applications/new`);

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled when empty
    // Note: The Hook0Button component uses an <a> tag with disabled attribute when not a submit button.
    // Playwright's toBeDisabled() doesn't work with <a> elements, so we check the attribute directly.
    await expect(page.locator('[data-test="application-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Clear if any value
    await page.locator('[data-test="application-name-input"]').clear();

    // Still disabled
    await expect(page.locator('[data-test="application-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );
  });

  test("should delete application and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-delete-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Delete Test App ${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

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

    // Create an application first
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Wait for frontend's auto-refresh to complete after app creation
    // The frontend automatically refreshes the token when an application is created
    await page.waitForTimeout(2000);

    // Navigate to application settings page (where delete option is)
    // The navigation itself waits for the page to load
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify delete card is visible
    await expect(page.locator('[data-test="application-delete-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="application-delete-button"]')).toBeVisible();

    // Setup dialog handler for confirmation
    page.on("dialog", (dialog) => {
      dialog.accept();
    });

    // Step 2: Click delete and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-delete-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify redirect to organization dashboard
    await expect(page).toHaveURL(/\/organizations\/[^/]+\/dashboard/, {
      timeout: 15000,
    });
  });

  test("should cancel delete when dialog is dismissed", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-delete-cancel-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Cancel Delete App ${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

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

    // Create an application first
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Wait for frontend's auto-refresh to complete after app creation
    await page.waitForTimeout(2000);

    // Navigate to application settings page
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    await expect(page.locator('[data-test="application-delete-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click delete button
    await page.locator('[data-test="application-delete-button"]').click();

    // Should still be on the settings page (not redirected)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });

    // Application form should still be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible();
  });
});
