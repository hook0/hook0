import { test, expect, type Page } from "@playwright/test";
import { API_BASE_URL } from "../fixtures/email-verification";

/**
 * Resend-verification-email E2E tests for Hook0.
 *
 * Exercises the real check-email flow end to end (no mocks): the real resend
 * endpoint answers and the UI reacts. Follows the Three-Step Verification
 * Pattern used across the auth suite (act -> waitForResponse -> assert status +
 * UI state).
 *
 * The signup address is carried to /check-email through History API state, never
 * the URL (so analytics never records it). `landOnCheckEmailWith` reproduces
 * that: it seeds window.history.state and reloads, which also proves the resend
 * flow is refresh-safe from state alone.
 */

/** Regex for the disabled button's cooldown countdown label. */
const COOLDOWN_LABEL = /Resend available in \d+s/;
/** URL fragment identifying the real resend endpoint. */
const RESEND_ENDPOINT = "/auth/resend-verification-email";

/**
 * Land on /check-email carrying `email` exactly like the signup redirect does:
 * via History API state (never a query param). Reloading forces the page to read
 * the address back from state, proving it is refresh-safe. Also asserts the URL
 * never carries the address.
 */
async function landOnCheckEmailWith(page: Page, email: string): Promise<void> {
  await page.goto("/check-email");
  await page.evaluate((addr) => {
    window.history.replaceState({ ...window.history.state, email: addr }, "", "/check-email");
  }, email);
  await page.reload();
  await expect(page.locator('[data-test="check-email-page"]')).toBeVisible({ timeout: 10000 });
  // The address lives only in History API state — never in the URL.
  expect(new URL(page.url()).search).toBe("");
}

test.describe("Resend verification email", () => {
  test("resends from the check-email page and enters cooldown", async ({ page, request }) => {
    // Register a user via the API and leave the email unverified, so the resend
    // endpoint has a real unverified account to act on.
    const timestamp = Date.now();
    const email = `test-resend-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    await landOnCheckEmailWith(page, email);

    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeEnabled();

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await resendButton.click();

    const response = await responsePromise;

    // 204 No Content (always the same body/status either way, anti-enumeration),
    // and the button entered its cooldown: disabled with a countdown label.
    expect(response.status()).toBe(204);
    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(COOLDOWN_LABEL, { timeout: 10000 });
  });

  test("resend for an unknown email behaves identically (anti-enumeration)", async ({ page }) => {
    // No account is ever registered for this address. The endpoint must answer
    // the same 204 and the UI must behave exactly as for a real account: the
    // button enters cooldown, no error surfaces, nothing reveals the address is
    // unknown.
    const email = `never-registered-${Date.now()}@hook0.local`;

    await landOnCheckEmailWith(page, email);

    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeEnabled();

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await resendButton.click();

    const response = await responsePromise;

    // Identical 204 to the known-account case (anti-enumeration): the status
    // never discloses whether the address is registered.
    expect(response.status()).toBe(204);

    // Identical UI: cooldown starts, page stays put, no error toast is shown.
    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(COOLDOWN_LABEL, { timeout: 10000 });
    await expect(page.locator('[data-test="check-email-page"]')).toBeVisible();
    await expect(page.locator('[data-sonner-toast][data-type="error"]')).toHaveCount(0);
  });

  test("a second resend within the cooldown window is throttled", async ({ page, request }) => {
    // A real unverified account so the first resend actually sends.
    const timestamp = Date.now();
    const email = `test-resend-throttle-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    await landOnCheckEmailWith(page, email);

    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeEnabled();

    // First resend: succeeds and starts the cooldown.
    const firstResponse = page.waitForResponse(
      (response) =>
        response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await resendButton.click();
    expect((await firstResponse).status()).toBe(204);

    // The button is now disabled for the whole cooldown window.
    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(COOLDOWN_LABEL, { timeout: 10000 });

    // Attempt a second resend during the cooldown: because the button is
    // disabled, no second POST is issued. Prove it front-observably — no resend
    // request fires within a short window after a forced click.
    const secondPostFired = page
      .waitForResponse(
        (response) =>
          response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
        { timeout: 2000 }
      )
      .then(() => true)
      .catch(() => false);

    await resendButton.click({ force: true }).catch(() => undefined);

    expect(await secondPostFired).toBe(false);
    // Still disabled and still counting down: the throttle holds.
    await expect(resendButton).toBeDisabled();
    await expect(resendButton).toContainText(COOLDOWN_LABEL);
  });
});
