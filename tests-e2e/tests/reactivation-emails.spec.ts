import { test, expect, APIRequestContext } from "@playwright/test";
import { Client } from "pg";
import { API_BASE_URL, getEmailFromMailpit } from "../fixtures/email-verification";

/**
 * End-to-end coverage of the "0 event sent" reactivation drip.
 *
 * The drip is a background job inside the API: every pass it selects verified
 * accounts whose organization never ingested an event, claims one step per
 * organization and sends the matching email over real SMTP. These tests drive
 * the real job — no stubbing of the scheduler, the query or the mailer: an
 * account is aged by moving its signup timestamp back in the database (the only
 * way to reach a J+1 threshold inside a CI run), then the assertion is made on
 * the message that actually lands in Mailpit.
 *
 * Timing: the job waits out a startup grace period, then runs every
 * REACTIVATION_EMAILS_PERIOD (set to a few seconds for CI in
 * tests-e2e/.gitlab-ci.yml). Waits below are bounded and generous enough to
 * cover a missed pass without becoming unbounded.
 */

const DATABASE_URL =
  process.env.DATABASE_URL || "postgres://postgres:postgres@localhost:5432/hook0";

/** Subject of the J+1 step — the first message of the sequence. */
const DAY1_SUBJECT = "Your first webhook is 5 minutes away";

/** Subject of the J+3 step — the next message the sequence would send. */
const DAY3_SUBJECT = "The missing piece: a URL to receive your webhook";

/** Upper bound for a reactivation email to show up after an account qualifies. */
const DRIP_WAIT_MS = 90_000;

/**
 * Window used to prove a *negative*: long enough that a pass certainly happened
 * (the control account below receives its email inside it), short enough to keep
 * the suite fast.
 */
const NEGATIVE_WINDOW_MS = 45_000;

// Each case uses its own freshly registered address, so they are independent;
// only the generous timeout is shared, to cover a missed job pass.
test.describe.configure({ timeout: 180_000 });

async function withDb<T>(fn: (client: Client) => Promise<T>): Promise<T> {
  const client = new Client({ connectionString: DATABASE_URL });
  await client.connect();
  try {
    return await fn(client);
  } finally {
    await client.end();
  }
}

interface ReactivationUser {
  email: string;
  password: string;
  organizationId: string;
}

/**
 * Register through the public API and mark the address verified, which is the
 * state the drip selects on. Returns the credentials plus the organization the
 * registration created.
 */
async function registerVerifiedUser(
  request: APIRequestContext,
  testId: string,
  firstName = "Nina"
): Promise<ReactivationUser> {
  const email = `reactivation-${testId}-${Date.now()}@hook0.local`;
  const password = `TestPassword123!${Date.now()}`;

  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    data: { email, first_name: firstName, last_name: "Dormant", password },
  });
  expect(registerResponse.status(), await registerResponse.text()).toBeLessThan(400);
  const organizationId = (await registerResponse.json()).organization_id as string;
  expect(organizationId).toBeTruthy();

  await withDb((client) =>
    client.query(
      "UPDATE iam.user SET email_verified_at = NOW() WHERE email = $1 AND email_verified_at IS NULL",
      [email]
    )
  );

  return { email, password, organizationId };
}

/**
 * Move the signup date back so the account clears a step's minimum age. This is
 * time travel for the test, not a stub: the job runs its real query against the
 * moved timestamp.
 */
async function ageSignupByDays(email: string, days: number): Promise<void> {
  const result = await withDb((client) =>
    client.query(
      `UPDATE iam.user SET created_at = NOW() - MAKE_INTERVAL(days => $2) WHERE email = $1`,
      [email, days]
    )
  );
  expect(result.rowCount, `signup date of ${email} should have been aged`).toBe(1);
}

/** Count reactivation steps recorded for an organization. */
async function recordedSteps(organizationId: string): Promise<number[]> {
  const result = await withDb((client) =>
    client.query(
      "SELECT step FROM iam.reactivation_email WHERE organization__id = $1 ORDER BY step",
      [organizationId]
    )
  );
  return result.rows.map((row: { step: number }) => Number(row.step));
}

/**
 * Push a recorded step's send time into the past so the minimum spacing before
 * the next step of the sequence is satisfied.
 */
