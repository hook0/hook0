import { test, expect } from "@playwright/test";

/**
 * The forgot-password page reports almost everything as success on purpose:
 * whether an address belongs to an account is exactly what the API stopped
 * answering, so any difference the page draws from a failure hands that oracle
 * back. The exception is a request that never got as far as a decision about
 * the address — it timed out, it failed in transport, or the call that claims
 * the send failed. Reporting success there promises a mail that was never
 * minted, on the one route back into an account, and withholding it protects
 * nobody: none of those outcomes depends on whether an account exists.
 *
 * This is the only place that branch can be checked. The predicate behind it is
 * unit-tested, but the wiring between it and the page is not: the components
 * are out of reach of this project's unit runner, so deleting the branch from
 * the page would leave every other test green.
 */
test.describe("Forgot password page, request never fulfilled", () => {
  test("says so instead of claiming a mail is on its way", async ({ page }) => {
    await page.goto("/begin-reset-password");
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="reset-password-email-input"]')
      .fill(`unfulfilled-${Date.now()}@hook0.local`);

    // Neither outcome is on screen yet. Read with the assertions below, this is
    // what keeps them from being satisfied by a page rendering one of them
    // unconditionally.
    await expect(page.locator('[data-test="reset-password-request-failed"]')).toHaveCount(0);
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);

    // The request never reaches the API.
    await page.route("**/api/v1/auth/begin-reset-password", (route) =>
      route.abort("connectionfailed")
    );
    await page.locator('[data-test="reset-password-submit-button"]').click();

    // Both faces are needed: a page showing the warning *and* the confirmation
    // would satisfy either one alone while telling the reader two contradictory
    // things — that nothing was sent, and that something was.
    await expect(page.locator('[data-test="reset-password-request-failed"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);

    // And the page is not stranded: with the connection back, the same address
    // goes through and the confirmation replaces the warning. Showing the
    // failure is only right if the reader can still act on it.
    await page.unrouteAll({ behavior: "ignoreErrors" });
    const answered = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect((await answered).status()).toBe(204);
    await expect(page.locator('[data-test="reset-password-success"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="reset-password-request-failed"]')).toHaveCount(0);
  });
});
