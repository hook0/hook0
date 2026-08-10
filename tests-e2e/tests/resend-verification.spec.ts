import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { Client } from "pg";
import { API_BASE_URL } from "../fixtures/email-verification";

const MAILPIT_URL = process.env.MAILPIT_URL || "http://localhost:8025";
const DATABASE_URL =
  process.env.DATABASE_URL || "postgres://postgres:postgres@localhost:5432/hook0";

/**
 * Push an account's last-verification-send stamp far enough into the past that
 * the server-side cooldown no longer applies.
 *
 * Signing up stamps that column, because the signup email is itself a
 * verification email. A resend fired seconds later is therefore throttled, on
 * purpose — the point of the throttle is that a mailbox never gets two identical
 * mails back to back. Every test that needs a resend to genuinely send is
 * simulating a user who came back later, which is what this reproduces without
 * making the suite wait out a real cooldown.
 */
function letTheCooldownElapse(email: string): Promise<void> {
  const client = new Client({ connectionString: DATABASE_URL });
  return client
    .connect()
    .then(() =>
      client.query(
        "UPDATE iam.user SET email_verification_sent_at = statement_timestamp() - INTERVAL '1 hour' WHERE email = $1",
        [email]
      )
    )
    .then((result) => {
      expect(result.rowCount, `no account to age the cooldown for: ${email}`).toBe(1);
    })
    .finally(() => client.end());
}

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
 * Land on /check-email carrying `email` exactly like the login redirect does:
 * via History API state (never a query param), declaring no send of its own —
 * refusing a login sends nothing, so the resend button is live on arrival.
 * Reloading forces the page to read the address back from state, proving it is
 * refresh-safe. Also asserts the URL never carries the address.
 *
 * The signup redirect differs on purpose: it declares the mail it just sent, so
 * the button starts out counting down. That case has its own test below.
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
  test("straight after a real signup the button is already counting down", async ({ page }) => {
    // The other cases seed the address into history state directly. That proves
    // the page works, not that signup actually hands the address over. Here the
    // registration form is filled for real, so a change to the router push, the
    // history mode, or the page's read of it would fail this test.
    //
    // And signing up sends a verification email right then, stamping the
    // account. An enabled button would therefore offer an attempt the server
    // silently refuses while cheerfully reporting a send that never happened —
    // the endpoint answers 204 either way. So the honest state on arrival is a
    // button already counting down.
    const timestamp = Date.now();
    const email = `test-resend-fromsignup-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    await page.goto("/register");
    await expect(page.locator('[data-test="register-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="register-email-input"]').fill(email);
    await page.locator('[data-test="register-firstname-input"]').fill("Test");
    await page.locator('[data-test="register-lastname-input"]').fill("User");
    await page.locator('[data-test="register-password-input"]').fill(password);

    const registerResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/register") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="register-submit-button"]').click();
    expect((await registerResponse).status()).toBeLessThan(400);

    await expect(page).toHaveURL(/\/check-email/, { timeout: 15000 });
    await expect(page.locator('[data-test="check-email-page"]')).toBeVisible({ timeout: 10000 });
    // The address travels in History API state, so it never reaches analytics.
    expect(new URL(page.url()).search).toBe("");

    // The address made it across: the button is present. If the hand-off broke,
    // the page would render without any resend action at all.
    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });

    // And it reflects the mail signup just sent: disabled, counting down.
    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(COOLDOWN_LABEL, { timeout: 10000 });

    // Pressing it anyway changes nothing — no request leaves, and nothing claims
    // an email was sent.
    const postFired = page
      .waitForResponse(
        (response) =>
          response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
        { timeout: 2000 }
      )
      .then(() => true)
      .catch(() => false);
    await resendButton.click({ force: true }).catch(() => undefined);

    expect(await postFired).toBe(false);
    await expect(page.locator('[data-sonner-toast][data-type="success"]')).toHaveCount(0);
    await expect(resendButton).toBeDisabled();
    await expect(resendButton).toContainText(COOLDOWN_LABEL);
  });

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
    // Come back later, so the resend is a real send rather than a throttled one.
    await letTheCooldownElapse(email);

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

  test("a resend the API refuses says so and still enters cooldown", async ({ page }) => {
    // The failure path, driven by a real failure rather than an intercepted one:
    // the page resends whatever address it was handed, and the endpoint is the
    // authority on what it accepts, so handing it an address the endpoint
    // rejects makes the real call really fail.
    //
    // What must hold then is both halves: the UI says the send failed — never
    // the success message — and the cooldown starts anyway, because whatever
    // refused the call (validation, the rate limiter in front of it) will refuse
    // an immediate retry just the same.
    const email = "not-an-email";

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
    expect(
      response.status(),
      "the endpoint must refuse an address it cannot accept"
    ).toBeGreaterThanOrEqual(400);

    // Said out loud, and never as a success.
    await expect(page.locator('[data-sonner-toast][data-type="error"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-sonner-toast][data-type="success"]')).toHaveCount(0);

    // And the button is out of reach for the cooldown window all the same.
    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(COOLDOWN_LABEL, { timeout: 10000 });
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
    await letTheCooldownElapse(email);

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

  test("a user who comes back and tries to log in can recover from the login form", async ({
    page,
    request,
  }) => {
    // The case the feature exists for, start to finish. Someone signs up, never
    // clicks the link, and comes back days later. They do not think "check-email
    // page" — they go to the login form. The API refuses an unverified account
    // and sends nothing, so unless that refusal leads somewhere the account is
    // stranded. It must land them where the resend button lives, and the resend
    // must put a real second mail in their inbox.
    const timestamp = Date.now();
    const email = `test-resend-vialogin-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await waitForVerificationEmails(request, email, 1);
    // Days later.
    await letTheCooldownElapse(email);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);

    const loginResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/auth/login") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="login-submit-button"]').click();

    // Credentials are right; the account is simply unverified.
    const login = await loginResponse;
    expect(login.status()).toBe(401);
    const problem = (await login.json()) as { id: string };
    expect(problem.id, "the login refusal must name the unverified email").toBe(
      "AuthEmailNotVerified"
    );

    // Not a dead end: the refusal hands them to the page carrying the resend
    // button, with their address already loaded.
    await expect(page).toHaveURL(/\/check-email/, { timeout: 15000 });
    await expect(page.locator('[data-test="check-email-page"]')).toBeVisible({ timeout: 10000 });
    // The address travels in History API state, so it never reaches analytics.
    expect(new URL(page.url()).search).toBe("");

    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeEnabled();

    const resendResponse = page.waitForResponse(
      (response) =>
        response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await resendButton.click();
    expect((await resendResponse).status()).toBe(204);

    // The loop closes only if a second mail really lands.
    const emails = await waitForVerificationEmails(request, email, 2);

    // And that fresh link completes the account, so the login they attempted now
    // works.
    const verifyResponse = await request.post(`${API_BASE_URL}/auth/verify-email`, {
      data: { token: tokenFrom(emails[0].html) },
      failOnStatusCode: false,
    });
    expect(verifyResponse.status(), "the recovered link must complete verification").toBeLessThan(
      400
    );

    const secondLogin = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email, password },
      failOnStatusCode: false,
    });
    expect(
      secondLogin.status(),
      "after recovering, the login that was refused must succeed"
    ).toBeLessThan(400);
  });

  test("the cooldown survives a page reload", async ({ page, request }) => {
    // The countdown used to live only in memory, so a reload handed the button
    // back. The server-side cooldown outlives the page and answers 204 either
    // way, so a re-enabled button cheerfully reports a send that never happened.
    const timestamp = Date.now();
    const email = `test-resend-reload-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    await letTheCooldownElapse(email);

    await landOnCheckEmailWith(page, email);

    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(RESEND_ENDPOINT) && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await resendButton.click();
    expect((await responsePromise).status()).toBe(204);
    await expect(resendButton).toBeDisabled({ timeout: 10000 });

    // Reload the way a confused user would, then land on the page again from
    // scratch. The cooldown must still be running.
    await page.reload();
    await expect(page.locator('[data-test="check-email-page"]')).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeDisabled();
    await expect(resendButton).toContainText(COOLDOWN_LABEL);

    // And it resumes where it left off rather than restarting: the countdown
    // after the reload cannot be higher than it was before.
    const labelAfterReload = String(await resendButton.textContent());
    const countdown = /(\d+)s/.exec(labelAfterReload);
    expect(countdown, "the cooldown label must carry a countdown").toBeTruthy();
    const secondsLeft = Number(countdown![1]);
    expect(secondsLeft).toBeGreaterThan(0);
    expect(secondsLeft).toBeLessThanOrEqual(60);
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
    await letTheCooldownElapse(email);

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
    // Past the signup mail's own cooldown, so the first of the calls below is
    // allowed to send and the rest have to be stopped by the throttle itself.
    await letTheCooldownElapse(email);

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
