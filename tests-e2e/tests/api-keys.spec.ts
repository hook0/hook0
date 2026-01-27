import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * API Keys (Application Secrets) E2E tests for Hook0.
 *
 * Tests for creating, viewing, and deleting API keys.
 * Following the Three-Step Verification Pattern.
 */
test.describe("API Keys", () => {
  /**
   * Helper to setup test environment: user, organization, and application
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-apikeys-${testId}-${timestamp}@hook0.local`;
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
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Create an application
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(`Test App ${timestamp}`);

    // Capture response body inside the predicate to avoid race condition with navigation
    let applicationId: string = "";
    const createAppResponse = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/applications") && response.request().method() === "POST") {
          if (response.status() < 400) {
            const app = await response.json();
            applicationId = app.application_id;
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);

    return {
      email,
      password,
      organizationId,
      applicationId,
      timestamp,
    };
  }

  test("should display API keys list with created key", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list");
    const keyName = `Test API Key ${env.timestamp}`;

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/secrets`
    );

    // Verify API keys card is visible
    await expect(page.locator('[data-test="api-keys-card"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Create an API key via UI (uses prompt dialog)
    page.on("dialog", (dialog) => {
      dialog.accept(keyName);
    });

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="api-keys-create-button"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Step 2: Verify list has at least 1 row
    await page.waitForTimeout(1000); // Wait for UI to refresh
    const rows = page.locator('[data-test="api-keys-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 3: Verify first row contains expected key name
    const firstRow = rows.first();
    await expect(firstRow).toContainText(keyName);
  });

  test("should display create button and API keys card", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "display");

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/secrets`
    );

    // Verify page elements
    await expect(page.locator('[data-test="api-keys-card"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="api-keys-create-button"]')).toBeVisible();
  });

  test("should create new API key and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "create");
    const keyName = `New API Key ${env.timestamp}`;

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/secrets`
    );
    await expect(page.locator('[data-test="api-keys-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to enter key name
    page.on("dialog", (dialog) => {
      dialog.accept(keyName);
    });

    // Step 2: Click create and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { token?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/application_secrets") && response.request().method() === "POST") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="api-keys-create-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    expect(responseBody).toHaveProperty("token");
    expect(responseBody.name).toBe(keyName);
  });

  test("should cancel create when dialog is dismissed", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "cancel");

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/secrets`
    );
    await expect(page.locator('[data-test="api-keys-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to dismiss (cancel)
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click create button
    await page.locator('[data-test="api-keys-create-button"]').click();

    // Should still be on the same page
    await expect(page).toHaveURL(/\/secrets/, {
      timeout: 5000,
    });

    // Card should still be visible
    await expect(page.locator('[data-test="api-keys-card"]')).toBeVisible();
  });

  test("should delete API key and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "delete");
    const keyName = `Delete Test Key ${env.timestamp}`;

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/secrets`
    );

    // Create an API key first
    let dialogCount = 0;
    page.on("dialog", (dialog) => {
      dialogCount++;
      if (dialogCount === 1) {
        // First dialog is the create prompt
        dialog.accept(keyName);
      } else {
        // Second dialog is the delete confirmation
        dialog.accept();
      }
    });

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="api-keys-create-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for table to show the key
    await page.waitForTimeout(1000);
    const rows = page.locator('[data-test="api-keys-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Click delete on the first row
    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    // Find and click the delete link in the first row
    const deleteLink = rows.first().locator('text="Delete"');
    await deleteLink.click();

    const deleteResponse = await deleteResponsePromise;
    expect(deleteResponse.status()).toBeLessThan(400);
  });
});
