import { test, expect } from "@playwright/test";
import { getVerificationTokenFromMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Auto-login-after-email-verification E2E guard.
 *
 * The headline UX of the auto-login feature: a freshly registered (unverified)
 * user opens the verification link and lands authenticated on the onboarding
 * wizard / dashboard WITHOUT ever touching the login form. This exercises the
 * real browser path VerifyEmail.vue -> authStore.setSession -> navigateAfterAuth.
 *
 * The verification token is pulled straight from the email in Mailpit (the same
 * mechanism the shared fixtures use, available both locally and in CI) and is
 * consumed by the BROWSER — no API-side shortcut, no mocks — so the assertion
 * proves the end-to-end auto-login redirect.
 */
test.describe("Email verification auto-login", () => {
  test("verifies email in the browser and lands authenticated without a manual login", async ({
    page,
    request,
  }) => {
    // Register a brand-new, unverified user via API.
    const timestamp = Date.now();
    const email = `test-verify-autologin-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Pull the verification token from the email (Mailpit) WITHOUT consuming it,
    // so the BROWSER is the one that verifies + auto-logs-in.
    const token = await getVerificationTokenFromMailpit(request, email);
    expect(token.length).toBeGreaterThan(0);

    // Open the verification link in the browser — the real user path. We never
    // visit /login and never fill a login form.
    await page.goto(`/verify-email?token=${encodeURIComponent(token)}`);

    // Auto-login lands the user straight on the wizard/dashboard, NOT the login
    // form and NOT the verify-email "Back to login" error card.
    await expect(page).toHaveURL(/\/tutorial|\/organizations|\/dashboard/, {
      timeout: 15000,
    });
    await expect(page.locator('[data-test="verify-email-error"]')).toHaveCount(0);
    await expect(page.locator('[data-test="login-form"]')).toHaveCount(0);

    // Prove the session actually authenticates protected navigation: reloading
    // the protected landing stays put instead of bouncing to /login.
    await page.reload();
    await expect(page).not.toHaveURL(/\/login/, { timeout: 10000 });
  });

  test("drops the token from the address bar so it is never observed as a URL", async ({
    page,
    request,
  }) => {
    // The token in this URL now mints a session, and anything that watches URLs
    // watches it too: the analytics plugin tracks the full path of every
    // navigation and reports the previous one as the referrer of the next.
    // Whoever can read those URLs could replay the link, so it must not survive
    // in the address bar past the moment the page reads it.
    const timestamp = Date.now();
    const email = `test-verify-nourl-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const token = await getVerificationTokenFromMailpit(request, email);
    await page.goto(`/verify-email?token=${encodeURIComponent(token)}`);
    await expect(page).toHaveURL(/\/tutorial|\/organizations|\/dashboard/, { timeout: 15000 });

    // Neither the page the user landed on nor any entry left behind in session
    // history may still carry it.
    expect(page.url(), "the landing URL must not carry the token").not.toContain(token);
    await page.goBack();
    expect(page.url(), "no history entry may keep the token in its URL").not.toContain(token);
  });

  test("a verification link that was already used never opens a second session", async ({
    page,
    request,
  }) => {
    // This is the security boundary of the feature: the link is what mints the
    // session, so replaying one that has already been consumed — a forwarded
    // email, a prefetching mail client, a shared inbox — must not hand anybody
    // an account. The token is still well within its lifetime here, so
    // what is under test is the single unverified→verified transition, not
    // expiry.
    const timestamp = Date.now();
    const email = `test-verify-replay-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const token = await getVerificationTokenFromMailpit(request, email);

    // First use consumes the link and signs the user in.
    await page.goto(`/verify-email?token=${encodeURIComponent(token)}`);
    await expect(page).toHaveURL(/\/tutorial|\/organizations|\/dashboard/, { timeout: 15000 });

    // Drop the session so the replay is judged on the link alone, exactly as it
    // would be for someone else opening a forwarded email.
    await page.context().clearCookies();
    await page.evaluate(() => {
      window.localStorage.clear();
      window.sessionStorage.clear();
    });

    // Replay the very same link: the page must refuse it and must not land on
    // an authenticated area.
    await page.goto(`/verify-email?token=${encodeURIComponent(token)}`);
    await expect(page.locator('[data-test="verify-email-error"]')).toBeVisible({ timeout: 15000 });
    await expect(page).not.toHaveURL(/\/tutorial|\/organizations|\/dashboard/);

    // And the API itself refuses to mint a second session for that token, which
    // is the guarantee the UI is relying on.
    const replay = await request.post(`${API_BASE_URL}/auth/verify-email`, {
      data: { token },
      failOnStatusCode: false,
    });
    expect(replay.status(), "a consumed verification token must never return a session").not.toBe(
      201
    );
    expect(
      (await replay.json()).id,
      "the account is verified by then, so the user must be told to sign in rather than start over"
    ).toBe("AuthEmailAlreadyVerified");
  });
});
