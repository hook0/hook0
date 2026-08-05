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
        if (
          response.url().includes("/api/v1/applications") &&
          response.request().method() === "POST"
        ) {
          if (response.status() < 400) {
            try {
              const app = await response.json();
              applicationId = app.application_id;
            } catch {
              // Response body may be unavailable due to navigation
            }
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

    // Wait for redirect to complete - URL should contain a UUID application ID, not "new"
    // UUID pattern: 8-4-4-4-12 hex characters
    const uuidPattern =
      /\/applications\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;
    await expect(page).toHaveURL(uuidPattern, { timeout: 15000 });
    const url = page.url();
    const match = url.match(uuidPattern);
    expect(match, "Failed to extract application ID (UUID) from URL").toBeTruthy();
    applicationId = match![1];

    return {
      email,
      password,
      organizationId,
      applicationId,
      timestamp,
    };
  }

  test("provisions a usable Default key as soon as an application is created", async ({
    page,
    request,
  }) => {
    // The whole point of auto-provisioning: a brand-new application is ready to
    // receive its first event with no "now create a key" detour. Nothing is
    // created by this test beyond the application itself.
    const env = await setupTestEnvironment(page, request, "default");

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
    );
    await expect(page.locator('[data-test="api-keys-card"]')).toBeVisible({ timeout: 10000 });

    const defaultRow = page.locator('[data-test="api-keys-table"] [row-id]', {
      hasText: "Default",
    });
    await expect(defaultRow, "a Default key exists without any manual step").toBeVisible({
      timeout: 15000,
    });

    // Read the provisioned token from the API and prove it actually
    // authenticates — a key that exists but cannot send an event would defeat
    // the purpose.
    const loginResponse = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email: env.email, password: env.password },
    });
    expect(loginResponse.status()).toBeLessThan(400);
    const accessToken = (await loginResponse.json()).access_token;

    const secretsResponse = await request.get(
      `${API_BASE_URL}/application_secrets?application_id=${env.applicationId}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    expect(secretsResponse.status()).toBeLessThan(400);
    const secrets = (await secretsResponse.json()) as Array<{ name?: string; token: string }>;
    expect(secrets.length, "exactly one key is provisioned at creation").toBe(1);
    expect(secrets[0].name).toBe("Default");

    // An event references a registered event type (foreign key), so declare one
    // first — still using only the provisioned key for the ingestion itself.
    const eventTypeResponse = await request.post(`${API_BASE_URL}/event_types`, {
      headers: { Authorization: `Bearer ${accessToken}` },
      data: {
        application_id: env.applicationId,
        service: "test",
        resource_type: "entity",
        verb: "created",
      },
    });
    expect(eventTypeResponse.status()).toBeLessThan(400);

    const eventResponse = await request.post(`${API_BASE_URL}/event/`, {
      headers: { Authorization: `Bearer ${secrets[0].token}` },
      data: {
        application_id: env.applicationId,
        event_type: "test.entity.created",
        labels: { all: "yes" },
        occurred_at: new Date().toISOString(),
        payload_content_type: "application/json",
        payload: '{"provisioned": true}',
      },
    });
    expect(eventResponse.status(), "the provisioned key must be able to send the first event").toBe(
      201
    );
  });

  test("pre-fills the copyable curl snippet with the provisioned key", async ({
    page,
    request,
  }) => {
    // The point of provisioning a key is that the copy-paste path to the first
    // API call works with no detour. Reading the code shows the snippet is built
    // from the application's first secret; this pins that behaviour so a change
    // to the secrets query, its ordering, or the token lookup cannot silently
    // turn the snippet back into a placeholder.
    const env = await setupTestEnvironment(page, request, "curl");

    const loginResponse = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email: env.email, password: env.password },
    });
    expect(loginResponse.status()).toBeLessThan(400);
    const accessToken = (await loginResponse.json()).access_token;

    const secretsResponse = await request.get(
      `${API_BASE_URL}/application_secrets?application_id=${env.applicationId}`,
      { headers: { Authorization: `Bearer ${accessToken}` } }
    );
    expect(secretsResponse.status()).toBeLessThan(400);
    const provisionedToken = ((await secretsResponse.json()) as Array<{ token: string }>)[0].token;
    expect(provisionedToken).toBeTruthy();

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({ timeout: 15000 });

    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curlPanel = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curlPanel).toBeVisible({ timeout: 10000 });

    // The real token, not a placeholder the user still has to replace.
    await expect(curlPanel).toContainText(provisionedToken, { timeout: 10000 });
    await expect(curlPanel).toContainText("curl");
    await expect(curlPanel).not.toContainText("YOUR_APPLICATION_SECRET");
  });

  test("should display API keys list with created key", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list");
    const keyName = `Test API Key ${env.timestamp}`;

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
    );

    // Verify API keys card is visible
    await expect(page.locator('[data-test="api-keys-card"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Create an API key via UI (uses Hook0Dialog modal)
    await page.locator('[data-test="api-keys-create-button"]').click();

    // Fill the name in the dialog modal
    const dialogInput = page.locator('[data-test="api-key-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(keyName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    // Click confirm button in the dialog
    await page.locator(".hook0-dialog .hook0-dialog__actions button:last-child").click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Step 2: Verify list has at least 1 row (wait for UI to refresh using expect.toPass)
    const rows = page.locator('[data-test="api-keys-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

    // Step 3: Verify the created key appears in the list.
    // An application also has an auto-provisioned "Default" key, so don't assume position.
    await expect(
      page.locator('[data-test="api-keys-table"] [row-id]', { hasText: keyName })
    ).toBeVisible();
  });

  test("should display create button and API keys card", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "display");

    // Navigate to API keys page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
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
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
    );
    await expect(page.locator('[data-test="api-keys-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 2: Click create to open Hook0Dialog modal
    await page.locator('[data-test="api-keys-create-button"]').click();

    // Fill the name in the dialog modal
    const dialogInput = page.locator('[data-test="api-key-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(keyName);

    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { token?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (
          response.url().includes("/api/v1/application_secrets") &&
          response.request().method() === "POST"
        ) {
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
    await page.locator(".hook0-dialog .hook0-dialog__actions button:last-child").click();

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
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
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
    await expect(page).toHaveURL(/\/application_secrets/, {
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
      `/organizations/${env.organizationId}/applications/${env.applicationId}/application_secrets`
    );

    // Create an API key first via Hook0Dialog modal
    await page.locator('[data-test="api-keys-create-button"]').click();

    const createDialogInput = page.locator('[data-test="api-key-name-input"]');
    await expect(createDialogInput).toBeVisible({ timeout: 5000 });
    await createDialogInput.fill(keyName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator(".hook0-dialog .hook0-dialog__actions button:last-child").click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for table to show the key using expect.toPass
    const rows = page.locator('[data-test="api-keys-table"] [row-id]');
    await expect(async () => {
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);
    }).toPass({ timeout: 10000 });

    // Click delete on the key we just created. An application also has a "Default" key,
    // so target our row by name rather than the first row.
    const deleteLink = page
      .locator('[data-test="api-keys-table"] [row-id]', { hasText: keyName })
      .locator('[data-test="api-key-delete-button"]');
    await deleteLink.click();

    // Wait for delete confirmation dialog and click confirm
    const deleteConfirmButton = page.locator(
      ".hook0-dialog--danger .hook0-dialog__actions button:last-child"
    );
    await expect(deleteConfirmButton).toBeVisible({ timeout: 5000 });

    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/application_secrets") &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await deleteConfirmButton.click();

    const deleteResponse = await deleteResponsePromise;
    expect(deleteResponse.status()).toBeLessThan(400);
  });
});
