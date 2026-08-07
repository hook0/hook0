import { test, expect } from "@playwright/test";
import {
  verifyEmailViaMailpit,
  getPasswordResetTokenFromMailpit,
  API_BASE_URL,
} from "../fixtures/email-verification";
import { expectToast } from "../fixtures/test-setup";

/**
 * The password policy, walked as a user walks it.
 *
 * The suite used to exercise only passwords that pass, so nothing here was
 * covered: not the reported vulnerability (typing your own email address as
 * your password), and not what the forms do when the API refuses. The
 * reset-password form in particular used to be unmounted by any rejection,
 * which cost the user their reset link, and no test could see it.
 */
test.describe("Password policy", () => {
  test.describe("Register page", () => {
    test("states the rules before anything is typed", async ({ page }) => {
      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      await expect(
        page.getByText(/not built from your email address or your name/i)
      ).toBeVisible();
    });

    /**
     * The reported vulnerability, from the user's side: the account's own email
     * address as its password. The form knows both, so it must refuse without a
     * round trip.
     */
    test("refuses the account email address as a password, without asking the API", async ({
      page,
    }) => {
      const email = `test-policy-${Date.now()}@hook0.local`;

      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      let registerCalled = false;
      page.on("request", (request) => {
        if (request.url().includes("/api/v1/register") && request.method() === "POST") {
          registerCalled = true;
        }
      });

      await page.locator('[data-test="register-email-input"]').fill(email);
      await page.locator('[data-test="register-firstname-input"]').fill("Policy");
      await page.locator('[data-test="register-lastname-input"]').fill("Tester");
      await page.locator('[data-test="register-password-input"]').fill(email);
      await page.locator('[data-test="register-submit-button"]').click();

      await expect(
        page.getByText(/must not be built from your email address/i)
      ).toBeVisible({ timeout: 10000 });

      await expect(page).toHaveURL(/\/register/);
      expect(registerCalled).toBe(false);
    });

    /**
     * A reason only the server knows — the blocklist does not ship to the
     * browser — must still reach the user as something readable.
     */
    test("surfaces a common password refused by the API", async ({ page }) => {
      const email = `test-policy-common-${Date.now()}@hook0.local`;

      await page.goto("/register");
      await expect(page.locator('[data-test="register-form"]')).toBeVisible({
        timeout: 10000,
      });

      await page.locator('[data-test="register-email-input"]').fill(email);
      await page.locator('[data-test="register-firstname-input"]').fill("Policy");
      await page.locator('[data-test="register-lastname-input"]').fill("Tester");
      await page.locator('[data-test="register-password-input"]').fill("2026letmein!");

      const responsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/register") && response.request().method() === "POST",
        { timeout: 15000 }
      );

      await page.locator('[data-test="register-submit-button"]').click();

      const response = await responsePromise;
      expect(response.status()).toBe(400);
      expect((await response.json()).id).toBe("PasswordTooCommon");

      await expectToast(page, { contains: "common" });
      await expect(page).toHaveURL(/\/register/);
    });
  });

  test.describe("Reset password page", () => {
    /**
     * The regression this suite exists to catch: a rejection here used to
     * unmount the form, so a weak password cost the user their reset link. The
     * form must survive the refusal and accept a second attempt.
     */
    test("keeps the form usable after a refusal, and accepts the next password", async ({
      page,
      request,
    }) => {
      const timestamp = Date.now();
      const email = `test-policy-reset-${timestamp}@hook0.local`;
      const originalPassword = `OriginalPass123!${timestamp}`;
      const acceptedPassword = `QuiltLanternHarbour${timestamp}`;

      const registerResponse = await request.post(`${API_BASE_URL}/register`, {
        data: {
          email,
          first_name: "Policy",
          last_name: "Tester",
          password: originalPassword,
        },
      });
      expect(registerResponse.status()).toBeLessThan(400);

      await verifyEmailViaMailpit(request, email);

      const beginResetResponse = await request.post(`${API_BASE_URL}/auth/begin-reset-password`, {
        data: { email },
      });
      expect(beginResetResponse.status()).toBeLessThan(400);

      const resetToken = await getPasswordResetTokenFromMailpit(request, email, 20000);
      expect(resetToken).toBeTruthy();

      await page.goto(`/reset-password?token=${resetToken}`);
      await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({
        timeout: 10000,
      });

      // First attempt: the account's own address. The reset page never learns
      // the email — only the token — so this one can only be caught by the API.
      await page.locator('[data-test="reset-password-new-password-input"]').fill(email);
      await page.locator('[data-test="reset-password-confirm-password-input"]').fill(email);

      const refusalPromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/auth/reset-password") &&
          response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="reset-password-submit-button"]').click();

      const refusal = await refusalPromise;
      expect(refusal.status()).toBe(400);
      expect((await refusal.json()).id).toBe("PasswordSimilarToEmail");

      // The form is still there — this is the whole point of the test.
      await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible();
      await expect(
        page.locator('[data-test="reset-password-new-password-input"]')
      ).toBeVisible();

      // Second attempt, with a password the policy accepts.
      await page.locator('[data-test="reset-password-new-password-input"]').fill(acceptedPassword);
      await page
        .locator('[data-test="reset-password-confirm-password-input"]')
        .fill(acceptedPassword);

      const acceptedPromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/auth/reset-password") &&
          response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="reset-password-submit-button"]').click();

      const accepted = await acceptedPromise;
      expect(accepted.status()).toBeLessThan(400);

      await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

      // The reset really took effect: the new password logs in.
      await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
      await page.locator('[data-test="login-email-input"]').fill(email);
      await page.locator('[data-test="login-password-input"]').fill(acceptedPassword);

      const loginPromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/auth/login") && response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="login-submit-button"]').click();

      expect((await loginPromise).status()).toBeLessThan(400);
    });
  });
});
