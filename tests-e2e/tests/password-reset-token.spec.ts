import { test, expect, type APIRequestContext } from "@playwright/test";
import { Client } from "pg";
import { API_BASE_URL, verifyEmailViaMailpit } from "../fixtures/email-verification";
import { fromItsOwnAddress } from "../fixtures/test-setup";

/**
 * What a password reset link is allowed to do, driven end to end against the
 * real API, a real database and a real mailbox — no mocks, no interception.
 *
 * A reset link is a signed token that travels by email and comes back from
 * whoever holds it. Nothing about holding it proves the holder is the account
 * owner: a link can be forwarded, sit in a mailbox someone else later reads, or
 * be pulled off a shared machine. So the answer to "is this link still good?"
 * cannot live in the link. Every property below is about the server's ability to
 * say no to a link it has already honoured, replaced, or outlived — and each one
 * is checked by which password actually opens the account afterwards, because a
 * refusal that writes anyway would answer 4xx just as convincingly.
 */

/** Mailpit's API base: CI points this at the service container. */
function mailpitUrl(): string {
  const configured = process.env.MAILPIT_URL;
  if (configured === undefined) {
    return "http://localhost:8025";
  }
  return configured;
}

/** The database the API runs against, shared with the local docker stack. */
function databaseUrl(): string {
  const configured = process.env.DATABASE_URL;
  if (configured === undefined) {
    return "postgres://postgres:postgres@localhost:5432/hook0";
  }
  return configured;
}

/**
 * The server's own bounds on how much reset mail one address may receive: no
 * more than one message per minute, and no more than this many per day. A
 * browser suite cannot read the constants the API compiles in, so the cap is
 * restated here; what the tests below assert is the mailbox, which is where the
 * bound is either honoured or not.
 */
const RESET_EMAILS_ALLOWED_PER_DAY = 5;

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * The rendered body of every password reset message currently in Mailpit for an
 * address.
 *
 * Read by recipient rather than by position: the endpoint answers the same 204
 * whether it sent anything or not, so the mailbox is the only place the
 * difference shows, and a test that asked for "the first message" would be
 * reading whichever one Mailpit happened to list first.
 */
async function resetEmails(request: APIRequestContext, email: string): Promise<string[]> {
  const search = await request.get(
    `${mailpitUrl()}/api/v1/search?query=to:${encodeURIComponent(email)}`,
    { timeout: 5000 }
  );
  if (!search.ok()) {
    return [];
  }

  const result = (await search.json()) as { messages: Array<{ ID: string }> };
  expect(Array.isArray(result.messages), "Mailpit must answer a list of messages").toBe(true);

  const found: string[] = [];
  for (const message of result.messages) {
    const detail = await request.get(`${mailpitUrl()}/api/v1/message/${message.ID}`, {
      timeout: 5000,
    });
    if (!detail.ok()) {
      continue;
    }
    const full = (await detail.json()) as { HTML: string };
    expect(typeof full.HTML, `message ${message.ID} must carry an HTML body`).toBe("string");
    if (full.HTML.includes("reset-password")) {
      found.push(full.HTML);
    }
  }
  return found;
}

/** Wait until at least `count` reset messages have reached `email`. */
async function waitForResetEmails(
  request: APIRequestContext,
  email: string,
  count: number,
  maxWaitMs = 20000
): Promise<string[]> {
  const startedAt = Date.now();
  let latest: string[] = [];
  while (Date.now() - startedAt < maxWaitMs) {
    latest = await resetEmails(request, email);
    if (latest.length >= count) {
      return latest;
    }
    await sleep(500);
  }
  throw new Error(`expected at least ${count} reset email(s) for ${email}, saw ${latest.length}`);
}

/** Pull the reset token out of a rendered message. */
function tokenFrom(html: string): string {
  // The link is wrapped across lines in the delivered message, so the body is
  // flattened before the token is read out of it.
  const match = html.replace(/[\r\n]+/g, "").match(/reset-password\?token=([A-Za-z0-9_\-+/=]+)/i);
  if (match === null) {
    throw new Error("the reset email must carry a reset-password link");
  }
  return match[1];
}

