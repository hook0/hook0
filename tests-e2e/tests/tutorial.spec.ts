import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Tutorial E2E tests for Hook0.
 *
 * Tests for the onboarding tutorial flow.
 * Following the Three-Step Verification Pattern.
 *
 * These tests directly navigate to the tutorial page to ensure the tutorial
 * functionality works correctly, regardless of whether the user is automatically
 * redirected to it after login.
 */
test.describe("Tutorial", () => {
  /**
   * Helper to setup test environment with authenticated user
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-tutorial-${testId}-${timestamp}@hook0.local`;
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

    // Wait for redirect to any authenticated area
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    return { email, password, timestamp };
  }

  test("should display tutorial introduction page with all required elements", async ({
    page,
    request,
  }) => {
    await setupTestEnvironment(page, request, "intro");

    // Navigate directly to tutorial page
    await page.goto("/tutorial");

    // Verify tutorial page URL
    await expect(page).toHaveURL(/\/tutorial/, { timeout: 10000 });

    // Verify tutorial page contains expected elements
    await expect(page.locator("text=Welcome to Hook0")).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeVisible();
    await expect(page.locator('[data-test="tutorial-skip-button"]')).toBeVisible();
  });

  test("should skip tutorial and redirect to organizations dashboard", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "skip");

    // Navigate directly to tutorial page
    await page.goto("/tutorial");

    // Wait for tutorial page to load
    await expect(page.locator('[data-test="tutorial-skip-button"]')).toBeVisible({ timeout: 10000 });

    // Click skip button
    await page.locator('[data-test="tutorial-skip-button"]').click();

    // Should be redirected to home (which is "/" - the Home route)
    // Note: The Home route in Hook0 is "/" which redirects based on user state
    // The full URL is like "http://localhost:8001/" so we match the path ending with / or continuing
    await expect(page).toHaveURL(/\/$|\/organizations|\/dashboard/, {
      timeout: 15000,
    });
  });

  test("should start tutorial and navigate to create organization step", async ({
    page,
    request,
  }) => {
    await setupTestEnvironment(page, request, "start");

    // Navigate directly to tutorial page
    await page.goto("/tutorial");

    // Wait for tutorial page to load
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeVisible({ timeout: 10000 });

    // Click start button
    await page.locator('[data-test="tutorial-start-button"]').click();

    // Should be redirected to the first tutorial step (create organization)
    // The actual route is /tutorial/organization
    await expect(page).toHaveURL(/\/tutorial\/organization/, {
      timeout: 15000,
    });
  });

  test("should complete tutorial create organization step and verify API response", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "create-org");

    // Navigate directly to tutorial page
    await page.goto("/tutorial");

    // Wait for tutorial page to load and click Start
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="tutorial-start-button"]').click();

    // Wait for create organization step
    // The actual route is /tutorial/organization
    await expect(page).toHaveURL(/\/tutorial\/organization/, {
      timeout: 15000,
    });

    // Select "Create a new organization" option (radio button)
    // The form is only shown after selecting this option
    const createOrgRadio = page.locator('input[type="radio"][value="create_organization"]');
    await expect(createOrgRadio).toBeVisible({ timeout: 10000 });
    await createOrgRadio.click();

    // Fill organization name
    await expect(page.locator('[data-test="organization-name-input"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="organization-name-input"]').fill(`Tutorial Org ${env.timestamp}`);

    // Submit and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { organization_id?: string } = {};
    const orgResponsePromise = page.waitForResponse(
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

    const orgResponse = await orgResponsePromise;

    // Verify API response
    expect(orgResponse.status()).toBeLessThan(400);
    expect(responseBody).toHaveProperty("organization_id");

    // Should proceed to next tutorial step
    await expect(page).toHaveURL(/\/tutorial/, {
      timeout: 15000,
    });
  });

  test("should be accessible directly via URL", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "direct");

    // Navigate directly to tutorial page
    await page.goto("/tutorial");

    // Verify tutorial page is accessible and loaded
    await expect(page).toHaveURL(/\/tutorial/, { timeout: 10000 });

    // Verify main content is present
    await expect(page.locator("text=Hook0")).toBeVisible({ timeout: 10000 });
  });
});
