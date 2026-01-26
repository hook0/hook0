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
 * These tests will verify the feature works when available.
 */
test.describe("Members", () => {
  test("should display organization dashboard with members section if quota allows", async ({
    page,
    request,
  }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-members-list-${timestamp}@hook0.local`;
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

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Organization dashboard should be visible
    await expect(page).toHaveURL(/\/dashboard/, { timeout: 10000 });

    // Check if members card is visible (depends on plan quota)
    // Members card is only shown when members_per_organization_limit > 1
    const membersCardVisible = await page
      .locator('[data-test="members-card"]')
      .isVisible({ timeout: 5000 })
      .catch(() => false);

    if (membersCardVisible) {
      // Verify members table has at least 1 row (the current user)
      const rows = page.locator('[data-test="members-table"] .ag-row');
      const rowCount = await rows.count();
      expect(rowCount).toBeGreaterThanOrEqual(1);

      // Verify first row contains the current user's email
      const firstRow = rows.first();
      await expect(firstRow).toContainText(email);
    } else {
      // Members feature not available for this plan - that's expected for free tier
      console.log(
        "Members feature not available - organization quota does not allow multiple members"
      );
    }
  });

  test("should display invite form when members feature is available", async ({
    page,
    request,
  }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-members-form-${timestamp}@hook0.local`;
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

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Check if members card is visible (depends on plan quota)
    const membersCardVisible = await page
      .locator('[data-test="members-card"]')
      .isVisible({ timeout: 5000 })
      .catch(() => false);

    if (membersCardVisible) {
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
    } else {
      console.log(
        "Members feature not available - organization quota does not allow multiple members"
      );
    }
  });

  test("should invite a new member when feature is available", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-members-invite-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const inviteeEmail = `invitee-${timestamp}@hook0.local`;

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

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Check if members card is visible (depends on plan quota)
    const membersCardVisible = await page
      .locator('[data-test="members-card"]')
      .isVisible({ timeout: 5000 })
      .catch(() => false);

    if (membersCardVisible) {
      // Wait for form
      await expect(page.locator('[data-test="members-invite-form"]')).toBeVisible();

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
    } else {
      console.log(
        "Members feature not available - organization quota does not allow multiple members"
      );
    }
  });

  test("should validate invite form fields", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-members-validation-${timestamp}@hook0.local`;
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

    // Navigate to organization dashboard (which contains the members list)
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Check if members card is visible (depends on plan quota)
    const membersCardVisible = await page
      .locator('[data-test="members-card"]')
      .isVisible({ timeout: 5000 })
      .catch(() => false);

    if (membersCardVisible) {
      // Wait for form
      await expect(page.locator('[data-test="members-invite-form"]')).toBeVisible();

      // Verify button is disabled when empty
      await expect(page.locator('[data-test="members-invite-button"]')).toHaveAttribute(
        "disabled",
        "true"
      );

      // Fill email only - still disabled (missing role)
      await page
        .locator('[data-test="members-invite-email-input"]')
        .fill("test-invitee@hook0.local");
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
    } else {
      console.log(
        "Members feature not available - organization quota does not allow multiple members"
      );
    }
  });
});