/**
 * Push an account's last-reset-send stamp far enough into the past that the
 * server-side cooldown no longer applies.
 *
 * Asking for a second link seconds after the first is throttled on purpose — a
 * mailbox never gets two of these back to back. Every test here that needs a
 * second link genuinely sent is playing someone who came back later, which this
 * reproduces without the suite sitting out a real minute.
 */
function letTheResetCooldownElapse(email: string): Promise<void> {
  const client = new Client({ connectionString: databaseUrl() });
  return client
    .connect()
    .then(() =>
      client.query(
        `UPDATE iam."user"
            SET password_reset_sent_at = statement_timestamp() - INTERVAL '1 hour'
          WHERE email = $1`,
        [email]
      )
    )
    .then((result) => {
      expect(result.rowCount, `no account to age the cooldown for: ${email}`).toBe(1);
    })
    .finally(() => client.end());
}

interface Account {
  email: string;
  password: string;
}

/** A registered account with a verified address, so it can log in. */
async function createVerifiedAccount(request: APIRequestContext, testId: string): Promise<Account> {
  const timestamp = Date.now();
  const email = `test-reset-token-${testId}-${timestamp}@hook0.local`;
  const password = `OriginalPass123!${timestamp}`;

  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    headers: fromItsOwnAddress(),
    data: { email, first_name: "Reset", last_name: "Tester", password },
  });
  expect(registerResponse.status()).toBeLessThan(400);
  await verifyEmailViaMailpit(request, email);

  return { email, password };
}

/**
 * Ask for a reset link, as a caller with its own source address.
 *
 * Every call here comes from a different address on purpose. The quota these
 * tests are about is the one held per account in the database, and it is the
 * one that has to hold whatever address the caller arrives from — a script
 * walking a proxy pool is the case it exists for. Handing each call its own
 * address takes the per-IP limiter out of the picture entirely, so what the
 * mailbox shows at the end is the per-account bound and nothing else.
 */
function beginReset(request: APIRequestContext, email: string) {
  return request.post(`${API_BASE_URL}/auth/begin-reset-password`, {
    headers: fromItsOwnAddress(),
    data: { email },
    failOnStatusCode: false,
  });
}

function resetWith(request: APIRequestContext, token: string, newPassword: string) {
  return request.post(`${API_BASE_URL}/auth/reset-password`, {
    data: { token, new_password: newPassword },
    failOnStatusCode: false,
  });
}

function login(request: APIRequestContext, email: string, password: string) {
  return request.post(`${API_BASE_URL}/auth/login`, {
    data: { email, password },
    failOnStatusCode: false,
  });
}

/** Ask for a link and hand back the token it carries. */
async function requestResetLink(request: APIRequestContext, email: string): Promise<string> {
  const response = await beginReset(request, email);
  expect(response.status()).toBe(204);
  const emails = await waitForResetEmails(request, email, 1);
  return tokenFrom(emails[0]);
}