async function backdateStepSent(organizationId: string, step: number, days: number): Promise<void> {
  const result = await withDb((client) =>
    client.query(
      `UPDATE iam.reactivation_email SET sent_at = NOW() - MAKE_INTERVAL(days => $3) WHERE organization__id = $1 AND step = $2`,
      [organizationId, step, days]
    )
  );
  expect(result.rowCount, `step ${step} of ${organizationId} should have been aged`).toBe(1);
}

/** Whether the account asked to stop receiving the drip. */
async function hasOptedOut(email: string): Promise<boolean> {
  const result = await withDb((client) =>
    client.query(
      `SELECT reactivation_opted_out_at IS NOT NULL AS opted_out FROM iam.user WHERE email = $1`,
      [email]
    )
  );
  expect(result.rowCount, `user ${email} should exist`).toBe(1);
  return result.rows[0].opted_out as boolean;
}

/** When the account opted out. Fails the test if it never did. */
async function optOutTimestamp(email: string): Promise<string> {
  const result = await withDb((client) =>
    client.query(
      `SELECT reactivation_opted_out_at FROM iam.user WHERE email = $1 AND reactivation_opted_out_at IS NOT NULL`,
      [email]
    )
  );
  expect(result.rowCount, `the opt-out of ${email} should have been recorded`).toBe(1);
  return String(result.rows[0].reactivation_opted_out_at);
}

/**
 * The opt-out link the email carries, exactly as a reader would click it.
 * Fails the test rather than returning a fallback: a missing link is the bug
 * these tests exist to catch.
 */
function unsubscribeLinkFrom(html: string): URL {
  const match = html.match(/href="([^"]*\/unsubscribe\?[^"]*)"/);
  expect(match, "the reactivation email must carry an opt-out link").not.toBeNull();
  return new URL(match![1].replace(/&amp;/g, "&"));
}

/** Number of messages Mailpit holds for an address whose body matches a filter. */
async function countEmails(
  request: APIRequestContext,
  email: string,
  filter: string
): Promise<number> {
  const mailpitUrl = process.env.MAILPIT_URL || "http://localhost:8025";
  const response = await request.get(
    `${mailpitUrl}/api/v1/search?query=to:${encodeURIComponent(email)}`,
    { timeout: 5000 }
  );
  if (!response.ok()) return 0;
  const messages = ((await response.json()).messages ?? []) as Array<{ ID: string }>;

  let matches = 0;
  for (const message of messages) {
    const detail = await request.get(`${mailpitUrl}/api/v1/message/${message.ID}`, {
      timeout: 5000,
    });
    if (!detail.ok()) continue;
    const full = (await detail.json()) as { Subject?: string; HTML?: string; Text?: string };
    if (`${full.Subject ?? ""}\n${full.HTML ?? ""}\n${full.Text ?? ""}`.includes(filter)) {
      matches += 1;
    }
  }
  return matches;
}

/** Obtain a user access token through the real login endpoint. */
async function login(request: APIRequestContext, user: ReactivationUser): Promise<string> {
  const response = await request.post(`${API_BASE_URL}/auth/login`, {
    data: { email: user.email, password: user.password },
  });
  expect(response.status(), await response.text()).toBeLessThan(400);
  return (await response.json()).access_token as string;
}

/**
 * Take the account all the way to "has ingested an event" through the public
 * API: application, application secret, then one event. This is the exact signal
 * the drip stops on.
 */
async function ingestFirstEvent(request: APIRequestContext, user: ReactivationUser): Promise<void> {
  const accessToken = await login(request, user);
  const auth = { Authorization: `Bearer ${accessToken}` };

  const application = await request.post(`${API_BASE_URL}/applications`, {
    headers: auth,
    data: { organization_id: user.organizationId, name: "Activated app" },
  });
  expect(application.status(), await application.text()).toBeLessThan(400);
  const applicationId = (await application.json()).application_id as string;

  const secret = await request.post(`${API_BASE_URL}/application_secrets`, {
    headers: auth,
    data: { application_id: applicationId, name: "e2e" },
  });
  expect(secret.status(), await secret.text()).toBeLessThan(400);
  const applicationSecret = (await secret.json()).token as string;

  // Events reference a registered event type (foreign key), so declare it first.
  const eventType = await request.post(`${API_BASE_URL}/event_types`, {
    headers: auth,
    data: {
      application_id: applicationId,
      service: "test",
      resource_type: "entity",
      verb: "created",
    },
  });
  expect(eventType.status(), await eventType.text()).toBeLessThan(400);

  const event = await request.post(`${API_BASE_URL}/event/`, {
    headers: { Authorization: `Bearer ${applicationSecret}` },
    data: {
      application_id: applicationId,
      event_type: "test.entity.created",
      labels: { all: "yes" },
      occurred_at: new Date().toISOString(),
      payload_content_type: "application/json",
      payload: '{"activated": true}',
    },
  });
  expect(event.status(), await event.text()).toBe(201);
}

