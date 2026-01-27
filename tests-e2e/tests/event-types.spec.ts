import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Event Types E2E tests for Hook0.
 *
 * Tests for creating, viewing, and deactivating event types.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Event Types", () => {
  test("should display event types list with created event type", async ({ page, request }) => {
    // Setup: Create test user and application
    const timestamp = Date.now();
    const email = `test-event-types-list-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const service = "user";
    const resourceType = "account";
    const verb = "created";
    const expectedEventTypeName = `${service}.${resourceType}.${verb}`;

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

    // Capture response body inside the predicate to avoid race condition with navigation
    let applicationId: string = "";
    const createAppResponse = page.waitForResponse(
      async (response) => {
        if (response.url().includes("/api/v1/applications") && response.request().method() === "POST") {
          if (response.status() < 400) {
            const app = await response.json();
            applicationId = app.application_id;
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

    // Step 1: CREATE an event type via UI
    await page.goto(
      `/organizations/${organizationId}/applications/${applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill(service);
    await page.locator('[data-test="event-type-resource-input"]').fill(resourceType);
    await page.locator('[data-test="event-type-verb-input"]').fill(verb);

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    const eventTypeResponse = await createEventTypeResponse;
    expect(eventTypeResponse.status()).toBeLessThan(400);

    // Step 2: Navigate to event types list
    await page.goto(
      `/organizations/${organizationId}/applications/${applicationId}/event_types`
    );

    // Verify event types card is visible
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="event-types-create-button"]')).toBeVisible();

    // Step 3: Verify list has at least 1 row (AG Grid uses .ag-row class)
    const rows = page.locator('[data-test="event-types-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 4: Verify first row contains expected event type data
    const firstRow = rows.first();
    await expect(firstRow).toContainText(expectedEventTypeName);
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

  test("should deactivate event type and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-event-types-deactivate-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const service = "notification";
    const resourceType = "email";
    const verb = "sent";
    const eventTypeName = `${service}.${resourceType}.${verb}`;

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

    // Create an event type to deactivate
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill(service);
    await page.locator('[data-test="event-type-resource-input"]').fill(resourceType);
    await page.locator('[data-test="event-type-verb-input"]').fill(verb);

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createEventTypeResponse;

    // Navigate to event types list
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types`
    );
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

    // Verify the event type exists in the list
    const rows = page.locator('[data-test="event-types-table"] .ag-row');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Find the row with our event type
    const targetRow = rows.filter({ hasText: eventTypeName });
    await expect(targetRow).toBeVisible();

    // Setup dialog handler for deactivate confirmation
    page.on("dialog", (dialog) => {
      dialog.accept();
    });

    // Click deactivate button (it's in the row, text "Deactivate")
    const deactivateResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") &&
        response.url().includes(encodeURIComponent(eventTypeName)) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await targetRow.locator('[data-test="event-type-deactivate-button"]').click();

    const deactivateResponse = await deactivateResponsePromise;

    // Verify API response
    expect(deactivateResponse.status()).toBeLessThan(400);

    // Verify the event type is no longer in the list
    await expect(targetRow).not.toBeVisible({ timeout: 10000 });
  });

  test("should cancel deactivation when dialog is dismissed", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-event-types-cancel-deact-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const service = "order";
    const resourceType = "payment";
    const verb = "received";
    const eventTypeName = `${service}.${resourceType}.${verb}`;

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

    // Create an event type
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill(service);
    await page.locator('[data-test="event-type-resource-input"]').fill(resourceType);
    await page.locator('[data-test="event-type-verb-input"]').fill(verb);

    const createEventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    await createEventTypeResponse;

    // Navigate to event types list
    await page.goto(
      `/organizations/${organizationId}/applications/${app.application_id}/event_types`
    );
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 10000 });

    // Find the row with our event type
    const rows = page.locator('[data-test="event-types-table"] .ag-row');
    const targetRow = rows.filter({ hasText: eventTypeName });
    await expect(targetRow).toBeVisible();

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click deactivate button
    await targetRow.locator('[data-test="event-type-deactivate-button"]').click();

    // Verify the event type is still in the list (deactivation was cancelled)
    await expect(targetRow).toBeVisible();
    await expect(targetRow).toContainText(eventTypeName);
  });
});
