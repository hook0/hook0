import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Members management E2E tests for Hook0.
 *
 * Tests for viewing organization members and inviting new users.
 * Following the Three-Step Verification Pattern.
 *
 * NOTE: Members list is only shown on the organization dashboard when
 * members_per_organization_limit > 1 (based on the organization's plan).
 * These tests require the MEMBERS_FEATURE_ENABLED environment variable to be set to "true",
 * otherwise they will be skipped.
 *
 * To run these tests:
 * 1. Ensure your test database has organizations with members_per_organization_limit > 1
 * 2. Set MEMBERS_FEATURE_ENABLED=true environment variable
 */

// Members feature requires environment variable to be set
const membersFeatureEnabled = process.env.MEMBERS_FEATURE_ENABLED === "true";

// Skip entire test suite if members feature is not enabled
// This is a feature-flag controlled skip, not a silent skip
test.describe("Members", () => {
  test.skip(
    !membersFeatureEnabled,
    "Members feature tests require MEMBERS_FEATURE_ENABLED=true environment variable. " +
    "Set this variable in .envrc or your CI configuration to enable these tests."
  );

  /**
   * Helper to setup test environment with a user who has members quota > 1
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-members-${testId}-${timestamp}@hook0.local`;
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

    return { email, password, organizationId, timestamp };
  }

  test("should display members list with current user on organization dashboard", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "list");

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${env.organizationId}/dashboard`);

    // Organization dashboard should be visible
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 10000 });

    // Verify members card is visible
    await expect(page.locator('[data-test="members-card"]')).toBeVisible({ timeout: 10000 });

    // Verify members table has at least 1 row (the current user)
    const rows = page.locator('[data-test="members-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Verify first row contains the current user's email
    const firstRow = rows.first();
    await expect(firstRow).toContainText(env.email);
  });

  test("should display invite form with all required elements", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "form");

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${env.organizationId}/dashboard`);

    // Verify members card is visible
    await expect(page.locator('[data-test="members-card"]')).toBeVisible({ timeout: 10000 });

    // Verify invite form elements are present
    await expect(page.locator('[data-test="members-invite-form"]')).toBeVisible();
    await expect(page.locator('[data-test="members-invite-email-input"]')).toBeVisible();
    await expect(page.locator('[data-test="members-invite-role-select"]')).toBeVisible();
    await expect(page.locator('[data-test="members-invite-button"]')).toBeVisible();

    // Verify invite button is disabled when fields are empty
    await expect(page.locator('[data-test="members-invite-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );
  });

  test("should invite a new member and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "invite");
    const inviteeEmail = `invitee-${env.timestamp}@hook0.local`;

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${env.organizationId}/dashboard`);

    // Wait for form
    await expect(page.locator('[data-test="members-invite-form"]')).toBeVisible({ timeout: 10000 });

    // Step 1: Fill invite form
    await page.locator('[data-test="members-invite-email-input"]').fill(inviteeEmail);
    await page.locator('[data-test="members-invite-role-select"]').selectOption("editor");

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/organizations") &&
        response.url().includes("/invite") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="members-invite-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify form is cleared after successful invite
    await expect(page.locator('[data-test="members-invite-email-input"]')).toHaveValue("");
  });

  test("should validate invite form fields - button disabled without all required fields", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "validation");

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${env.organizationId}/dashboard`);

    // Wait for form
    await expect(page.locator('[data-test="members-invite-form"]')).toBeVisible({ timeout: 10000 });

    // Verify button is disabled when empty
    await expect(page.locator('[data-test="members-invite-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill email only - still disabled (missing role)
    await page.locator('[data-test="members-invite-email-input"]').fill("test-invitee@hook0.local");
    await expect(page.locator('[data-test="members-invite-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Select role - now enabled
    await page.locator('[data-test="members-invite-role-select"]').selectOption("viewer");
    await expect(page.locator('[data-test="members-invite-button"]')).not.toHaveAttribute(
      "disabled",
      "true"
    );
  });
});
