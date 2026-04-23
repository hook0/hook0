import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Service Token Detail E2E tests for Hook0.
 *
 * Tests navigating to and viewing service token detail pages.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Service Token Detail", () => {
  /**
   * Helper to register, verify, login, and create a service token.
   * Returns the token name, organization ID, and navigates to the token list.
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ): Promise<{
    email: string;
    password: string;
    organizationId: string;
    tokenName: string;
    timestamp: number;
  }> {
    const timestamp = Date.now();
    const email = `test-token-detail-${testId}-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const tokenName = `Detail Token ${timestamp}`;

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
    if (!organizationId) {
      throw new Error("Organization ID is required");
    }

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
    await page.goto(`/organizations/${organizationId}/service_tokens`);
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Create a service token via UI (uses Hook0Dialog modal)
    await page.locator('[data-test="service-tokens-create-button"]').click();
    await page.waitForURL("**/service_tokens/new");

    const dialogInput = page.locator('[data-test="service-token-name-input"]');
    await expect(dialogInput).toBeVisible({ timeout: 5000 });
    await dialogInput.fill(tokenName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/service_token") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    // Click confirm button in the dialog
    await page.locator('[data-test="dialog-confirm-button"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for the token to appear in the table (name column is a RouterLink)
    const tokenLink = page
      .locator('[data-test="service-tokens-table"] [data-test="token-name-link"]')
      .first();
    await expect(tokenLink).toBeVisible({ timeout: 10000 });

    return {
      email,
      password,
      organizationId,
      tokenName,
      timestamp,
    };
  }

  test("should navigate to service token detail page", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "nav");

    // Step 1: Click "Show" button to navigate to detail page
    const tokenLink = page
      .locator('[data-test="service-tokens-table"] [data-test="token-name-link"]')
      .first();
    await tokenLink.click();

    // Step 2: Verify URL changed to the service token detail page
    await expect(page).toHaveURL(/\/organizations\/[^/]+\/service_tokens\/[0-9a-f-]+/, {
      timeout: 10000,
    });

    // Step 3: Verify service token detail content is visible
    // The detail page renders Hook0Card with the token name inside
    await expect(page.locator('[data-test="service-token-detail-card"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display token name on detail page", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "name");

    // Step 1: Navigate to token detail by clicking the Show button
    const tokenLink = page
      .locator('[data-test="service-tokens-table"] [data-test="token-name-link"]')
      .first();
    await tokenLink.click();

    // Step 2: Wait for detail page to load
    await expect(page).toHaveURL(/\/organizations\/[^/]+\/service_tokens\/[0-9a-f-]+/, {
      timeout: 10000,
    });

    // Step 3: Verify token name is displayed on the page
    await expect(page.getByText(env.tokenName)).toBeVisible({ timeout: 10000 });
  });

  test("should display token details including biscuit and detail card", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "details");

    // Step 1: Navigate to token detail page
    const tokenLink = page
      .locator('[data-test="service-tokens-table"] [data-test="token-name-link"]')
      .first();
    await tokenLink.click();

    await expect(page).toHaveURL(/\/organizations\/[^/]+\/service_tokens\/[0-9a-f-]+/, {
      timeout: 10000,
    });

    // Step 2: Verify the detail card is visible with correct data-test attribute
    const detailCard = page.locator('[data-test="service-token-detail-card"]');
    await expect(detailCard).toBeVisible({ timeout: 10000 });

    // Step 3: Verify the token name is displayed on the page (in page title)
    await expect(page.getByText(env.tokenName)).toBeVisible({ timeout: 10000 });

    // Verify the biscuit token value is displayed
    await expect(detailCard.locator('[data-test="service-token-value"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should navigate back to token list", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "back");

    // Step 1: Navigate to token detail page
    const tokenLink = page
      .locator('[data-test="service-tokens-table"] [data-test="token-name-link"]')
      .first();
    await tokenLink.click();

    await expect(page).toHaveURL(/\/organizations\/[^/]+\/service_tokens\/[0-9a-f-]+/, {
      timeout: 10000,
    });

    // Verify detail card loaded
    await expect(page.locator('[data-test="service-token-detail-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 2: Click the Cancel/Back button (navigates back to list)
    await page.locator('[data-test="service-token-back-button"]').click();

    // Step 3: Verify URL changed back to the service tokens list
    await expect(page).toHaveURL(
      new RegExp(`/organizations/${env.organizationId}/service_tokens$`),
      { timeout: 10000 }
    );

    // Verify the tokens list card is visible again
    await expect(page.locator('[data-test="service-tokens-card"]')).toBeVisible({ timeout: 10000 });
  });
});
