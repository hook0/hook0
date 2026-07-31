import { test, expect } from "@playwright/test";
import { API_BASE_URL } from "../fixtures/email-verification";

/**
 * Resend-verification-email E2E test for Hook0.
 *
 * Exercises the real check-email flow end to end (no mocks): an unverified user
 * lands on /check-email?email=…, sees the "Resend verification email" button,
 * clicks it, the real resend endpoint answers, and the button enters its
 * client-side cooldown. Follows the Three-Step Verification Pattern used across
 * the auth suite (act → waitForResponse → assert status < 400 + UI state).
 */
test.describe("Resend verification email", () => {
  test("resends from the check-email page and enters cooldown", async ({ page, request }) => {
    // Register a user via the API and leave the email unverified (we never
    // verify it), so the resend endpoint has a real unverified account to act on.
    const timestamp = Date.now();
    const email = `test-resend-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Land on the check-email page WITH the ?email= query the signup redirect carries.
    await page.goto(`/check-email?email=${encodeURIComponent(email)}`);
    await expect(page.locator('[data-test="check-email-page"]')).toBeVisible({
      timeout: 10000,
    });

    // The resend button is visible and enabled before any click.
    const resendButton = page.locator('[data-test="resend-verification-email-button"]');
    await expect(resendButton).toBeVisible({ timeout: 10000 });
    await expect(resendButton).toBeEnabled();

    // Step 2: click and wait for the real resend endpoint to answer.
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/auth/resend-verification-email") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await resendButton.click();

    const response = await responsePromise;

    // Step 3: the endpoint accepted the request (204 No Content — always the
    // same body/status either way, anti-enumeration), and the button entered
    // its cooldown: disabled with a countdown label.
    expect(response.status()).toBeLessThan(400);

    await expect(resendButton).toBeDisabled({ timeout: 10000 });
    await expect(resendButton).toContainText(/Resend available in \d+s/, {
      timeout: 10000,
    });
  });
});
