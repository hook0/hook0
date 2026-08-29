import { test, expect } from "@playwright/test";
import {
  verifyEmailViaMailpit,
  API_BASE_URL,
  getPasswordResetTokenFromMailpit,
} from "../fixtures/email-verification";
import { expectToast, fromItsOwnAddress } from "../fixtures/test-setup";

/**
 * User Settings E2E tests for Hook0.
 *
 * Tests for viewing and updating user settings including password change.
 * Following the Three-Step Verification Pattern.
 */
test.describe("User Settings", () => {
  test("should display user settings page with all sections", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-settings-display-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

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

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    // Navigate to settings page
    await page.goto("/settings");

    // Verify user info card is visible
    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="user-email-input"]')).toBeVisible();

    // Verify change password card is visible
    await expect(page.locator('[data-test="change-password-card"]')).toBeVisible();
    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible();
    await expect(page.locator('[data-test="current-password-input"]')).toBeVisible();
    await expect(page.locator('[data-test="new-password-input"]')).toBeVisible();
    await expect(page.locator('[data-test="confirm-password-input"]')).toBeVisible();
    await expect(page.locator('[data-test="change-password-button"]')).toBeVisible();

    // Verify delete account card is visible
    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();
    await expect(page.locator('[data-test="delete-account-form"]')).toBeVisible();
    await expect(page.locator('[data-test="delete-account-button"]')).toBeVisible();
  });

  test("should display user email in personal information section", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-email-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify email is displayed and disabled (read-only)
    await expect(page.locator('[data-test="user-email-input"]')).toHaveValue(email);
    await expect(page.locator('[data-test="user-email-input"]')).toBeDisabled();
  });

  test("refuses a wrong current password, then changes the password for real", async ({
    page,
    request,
  }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-password-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const newPassword = `NewPassword456!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Holding a session is not enough to take the account over: whoever asks
    // for a new password has to know the one it replaces. Someone who walked up
    // to an unlocked browser knows the session, never the password.
    await page.locator('[data-test="current-password-input"]').fill(`Wrong${password}`);
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);

    const refusalPromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/password") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="change-password-button"]').click();
    expect(
      (await refusalPromise).status(),
      "a change offered with the wrong current password must be refused"
    ).toBeGreaterThanOrEqual(400);

    // The status alone would pass on a handler that refuses and writes anyway.
    // What settles it is which password opens the account afterwards: the one
    // the owner knows, and not the one the attempt tried to install.
    const loginWithOriginal = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
      failOnStatusCode: false,
    });
    expect(
      loginWithOriginal.status(),
      "a refused change must leave the password it failed to replace working"
    ).toBeLessThan(400);

    const loginWithRefused = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password: newPassword },
      failOnStatusCode: false,
    });
    expect(
      loginWithRefused.status(),
      "a refused change must not install the password it was offered"
    ).toBeGreaterThanOrEqual(400);

    // The same form, with the current password the account actually has.
    await page.locator('[data-test="current-password-input"]').fill(password);
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/password") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="change-password-button"]').click();

    const response = await responsePromise;

    expect(response.status()).toBeLessThan(400);

    // Verify success notification is shown
    await expectToast(page, { type: "success" });

    // And the rotation really happened: the old password is dead, the new one
    // opens the account.
    const loginWithRotatedOut = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
      failOnStatusCode: false,
    });
    expect(
      loginWithRotatedOut.status(),
      "after the change the previous password must stop working"
    ).toBeGreaterThanOrEqual(400);

    const loginWithNew = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password: newPassword },
      failOnStatusCode: false,
    });
    expect(loginWithNew.status(), "after the change the new password must work").toBeLessThan(400);
  });

  test("a session cannot change the password without presenting the current one", async ({
    request,
  }) => {
    // The form always sends the field, so only a caller bypassing it can leave
    // it out — which is exactly the caller this guard exists for. The endpoint
    // has to refuse the body itself rather than trust the page that built it.
    const timestamp = Date.now();
    const email = `test-settings-nocurrent-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const newPassword = `NewPassword456!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

    const loginResponse = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
    });
    expect(loginResponse.status()).toBeLessThan(400);
    const session = (await loginResponse.json()) as { access_token: string };

    const changeResponse = await request.post(`${API_BASE_URL}/auth/password`, {
      headers: { Authorization: `Bearer ${session.access_token}` },
      data: { new_password: newPassword },
      failOnStatusCode: false,
    });
    expect(
      changeResponse.status(),
      "a change with no current password must not be accepted"
    ).toBeGreaterThanOrEqual(400);

    // And nothing was written: the account still answers to the password it had.
    const loginAfter = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
      failOnStatusCode: false,
    });
    expect(loginAfter.status(), "the refused call must leave the password alone").toBeLessThan(400);

    const loginWithAttempted = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password: newPassword },
      failOnStatusCode: false,
    });
    expect(
      loginWithAttempted.status(),
      "the password the refused call carried must not have been installed"
    ).toBeGreaterThanOrEqual(400);
  });

  test("an impatient second click cannot reach the API", async ({ page, request }) => {
    // Changing a password costs two deliberate hashes in series — one to check
    // the password being replaced, one to store the password replacing it — so
    // the form sits there looking untouched for the best part of a second, and
    // a second click is the ordinary human response to that.
    //
    // Left to travel, that second request arrives after the change and presents
    // a current password the account no longer has: the user is told the change
    // succeeded and that their current password is wrong, in that order, about
    // the same click.
    const timestamp = Date.now();
    const email = `test-settings-doubleclick-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const newPassword = `QuiltLanternHarbour${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    await page.goto("/settings");
    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Counted on the requests that leave, not on the responses that come back:
    // a second request refused by the API would still be the defect, since what
    // it is refused for is a password that is no longer current.
    let attempts = 0;
    page.on("request", (request_) => {
      if (request_.url().includes("/api/v1/auth/password") && request_.method() === "POST") {
        attempts += 1;
      }
    });

    await page.locator('[data-test="current-password-input"]').fill(password);
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);

    // Two clicks in the same breath, the second one delivered as a hand
    // delivers it: to the place on the screen where the button was, without
    // asking anything about what is there now. Aiming the second click at the
    // element would be waiting for the button to be pressable again — the very
    // state that is supposed to stop it — and this test would then pass for a
    // page that had nothing stopping it at all. It would not even find the same
    // element: a button that is disabled and carries a reason is rendered
    // through a different branch, so the node under the pointer is replaced
    // mid-press.
    const submit = page.locator('[data-test="change-password-button"]');
    await expect(submit, "the form has to be pressable before it is pressed twice").toBeEnabled();
    // Scrolled into view before its position is read: clicking it does that on
    // its own, and a position read beforehand would point at whatever the
    // scroll moved into that spot — a second click that lands nowhere passes
    // this test for the wrong reason.
    await submit.scrollIntoViewIfNeeded();
    const where = await submit.boundingBox();
    expect(where, "the button must be on screen to be clicked twice").not.toBeNull();
    await submit.click();
    await page.mouse.click(where!.x + where!.width / 2, where!.y + where!.height / 2);

    await expect(page).toHaveURL(/\/login/, { timeout: 20000 });
    expect(attempts, "one click's worth of change, whatever the hand did").toBe(1);

    // And the one that did leave was the real one.
    const loginWithNew = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password: newPassword },
      failOnStatusCode: false,
    });
    expect(loginWithNew.status(), "the password the form carried must be the one set").toBeLessThan(
      400
    );
  });

  test("changing the password ends this session too", async ({ page, request }) => {
    // The change signs every session out, including the one that asked for it
    // (`store_new_password`, api/src/handlers/auth.rs). Left where they were,
    // the user found that out from whichever request happened to fail next —
    // a settings page that suddenly could not load anything, with no
    // explanation and no obvious way back.
    const timestamp = Date.now();
    const email = `test-settings-signout-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;
    const newPassword = `MarbleCanyonFerry${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    await page.goto("/settings");
    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    const changed = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/password") && response.request().method() === "POST",
      { timeout: 20000 }
    );
    await page.locator('[data-test="current-password-input"]').fill(password);
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);
    await page.locator('[data-test="change-password-button"]').click();
    expect((await changed).status()).toBeLessThan(400);

    // Taken to the form they now have to use, and told why they are there.
    await expect(page).toHaveURL(/\/login/, { timeout: 20000 });
    await expectToast(page, { type: "success", contains: "signed out" });

    // Not merely redirected: the session is gone. Asking for the page they were
    // on has to send them back here, or the tokens were still on this browser
    // and only the routing pretended otherwise.
    await page.goto("/settings");
    await expect(page).toHaveURL(/\/login/, { timeout: 15000 });
    await expect(page.locator('[data-test="change-password-form"]')).toHaveCount(0);
  });

  test("the submit button says what is missing while it refuses to be pressed", async ({
    page,
    request,
  }) => {
    // The button is gated on the form being valid and touched, and the
    // validation library only writes a message under a field the user has
    // already visited. Fill in the two new passwords and never touch the
    // current one and there is nothing anywhere: a grey button, no error, and a
    // native `disabled` swallows the click, so pressing it does not even
    // produce a late explanation.
    const timestamp = Date.now();
    const email = `test-settings-hint-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    await page.goto("/settings");
    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    const newPassword = `QuiltLanternHarbour${timestamp}`;
    await page.locator('[data-test="new-password-input"]').fill(newPassword);
    await page.locator('[data-test="confirm-password-input"]').fill(newPassword);

    const submit = page.locator('[data-test="change-password-button"]');
    await expect(submit).toBeDisabled();
    await expect(
      page.locator('[data-test="change-password-card"] [data-test="input-error"]'),
      "the field left alone raises nothing on its own — which is the whole problem"
    ).toHaveCount(0);

    // So the reason has to come from the button itself, on hover and on focus.
    await submit.hover();
    const hint = page.getByRole("tooltip");
    await expect(hint).toBeVisible({ timeout: 10000 });
    await expect(hint).toContainText(/current password/i);
  });

  test("should display language selector", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-lang-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify language selector is visible
    await expect(page.locator('[data-test="language-select"]')).toBeVisible();
  });

  test("should change language and verify UI text changes", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-lang-change-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="user-info-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Verify language selector is visible and currently set to English
    const languageSelect = page.locator('[data-test="language-select"]');
    await expect(languageSelect).toBeVisible();

    // Language select is currently disabled (only English is supported)
    await expect(languageSelect).toBeDisabled();

    // Verify the selected option shows English
    // Hook0Select renders a native <select> — check its current value
    await expect(languageSelect).toHaveValue("en");

    // Verify UI text is in English by checking known page headings
    await expect(page.locator('[data-test="change-password-card"]')).toBeVisible();
    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();

    // Verify theme selector is also present (sibling preference control)
    await expect(page.locator('[data-test="theme-select"]')).toBeVisible();
  });

  test("should show error when passwords do not match", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-settings-mismatch-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="change-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // The current password comes first, and not as a formality: the mismatch
    // rule compares two fields, so it is only reached once every field on the
    // form parses. Leaving this one empty disables the button on its own, and
    // the assertion below would then hold without the mismatch ever being the
    // reason.
    await page.locator('[data-test="current-password-input"]').fill(password);

    // Fill mismatching passwords
    await page.locator('[data-test="new-password-input"]').fill("NewPassword123!");
    await page.locator('[data-test="confirm-password-input"]').fill("DifferentPassword456!");

    // Blur to trigger cross-field validation (Zod refine evaluates after both fields touched)
    await page.locator('[data-test="confirm-password-input"]').blur();

    // VeeValidate/Zod cross-field refine renders inline error and disables submit button
    // Verify the submit button is disabled (passwords don't match → meta.valid is false)
    await expect(page.locator('[data-test="change-password-button"]')).toBeDisabled({
      timeout: 10000,
    });

    // Verify the validation error message appears inline
    await expect(page.locator('[data-test="input-error"]')).toBeVisible({
      timeout: 10000,
    });
  });

  test("should show not implemented error when trying to delete account", async ({
    page,
    request,
  }) => {
    // Note: The delete account feature is not implemented yet.
    // The frontend shows "Not implemented yet" error when clicking delete.
    // This test verifies the error notification appears.

    // Setup - create a test user
    const timestamp = Date.now();
    const email = `test-delete-account-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Click delete - opens Hook0Dialog confirmation
    await page.locator('[data-test="delete-account-button"]').click();

    // Wait for confirmation dialog and click confirm
    const confirmButton = page.locator('[data-test="dialog-confirm-button"]');
    await expect(confirmButton).toBeVisible({ timeout: 5000 });
    await confirmButton.click();

    // Verify error notification is shown (not implemented feature) or success toast
    await expectToast(page);

    // User should still be on settings page (not logged out)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });
  });

  test("should cancel account deletion when dialog is dismissed", async ({ page, request }) => {
    // Setup
    const timestamp = Date.now();
    const email = `test-cancel-delete-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await verifyEmailViaMailpit(request, email);

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

    // Navigate to settings
    await page.goto("/settings");

    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible({
      timeout: 10000,
    });

    // Click delete button to open Hook0Dialog confirmation
    await page.locator('[data-test="delete-account-button"]').click();

    // Wait for the dialog to appear, then click cancel
    await expect(page.locator('[data-test="dialog-cancel-button"]')).toBeVisible({ timeout: 5000 });
    await page.locator('[data-test="dialog-cancel-button"]').click();

    // Should still be on settings page (not logged out)
    await expect(page).toHaveURL(/\/settings/, {
      timeout: 5000,
    });

    // Delete account card should still be visible
    await expect(page.locator('[data-test="delete-account-card"]')).toBeVisible();
  });
});

