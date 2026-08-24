import { test, expect, type APIRequestContext } from "@playwright/test";
import { API_BASE_URL } from "../fixtures/email-verification";

/**
 * What the forgot-password page does when the API refuses the request for
 * pacing, driven against the real limiter — no interception, no faked response.
 *
 * The page has exactly two outcomes, and they are deliberately lopsided: a 429
 * is shown, and everything else — an address nobody registered, an SMTP outage,
 * a database that answered too late — is reported as success, because saying
 * anything else about those cases would answer the question the API stopped
 * answering. Showing the 429 gives nothing away in return: it comes from the
 * limiter keyed on the caller's own address, which behaves identically whether
 * or not an account exists. Staying silent about it would be the harmful
 * choice, promising a mail that was never going to be sent.
 *
 * This is the only place that branch can be checked. The predicate behind it is
 * unit-tested, but the wiring between it and the page is not: the components are
 * out of reach of this project's unit runner, so deleting the branch from the
 * page would leave every other test green.
 */

/** How many calls the test is willing to make before it gives up trying. */
const MOST_CALLS_WORTH_TRYING = 200;

/**
 * Ask for a reset link the way this machine does by default — carrying no
 * forwarded-for address, so the API bills the call to the source address the
 * browser will use in a moment. That sharing is the whole mechanism here: these
 * calls are what empties the allowance the page is about to run into.
 */
function beginResetFromThisMachine(request: APIRequestContext, email: string) {
  return request.post(`${API_BASE_URL}/auth/begin-reset-password`, {
    data: { email },
    failOnStatusCode: false,
  });
}

test.describe("Forgot password page @rate-limit", () => {
  test("says when the request was refused for pacing, and claims nothing was sent", async ({
    page,
    request,
  }) => {
    // The form is filled first, while requests are still being accepted. What
    // follows has to hold from the moment the allowance runs out until the
    // click, and doing the typing up front leaves nothing in that window.
    await page.goto("/begin-reset-password");
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="reset-password-email-input"]')
      .fill(`rate-limited-${Date.now()}@hook0.local`);

    // Nothing is refused yet, so neither outcome is on screen. Read with the
    // assertions at the end, this is what keeps them from being satisfied by a
    // page that renders one of them unconditionally.
    await expect(page.locator('[data-test="reset-password-rate-limited"]')).toHaveCount(0);
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);

    // Spend the allowance. Every address is one nobody registered, so this puts
    // no mail in anyone's mailbox: what is being consumed is the right to ask,
    // not the right to receive. The loop stops at the first refusal rather than
    // at a count, so it stays true to whatever the limiter is configured to
    // allow, and it is bounded so a limiter that never refuses fails the test
    // instead of running forever.
    let refusedForPacing = false;
    for (let attempt = 0; attempt < MOST_CALLS_WORTH_TRYING && !refusedForPacing; attempt += 1) {
      const status = (
        await beginResetFromThisMachine(request, `spend-the-allowance-${attempt}@hook0.local`)
      ).status();
      if (status === 429) {
        refusedForPacing = true;
      } else {
        expect(status, "an accepted request answers no content").toBe(204);
      }
    }
    expect(
      refusedForPacing,
      `the limiter must refuse a caller within ${MOST_CALLS_WORTH_TRYING} calls; check the burst configured for this stack`
    ).toBe(true);

    // The page now asks for the same thing from the same address, and meets the
    // same wall.
    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect(
      (await responsePromise).status(),
      "the page's own request must be the one the limiter refuses"
    ).toBe(429);

    // Both faces, and both are needed: a page that showed the warning *and* the
    // confirmation would satisfy either one alone while telling the reader two
    // contradictory things — that nothing was sent, and that something was.
    await expect(page.locator('[data-test="reset-password-rate-limited"]')).toBeVisible({
      timeout: 10000,
    });
    await expect(page.locator('[data-test="reset-password-success"]')).toHaveCount(0);
  });
});
