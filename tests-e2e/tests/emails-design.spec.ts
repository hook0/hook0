import { test, expect } from "@playwright/test";
import {
  API_BASE_URL,
  getEmailFromMailpit,
  type MailpitMessage,
} from "../fixtures/email-verification";

/**
 * Design / wording / tracking assertions for every transactional email.
 *
 * These tests inspect the real HTML rendered by Mailpit (not a mock) and
 * guarantee the redesign's non-negotiables:
 *
 *  - Logo URL is the website-served PNG (not the broken LFS pointer that
 *    used to be served by app.hook0.com).
 *  - First-name personalisation reaches the rendered body.
 *  - CTA labels are correct for each variant (anti-regression of the
 *    historic "Verify email" copy on the reset-password template).
 *  - Every clickable link carries `mtm_source=email`, `mtm_medium=transactional`
 *    and `mtm_campaign=<expected campaign>` so Matomo can attribute downstream
 *    activity. Mailto links are excluded.
 *  - Footer carries the legal entity identity (FGRibreau SARL) and the
 *    Privacy Policy link required by GDPR Art. 13.
 *  - Plain-text variant is wrapped at <= 80 columns.
 *
 * These tests rely on Mailpit being reachable. They do NOT fall back to
 * database-based verification, since the goal here is to inspect the email
 * content itself. Mailpit is provided by docker-compose locally and as a
 * GitLab CI service in CI.
 */

const REQUIRED_MTM = ["mtm_source=email", "mtm_medium=transactional"] as const;

function extractClickableHrefs(html: string): string[] {
  const matches = html.matchAll(/href="([^"]+)"/g);
  const hrefs: string[] = [];
  for (const m of matches) {
    const href = m[1];
    if (href.startsWith("mailto:")) continue;
    if (href.startsWith("#")) continue;
    hrefs.push(href);
  }
  return hrefs;
}

