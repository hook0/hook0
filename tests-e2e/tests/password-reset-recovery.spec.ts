import { test, expect, type APIRequestContext } from "@playwright/test";
import {
  API_BASE_URL,
  getPasswordResetTokenFromMailpit,
  getVerificationTokenFromMailpit,
  verifyEmailViaMailpit,
} from "../fixtures/email-verification";
import { fromItsOwnAddress } from "../fixtures/test-setup";

/**
 * How someone gets out of a password reset that went wrong, driven end to end
 * against the real API, a real database and a real mailbox.
 *
 * Both pages of the flow used to be dead ends, and for the same reason: the
 * endpoint stopped saying whether an address belongs to an account, so the two
 * mistakes a user actually makes — mistyping the address, and clicking the
 * older of two links — now look exactly like success and like a broken
 * product. Neither page said what to do next, and neither offered a way to do
 * it. What follows is about the way out being there when it helps, and absent
 * when offering it would cost the reader the link they are holding.
 */

/** Mailpit's API base: CI points this at the service container. */
function mailpitUrl(): string {
  const configured = process.env.MAILPIT_URL;
  if (configured === undefined) {
    return "http://localhost:8025";
  }
  return configured;
}

/** How many password reset messages Mailpit currently holds for an address. */
async function resetEmailCount(request: APIRequestContext, email: string): Promise<number> {
  const search = await request.get(
    `${mailpitUrl()}/api/v1/search?query=to:${encodeURIComponent(email)}`,
    { timeout: 5000 }
  );
  if (!search.ok()) {
    return 0;
  }
  const result = (await search.json()) as { messages: Array<{ ID: string }> };
  expect(Array.isArray(result.messages), "Mailpit must answer a list of messages").toBe(true);

  let found = 0;
  for (const message of result.messages) {
    const detail = await request.get(`${mailpitUrl()}/api/v1/message/${message.ID}`, {
      timeout: 5000,
    });
    if (!detail.ok()) {
      continue;
    }
    const full = (await detail.json()) as { HTML: string };
    if (full.HTML.includes("reset-password")) {
      found += 1;
    }
  }
  return found;
}

interface Account {
  email: string;
  password: string;
}

/** A registered account with a verified address, so it can log in. */
async function createVerifiedAccount(request: APIRequestContext, testId: string): Promise<Account> {
  const timestamp = Date.now();
  const email = `test-reset-recovery-${testId}-${timestamp}@hook0.local`;
  const password = `OriginalPass123!${timestamp}`;

  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    headers: fromItsOwnAddress(),
    data: { email, first_name: "Reset", last_name: "Tester", password },
  });
  expect(registerResponse.status()).toBeLessThan(400);
  await verifyEmailViaMailpit(request, email);

  return { email, password };
}

function login(request: APIRequestContext, email: string, password: string) {
  return request.post(`${API_BASE_URL}/auth/login`, {
    data: { email, password },
    failOnStatusCode: false,
  });
}

/**
 * Ask for a reset link from an address of the caller's own, and hand back the
 * token the mail carries.
 *
 * The forwarded address keeps this call off the allowance the browser draws
 * on: that one is held per source address, every browser-driven call in the
 * suite shares it, and a page that meets a 429 it did not expect fails for a
 * reason that has nothing to do with what is being tested.
 */
async function linkInTheMailbox(request: APIRequestContext, email: string): Promise<string> {
  const response = await request.post(`${API_BASE_URL}/auth/begin-reset-password`, {
    headers: fromItsOwnAddress(),
    data: { email },
    failOnStatusCode: false,
  });
  expect(response.status(), "asking for a link must be accepted").toBe(204);
  return getPasswordResetTokenFromMailpit(request, email, 20000);
}

