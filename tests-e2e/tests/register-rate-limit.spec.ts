import { test, expect, type APIRequestContext } from "@playwright/test";
import { API_BASE_URL } from "../fixtures/email-verification";
import { fromItsOwnAddress } from "../fixtures/test-setup";

/**
 * Registration mails a verification link to an address the caller names, which
 * is what begin-reset-password and resend-verification-email do, so it sits
 * behind the same per-IP limiter. Unlike those two it cannot be aimed at one
 * mailbox twice — the address is unique — so what this bounds is a sweep across
 * many addresses, one mail each.
 *
 * Driven against the real limiter: no interception, no faked response. Removing
 * the wrap from the route leaves every other test green, so this is the only
 * place that placement is checked.
 */

/**
 * Enough to run past the email limiter's burst and no further. The global
 * per-IP limiter allows 200 and refills about a hundred times a second, so a
 * refusal reached under this ceiling can only be the email one — that is what
 * the bound is for, not merely a guard against looping forever.
 */
const MOST_CALLS_WORTH_TRYING = 120;

/**
 * A registration the API refuses on its own terms: `first_name` is empty, which
 * its validators reject with 422. The limiter runs in front of the handler, so
 * the call still spends the allowance while no account is created and no mail
 * is sent — this test does not have to fill a mailbox to prove a point about
 * filling mailboxes.
 */
function registerFrom(request: APIRequestContext, headers: Record<string, string>, email: string) {
  return request.post(`${API_BASE_URL}/register`, {
    headers,
    data: { first_name: "", last_name: "Tester", email, password: "not-used-here" },
    failOnStatusCode: false,
  });
}

test.describe("Registration @rate-limit", () => {
  test("is bounded by the limiter that guards the endpoints mailing a named address", async ({
    request,
  }) => {
    // An address of this test's own, so the allowance it spends is one nothing
    // else has touched. Sharing the default one would let a sibling file that
    // already emptied the bucket hand this test a refusal on its first call —
    // green for a reason that has nothing to do with the route under test.
    const callerAddress = fromItsOwnAddress();

    let refusedForPacing = false;
    let callsMade = 0;

    for (let attempt = 0; attempt < MOST_CALLS_WORTH_TRYING && !refusedForPacing; attempt += 1) {
      callsMade += 1;
      const response = await registerFrom(
        request,
        callerAddress,
        `register-rate-limit-${Date.now()}-${attempt}@hook0.local`
      );
      if (response.status() === 429) {
        refusedForPacing = true;
        // The refusal has to come from a limiter rather than from anything the
        // handler decided, or a 429 raised for some other reason would do.
        const problem: unknown = await response.json();
        expect(
          (problem as { id?: unknown }).id,
          "a paced refusal names the rate-limited problem"
        ).toBe("RateLimited");
      } else {
        expect(
          response.status(),
          "a request that is not paced is refused on its own merits, not accepted"
        ).toBe(422);
      }
    }

    expect(
      refusedForPacing,
      `registration must be refused for pacing within ${MOST_CALLS_WORTH_TRYING} calls; without the limiter on this route it never is`
    ).toBe(true);
    // Starting from an untouched allowance, the burst has to be spent before the
    // refusal arrives. A refusal on the first call would mean the address was not
    // this test's own after all, and the loop above would have proven nothing.
    expect(callsMade, "the burst is spent before the limiter refuses").toBeGreaterThan(1);
    expect(
      callsMade,
      "reached this early, the refusal is the email limiter and not the global per-IP one"
    ).toBeLessThan(MOST_CALLS_WORTH_TRYING);
  });
});
