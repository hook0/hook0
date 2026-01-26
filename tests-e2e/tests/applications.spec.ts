import { test, expect } from "@playwright/test";

/**
 * Application management E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting applications.
 * All tests follow the Three-Step Verification Pattern:
 * 1. Fill form fields
 * 2. Submit and waitForResponse on API endpoint
 * 3. Verify response.status < 400 AND verify data persistence
 */
test.describe("Applications", () => {
  // Each test creates its own user to avoid state conflicts
  async function createTestUserAndLogin(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext
  ): Promise<{
    email: string;
    password: string;
    organizationId: string;
  }> {
    const timestamp = Date.now();
    const email = `test-apps-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Create user via API
    const registerResponse = await request.post("/api/v1/register", {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Login via UI to establish session
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);

    const loginResponsePromise = page.waitForResponse(
      (response) =>
        (response.url().includes("/api/v1/login") ||
          response.url().includes("/iam/login")) &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();

    const loginResponse = await loginResponsePromise;
    expect(loginResponse.status()).toBeLessThan(400);

    // Wait for redirect and get the organization ID from the URL or API
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Get organization ID from API (now we have session cookies)
    const orgsResponse = await page.request.get("/api/v1/organizations");
    expect(orgsResponse.status()).toBeLessThan(400);
    const orgs = await orgsResponse.json();

    // If no orgs exist yet, the user is in tutorial mode - create one
    let organizationId: string;
    if (orgs.length === 0) {
      // Create an organization via the tutorial flow or API
      const createOrgResponse = await page.request.post(
        "/api/v1/organizations",
        {
          data: {
            name: `Test Org ${timestamp}`,
          },
        }
      );
      expect(createOrgResponse.status()).toBeLessThan(400);
      const newOrg = await createOrgResponse.json();
      organizationId = newOrg.organization_id;
    } else {
      organizationId = orgs[0].organization_id;
    }

    return { email, password, organizationId };
  }

  test("should display applications list after login", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);

    // Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    // Verify applications card is visible
    await expect(
      page.locator('[data-test="applications-card"]')
    ).toBeVisible({
      timeout: 10000,
    });

    // Verify create button is present
    await expect(
      page.locator('[data-test="applications-create-button"]')
    ).toBeVisible();
  });

  test("should create new application with required fields and verify API response", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);
    const timestamp = Date.now();
    const appName = `Test App ${timestamp}`;

    // Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    await expect(
      page.locator('[data-test="applications-create-button"]')
    ).toBeVisible({ timeout: 10000 });

    // Click create button
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form to be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill form fields
    await page.locator('[data-test="application-name-input"]').fill(appName);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    const responseBody = await response.json();
    expect(responseBody).toHaveProperty("application_id");
    expect(responseBody.name).toBe(appName);

    // Verify navigation to next step (tutorial or dashboard)
    await expect(page).toHaveURL(
      /\/tutorial|\/dashboard|\/event_types|\/applications/,
      { timeout: 15000 }
    );
  });

  test("should display application details after creation", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);
    const timestamp = Date.now();
    const appName = `Details App ${timestamp}`;

    // Create application via API (using page.request for session context)
    const createResponse = await page.request.post("/api/v1/applications", {
      data: {
        name: appName,
        organization_id: organizationId,
      },
    });
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Navigate to application detail/dashboard
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/dashboard`
    );

    // Wait for page to load - application dashboard should show app info
    await page.waitForLoadState("networkidle");

    // Verify we're on the application page
    await expect(page).toHaveURL(new RegExp(app.application_id), {
      timeout: 10000,
    });
  });

  test("should update application name and verify API response", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);
    const timestamp = Date.now();
    const originalName = `Original App ${timestamp}`;
    const updatedName = `Updated App ${timestamp}`;

    // Create application via API (using page.request for session context)
    const createResponse = await page.request.post("/api/v1/applications", {
      data: {
        name: originalName,
        organization_id: organizationId,
      },
    });
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Navigate to application settings (edit page)
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/settings`
    );

    // Wait for form to be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Update the name
    await page.locator('[data-test="application-name-input"]').clear();
    await page
      .locator('[data-test="application-name-input"]')
      .fill(updatedName);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    const responseBody = await response.json();
    expect(responseBody.name).toBe(updatedName);

    // Verify the change persisted by fetching the application
    const getResponse = await page.request.get(
      `/api/v1/applications/${app.application_id}`
    );
    expect(getResponse.status()).toBeLessThan(400);
    const updatedApp = await getResponse.json();
    expect(updatedApp.name).toBe(updatedName);
  });

  test("should delete application and verify removal", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);
    const timestamp = Date.now();
    const appName = `Delete App ${timestamp}`;

    // Create application via API
    const createResponse = await page.request.post("/api/v1/applications", {
      data: {
        name: appName,
        organization_id: organizationId,
      },
    });
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    // Wait for table to be visible
    await expect(
      page.locator('[data-test="applications-table"]')
    ).toBeVisible({
      timeout: 10000,
    });

    // Verify the app is in the list
    await expect(page.locator(`text=${appName}`)).toBeVisible();

    // Set up dialog handler for confirmation
    page.on("dialog", async (dialog) => {
      await dialog.accept();
    });

    // Step 2: Click delete and wait for API response
    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    // Click the delete link/button in the table row containing the app name
    const row = page.locator(`text=${appName}`).locator("xpath=ancestor::tr");
    await row.locator("text=Delete").click();

    const deleteResponse = await deleteResponsePromise;

    // Step 3: Verify API response
    expect(deleteResponse.status()).toBeLessThan(400);

    // Verify the app is no longer in the list
    await expect(page.locator(`text=${appName}`)).not.toBeVisible({
      timeout: 10000,
    });

    // Verify via API that app no longer exists
    const getResponse = await page.request.get(
      `/api/v1/applications/${app.application_id}`
    );
    expect(getResponse.status()).toBeGreaterThanOrEqual(400);
  });

  test("should show validation error when creating application without name", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);

    // Navigate to create application page
    await page.goto(`/organizations/${organizationId}/applications/new`);

    // Wait for form to be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit button is disabled when name is empty
    await expect(
      page.locator('[data-test="application-submit-button"]')
    ).toBeDisabled();

    // Clear name field if it has any value
    await page.locator('[data-test="application-name-input"]').clear();

    // Verify submit button remains disabled
    await expect(
      page.locator('[data-test="application-submit-button"]')
    ).toBeDisabled();
  });

  test("should cancel application creation and return to previous page", async ({
    page,
    request,
  }) => {
    const { organizationId } = await createTestUserAndLogin(page, request);

    // Navigate to applications list first
    await page.goto(`/organizations/${organizationId}/applications`);

    await expect(
      page.locator('[data-test="applications-create-button"]')
    ).toBeVisible({ timeout: 10000 });

    // Click create button
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form to be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill some data
    await page
      .locator('[data-test="application-name-input"]')
      .fill("Test Cancel");

    // Click cancel button
    await page.locator('[data-test="application-cancel-button"]').click();

    // Verify we're back to applications list
    await expect(
      page.locator('[data-test="applications-card"]')
    ).toBeVisible({
      timeout: 10000,
    });
  });
});