function assertCommonFooter(html: string, campaign: string) {
  expect(html, "footer identity").toContain("Open-Source Webhooks-as-a-Service");
  expect(html, "footer legal name").toContain("FGRibreau SARL");
  expect(html, "footer postal address").toContain("Chantonnay");
  expect(html, "footer RCS").toContain("RCS La Roche-sur-Yon");
  // Anti-regression GDPR Art. 13 footer link — wording is "Privacy & data
  // rights" but we keep the assertion loose so a future rename to "Privacy
  // Policy" or "Privacy Notice" stays green.
  expect(html, "footer Privacy link").toMatch(/Privacy(\s*&|\s+(Policy|Notice))/i);
  // Anti-regression for the original LFS-pointer bug: the previous default
  // `https://app.hook0.com/256x256.png` served a 130-byte ASCII pointer
  // instead of a PNG. We accept any image served from the website mediakit.
  expect(html, "logo URL").toMatch(/www\.hook0\.com\/mediakit\/logo\/[^"]+\.png/);
  expect(html, "logo URL must not be the broken app.hook0.com path").not.toContain(
    "app.hook0.com/256x256.png"
  );
  expect(html, "no unsubstituted placeholder").not.toMatch(/\{\s*\$\w+\s*\}/);
  for (const href of extractClickableHrefs(html)) {
    for (const required of REQUIRED_MTM) {
      expect(href, `link must contain ${required}: ${href}`).toContain(required);
    }
    expect(
      href,
      `link must contain mtm_campaign=${campaign}: ${href}`
    ).toContain(`mtm_campaign=${campaign}`);
  }
}

function assertPlainTextWidth(message: MailpitMessage) {
  const text = message.Text ?? "";
  for (const line of text.split(/\r?\n/)) {
    expect(
      [...line].length,
      `plain text line >80 cols: ${JSON.stringify(line)}`
    ).toBeLessThanOrEqual(80);
  }
}

test.describe("Transactional emails — design, wording, tracking", () => {
  test("Verify email — design + Matomo + first-name personalisation", async ({
    request,
  }) => {
    const timestamp = Date.now();
    const email = `test-verify-design-${timestamp}@hook0.local`;
    const firstName = "Sarah";

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: firstName,
        last_name: "Hook0Tester",
        password: `TestPassword123!${timestamp}`,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const message = await getEmailFromMailpit(request, email, "Verify your Hook0 email");
    expect(message.Subject ?? "").toContain("Verify your Hook0 email");

    const html = message.HTML ?? "";
    expect(html, "personalised greeting with first name").toContain(firstName);
    expect(html, "CTA label").toContain("Verify email");
    expect(html, "no historic typo").not.toContain("you account");
    assertCommonFooter(html, "verify_email");
    assertPlainTextWidth(message);
  });

  test("Welcome — arrives after verification, links to dashboard + docs", async ({
    request,
  }) => {
    const timestamp = Date.now();
    const email = `test-welcome-design-${timestamp}@hook0.local`;
    const firstName = "Sarah";

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: firstName,
        last_name: "Hook0Tester",
        password: `TestPassword123!${timestamp}`,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Pull the verification token from the verify email and call the API.
    const verifyMail = await getEmailFromMailpit(request, email, "Verify your Hook0 email");
    const tokenMatch = (verifyMail.HTML ?? verifyMail.Text ?? "")
      .replace(/[\r\n]+/g, "")
      .match(/verify-email\?token=([A-Za-z0-9_\-+/=%]+)/i);
    expect(tokenMatch, "verification token in verify email").toBeTruthy();
    const token = decodeURIComponent(tokenMatch![1].replace(/&.*$/, ""));

    const verifyResponse = await request.post(`${API_BASE_URL}/auth/verify-email`, {
      data: { token },
    });
    expect(verifyResponse.status()).toBeLessThan(400);

    const message = await getEmailFromMailpit(
      request,
      email,
      "ship your first webhook"
    );
    const html = message.HTML ?? "";
    // Welcome H1 is "You're in — let's ship a webhook" (no name); the
    // personalisation lives in the body as "Hi {firstName}, your Hook0…".
    // The first-name presence in the body is the load-bearing part.
    expect(html, "welcome body greets by first name").toContain(`Hi ${firstName},`);
    expect(html, "primary CTA").toContain("Create your first webhook");
    expect(html, "documentation link").toContain("documentation.hook0.com");
    assertCommonFooter(html, "welcome");
    assertPlainTextWidth(message);
  });

  test("Reset password — correct CTA label, anti-regression on Verify", async ({
    request,
  }) => {
    const timestamp = Date.now();
    const email = `test-reset-design-${timestamp}@hook0.local`;
    const firstName = "Sarah";

    // Register a user first so the reset request finds a target.
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: firstName,
        last_name: "Hook0Tester",
        password: `TestPassword123!${timestamp}`,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Wait for the verify mail to arrive so we know the user is fully created
    // and the inbox is non-empty before we issue the reset.
    await getEmailFromMailpit(request, email, "Verify your Hook0 email");

    const beginResetResponse = await request.post(
      `${API_BASE_URL}/auth/begin-reset-password`,
      { data: { email } }
    );
    expect(beginResetResponse.status()).toBeLessThan(400);

    const message = await getEmailFromMailpit(request, email, "Reset your Hook0 password");
    const html = message.HTML ?? "";

    expect(html, "personalised greeting").toContain(firstName);
    expect(html, "CTA label says Reset password").toContain("Reset password");
    // Anti-regression: the legacy template shipped a "Verify email" button on
    // the reset-password mail.
    expect(html, "CTA label must NOT say Verify email").not.toContain(
      ">Verify email<"
    );
    expect(html, "5-minute expiration mention").toContain("5 minutes");
    expect(html, "no-action reassurance").toMatch(/no action is needed/i);
    assertCommonFooter(html, "reset_password");
    assertPlainTextWidth(message);
  });

  test("Logo image is fetchable (anti-regression for the LFS pointer bug)", async ({
    request,
  }) => {
    // The logo URL is the same constant injected into every email by the
    // backend. We assert on the live URL to catch a regression where the
    // website would silently revert to serving an LFS pointer text file.
    const logoUrl = "https://www.hook0.com/mediakit/logo/256x256.png";
    const response = await request.get(logoUrl);
    expect(response.status(), "logo HTTP status").toBe(200);
    expect(response.headers()["content-type"]).toContain("image/png");
    const body = await response.body();
    expect(
      body.length,
      "logo size must be a real PNG (>1 KB), not a 130-byte LFS pointer"
    ).toBeGreaterThan(1000);
  });

  // The two quota templates need:
  //   - QUOTA_BASED_EMAIL_NOTIFICATIONS=true on the API
  //   - A way to push the org's events_per_day above the warning threshold
  //     (default 80%) or to the hard limit.
  // This is doable but slow (requires sending dozens of real events). Pending
  // CI fixture work, these are added as design tests behind .skip so the spec
  // file documents the expectations and is ready to enable.
  test.skip("Quota warning — design (TODO: requires quota fixture)", async ({
    request,
  }) => {
    const email = "skipped@hook0.local";
    const message = await getEmailFromMailpit(request, email, "% of your daily events");
    const html = message.HTML ?? "";
    expect(html).toContain("Upgrade your plan");
    expect(html).toContain("/dashboard");
    assertCommonFooter(html, "quota_warning");
  });

  test.skip("Quota reached — design (TODO: requires quota fixture)", async ({
    request,
  }) => {
    const email = "skipped@hook0.local";
    const message = await getEmailFromMailpit(request, email, "Events paused");
    const html = message.HTML ?? "";
    expect(html).toContain("Upgrade now to resume");
    expect(html).toContain("events paused");
    assertCommonFooter(html, "quota_reached");
  });
});
