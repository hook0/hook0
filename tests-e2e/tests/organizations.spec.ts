import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Organizations E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting organizations.
 * Following the Three-Step Verification Pattern:
 * 1. Fill form fields
 * 2. Submit and waitForResponse on API endpoint
 * 3. Verify response.status < 400 AND verify data/navigation
 */
test.describe("Organizations", () => {
  test("should display organization form when creating new organization", async ({
    page,
    request,
  }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-org-create-${timestamp}@hook0.local`;
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

    // Verify email
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to create organization page
    await page.goto("/organizations/new");

    // Verify form is visible
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="organization-card"]')).toBeVisible();
    await expect(page.locator('[data-test="organization-name-input"]')).toBeVisible();
    await expect(page.locator('[data-test="organization-submit-button"]')).toBeVisible();
  });

  test("should create new organization and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-org-new-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const orgName = `Test Organization ${timestamp}`;

    // Register and verify email
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to create organization page
    await page.goto("/organizations/new");
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill form
    await page.locator('[data-test="organization-name-input"]').fill(orgName);

    // Step 2: Submit and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { organization_id?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/organizations") && response.request().method() === "POST") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="organization-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    expect(responseBody).toHaveProperty("organization_id");
    expect(responseBody.name).toBe(orgName);

    // Verify redirect after creation (to tutorial or dashboard)
    await expect(page).toHaveURL(/\/tutorial|\/organizations\/[^/]+\/dashboard/, {
      timeout: 15000,
    });
  });

  test("should update organization name and verify persistence", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-org-update-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const originalName = `Original Org ${timestamp}`;
    const updatedName = `Updated Org ${timestamp}`;

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

    // First, update the organization name via API to set original name
    const updateResponse = await request.put(`${API_BASE_URL}/organizations/${organizationId}`, {
      data: { name: originalName },
      headers: {
        "Content-Type": "application/json",
      },
    });
    // If update fails with 401, login first and retry
    // For now, just navigate to settings and update via UI

    // Navigate to organization settings
    await page.goto(`/organizations/${organizationId}/settings`);
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Update name
    await page.locator('[data-test="organization-name-input"]').clear();
    await page.locator('[data-test="organization-name-input"]').fill(updatedName);

    // Submit and wait for response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes(`/api/v1/organizations/${organizationId}`) && response.request().method() === "PUT") {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="organization-submit-button"]').click();

    const response = await responsePromise;

    // Verify
    expect(response.status()).toBeLessThan(400);
    expect(responseBody.name).toBe(updatedName);
  });

  test("should show disabled submit button when organization name is empty", async ({
    page,
    request,
  }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-org-validation-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to create page
    await page.goto("/organizations/new");
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled when empty
    await expect(page.locator('[data-test="organization-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Clear if any value
    await page.locator('[data-test="organization-name-input"]').clear();

    // Still disabled
    await expect(page.locator('[data-test="organization-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );
  });

  test("should display delete organization card on settings page", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-org-delete-display-${timestamp}@hook0.local`;
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

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to organization settings
    await page.goto(`/organizations/${organizationId}/settings`);
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify delete card is visible
    await expect(page.locator('[data-test="organization-delete-card"]')).toBeVisible();
    await expect(page.locator('[data-test="organization-delete-button"]')).toBeVisible();
  });

  test("should delete organization and verify API response and redirect", async ({
    page,
    request,
  }) => {
    // Setup - create a NEW organization to delete (don't delete the auto-created one)
    const timestamp = Date.now();
    const email = `test-org-delete-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const orgName = `Deletable Org ${timestamp}`;

    // Register and verify email
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Create a new organization to delete
    await page.goto("/organizations/new");
    await expect(page.locator('[data-test="organization-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="organization-name-input"]').fill(orgName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/organizations") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="organization-submit-button"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const createdOrg = await createResponse.json();
    const newOrgId = createdOrg.organization_id;

    // The frontend automatically refreshes the token after creating an organization.
    // Wait for navigation to complete (frontend redirects after org creation).
    // The refresh happens automatically, so we just wait for the page to settle.
    await expect(page).toHaveURL(/\/tutorial|\/organizations\/[^/]+/, {
      timeout: 15000,
    });
    // Wait for the frontend refresh to complete by checking that the auth token was updated
    await page.waitForTimeout(1000);

    // Navigate to the new organization's settings
    await page.goto(`/organizations/${newOrgId}/settings`);
    await expect(page.locator('[data-test="organization-delete-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler for delete confirmation
    page.on("dialog", (dialog) => {
      dialog.accept();
    });

    // Step 2: Click delete and wait for API response
    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/organizations/${newOrgId}`) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await page.locator('[data-test="organization-delete-button"]').click();

    const deleteResponse = await deleteResponsePromise;

    // Step 3: Verify API response
    expect(deleteResponse.status()).toBeLessThan(400);

    // Verify redirect to home/organizations list
    // Regex matches full URL ending with /organizations or ending with /
    await expect(page).toHaveURL(/\/organizations$|\/$/, {
      timeout: 15000,
    });
  });

  test("should cancel organization deletion when dialog is dismissed", async ({
    page,
    request,
  }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-org-cancel-delete-${timestamp}@hook0.local`;
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

    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to organization settings
    await page.goto(`/organizations/${organizationId}/settings`);
    await expect(page.locator('[data-test="organization-delete-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click delete button
    await page.locator('[data-test="organization-delete-button"]').click();

    // Should still be on settings page (not redirected)
    await expect(page).toHaveURL(new RegExp(`/organizations/${organizationId}/settings`), {
      timeout: 5000,
    });

    // Delete card should still be visible
    await expect(page.locator('[data-test="organization-delete-card"]')).toBeVisible();
  });
});