test.describe("Password reset links", () => {
  test("a link that has already set a password cannot set another one", async ({ request }) => {
    const account = await createVerifiedAccount(request, "replay");
    const timestamp = Date.now();
    const chosenPassword = `QuiltLanternHarbour${timestamp}`;
    const attackerPassword = `MarbleCanyonFerry${timestamp}`;

    const token = await requestResetLink(request, account.email);

    // The owner uses their link.
    expect((await resetWith(request, token, chosenPassword)).status()).toBe(204);

    // Whoever else holds the same link — it travelled by email, so that is not a
    // hypothetical — tries it again.
    const replay = await resetWith(request, token, attackerPassword);
    expect(replay.status(), "a spent reset link must be refused").toBeGreaterThanOrEqual(400);

    // The status is not the point; the account is. A handler that answered 4xx
    // and wrote anyway would pass the line above and hand the account over.
    expect(
      (await login(request, account.email, chosenPassword)).status(),
      "the password the owner chose must still open the account"
    ).toBeLessThan(400);
    expect(
      (await login(request, account.email, attackerPassword)).status(),
      "the password the replay tried to install must open nothing"
    ).toBeGreaterThanOrEqual(400);
  });

  test("the same link sent twice at once is honoured once", async ({ request }) => {
    // A double click on the button in the mail, or a client that prefetches
    // links, sends the same token twice with nothing in between. Setting a
    // password costs about a tenth of a second of deliberate hashing before
    // anything is written, and both calls spend it at the same time — so a check
    // made on a value read before that hash would still be true for both of them
    // when they came back. This is the case that tells such a check apart from
    // one carried by the write itself.
    const account = await createVerifiedAccount(request, "concurrent");
    const timestamp = Date.now();
    const oneChoice = `QuiltLanternHarbour${timestamp}`;
    const theOther = `MarbleCanyonFerry${timestamp}`;

    const token = await requestResetLink(request, account.email);

    const [first, second] = await Promise.all([
      resetWith(request, token, oneChoice),
      resetWith(request, token, theOther),
    ]);

    const accepted = [first, second].filter((response) => response.status() === 204);
    expect(accepted.length, "one link, one password set").toBe(1);

    // And the account holds the password of the call that was told yes. Two
    // writes landing in either order would leave someone holding a password the
    // server said no to.
    const settled = first.status() === 204 ? oneChoice : theOther;
    const refused = first.status() === 204 ? theOther : oneChoice;
    expect(
      (await login(request, account.email, settled)).status(),
      "the account must hold the password whose call was accepted"
    ).toBeLessThan(400);
    expect(
      (await login(request, account.email, refused)).status(),
      "the password whose call was refused must open nothing"
    ).toBeGreaterThanOrEqual(400);
  });

  test("asking for a second link retires the first, and the second one still works", async ({
    request,
  }) => {
    const account = await createVerifiedAccount(request, "reissue");
    const timestamp = Date.now();
    const staleAttempt = `AmberThicketPylon${timestamp}`;
    const chosenPassword = `QuiltLanternHarbour${timestamp}`;

    const firstToken = await requestResetLink(request, account.email);

    // Someone who did not get the first mail — or thinks they did not — asks
    // again. Coming back a minute later is the ordinary way that happens.
    await letTheResetCooldownElapse(account.email);
    expect((await beginReset(request, account.email)).status()).toBe(204);
    const delivered = await waitForResetEmails(request, account.email, 2);
    const secondTokens = delivered.map((body) => tokenFrom(body));
    const secondToken = secondTokens.find((candidate) => candidate !== firstToken);
    if (secondToken === undefined) {
      throw new Error("the second request must deliver a link different from the first");
    }

    // The link that was superseded is dead, even though its own clock has not
    // run out: two live links means two chances for the wrong person.
    const stale = await resetWith(request, firstToken, staleAttempt);
    expect(stale.status(), "a superseded reset link must be refused").toBeGreaterThanOrEqual(400);
    expect(
      (await login(request, account.email, staleAttempt)).status(),
      "the superseded link must not have set anything"
    ).toBeGreaterThanOrEqual(400);

    // And the retirement stops there. Refusing the newest link too would lock
    // people out of the one recovery path they have — every request would kill
    // the link it just sent.
    expect(
      (await resetWith(request, secondToken, chosenPassword)).status(),
      "the newest reset link must work"
    ).toBe(204);
    expect(
      (await login(request, account.email, chosenPassword)).status(),
      "the password set through the newest link must open the account"
    ).toBeLessThan(400);
  });

  test("changing the password from the settings retires a link still in flight", async ({
    request,
  }) => {
    // The case someone lives through after a link leaks: they still know their
    // password, they change it, and they expect that to be the end of it. If the
    // link outlives the change, the change is what hands the account over — it
    // signs every session out, so the owner is the one who ends up locked out.
    const account = await createVerifiedAccount(request, "changed");
    const timestamp = Date.now();
    const chosenPassword = `QuiltLanternHarbour${timestamp}`;
    const leakedLinkAttempt = `MarbleCanyonFerry${timestamp}`;

    const token = await requestResetLink(request, account.email);

    const session = await login(request, account.email, account.password);
    expect(session.status()).toBeLessThan(400);
    const { access_token } = (await session.json()) as { access_token: string };

    const change = await request.post(`${API_BASE_URL}/auth/password`, {
      headers: { Authorization: `Bearer ${access_token}` },
      data: { current_password: account.password, new_password: chosenPassword },
      failOnStatusCode: false,
    });
    expect(change.status(), "the owner must be able to change their own password").toBeLessThan(
      400
    );

    const leaked = await resetWith(request, token, leakedLinkAttempt);
    expect(
      leaked.status(),
      "a reset link must not survive the password change that came after it"
    ).toBeGreaterThanOrEqual(400);

    expect(
      (await login(request, account.email, chosenPassword)).status(),
      "the password the owner chose must still open the account"
    ).toBeLessThan(400);
    expect(
      (await login(request, account.email, leakedLinkAttempt)).status(),
      "the password the stale link tried to install must open nothing"
    ).toBeGreaterThanOrEqual(400);
  });

  test("a flood of requests puts no more than the day's allowance in the mailbox", async ({
    request,
  }) => {
    // The endpoint answers the same 204 to every caller, so nothing it returns
    // bounds anything: a script pointed at one address would otherwise turn the
    // mailbox into the attack. What has to hold is on the other side — how much
    // mail actually lands.
    //
    // Every request below arrives from a different source address, so the per-IP
    // limiter never sees a repeat caller and stops none of them. What is left
    // holding the line is the allowance kept against the account itself, which
    // is the one an attacker cannot walk away from.
    const account = await createVerifiedAccount(request, "flood");

    const burst: number[] = [];
    for (let attempt = 0; attempt < 8; attempt += 1) {
      burst.push((await beginReset(request, account.email)).status());
    }
    expect(new Set(burst).size, "every request must get the same answer, sent or not").toBe(1);
    expect(burst[0]).toBe(204);

    // Eight requests inside one minute, one message: the throttle is what keeps
    // a mailbox from being usable as a weapon, and it is enforced where the
    // requests cannot reach it.
    await waitForResetEmails(request, account.email, 1);
    await sleep(3000);
    expect(
      (await resetEmails(request, account.email)).length,
      "requests made inside one cooldown window may send at most one message"
    ).toBe(1);

    // Spacing the requests out gets past the cooldown but not past the day's
    // allowance, which is the bound that actually caps a mailbox: a caller
    // willing to wait a minute between requests would otherwise still be able to
    // deliver a message every minute, all day.
    const spaced: number[] = [];
    for (let attempt = 0; attempt < RESET_EMAILS_ALLOWED_PER_DAY + 2; attempt += 1) {
      await letTheResetCooldownElapse(account.email);
      spaced.push((await beginReset(request, account.email)).status());
    }
    expect(new Set(spaced).size, "reaching the allowance must stay invisible to the caller").toBe(
      1
    );
    expect(spaced[0]).toBe(204);

    await waitForResetEmails(request, account.email, RESET_EMAILS_ALLOWED_PER_DAY);
    await sleep(3000);
    expect(
      (await resetEmails(request, account.email)).length,
      "one address may not receive more reset mail in a day than its allowance"
    ).toBe(RESET_EMAILS_ALLOWED_PER_DAY);
  });

  test("an address nobody registered is answered exactly like one that exists", async ({
    request,
  }) => {
    // The endpoint used to answer 401 for an address it did not know, which made
    // it a plain list of who has an account here — readable from the login page
    // with no tooling at all.
    const unknown = `never-registered-${Date.now()}@hook0.local`;
    const unknownResponse = await beginReset(request, unknown);

    const account = await createVerifiedAccount(request, "enumeration");
    const knownResponse = await beginReset(request, account.email);

    expect(knownResponse.status(), "a known address gets no content").toBe(204);
    expect(
      unknownResponse.status(),
      "an unknown address must get the same status as a known one"
    ).toBe(knownResponse.status());
    expect(
      await unknownResponse.text(),
      "an unknown address must get the same body as a known one"
    ).toBe(await knownResponse.text());

    // Identical answers prove nothing if the endpoint does nothing for anyone,
    // so the known address has to have really been served.
    await waitForResetEmails(request, account.email, 1);

    // And the silence is real: no message was minted for an address with no
    // account behind it.
    await sleep(3000);
    expect(
      (await resetEmails(request, unknown)).length,
      "an address with no account must receive nothing"
    ).toBe(0);
  });
});
