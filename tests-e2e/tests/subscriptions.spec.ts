import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Subscriptions (Webhooks) E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting webhook subscriptions.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Subscriptions", () => {
  /**
   * Helper to setup test environment: user, organization, application, and event type
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-sub-${testId}-${timestamp}@hook0.local`;
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

    const createAppResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);
    const app = await appResponse.json();

    // Create an event type (required for subscriptions)
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("billing");
    await page.locator('[data-test="event-type-resource-input"]').fill("invoice");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createEventTypeResponse;

    return {
      email,
      password,
      organizationId,
      applicationId: app.application_id,
      timestamp,
    };
  }

  test("should display subscriptions list page", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list");

    // Navigate to subscriptions list
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions`
    );

    // Verify subscriptions card is visible
    await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="subscriptions-create-button"]')).toBeVisible();
  });

  test("should display subscription form with all required elements", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "form");

    // Navigate to create subscription page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );

    // Verify form is visible
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify all form elements are present
    await expect(page.locator('[data-test="subscription-description-input"]')).toBeVisible();
    await expect(page.locator('[data-test="subscription-method-select"]')).toBeVisible();
    await expect(page.locator('[data-test="subscription-url-input"]')).toBeVisible();
    await expect(page.locator('[data-test="subscription-submit-button"]')).toBeVisible();
    await expect(page.locator('[data-test="subscription-cancel-button"]')).toBeVisible();
  });

  test("should create new subscription with required fields and verify API response", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "create");
    const description = `Test Subscription ${env.timestamp}`;
    const webhookUrl = "https://webhook.site/test-hook";

    // Navigate to create subscription page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill form
    await page.locator('[data-test="subscription-description-input"]').fill(description);

    // Select HTTP method (POST is commonly used)
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");

    // Fill webhook URL
    await page.locator('[data-test="subscription-url-input"]').fill(webhookUrl);

    // Add a label (required for subscriptions)
    // Labels are in Hook0KeyValue component - let's find the first key/value inputs
    const labelKeyInput = page.locator('input[placeholder="Label key"]').first();
    const labelValueInput = page.locator('input[placeholder="Label value"]').first();

    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });
    await labelKeyInput.fill("env");
    await labelValueInput.fill("test");

    // Select an event type (checkbox)
    const eventTypeCheckbox = page.locator('input[type="checkbox"]').first();
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 5000 });
    await eventTypeCheckbox.click();

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    const responseBody = await response.json();
    expect(responseBody).toHaveProperty("subscription_id");
    expect(responseBody.description).toBe(description);
    expect(responseBody.target.url).toBe(webhookUrl);
    expect(responseBody.target.method).toBe("POST");
  });

  test("should show disabled submit when required fields are empty", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "validation");

    // Navigate to create subscription page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled initially (missing required fields)
    await expect(page.locator('[data-test="subscription-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill description only - still disabled
    await page.locator('[data-test="subscription-description-input"]').fill("Test");
    await expect(page.locator('[data-test="subscription-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill URL - still disabled (missing labels and event types)
    await page.locator('[data-test="subscription-url-input"]').fill("https://test.com");
    await expect(page.locator('[data-test="subscription-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );
  });

  test("should navigate back when clicking cancel button", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "cancel");

    // Navigate to subscriptions list first
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions`
    );
    await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

    // Click create button
    await page.locator('[data-test="subscriptions-create-button"]').click();

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Click cancel
    await page.locator('[data-test="subscription-cancel-button"]').click();

    // Should go back to subscriptions list
    await expect(page).toHaveURL(/\/subscriptions$/, {
      timeout: 10000,
    });
  });
});
