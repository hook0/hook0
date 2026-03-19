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

    // Step 1: Create a service token via UI (uses Hook0Dialog modal)
    await page.locator('[data-test="service-tokens-create-button"]').click();

    // Fill the name in the dialog modal
    const dialogInput = page.locator('[data-test="service-token-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(tokenName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    // Click confirm button in the dialog
    await page.locator('.hook0-dialog .hook0-dialog__actions button:last-child').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Step 2: Verify list has at least 1 row (wait for UI to refresh using expect.toPass)
    const rows = page.locator('[data-test="service-tokens-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

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

    // Step 2: Click create to open Hook0Dialog modal
    await page.locator('[data-test="service-tokens-create-button"]').click();

    // Fill the name in the dialog modal
    const dialogInput = page.locator('[data-test="service-token-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(tokenName);

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

    // Click confirm button in the dialog
    await page.locator('.hook0-dialog .hook0-dialog__actions button:last-child').click();

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

  test("should delete a service token and verify it is removed from the list", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-delete-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const tokenName = `Token To Remove ${timestamp}`;

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
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Create a service token via UI
    await page.locator('[data-test="service-tokens-create-button"]').click();

    const dialogInput = page.locator('[data-test="service-token-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(tokenName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('.hook0-dialog .hook0-dialog__actions button:last-child').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for token to appear in the table
    const rows = page.locator('[data-test="service-tokens-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

    await expect(rows.first()).toContainText(tokenName);

    // Step 2: Click "Delete" on the first row to open danger confirmation dialog
    const deleteLink = rows.first().getByText("Delete");
    await deleteLink.click();

    // Wait for the danger confirmation dialog and click confirm
    const deleteConfirmButton = page.locator('.hook0-dialog--danger .hook0-dialog__actions button:last-child');
    await expect(deleteConfirmButton).toBeVisible({ timeout: 5000 });

    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await deleteConfirmButton.click();

    const deleteResponse = await deleteResponsePromise;
    expect(deleteResponse.status()).toBeLessThan(400);

    // Step 3: Verify the token is no longer in the list
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBe(0);
    }).toPass({ timeout: 10000 });
  });

  test("should rename a service token", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tokens-rename-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const tokenName = `Rename Token ${timestamp}`;
    const newTokenName = `Renamed Token ${timestamp}`;

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
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Create a service token via UI
    await page.locator('[data-test="service-tokens-create-button"]').click();

    const createDialogInput = page.locator('[data-test="service-token-name-input"]');
    await expect(createDialogInput).toBeVisible({ timeout: 5000 });
    await createDialogInput.fill(tokenName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('.hook0-dialog .hook0-dialog__actions button:last-child').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for token to appear in the table
    const rows = page.locator('[data-test="service-tokens-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

    await expect(rows.first()).toContainText(tokenName);

    // Step 2: Click "Edit" on the first row to open edit dialog
    const editLink = rows.first().getByText("Edit");
    await editLink.click();

    // Fill the new name in the edit dialog
    const editDialogInput = page.locator('[data-test="service-token-edit-name-input"]');
    await expect(editDialogInput).toBeVisible({ timeout: 5000 });
    await editDialogInput.clear();
    await editDialogInput.fill(newTokenName);

    const updateResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('.hook0-dialog .hook0-dialog__actions button:last-child').click();

    const updateResponse = await updateResponsePromise;
    expect(updateResponse.status()).toBeLessThan(400);

    // Step 3: Verify the table now shows the new name
    await expect(async () => {
      await expect(rows.first()).toContainText(newTokenName);
    }).toPass({ timeout: 10000 });
  });
});
