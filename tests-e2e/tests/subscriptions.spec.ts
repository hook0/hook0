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
  ): Promise<{
    email: string;
    password: string;
    organizationId: string;
    applicationId: string;
    timestamp: number;
  }> {
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

    // Capture application ID inside predicate to avoid race condition with navigation
    let applicationId: string = "";
    const createAppResponse = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/applications") && response.request().method() === "POST") {
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
    // Always extract applicationId from URL (most reliable due to navigation timing)
    await expect(page).toHaveURL(/\/applications\/[^/]+/, { timeout: 10000 });
    const url = page.url();
    const match = url.match(/\/applications\/([^/]+)/);
    expect(match, "Failed to extract application ID from URL").toBeTruthy();
    applicationId = match![1];

    // Create an event type (required for subscriptions)
    await page.goto(
      `/organizations/${organizationId}/applications/${applicationId}/event_types/new`
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
      applicationId,
      timestamp,
    };
  }

  test("should display subscriptions list with created subscription", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "list");
    const description = `Test Subscription ${env.timestamp}`;
    const webhookUrl = "https://webhook.site/test-list";

    // Step 1: CREATE a subscription via UI
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="subscription-description-input"]').fill(description);
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");
    await page.locator('[data-test="subscription-url-input"]').fill(webhookUrl);

    // Add a label using data-test selectors
    const labelKeyInput = page.locator('[data-test="subscription-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="subscription-labels"] [data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });
    await labelKeyInput.fill("env");
    await labelValueInput.fill("test");

    // Select an event type using data-test selector
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 5000 });
    await eventTypeCheckbox.click();

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="subscription-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for navigation after subscription creation (router.back() is called)
    // The form will redirect back, so wait for the URL to change from /new
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Step 2: Navigate to subscriptions list
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions`
    );

    // Verify subscriptions card is visible
    await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="subscriptions-create-button"]')).toBeVisible();

    // Step 3: Verify list has at least 1 row (AG Grid uses [row-id] class)
    const rows = page.locator('[data-test="subscriptions-table"] [row-id]');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 4: Verify first row contains expected subscription data
    const firstRow = rows.first();
    await expect(firstRow).toContainText(description);
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

    // Add a label (required for subscriptions) using data-test selectors
    const labelKeyInput = page.locator('[data-test="subscription-labels"] [data-test="kv-key-input-0"]');
    const labelValueInput = page.locator('[data-test="subscription-labels"] [data-test="kv-value-input-0"]');

    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });
    await labelKeyInput.fill("env");
    await labelValueInput.fill("test");

    // Select an event type using data-test selector
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 5000 });
    await eventTypeCheckbox.click();

    // Step 2: Submit and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { subscription_id?: string; description?: string; target?: { url?: string; method?: string } } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/subscriptions") && response.request().method() === "POST") {
          if (response.status() < 400) {
            try {
              responseBody = await response.json();
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

    await page.locator('[data-test="subscription-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    // Verify response body was captured and contains expected data
    expect(responseBody.subscription_id, "Response body should contain subscription_id").toBeTruthy();
    expect(responseBody.description).toBe(description);
    expect(responseBody.target?.url).toBe(webhookUrl);
    expect(responseBody.target?.method).toBe("POST");
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

  /**
   * Helper to create a subscription and return its ID
   * Extracts ID from response body or from the subscriptions list page UI
   */
  async function createSubscription(
    page: import("@playwright/test").Page,
    _request: import("@playwright/test").APIRequestContext,
    env: { organizationId: string; applicationId: string; timestamp: number },
    description: string
  ): Promise<string> {
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/new`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="subscription-description-input"]').fill(description);
    await page.locator('[data-test="subscription-method-select"]').selectOption("POST");
    await page.locator('[data-test="subscription-url-input"]').fill("https://webhook.site/test");

    // Add label using data-test selectors (scoped to subscription-labels container)
    const labelKeyInput = page.locator(
      '[data-test="subscription-labels"] [data-test="kv-key-input-0"]'
    );
    const labelValueInput = page.locator(
      '[data-test="subscription-labels"] [data-test="kv-value-input-0"]'
    );
    await expect(labelKeyInput).toBeVisible({ timeout: 5000 });

    // Clear and fill key input, then blur to trigger debounced emit
    await labelKeyInput.clear();
    await labelKeyInput.fill("env");
    await labelKeyInput.blur();

    // Clear and fill value input, then blur to trigger debounced emit
    await labelValueInput.clear();
    await labelValueInput.fill("test");
    await labelValueInput.blur();

    // Wait for debounced label input to be processed
    await expect(labelKeyInput).toHaveValue("env");
    await expect(labelValueInput).toHaveValue("test");

    // Select event type using data-test selector
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 5000 });
    await eventTypeCheckbox.click();

    // Submit the form and wait for API response
    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/subscriptions") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-submit-button"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);

    // Wait for navigation after subscription creation (router.back() is called)
    await expect(page).not.toHaveURL(/\/subscriptions\/new/, { timeout: 10000 });

    // Navigate to subscriptions list and find our subscription by description
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions`
    );
    await expect(page.locator('[data-test="subscriptions-card"]')).toBeVisible({ timeout: 10000 });

    // Find the subscription row with our description
    const subscriptionRow = page.locator('[data-test="subscriptions-table"] [row-id]').filter({
      hasText: description,
    });
    await expect(subscriptionRow.first()).toBeVisible({ timeout: 10000 });

    // Get the link in the description column and wait for href to be set
    const descriptionLink = subscriptionRow.first().locator('[data-test="subscription-description-link"]');
    await expect(descriptionLink).toBeVisible({ timeout: 10000 });

    // Extract subscription ID from the href attribute
    const href = await descriptionLink.getAttribute("href");
    expect(href, "Link href attribute is missing").toBeTruthy();

    const match = href!.match(/\/subscriptions\/([a-f0-9-]+)$/);
    expect(match, "Could not extract subscription_id from href").toBeTruthy();
    const subscriptionId = match![1];

    return subscriptionId;
  }

  test("should update subscription description and verify API response", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "update");
    const originalDescription = `Original Subscription ${env.timestamp}`;
    const updatedDescription = `Updated Subscription ${env.timestamp}`;

    // Create a subscription first
    const subscriptionId = await createSubscription(page, request, env, originalDescription);

    // Navigate to subscription edit page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Update the description
    await page.locator('[data-test="subscription-description-input"]').clear();
    await page.locator('[data-test="subscription-description-input"]').fill(updatedDescription);

    // Step 2: Submit and wait for API response
    // Use simple predicate - don't try to access response body in predicate due to navigation race condition
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/subscriptions/${subscriptionId}`) &&
        response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response status (body may not be available due to navigation)
    expect(response.status()).toBeLessThan(400);

    // Step 4: Wait for the router.back() navigation that happens after form submission
    // This navigates to the subscriptions list page
    await expect(page).toHaveURL(/\/subscriptions$/, { timeout: 10000 });

    // Step 5: Verify the update persisted by navigating back to the subscription
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="subscription-description-input"]')).toHaveValue(
      updatedDescription
    );
  });

  test("should update subscription URL and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "update-url");
    const description = `URL Update Test ${env.timestamp}`;
    const updatedUrl = "https://webhook.site/updated-endpoint";

    // Create a subscription first
    const subscriptionId = await createSubscription(page, request, env, description);

    // Navigate to subscription edit page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Update the URL
    await page.locator('[data-test="subscription-url-input"]').clear();
    await page.locator('[data-test="subscription-url-input"]').fill(updatedUrl);

    // Step 2: Submit and wait for API response
    // Use simple predicate - don't try to access response body in predicate due to navigation race condition
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/subscriptions/${subscriptionId}`) &&
        response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response status (body may not be available due to navigation)
    expect(response.status()).toBeLessThan(400);

    // Step 4: Wait for the router.back() navigation that happens after form submission
    // This navigates to the subscriptions list page
    await expect(page).toHaveURL(/\/subscriptions$/, { timeout: 10000 });

    // Step 5: Verify the update persisted by navigating back to the subscription
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="subscription-url-input"]')).toHaveValue(updatedUrl);
  });

  test("should delete subscription and verify API response", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "delete");
    const description = `Delete Test Subscription ${env.timestamp}`;

    // Create a subscription first
    const subscriptionId = await createSubscription(page, request, env, description);

    // Navigate to subscription edit page (where delete button is)
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );

    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify delete card is visible
    await expect(page.locator('[data-test="subscription-delete-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="subscription-delete-button"]')).toBeVisible();

    // Setup dialog handler for confirmation
    page.on("dialog", (dialog) => {
      dialog.accept();
    });

    // Step 2: Click delete and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/subscriptions`) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-delete-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify redirect to subscriptions list
    await expect(page).toHaveURL(/\/subscriptions$/, {
      timeout: 15000,
    });
  });

  test("should cancel delete when dialog is dismissed", async ({ page, request }) => {
    const env = await setupTestEnvironment(page, request, "delete-cancel");
    const description = `Cancel Delete Test ${env.timestamp}`;

    // Create a subscription first
    const subscriptionId = await createSubscription(page, request, env, description);

    // Navigate to subscription edit page
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/subscriptions/${subscriptionId}`
    );

    await expect(page.locator('[data-test="subscription-delete-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click delete button
    await page.locator('[data-test="subscription-delete-button"]').click();

    // Should still be on the edit page (not redirected)
    await expect(page).toHaveURL(new RegExp(`/subscriptions/${subscriptionId}$`), {
      timeout: 5000,
    });

    // Form should still be visible
    await expect(page.locator('[data-test="subscription-form"]')).toBeVisible();
  });
});
