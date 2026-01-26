import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Event Types E2E tests for Hook0.
 *
 * Tests for creating, viewing, and deactivating event types.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Event Types", () => {
  test("should display event types list page", async ({ page, request }) => {
    // Setup: Create test user and application
    const timestamp = Date.now();
    const email = `test-event-types-list-${timestamp}@hook0.local`;
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

    // Create an application first
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

    // Navigate to event types list
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types`
    );

    // Verify event types card is visible
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="event-types-create-button"]')).toBeVisible();
  });

  test("should create new event type with required fields and verify API response", async ({
    page,
    request,
  }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-event-types-create-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const service = "billing";
    const resourceType = "invoice";
    const verb = "created";

    // Register and verify email
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

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

    // Create an application first
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

    // Navigate to create event type page
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );

    // Wait for form
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 1: Fill form
    await page.locator('[data-test="event-type-service-input"]').fill(service);
    await page.locator('[data-test="event-type-resource-input"]').fill(resourceType);
    await page.locator('[data-test="event-type-verb-input"]').fill(verb);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="event-type-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);
    const responseBody = await response.json();
    expect(responseBody.event_type_name).toBe(`${service}.${resourceType}.${verb}`);

    // Verify redirect to event types list
    await expect(page).toHaveURL(/\/event_types$/, {
      timeout: 15000,
    });

    // Verify the event type appears in the list
    await expect(page.locator('[data-test="event-types-table"]')).toBeVisible({ timeout: 10000 });
  });

  test("should show disabled submit when event type fields are empty", async ({
    page,
    request,
  }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-event-types-validation-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
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

    // Create an application first
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

    // Navigate to create event type page
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );

    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled when fields are empty
    await expect(page.locator('[data-test="event-type-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill only service - still disabled
    await page.locator('[data-test="event-type-service-input"]').fill("billing");
    await expect(page.locator('[data-test="event-type-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill resource type - still disabled
    await page.locator('[data-test="event-type-resource-input"]').fill("invoice");
    await expect(page.locator('[data-test="event-type-submit-button"]')).toHaveAttribute(
      "disabled",
      "true"
    );

    // Fill verb - now enabled
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await expect(page.locator('[data-test="event-type-submit-button"]')).not.toHaveAttribute(
      "disabled",
      "true"
    );
  });

  test("should navigate back when clicking cancel button", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-event-types-cancel-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
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

    // Create an application first
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

    // Navigate to event types list first (to have a history entry)
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types`
    );
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

    // Click create button
    await page.locator('[data-test="event-types-create-button"]').click();

    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Click cancel
    await page.locator('[data-test="event-type-cancel-button"]').click();

    // Should go back to event types list
    await expect(page).toHaveURL(/\/event_types$/, {
      timeout: 10000,
    });
  });
});