test.describe("Password Reset Flow", () => {
  test("should display reset password form", async ({ page }) => {
    await page.goto("/begin-reset-password");

    // Verify form is visible
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="reset-password-email-input"]')).toBeVisible();
    await expect(page.locator('[data-test="reset-password-submit-button"]')).toBeVisible();
    await expect(page.locator('[data-test="reset-password-back-link"]')).toBeVisible();
  });

  test("should navigate back to login when clicking back link", async ({ page }) => {
    await page.goto("/begin-reset-password");

    await expect(page.locator('[data-test="reset-password-back-link"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="reset-password-back-link"]').click();

    await expect(page).toHaveURL(/\/login/);
    await expect(page.locator('[data-test="login-form"]')).toBeVisible();
  });

  test("should submit reset password request and verify API response", async ({ page }) => {
    const timestamp = Date.now();
    const email = `test-reset-${timestamp}@hook0.local`;

    await page.goto("/begin-reset-password");

    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Nothing is being confirmed yet, so the confirmation is not on screen. Read
    // together with the assertion at the end, this is what makes that one worth
    // making: a card that were always mounted would satisfy it without the
    // request ever having been made.
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);

    // Step 1: Fill email
    await page.locator('[data-test="reset-password-email-input"]').fill(email);

    // Step 2: Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="reset-password-submit-button"]').click();

    const response = await responsePromise;

    // Step 3: an address nobody registered gets the same answer a real one
    // gets. "Below 500" was true of the 401 that used to name unknown
    // addresses, so it has to be the exact status a known address receives.
    expect(response.status(), "an unknown address must get the ordinary answer").toBe(204);

    // And the page says the same thing it says to someone whose address does
    // exist: the confirmation card, which is what a reader would compare
    // against. This page has never raised a toast on success — asserting one
    // could only ever catch the error toast that used to give the answer away.
    await expect(page.locator('[data-test="reset-password-success"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator("[data-sonner-toast]")).toHaveCount(0);
  });

  test("an address the API refuses is shown on the field, never as a pacing warning", async ({
    page,
  }) => {
    // The page keeps exactly one failure to itself — anything that would say
    // whether an address belongs to an account — and shows two that say nothing
    // about the account: the caller going too fast, and the address itself
    // being unusable. They must not be shown as each other. A typo reported as
    // a pacing problem tells the reader to wait for a mail that will never
    // come; a pacing refusal reported on the field blames an address that is
    // fine.
    //
    // The refusal is a real one, not a simulated one: the address is longer
    // than the request struct accepts, so the API turns it down on its own,
    // before it looks anything up — which is why showing it gives nothing away.
    //
    // Planted rather than typed, and that is the whole reason this test is
    // worth its length. The field now carries `maxlength` and `type="email"`,
    // and between them no keyboard can produce an address this endpoint
    // refuses: typing is truncated at the bound, and every address the API's
    // own validator turns down is one the browser marks invalid first, so the
    // form never submits. Handing the field a value it would not have accepted
    // from a keyboard is what puts the page back in front of the answer it has
    // to handle — the state it lands in the day either bound is relaxed here or
    // tightened there. `maxlength` does not apply to a value a script last
    // wrote, so these characters do reach the API.
    const overLongAddress = `${"a".repeat(95)}@hook0.local`;

    await page.goto("/begin-reset-password");
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="reset-password-email-input"]').evaluate((field, address) => {
      (field as HTMLInputElement).value = address as string;
      field.dispatchEvent(new Event("input", { bubbles: true }));
    }, overLongAddress);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-submit-button"]').click();

    const response = await responsePromise;
    expect(
      response.status(),
      "the API must refuse an address longer than the field allows"
    ).toBeGreaterThanOrEqual(400);
    expect(response.status(), "and refuse it for the input, not for pacing").not.toBe(429);

    // Refused, and said under the field the reader has to fix — naming the
    // bound, because "not accepted" without it leaves them retyping the same
    // address. Reword freely; keep the reader told what the limit is.
    const fieldError = page.locator('[data-test="input-error"]');
    await expect(fieldError).toBeVisible({ timeout: 10000 });
    await expect(fieldError).toContainText(/at most \d+ characters/i);

    // And the two things it is not. The pacing warning belongs to the 429 alone
    // — this refusal never was one — and the confirmation would be an outright
    // lie: nothing was sent, and the reader would be waiting on it.
    await expect(page.locator('[data-test="reset-password-rate-limited"]')).toHaveCount(0);
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);
  });

  test("should complete password reset flow with valid token and login with new password", async ({
    page,
    request,
  }) => {
    // Step 1: Create a verified user first
    const timestamp = Date.now();
    const email = `test-reset-complete-${timestamp}@hook0.local`;
    const originalPassword = `OriginalPass123!${timestamp}`;
    const newPassword = `NewSecurePass456!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: {
        email,
        first_name: "Reset",
        last_name: "Tester",
        password: originalPassword,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Step 2: Initiate password reset via API
    const beginResetResponse = await request.post(`${API_BASE_URL}/auth/begin-reset-password`, {
      data: { email },
    });
    expect(beginResetResponse.status()).toBeLessThan(400);

    // Step 3: Get the password reset token from Mailpit
    const resetToken = await getPasswordResetTokenFromMailpit(request, email, 20000);
    expect(resetToken).toBeTruthy();

    // Step 4: Navigate to the reset password page with token
    await page.goto(`/reset-password?token=${resetToken}`);

    // Verify form is visible
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Step 5: Fill in new password
    await page.locator('[data-test="reset-password-new-password-input"]').fill(newPassword);
    await page.locator('[data-test="reset-password-confirm-password-input"]').fill(newPassword);

    // Step 6: Submit and wait for API response
    const resetResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="reset-password-submit-button"]').click();

    const resetResponse = await resetResponsePromise;
    expect(resetResponse.status()).toBeLessThan(400);

    // Step 7: Should redirect to login page
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Step 8: Login with new password
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(newPassword);

    const loginResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/login") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();

    const loginResponse = await loginResponsePromise;
    expect(loginResponse.status()).toBeLessThan(400);

    // Step 9: Verify login succeeded - should redirect to dashboard/home
    await expect(page).toHaveURL(/\/(organizations|tutorial)/, { timeout: 15000 });
  });
});
