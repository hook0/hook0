import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Command Palette E2E tests for Hook0.
 *
 * Tests the command palette overlay, filtering, navigation, and dismissal.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Command Palette", () => {
  /**
   * Helper to setup test environment: register, verify, login, create org + app
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ): Promise<{
    email: string;
    password: string;
    organizationId: string;
    applicationId: string;
    timestamp: number;
  }> {
    const timestamp = Date.now();
    const email = `test-cmdpal-${testId}-${timestamp}@hook0.local`;
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

    // Create an application (needed for command palette navigation items)
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(`CmdPal App ${timestamp}`);

    const uuidPattern = /\/applications\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;
    const createAppResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);
    await expect(page).toHaveURL(uuidPattern, { timeout: 15000 });
    const url = page.url();
    const match = url.match(uuidPattern);
    expect(match, "Failed to extract application ID from URL").toBeTruthy();
    const applicationId = match![1];

    return {
      email,
      password,
      organizationId,
      applicationId,
      timestamp,
    };
  }

  const overlaySelector = '[data-test="command-palette-overlay"]';
  const inputSelector = '[data-test="command-palette-input"]';
  // Items have dynamic data-test: "command-palette-item-{id}", match any with prefix
  const itemSelector = '[data-test^="command-palette-item-"]';

  test("should open command palette via Search button", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "open");

    // Navigate to an application page so command palette has navigation items
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/dashboard`
    );
    await expect(
      page.locator('[data-test="event-types-card"], [data-test="application-dashboard-card"]').first()
    ).toBeVisible({ timeout: 15000 });

    // Step 1: Click the Search button in top nav
    const searchButton = page.locator('[data-test="search-button"]');
    await expect(searchButton).toBeVisible({ timeout: 10000 });
    await searchButton.click();

    // Step 2: Verify command palette overlay is visible
    await expect(page.locator(overlaySelector)).toBeVisible({ timeout: 10000 });

    // Step 3: Verify command palette input is visible
    await expect(page.locator(inputSelector)).toBeVisible();
  });

  test("should filter results when typing", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "filter");

    // Navigate to an application page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/dashboard`
    );
    await expect(
      page.locator('[data-test="event-types-card"], [data-test="application-dashboard-card"]').first()
    ).toBeVisible({ timeout: 15000 });

    // Step 1: Open command palette
    const searchButton = page.locator('[data-test="search-button"]');
    await expect(searchButton).toBeVisible({ timeout: 10000 });
    await searchButton.click();
    await expect(page.locator(inputSelector)).toBeVisible({ timeout: 10000 });

    // Step 2: Type "event" in the input
    await page.locator(inputSelector).fill("event");

    // Step 3: Verify at least one command palette item is visible
    await expect(page.locator(itemSelector).first()).toBeVisible({ timeout: 5000 });
  });

  test("should navigate to result", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "navigate");

    // Navigate to an application page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/dashboard`
    );
    await expect(
      page.locator('[data-test="event-types-card"], [data-test="application-dashboard-card"]').first()
    ).toBeVisible({ timeout: 15000 });

    // Step 1: Open command palette and type "Event Types"
    const searchButton = page.locator('[data-test="search-button"]');
    await expect(searchButton).toBeVisible({ timeout: 10000 });
    await searchButton.click();
    await expect(page.locator(inputSelector)).toBeVisible({ timeout: 10000 });
    await page.locator(inputSelector).fill("Event Types");

    // Step 2: Wait for results and click first one
    const firstResult = page.locator(itemSelector).first();
    await expect(firstResult).toBeVisible({ timeout: 5000 });
    await firstResult.click();

    // Step 3: Verify URL changed to an event types page
    await expect(page).toHaveURL(/\/event_types/, { timeout: 10000 });
  });

  test("should close palette with Escape", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "escape");

    // Navigate to an application page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/dashboard`
    );
    await expect(
      page.locator('[data-test="event-types-card"], [data-test="application-dashboard-card"]').first()
    ).toBeVisible({ timeout: 15000 });

    // Step 1: Open command palette
    const searchButton = page.locator('[data-test="search-button"]');
    await expect(searchButton).toBeVisible({ timeout: 10000 });
    await searchButton.click();
    await expect(page.locator(overlaySelector)).toBeVisible({ timeout: 10000 });

    // Step 2: Press Escape
    await page.keyboard.press("Escape");

    // Step 3: Verify overlay is not visible
    await expect(page.locator(overlaySelector)).not.toBeVisible({ timeout: 5000 });
  });
});