test.describe("Reactivation drip for accounts that never sent an event", () => {
  test("sends the J+1 email, with its CTA, campaign tagging and opt-out", async ({ request }) => {
    // The greeting is the one place a recipient's own text reaches the message,
    // so this account carries markup in its first name: the escaping assertion
    // below is only worth anything if something was there to escape.
    const injectedName = '<script>alert("x")</script>';
    const user = await registerVerifiedUser(request, "day1", injectedName);
    await ageSignupByDays(user.email, 2);

    const message = await getEmailFromMailpit(request, user.email, DAY1_SUBJECT, DRIP_WAIT_MS);
    const html = message.HTML ?? "";

    expect(html, "the J+1 mail drives the user to send a first webhook").toContain(
      "Send your first webhook"
    );

    // Lifecycle mails must stay attributable and must carry a way out, which is
    // what makes shipping them on by default acceptable.
    expect(html, "reactivation links are Matomo-tagged for funnel analysis").toContain(
      "mtm_campaign=reactivation_no_event_d1"
    );
    const unsubscribeLink = unsubscribeLinkFrom(html);
    expect(unsubscribeLink.pathname, "the opt-out link points at the unsubscribe page").toBe(
      "/unsubscribe"
    );
    expect(
      unsubscribeLink.searchParams.get("token"),
      "the opt-out link carries the token that identifies the reader"
    ).toBeTruthy();

    // The recipient's own data must never be rendered raw into the mail.
    expect(html, "the greeting must carry the recipient's name at all").toContain("&lt;script&gt;");
    expect(html).not.toContain("<script");

    expect(await recordedSteps(user.organizationId), "step 1 is recorded as sent").toContain(1);
  });

  test("never sends twice for the same step", async ({ request }) => {
    const user = await registerVerifiedUser(request, "once");
    await ageSignupByDays(user.email, 2);

    await getEmailFromMailpit(request, user.email, DAY1_SUBJECT, DRIP_WAIT_MS);

    // Let several further passes go by: the claim table must keep them idempotent.
    await new Promise((resolve) => setTimeout(resolve, NEGATIVE_WINDOW_MS));

    expect(
      await countEmails(request, user.email, DAY1_SUBJECT),
      "a recorded step must never be re-sent on later passes"
    ).toBe(1);
  });

  test("stops the whole series when the reader clicks the opt-out link", async ({ request }) => {
    // Control account: same age, same state, never opts out. It proves the job
    // really did run during the observation window, so the absence of a J+3
    // email for the opted-out account below means something.
    const control = await registerVerifiedUser(request, "optout-control");
    const leaver = await registerVerifiedUser(request, "optout");

    await ageSignupByDays(control.email, 2);
    await ageSignupByDays(leaver.email, 2);

    const message = await getEmailFromMailpit(request, leaver.email, DAY1_SUBJECT, DRIP_WAIT_MS);
    await getEmailFromMailpit(request, control.email, DAY1_SUBJECT, DRIP_WAIT_MS);

    // Click the link exactly as a mail reader would: no session, no sign-in.
    const unsubscribeLink = unsubscribeLinkFrom(message.HTML ?? "");
    const token = unsubscribeLink.searchParams.get("token");
    const unsubscribe = await request.post(
      `${API_BASE_URL}/email-preferences/unsubscribe-reactivation`,
      { data: { token } }
    );
    expect(unsubscribe.status(), await unsubscribe.text()).toBeLessThan(400);

    const firstOptOut = await optOutTimestamp(leaver.email);

    // Clicking again (or a mail client prefetching the page) must not move the
    // recorded date, and must not start failing.
    const replay = await request.post(
      `${API_BASE_URL}/email-preferences/unsubscribe-reactivation`,
      {
        data: { token },
      }
    );
    expect(replay.status(), await replay.text()).toBeLessThan(400);
    expect(await optOutTimestamp(leaver.email), "opting out twice is idempotent").toEqual(
      firstOptOut
    );

    // Both accounts now qualify for J+3 (age and spacing since J+1 satisfied).
    // Only the one that stayed subscribed may hear from us again.
    for (const account of [control, leaver]) {
      await ageSignupByDays(account.email, 4);
      await backdateStepSent(account.organizationId, 1, 3);
    }

    await getEmailFromMailpit(request, control.email, DAY3_SUBJECT, DRIP_WAIT_MS);

    expect(
      await countEmails(request, leaver.email, DAY3_SUBJECT),
      "an account that opted out must never receive the next step"
    ).toBe(0);
    expect(
      await recordedSteps(leaver.organizationId),
      "no further step may be claimed after an opt-out"
    ).toEqual([1]);
  });

  test("opting out from the browser, by clicking the link exactly as it was sent", async ({
    page,
    request,
  }) => {
    // The API-level test above proves the endpoint; this one proves the thing
    // the reader actually does — following the href in the mail — reaches a
    // page that confirms it worked.
    const user = await registerVerifiedUser(request, "optout-browser");
    await ageSignupByDays(user.email, 2);

    const message = await getEmailFromMailpit(request, user.email, DAY1_SUBJECT, DRIP_WAIT_MS);
    const unsubscribeLink = unsubscribeLinkFrom(message.HTML ?? "");

    await page.goto(`${unsubscribeLink.pathname}${unsubscribeLink.search}`);
    await expect(page.locator('[data-test="unsubscribe-confirmation"]')).toBeVisible({
      timeout: 15000,
    });
    expect(await hasOptedOut(user.email), "the visit must record the opt-out").toBe(true);

    // The token is long-lived, so it must not stay in the address bar where
    // analytics and browser history would keep it.
    expect(page.url(), "the opt-out token must not survive in the URL").not.toContain(
      unsubscribeLink.searchParams.get("token")
    );
  });

  test("tells the reader what happened when the link is not usable", async ({ page }) => {
    // A link mangled by a mail client is the common case, and it must produce an
    // explanation rather than a spinner that never resolves.
    await page.goto("/unsubscribe?token=not-a-real-token");
    await expect(page.locator('[data-test="unsubscribe-error"]')).toBeVisible({ timeout: 15000 });

    // And a link with no token at all.
    await page.goto("/unsubscribe");
    await expect(page.locator('[data-test="unsubscribe-error"]')).toBeVisible({ timeout: 15000 });
  });

  test("rejects an unsubscribe link that was tampered with", async ({ request }) => {
    const user = await registerVerifiedUser(request, "optout-tampered");
    await ageSignupByDays(user.email, 2);

    const message = await getEmailFromMailpit(request, user.email, DAY1_SUBJECT, DRIP_WAIT_MS);
    const token = unsubscribeLinkFrom(message.HTML ?? "").searchParams.get("token") ?? "";

    // Flip the last character of the signed token: the signature no longer
    // matches, so nobody can forge an opt-out for an address they do not own.
    const tampered = token.slice(0, -1) + (token.endsWith("a") ? "b" : "a");
    const response = await request.post(
      `${API_BASE_URL}/email-preferences/unsubscribe-reactivation`,
      { data: { token: tampered } }
    );
    expect(response.status(), "a forged token must be refused").toBe(401);
    expect(await hasOptedOut(user.email), "a forged token must not opt anybody out").toBe(false);
  });

  test("stops as soon as the account sends its first event", async ({ request }) => {
    // Control account: same age, no event. It proves the job really ran during
    // the observation window, so the absence of mail below means something.
    const control = await registerVerifiedUser(request, "control");
    const activated = await registerVerifiedUser(request, "activated");

    await ingestFirstEvent(request, activated);

    await ageSignupByDays(control.email, 8);
    await ageSignupByDays(activated.email, 8);

    await getEmailFromMailpit(request, control.email, DAY1_SUBJECT, DRIP_WAIT_MS);
    await new Promise((resolve) => setTimeout(resolve, NEGATIVE_WINDOW_MS));

    expect(
      await countEmails(request, activated.email, DAY1_SUBJECT),
      "an account that already ingested an event must be left alone"
    ).toBe(0);
    expect(
      await recordedSteps(activated.organizationId),
      "no step may be claimed for an activated organization"
    ).toEqual([]);
  });
});
