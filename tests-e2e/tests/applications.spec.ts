import { test, expect, type Page, type Route } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";
import { fromItsOwnAddress, loginAsNewUser } from "../fixtures/test-setup";

/**
 * Application management E2E tests for Hook0.
 *
 * Tests for creating, viewing, updating, and deleting applications.
 * Uses UI-based login to avoid auth state conflicts.
 * Gets organization ID from database during email verification.
 */
test.describe("Applications", () => {
  test("should display applications list with created application", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-apps-list-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `List Test App ${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email and get organization ID from database
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

    // Wait for redirect to authenticated area (could be tutorial, dashboard, or organizations)
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Step 1: CREATE at least one application via UI before verifying list
    await page.goto(`/organizations/${organizationId}/applications/new`);
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const createdApp = await createResponse.json();

    // Step 2: Navigate to applications list
    await page.goto(`/organizations/${organizationId}/applications`);

    // Verify applications card is visible
    await expect(page.locator('[data-test="applications-card"]')).toBeVisible({ timeout: 10000 });

    // Verify create button is present
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible();

    // Step 3: Verify list has at least 1 row (AG Grid uses [row-id] class)
    const rows = page.locator('[data-test="applications-table"] [row-id]');
    const rowCount = await rows.count();
    expect(rowCount).toBeGreaterThanOrEqual(1);

    // Step 4: Verify first row contains expected application data
    const firstRow = rows.first();
    await expect(firstRow).toContainText(appName);
  });

  test("should create new application and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-create-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Test App ${timestamp}`;

    // Register and verify email, get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
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

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to applications list using the organization ID from database
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });

    // Click create button
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill form
    await page.locator('[data-test="application-name-input"]').fill(appName);

    // Submit and wait for API response
    // Capture response body inside the predicate to avoid race condition with navigation
    let responseBody: { application_id?: string; name?: string } = {};
    const responsePromise = page.waitForResponse(
      async (response) => {
        if (
          response.url().includes("/api/v1/applications") &&
          response.request().method() === "POST"
        ) {
          if (response.status() < 400) {
            responseBody = await response.json();
          }
          return true;
        }
        return false;
      },
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;

    // Verify API response
    expect(response.status()).toBeLessThan(400);
    expect(responseBody).toHaveProperty("application_id");
    expect(responseBody.name).toBe(appName);
  });

  test("should update application name and verify persistence", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-update-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const originalName = `Original App ${timestamp}`;
    const updatedName = `Updated App ${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
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

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to applications list and create application via UI
    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-create-button"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="applications-create-button"]').click();

    // Wait for form and create application
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="application-name-input"]').fill(originalName);

    // Submit and wait for API response to get the application ID
    // Creating, renaming or deleting an application makes the frontend rotate
    // the session (POST /auth/refresh) so the org/app selector picks up the
    // change. The server revokes the old refresh token as soon as it answers,
    // so leaving the page with page.goto() while that call is in flight drops
    // the new token pair: the next forced refresh then sends a revoked token,
    // gets a 403 and the app logs the user out. Each step below therefore
    // waits for the refresh and for the redirect that follows it.
    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    const createRefreshPromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/refresh") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();
    expect((await createRefreshPromise).status()).toBeLessThan(400);
    await expect(page).not.toHaveURL(/\/applications\/new/, { timeout: 10000 });

    // Navigate to settings
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Update name
    await page.locator('[data-test="application-name-input"]').clear();
    await page.locator('[data-test="application-name-input"]').fill(updatedName);

    // Submit and wait for the PUT, then for the session refresh it triggers.
    // Only the status of the PUT is read: on success the page calls
    // router.back(), which can leave the document before its body is read.
    // Persistence is checked by reloading the settings page instead.
    //
    // The refresh response is not awaited, only its effect is.
    // The page was reached with goto.
    // So router.back() loads a document.
    // That load starts moments after the refresh answers.
    // Playwright then sometimes never hears that answer.
    // The page still received and applied it.
    // The session in localStorage outlives the document.
    // That stored session is what the test reads.
    const sessionBefore = await storedSession(page);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "PUT",
      { timeout: 15000 }
    );

    await page.locator('[data-test="application-submit-button"]').click();

    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    await expect(page).not.toHaveURL(/\/settings$/, { timeout: 10000 });
    await page.waitForFunction(
      (before) => {
        const stored = globalThis.localStorage.getItem("auth");
        return typeof stored === "string" && stored !== before;
      },
      sessionBefore,
      { timeout: 15000 }
    );

    // Verify the new name persisted
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="application-name-input"]')).toHaveValue(updatedName);
  });

  test("should show disabled submit when name is empty", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-validation-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
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

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to create page using the organization ID from database
    await page.goto(`/organizations/${organizationId}/applications/new`);

    // Wait for form
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify submit is disabled when empty
    // Note: The Hook0Button component uses an <a> tag with disabled attribute when not a submit button.
    // Playwright's toBeDisabled() doesn't work with <a> elements, so we check the attribute directly.
    await expect(page.locator('[data-test="application-submit-button"]')).toBeDisabled();

    // Clear if any value
    await page.locator('[data-test="application-name-input"]').clear();

    // Still disabled
    await expect(page.locator('[data-test="application-submit-button"]')).toBeDisabled();
  });

  test("should delete application and verify API response", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-delete-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Delete Test App ${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
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
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Wait for navigation after app creation to complete before proceeding
    // The frontend redirects after creation, so we wait for that redirect
    await expect(page).toHaveURL(/\/applications\/[^/]+/, { timeout: 10000 });

    // Navigate to application settings page (where delete option is)
    // The navigation itself waits for the page to load
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    await expect(page.locator('[data-test="application-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify delete card is visible
    await expect(page.locator('[data-test="application-delete-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="application-delete-button"]')).toBeVisible();

    // Step 2: Click delete to open Hook0Dialog confirmation
    await page.locator('[data-test="application-delete-button"]').click();

    // Wait for confirmation dialog and click confirm
    const confirmButton = page.locator(
      ".hook0-dialog--danger .hook0-dialog__actions button:last-child"
    );
    await expect(confirmButton).toBeVisible({ timeout: 5000 });

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "DELETE",
      { timeout: 15000 }
    );

    await confirmButton.click();

    const response = await responsePromise;

    // Step 3: Verify API response
    expect(response.status()).toBeLessThan(400);

    // Verify redirect away from settings page (to dashboard, or login if session expired)
    await expect(page).not.toHaveURL(/\/settings/, {
      timeout: 15000,
    });
  });

  test("should cancel delete when dialog is dismissed", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-apps-delete-cancel-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const appName = `Cancel Delete App ${timestamp}`;

    // Register and get organization ID
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
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
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="application-submit-button"]').click();
    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    // Wait for navigation after app creation to complete before proceeding
    await expect(page).toHaveURL(/\/applications\/[^/]+/, { timeout: 10000 });

    // Navigate to application settings page
    await page.goto(`/organizations/${organizationId}/applications/${app.application_id}/settings`);

    await expect(page.locator('[data-test="application-delete-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Setup dialog handler to DISMISS the confirmation
    page.on("dialog", (dialog) => {
      dialog.dismiss();
    });

    // Click delete button
    await page.locator('[data-test="application-delete-button"]').click();

    // Should still be on the settings page (not redirected)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });

    // Application form should still be visible
    await expect(page.locator('[data-test="application-form"]')).toBeVisible();
  });

  /**
   * Creating an application forces a token refresh, and the API rotates the
   * refresh token as soon as it answers. When the page goes away before that
   * answer arrives, the browser keeps the refresh token the API just spent.
   * The session must survive: the reloaded page replays the refresh at once,
   * and the next mutation (which refreshes again) keeps the user logged in.
   */
  test("should keep the session when the page reloads during the post-mutation refresh", async ({
    page,
    request,
  }) => {
    const { organizationId, timestamp } = await loginAsNewUser(
      page,
      request,
      "apps-refresh-reload"
    );

    // Let the forced refresh reach the API (which rotates the token), then
    // hold its response so the page never sees the new pair.
    let heldRefresh: Route | null = null;
    let markRotated!: () => void;
    const rotated = new Promise<void>((resolve) => (markRotated = resolve));
    await page.route(
      "**/api/v1/auth/refresh",
      async (route) => {
        const response = await route.fetch();
        expect(response.status()).toBe(201);
        heldRefresh = route;
        markRotated();
      },
      { times: 1 }
    );

    await createApplicationViaUi(page, organizationId, `Interrupted Refresh App ${timestamp}`);
    await rotated;

    // The page goes away mid-refresh. Once reloaded, it must replay the
    // refresh right away rather than wait for the usual timer, which would
    // fire minutes later, long after the API stopped accepting the token.
    const replayedRefresh = page.waitForResponse(isRefreshResponse, { timeout: 15000 });
    await page.reload();
    await (heldRefresh as Route | null)?.abort().catch(() => {});
    expect((await replayedRefresh).status()).toBe(201);

    // A new mutation refreshes the token again and must keep the session.
    const nextRefresh = page.waitForResponse(isRefreshResponse, { timeout: 15000 });
    await createApplicationViaUi(page, organizationId, `After Reload App ${timestamp}`);
    expect((await nextRefresh).status()).toBe(201);

    await page.goto(`/organizations/${organizationId}/applications`);
    await expect(page.locator('[data-test="applications-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="login-form"]')).toHaveCount(0);
  });
});

function isRefreshResponse(response: { url(): string; request(): { method(): string } }): boolean {
  return response.url().includes("/api/v1/auth/refresh") && response.request().method() === "POST";
}

/** The session the page holds in localStorage, as stored. */
async function storedSession(page: Page): Promise<string> {
  const session = await page.evaluate(() => globalThis.localStorage.getItem("auth"));
  expect(typeof session, "the page holds no session").toBe("string");
  return String(session);
}

/** Fill and submit the application form, and wait for the creation to succeed. */
async function createApplicationViaUi(
  page: Page,
  organizationId: string,
  name: string
): Promise<void> {
  await page.goto(`/organizations/${organizationId}/applications/new`);
  await expect(page.locator('[data-test="application-form"]')).toBeVisible({
    timeout: 10000,
  });
  await page.locator('[data-test="application-name-input"]').fill(name);

  const createResponse = page.waitForResponse(
    (response) =>
      response.url().includes("/api/v1/applications") && response.request().method() === "POST",
    { timeout: 15000 }
  );
  await page.locator('[data-test="application-submit-button"]').click();
  expect((await createResponse).status()).toBeLessThan(400);
}