test.describe("Recovering from a password reset that went wrong", () => {
  test("the way to a new link is offered once the old one is beyond saving, and not before", async ({
    page,
    request,
  }) => {
    // Both faces of one decision, walked on a single real link, because the
    // failure that matters is the page getting them the wrong way round.
    //
    // Offering a new link retires the one the reader is holding — that is what
    // asking again does now. So it is the right thing to offer for a link that
    // is already dead, and a way to lose a working reset for a server that was
    // briefly unreachable.
    const account = await createVerifiedAccount(request, "way-out");
    const timestamp = Date.now();
    const chosenPassword = `QuiltLanternHarbour${timestamp}`;
    const afterTheLinkDied = `MarbleCanyonFerry${timestamp}`;

    const token = await linkInTheMailbox(request, account.email);
    await page.goto(`/reset-password?token=${token}`);
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });

    // First face: the request never reaches the API. Nothing about that says
    // anything about the link, so the form stays and the way out is withheld —
    // taking it would cost this reader a link that is still good.
    await page.route("**/api/v1/auth/reset-password", (route) => route.abort("connectionfailed"));
    await page.locator('[data-test="reset-password-new-password-input"]').fill(chosenPassword);
    await page.locator('[data-test="reset-password-confirm-password-input"]').fill(chosenPassword);
    await page.locator('[data-test="reset-password-submit-button"]').click();

    await expect(page.getByText(/an unknown error occurred/i)).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="reset-password-request-new-link"]')).toHaveCount(0);
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible();

    // And the link really was still good: the same one, once the connection is
    // back, sets the password. Withholding the way out is only right if this
    // holds — otherwise the page would have stranded the reader.
    await page.unrouteAll({ behavior: "ignoreErrors" });
    const acceptedPromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect((await acceptedPromise).status(), "the link must still work").toBe(204);
    await expect(page).toHaveURL(/\/login/, { timeout: 10000 });

    // Second face: the same link, now spent. This is the one a reader meets by
    // accident — a link that was used, or superseded by a newer mail — and the
    // page it lands on is the only one they can reach without an account.
    await page.goto(`/reset-password?token=${token}`);
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });
    const refusalPromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-new-password-input"]').fill(afterTheLinkDied);
    await page
      .locator('[data-test="reset-password-confirm-password-input"]')
      .fill(afterTheLinkDied);
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect(
      (await refusalPromise).status(),
      "a spent link must be refused as unauthenticated, not as a permission the reader lacks"
    ).toBe(401);

    // What the API said about it has to be what the reader reads. The document
    // is written for someone holding two mails and opening the wrong one, and
    // it is the whole of what they get — a page that swapped in wording of its
    // own would send them back to the dead link. Pinned on the one thing that
    // reader has to act on, not on the sentence carrying it: the copy has been
    // reworded twice without ever ceasing to point at the newest mail.
    await expect(page.getByText(/most recent/i)).toBeVisible({ timeout: 10000 });

    const wayOut = page.locator('[data-test="reset-password-request-new-link"]');
    await expect(wayOut).toBeVisible();
    await wayOut.click();
    await expect(page).toHaveURL(/\/begin-reset-password/, { timeout: 10000 });
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible();

    // And the refusal refused: a handler that answered 401 and wrote anyway
    // would have satisfied every assertion above and handed the account over.
    expect(
      (await login(request, account.email, chosenPassword)).status(),
      "the password set through the live link must still open the account"
    ).toBeLessThan(400);
    expect(
      (await login(request, account.email, afterTheLinkDied)).status(),
      "the password the dead link carried must open nothing"
    ).toBeGreaterThanOrEqual(400);
  });

  test("a link the API will not authorize reads as a dead link, not as a permission problem", async ({
    page,
    request,
  }) => {
    // The other way a link dies, and the one nobody clicks on purpose: the
    // authorizer refuses it before anything is looked up. A link past its
    // lifetime lands here, and so does every link that left before the nonce
    // guard shipped — that is, every mail in flight during a deployment.
    //
    // Reached here with a signed link of the wrong kind, which is the same
    // refusal by the same authorizer and the only one a browser can produce on
    // demand. What it used to answer was "Insufficient rights", which sends
    // someone whose reset mail is minutes old looking for an account problem
    // they do not have, on the one page they can reach without an account.
    const timestamp = Date.now();
    const email = `test-reset-recovery-wrong-kind-${timestamp}@hook0.local`;
    const registered = await request.post(`${API_BASE_URL}/register`, {
      headers: fromItsOwnAddress(),
      data: {
        email,
        first_name: "Reset",
        last_name: "Tester",
        password: `OriginalPass123!${timestamp}`,
      },
    });
    expect(registered.status()).toBeLessThan(400);
    const wrongKind = await getVerificationTokenFromMailpit(request, email, 20000);

    await page.goto(`/reset-password?token=${wrongKind}`);
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });

    const refusalPromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    const chosenPassword = `AmberThicketPylon${timestamp}`;
    await page.locator('[data-test="reset-password-new-password-input"]').fill(chosenPassword);
    await page.locator('[data-test="reset-password-confirm-password-input"]').fill(chosenPassword);
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect(
      (await refusalPromise).status(),
      "refused for the link, which is a 401, and not for rights the reader was never asked for"
    ).toBe(401);

    // Read on the page, because a status nobody sees is not what the refusal
    // was changed for.
    await expect(page.getByText(/no longer works/i)).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-test="reset-password-request-new-link"]')).toBeVisible();
    await expect(page.locator('[data-test="reset-password-form"]')).toHaveCount(0);
  });

  test("the confirmation takes the focus, so it is not only visible", async ({ page }) => {
    // The card replaces the form, so whatever was focused goes with it and the
    // focus falls back to the document: a reader who cannot see the card is
    // left on a page that appears not to have answered at all. It carries a
    // live region as well, and that is the weaker half — it is mounted with its
    // content already in it, which is the case screen readers most often stay
    // silent through, so the focus is what has to land.
    await page.goto("/begin-reset-password");
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });

    const answered = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page
      .locator('[data-test="reset-password-email-input"]')
      .fill(`focus-${Date.now()}@hook0.local`);
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect((await answered).status()).toBe(204);

    const success = page.locator('[data-test="reset-password-success"]');
    await expect(success).toBeVisible({ timeout: 10000 });
    await expect(success).toBeFocused();
  });

  test("a mistyped address can still be corrected", async ({ page, request }) => {
    // The confirmation is deliberately the same for an address that has an
    // account and one that does not, which is what makes a typo invisible: the
    // reader is told a mail is on its way to an address that will never
    // receive one. Until this button existed there was nothing to do about it
    // — the form was gone, and the page offered no way back to it.
    const account = await createVerifiedAccount(request, "different-address");
    const mistyped = `mistyped-${Date.now()}@hook0.local`;

    await page.goto("/begin-reset-password");
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });

    const success = page.locator('[data-test="reset-password-success"]');
    await expect(success, "nothing is confirmed before anything is asked").toHaveCount(0);

    const firstAttempt = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-email-input"]').fill(mistyped);
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect((await firstAttempt).status()).toBe(204);

    await expect(success).toBeVisible({ timeout: 10000 });
    await expect(success).toHaveAttribute("role", "status");
    await expect(success).toHaveAttribute("aria-live", "polite");

    await page.locator('[data-test="reset-password-different-address"]').click();
    await expect(page.locator('[data-test="reset-password-form"]')).toBeVisible({ timeout: 10000 });
    await expect(
      page.locator('[data-test="reset-password-email-input"]'),
      "the address comes back as typed: this is a correction, not a fresh start"
    ).toHaveValue(mistyped);

    const secondAttempt = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/auth/begin-reset-password") &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="reset-password-email-input"]').fill(account.email);
    await page.locator('[data-test="reset-password-submit-button"]').click();
    expect((await secondAttempt).status()).toBe(204);
    await expect(success).toBeVisible({ timeout: 10000 });

    // The correction is only worth anything if the mail follows it. Both sides
    // are checked: the endpoint answers 204 to an address it has never heard
    // of, so the mailbox is the only place a correction that did nothing would
    // show up as different from one that worked.
    await getPasswordResetTokenFromMailpit(request, account.email, 20000);
    expect(
      await resetEmailCount(request, mistyped),
      "the address nobody registered must still have received nothing"
    ).toBe(0);
  });
});
