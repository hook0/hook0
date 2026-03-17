import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Tutorial Wizard Flow E2E tests for Hook0.
 *
 * Tests the complete tutorial wizard from intro through all steps to success.
 * Following the Three-Step Verification Pattern:
 * 1. Fill form fields
 * 2. Submit and waitForResponse on API endpoint
 * 3. Verify response.status < 400 AND verify data/navigation
 */
test.describe("Tutorial Wizard Flow", () => {
  /**
   * Helper to setup test environment with authenticated user.
   */
  async function setupTestEnvironment(
    page: import("@playwright/test").Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ) {
    const timestamp = Date.now();
    const email = `test-wizard-${testId}-${timestamp}@hook0.local`;
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

    const loginResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/auth/login") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();
    const loginResponse = await loginResponsePromise;
    expect(loginResponse.status()).toBeLessThan(400);

    // Wait for redirect to any authenticated area
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    return { email, password, timestamp };
  }

  test("should complete full tutorial wizard flow from intro to success", async ({
    page,
    request,
  }) => {
    const env = await setupTestEnvironment(page, request, "full-flow");

    // Use a larger viewport so wizard modal content fits without scrolling
    await page.setViewportSize({ width: 1280, height: 1024 });

    // Navigate to tutorial intro
    await page.goto("/tutorial");
    await expect(page).toHaveURL(/\/tutorial/, { timeout: 10000 });

    // --- Intro: Click Start ---
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="tutorial-start-button"]').click();

    // --- Step 1: Create Organization ---
    await expect(page).toHaveURL(/\/tutorial\/organization/, { timeout: 15000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // The "Create a new organization" card may be auto-selected when no orgs exist.
    // Click it if the selectable card is visible, otherwise the form is already shown.
    const createOrgRadio = page.locator('[data-test="tutorial-create-org-radio"]');
    if (await createOrgRadio.isVisible({ timeout: 3000 }).catch(() => false)) {
      await createOrgRadio.click();
    }

    // Fill organization name
    const orgNameInput = page.locator('[data-test="organization-name-input"]');
    await expect(orgNameInput).toBeVisible({ timeout: 10000 });
    await orgNameInput.fill(`Wizard Org ${env.timestamp}`);

    // Step 1: Submit org creation and wait for API response
    const orgResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/organizations") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    // Also wait for the token refresh that happens after org creation (new org permissions in JWT)
    const refreshResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/refresh") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="organization-submit-button"]').click();
    const orgResponse = await orgResponsePromise;

    // Step 1: Verify response
    expect(orgResponse.status()).toBeLessThan(400);

    // Wait for token refresh to complete (needed for subsequent API calls with new org permissions)
    await refreshResponsePromise;

    // After org creation, a continue button appears in the entity step footer.
    const orgContinueButton = page.locator(
      '[data-test="tutorial-wizard-modal"] .wizard-modal__footer button.hook0-button.primary'
    );
    await expect(orgContinueButton).toBeVisible({ timeout: 10000 });
    await orgContinueButton.click();

    // --- Step 2: Create Application ---
    await expect(page).toHaveURL(/\/tutorial\/application/, { timeout: 15000 });

    // Reload page to ensure fresh state with the updated JWT token
    await page.reload({ waitUntil: "networkidle" });

    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // When no apps exist and requireOptions=true, the entity step auto-selects "Create"
    // and hides the selectable cards. The ApplicationsEdit form is shown directly.
    const createAppRadio = page.locator('[data-test="tutorial-create-app-radio"]');
    if (await createAppRadio.isVisible({ timeout: 3000 }).catch(() => false)) {
      await createAppRadio.click();
    }

    // Fill application name (may take time to load the app list and show the form)
    const appNameInput = page.locator('[data-test="application-name-input"]');
    await expect(appNameInput).toBeVisible({ timeout: 20000 });
    await appNameInput.fill(`Wizard App ${env.timestamp}`);

    // Step 2: Submit app creation and verify via UI (continue button appears on success)
    await page.locator('[data-test="application-submit-button"]').click();

    // After app creation, the continue button appears in the footer.
    const appContinueButton = page.locator(
      '[data-test="tutorial-wizard-modal"] .wizard-modal__footer button.hook0-button.primary'
    );
    await expect(appContinueButton).toBeVisible({ timeout: 15000 });
    await appContinueButton.click();

    // --- Step 3: Create Event Type (3 segments) ---
    await expect(page).toHaveURL(/\/tutorial\/event_type/, { timeout: 15000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill the 3 event type segments
    await expect(page.locator('[data-test="event-type-service-input"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("billing");
    await page.locator('[data-test="event-type-resource-input"]').fill("invoice");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");

    // Step 3: Submit and wait for API response
    const eventTypeResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="event-type-submit-button"]').click();
    const eventTypeResponse = await eventTypeResponsePromise;

    // Step 3: Verify
    expect(eventTypeResponse.status()).toBeLessThan(400);

    // Steps 3-5 auto-advance via TutorialWizardStepForm -> emit('advance')

    // --- Step 4: Create Subscription ---
    await expect(page).toHaveURL(/\/tutorial\/subscription/, { timeout: 15000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill description (required)
    const descriptionInput = page.locator('[data-test="subscription-description-input"]');
    await expect(descriptionInput).toBeVisible({ timeout: 10000 });
    await descriptionInput.fill("Test webhook subscription");

    // Fill URL (required)
    await page.locator('[data-test="subscription-url-input"]').fill("https://example.com/webhook");

    // Add a label key-value pair (required: hasRequiredLabels needs at least one)
    // Scope to the subscription-labels container to avoid matching the headers KV section
    const labelsContainer = page.locator('[data-test="subscription-labels"]');
    const labelKeyInput = labelsContainer.locator('[data-test="kv-key-input-0"]');
    const labelValueInput = labelsContainer.locator('[data-test="kv-value-input-0"]');
    await expect(labelKeyInput).toBeVisible({ timeout: 10000 });
    await labelKeyInput.fill("env");
    await labelValueInput.fill("test");

    // Select at least one event type (required: hasSelectedEventTypes)
    const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
    await expect(eventTypeCheckbox).toBeVisible({ timeout: 10000 });
    await eventTypeCheckbox.click();

    // Step 4: Submit and wait for API response
    const subscriptionResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/subscriptions") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="subscription-submit-button"]').click();
    const subscriptionResponse = await subscriptionResponsePromise;

    // Step 4: Verify
    expect(subscriptionResponse.status()).toBeLessThan(400);

    // --- Step 5: Send Event ---
    await expect(page).toHaveURL(/\/tutorial\/event/, { timeout: 15000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // Select event type (Hook0Select auto-selects first option on mount via initValue)
    // Fill occurredAt (required, starts empty)
    const occurredAtInput = page.locator('[data-test="send-event-occurred-at-input"]');
    await expect(occurredAtInput).toBeVisible({ timeout: 10000 });
    const now = new Date();
    const datetimeLocal = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}T${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`;
    await occurredAtInput.fill(datetimeLocal);

    // Step 5: Submit and wait for API response (endpoint is /event singular)
    const sendEventResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/event") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await expect(page.locator('[data-test="send-event-submit-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="send-event-submit-button"]').click();
    const sendEventResponse = await sendEventResponsePromise;

    // Step 5: Verify
    expect(sendEventResponse.status()).toBeLessThan(400);

    // --- Step 6: Verify Success ---
    await expect(page).toHaveURL(/\/tutorial\/success/, { timeout: 15000 });

    // Verify success page content via the dashboard button
    await expect(page.locator('[data-test="tutorial-success-dashboard-button"]')).toBeVisible({
      timeout: 15000,
    });

    // Click "Go to dashboard" button
    await page.locator('[data-test="tutorial-success-dashboard-button"]').click();

    // Verify redirect to dashboard/organizations area
    await expect(page).toHaveURL(/\/dashboard|\/organizations/, {
      timeout: 15000,
    });
  });

  test("should dismiss wizard and return to home", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "dismiss");

    // Navigate to tutorial and start wizard
    await page.goto("/tutorial");
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="tutorial-start-button"]').click();

    // Wait for step 1 (create organization)
    await expect(page).toHaveURL(/\/tutorial\/organization/, { timeout: 15000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });

    // Click the close (X) button in the wizard modal header
    const closeButton = page.locator(
      '[data-test="tutorial-wizard-modal"] .wizard-modal__close'
    );
    await expect(closeButton).toBeVisible({ timeout: 10000 });
    await closeButton.click();

    // Verify redirect to home
    await expect(page).toHaveURL(/\/$|\/organizations|\/dashboard/, {
      timeout: 15000,
    });
  });

  test("should access tutorial steps directly via URL", async ({ page, request }) => {
    await setupTestEnvironment(page, request, "direct-url");

    // Navigate directly to the organization step
    await page.goto("/tutorial/organization");

    // Verify wizard renders at step 1
    await expect(page).toHaveURL(/\/tutorial\/organization/, { timeout: 10000 });
    await expect(page.locator('[data-test="tutorial-wizard-modal"]')).toBeVisible({
      timeout: 10000,
    });
  });
});
