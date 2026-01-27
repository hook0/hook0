import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Service Tokens E2E tests for Hook0.
 *
 * Tests for creating, viewing, and managing service tokens.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Service Tokens", () => {
  test("should display service tokens list with created token", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-list-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const tokenName = `Test Token ${timestamp}`;

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

    // Navigate to service tokens page
    await page.goto(`/organizations/${organizationId}/services_tokens`);

    // Verify service tokens card is visible
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Create a service token via UI (uses prompt dialog)
    page.on("dialog", (dialog) => {
      dialog.accept(tokenName);
    });

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="service-tokens-create-button"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Step 2: Verify list has at least 1 row
    await page.waitForTimeout(1000); // Wait for UI to refresh
    const rows = page.locator('[data-test="service-tokens-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 3: Verify first row contains expected token name
    const firstRow = rows.first();
    await expect(firstRow).toContainText(tokenName);
  });

  test("should display create button and service tokens card", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-display-${timestamp}@hook0.local`;
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

    // Navigate to service tokens page
    await page.goto(`/organizations/${organizationId}/services_tokens`);

    // Verify page elements
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="service-tokens-create-button"]')).toBeVisible();
  });

  test("should create new service token and verify API response", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-create-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const tokenName = `API Token ${timestamp}`;

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

    // Navigate to service tokens page
    await page.goto(`/organizations/${organizationId}/services_tokens`);
    await expect(page.locator('[data-test="service-tokens-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to enter token name
    page.on("dialog", (dialog) => {
      dialog.accept(tokenName);
    });

    // Step 2: Click create and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { token_id?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/service_token") && response.request().method() === "POST") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="service-tokens-create-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    expect(responseBody).toHaveProperty("token_id");
    expect(responseBody.name).toBe(tokenName);
  });

  test("should cancel create when dialog is dismissed", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-cancel-${timestamp}@hook0.local`;
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

    // Navigate to service tokens page
    await page.goto(`/organizations/${organizationId}/services_tokens`);
    await expect(page.locator('[data-test="service-tokens-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to dismiss (cancel)
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click create button
    await page.locator('[data-test="service-tokens-create-button"]').click();

    // Should still be on the same page
    await expect(page).toHaveURL(/\/services_tokens/, {
      timeout: 5000,
    });

    // Card should still be visible
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible();
  });
});
