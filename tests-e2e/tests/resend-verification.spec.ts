import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { API_BASE_URL } from "../fixtures/email-verification";

const MAILPIT_URL = process.env.MAILPIT_URL || "http://localhost:8025";

/**
 * Every verification email currently sitting in Mailpit for an address, newest
 * first. Used to tell "a fresh link was really sent" from "the endpoint merely
 * answered 204" — the resend endpoint answers identically either way, so the
 * mailbox is the only place the difference is observable.
 */
async function verificationEmails(
  request: APIRequestContext,
  email: string
): Promise<Array<{ html: string }>> {
  const search = await request.get(
    `${MAILPIT_URL}/api/v1/search?query=to:${encodeURIComponent(email)}`,
    { timeout: 5000 }
  );
  if (!search.ok()) return [];
  const messages = ((await search.json()).messages ?? []) as Array<{ ID: string }>;

  const found: Array<{ html: string }> = [];
  for (const message of messages) {
    const detail = await request.get(`${MAILPIT_URL}/api/v1/message/${message.ID}`, {
      timeout: 5000,
    });
    if (!detail.ok()) continue;
    const full = (await detail.json()) as { HTML?: string; Text?: string };
    const html = `${full.HTML ?? ""}${full.Text ?? ""}`;
    if (html.includes("verify-email")) found.push({ html });
  }
  return found;
}

/** Wait until at least `count` verification emails reached `email`. */
async function waitForVerificationEmails(
  request: APIRequestContext,
  email: string,
  count: number,
  maxWaitMs = 20000
): Promise<Array<{ html: string }>> {
  const startedAt = Date.now();
  let latest: Array<{ html: string }> = [];
  while (Date.now() - startedAt < maxWaitMs) {
    latest = await verificationEmails(request, email);
    if (latest.length >= count) return latest;
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(
    `expected at least ${count} verification email(s) for ${email}, saw ${latest.length}`
  );
}

/** Pull the verification token out of a rendered email body. */
function tokenFrom(html: string): string {
  const match = html.replace(/[\r\n]+/g, "").match(/verify-email\?token=([A-Za-z0-9_\-+/=%]+)/i);
  expect(match, "the email must carry a verification link").toBeTruthy();
  return decodeURIComponent(match![1]);
}

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

  test("the resent link actually arrives and verifies the account", async ({ request }) => {
    // The reason the feature exists: someone lost the first email. A 204 proves
    // nothing on its own — what matters is that a second, working link lands in
    // the mailbox and completes the account.
    const timestamp = Date.now();
    const email = `test-resend-delivers-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // The signup email itself is the first one; wait for it so the resend is
    // measured against a known baseline.
    await waitForVerificationEmails(request, email, 1);

    const resendResponse = await request.post(`${API_BASE_URL}/auth/resend-verification-email`, {
      data: { email },
    });
    expect(resendResponse.status()).toBe(204);

    const emails = await waitForVerificationEmails(request, email, 2);

    // The freshly delivered link must verify the account for real.
    const verifyResponse = await request.post(`${API_BASE_URL}/auth/verify-email`, {
      data: { token: tokenFrom(emails[0].html) },
      failOnStatusCode: false,
    });
    expect(verifyResponse.status(), "the resent link must complete verification").toBeLessThan(400);

    // And the account is genuinely usable afterwards — the point of recovering.
    const loginResponse = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
      failOnStatusCode: false,
    });
    expect(loginResponse.status(), "the recovered account must be able to log in").toBeLessThan(
      400
    );
  });

  test("the cooldown is enforced server-side, not just by the disabled button", async ({
    request,
  }) => {
    // The countdown in the UI is a courtesy; anyone can call the endpoint
    // directly. The control that actually prevents mailbox flooding lives in the
    // database, so drive the API straight past the UI and check the mailbox.
    const timestamp = Date.now();
    const email = `test-resend-server-cooldown-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await waitForVerificationEmails(request, email, 1);

    // Five back-to-back calls, no UI involved.
    for (let attempt = 0; attempt < 5; attempt += 1) {
      const response = await request.post(`${API_BASE_URL}/auth/resend-verification-email`, {
        data: { email },
      });
      // Always the same answer — the endpoint never discloses whether it sent.
      expect(response.status()).toBe(204);
    }

    await waitForVerificationEmails(request, email, 2);
    // Give any wrongly-permitted extra send time to land before counting.
    await new Promise((resolve) => setTimeout(resolve, 3000));

    const delivered = await verificationEmails(request, email);
    expect(
      delivered.length,
      "within one cooldown window only a single resend may actually be sent"
    ).toBe(2);
  });
});
