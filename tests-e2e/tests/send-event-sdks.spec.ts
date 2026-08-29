import { test, expect, type Locator, type Page } from "@playwright/test";
import {
  loginAndCreateApp,
  loginAndCreateAppWithEventType,
  loginAsNewUser,
  submitEventWithLabels,
  API_BASE_URL,
  type TestEnv,
} from "../fixtures/test-setup";
import { captureTrackingQueue, trackedEvents } from "../fixtures/tracking";
import { HOOK0_SDKS, type Hook0Sdk } from "../../frontend/src/generated/sdkExamples";
import { isPlainTextLanguage } from "../../frontend/src/components/Hook0CodeColouring";
import type { Hook0CodeLanguage } from "../../frontend/src/components/Hook0Code";

/**
 * "Send an event", held against every SDK the registry declares.
 *
 * The languages are read from the generated artefact rather than written here. That is the whole
 * point: a twelfth SDK is covered on the day it is added, by whoever adds it, without anybody
 * remembering to come back to this file. A list written here would be the one place the registry
 * does not reach, and it would go stale silently — which is the defect this screen was rebuilt to
 * remove, not one to reintroduce in its test.
 *
 * So every assertion below is a property that holds for all of them: no marker survives, the token
 * is the real one, the three blocks are all there, and what the copy button yields is what the
 * screen shows.
 *
 * One limit, stated rather than papered over: the snippets are rendered by CodeMirror, which is
 * free to keep only the visible lines in the DOM. A `not.toContainText("__HOOK0_")` is therefore a
 * lower bound — it cannot see a marker below the fold. Two things answer that. The tail check
 * below asserts the snippet's own last line is rendered, so a truncated render fails loudly
 * instead of passing vacuously; and the clipboard check reads the whole document, since the copy
 * button copies the editor's value rather than its DOM.
 */

/** How long a panel is given to render. The snippets are static, so this is generous already. */
const PANEL_TIMEOUT = 15000;

/**
 * A line from deep in a snippet that a rendered panel must contain.
 *
 * Finding it proves the whole document reached the DOM rather than only its first screenful, which
 * is what makes the marker assertion beside it mean anything. It is derived from the artefact, so
 * it stays right for a language nobody here has heard of.
 *
 * The length floor is the part that took measuring. Taking simply the last marker-free line gives
 * a bare `}` for six of the eleven — an assertion that passes against any rendered code at all,
 * which is no assertion. Requiring some substance instead yields a real line for all eleven, and
 * one sitting at or near the end of every one of them: the furthest from the bottom is C#, at line
 * 20 of 26, which is still well past anything a first screenful would hold.
 */
const DISTINCTIVE_LENGTH = 12;

function tailOf(snippet: string): string {
  const lines = snippet
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length >= DISTINCTIVE_LENGTH && !line.includes("__HOOK0_"));
  return lines.length > 0 ? lines[lines.length - 1] : "";
}

/** The token the application was provisioned with, which every snippet must carry. */
async function provisionedToken(
  request: import("@playwright/test").APIRequestContext,
  email: string,
  password: string,
  applicationId: string
): Promise<string> {
  const login = await request.post(`${API_BASE_URL}/auth/login`, { data: { email, password } });
  expect(login.status(), "the test user must be able to log in").toBeLessThan(400);
  const accessToken = (await login.json()).access_token;

  const secrets = await request.get(
    `${API_BASE_URL}/application_secrets?application_id=${applicationId}`,
    { headers: { Authorization: `Bearer ${accessToken}` } }
  );
  expect(secrets.status(), "the application must expose its secrets").toBeLessThan(400);
  const token = ((await secrets.json()) as Array<{ token: string }>)[0].token;
  expect(token, "an application is provisioned with a secret when it is created").toBeTruthy();
  return token;
}

/**
 * Opens a language, the way a reader would.
 *
 * Which control carries it is read off the page rather than declared here: a language given its own
 * tab is clicked, and one behind the picker is selected. That keeps the split between the two — a
 * presentation decision that is expected to move as the usage figures are revisited — out of this
 * file entirely.
 */
async function openLanguage(page: Page, target: string): Promise<Locator> {
  const tab = page.locator(`[data-test="send-event-tab-${target}"]`);
  if ((await tab.count()) > 0) {
    await tab.click();
  } else {
    await page.locator('[data-test="send-event-language-select"]').selectOption(target);
  }
  const panel = page.locator(`[data-test="send-event-panel-${target}"]`);
  await expect(panel).toBeVisible({ timeout: PANEL_TIMEOUT });
  return panel;
}

/**
 * What the tab strip and the panel it opens say about each other, read in one pass over the page.
 *
 * Read inside the page rather than through locators, for two reasons. An id is not a selector:
 * looking one up as `#value` breaks on a value carrying a character CSS reads specially, and the
 * assertion then fails for a reason that has nothing to do with the property it was written for.
 * And one read sees the tabs and the panel in the same state, which two queries against a page
 * that is still settling cannot promise.
 *
 * The faults come back named rather than counted, so a failure says which tab and which id.
 */
type Hook0TabStrip = {
  readonly tabs: number;
  readonly focusableInTablist: number;
  readonly faults: string[];
};

function tabStripOf(page: Page): Promise<Hook0TabStrip> {
  return page.evaluate(() => {
    const readable = (value: string | null): string => (value === null ? "" : value.trim());

    const tablist = document.querySelector('[role="tablist"]');
    const tabs = Array.from(document.querySelectorAll('[role="tab"]'));
    const faults: string[] = [];

    if (tablist === null) {
      faults.push("the screen carries no tablist");
    }

    for (const tab of tabs) {
      const named = readable(tab.textContent);

      const controls = readable(tab.getAttribute("aria-controls"));
      if (controls.length === 0) {
        faults.push(`the ${named} tab does not say which panel it opens`);
        continue;
      }

      const panel = document.getElementById(controls);
      if (panel === null) {
        faults.push(`the ${named} tab opens "${controls}", which is not on the page`);
        continue;
      }
      if (panel.getAttribute("role") !== "tabpanel") {
        faults.push(`the ${named} tab opens "${controls}", which is not a tabpanel`);
        continue;
      }

      const labelledBy = readable(panel.getAttribute("aria-labelledby"));
      if (labelledBy.length === 0) {
        // No tab opened this panel, so nothing can name it but itself.
        if (readable(panel.getAttribute("aria-label")).length === 0) {
          faults.push(`the panel the ${named} tab opens is named by nothing at all`);
        }
        continue;
      }

      const namer = document.getElementById(labelledBy);
      if (namer === null || namer.getAttribute("role") !== "tab") {
        faults.push(`the open panel is named by "${labelledBy}", which is not a tab on the page`);
        continue;
      }
      if (namer.getAttribute("aria-selected") !== "true") {
        faults.push(
          `the open panel is named by the ${readable(namer.textContent)} tab, which is not the open one`
        );
      }
    }

    const focusableInTablist =
      tablist === null ? 0 : tablist.querySelectorAll('[role="tab"][tabindex="0"]').length;

    return { tabs: tabs.length, focusableInTablist, faults };
  });
}

/** How long the labels editor waits after a keystroke before handing its rows up. */
const KV_DEBOUNCE_MS = 400;

/** How long a query stays fresh, from `frontend/src/plugins/query.ts`. */
const STALE_MS = 30000;

/** The two calls every panel on this screen is written against. */
const EVENT_TYPES_URL = "**/api/v1/event_types*";
const SECRETS_URL = "**/api/v1/application_secrets*";

/**
 * What the one panel is showing, counted in a single read.
 *
 * Counted rather than waited for. A test that waits for one element fails with "not found" and says
 * neither what went wrong nor what the reader saw; these numbers are the bug written out — an error
 * card standing where a form was, a skeleton that never gave way, an example printed beside a
 * refusal.
 */
type Hook0PanelState = Record<string, number>;

function panelState(page: Page): Promise<Hook0PanelState> {
  return page.evaluate(() => {
    const count = (selector: string) => document.querySelectorAll(selector).length;
    return {
      errorCards: count('[data-test="error-card"]'),
      skeletons: count('[data-test="send-event-loading"]'),
      forms: count('[data-test="send-event-form"]'),
      commands: count('[data-test="send-event-curl-panel"] .cm-content'),
      snippets: count('[data-test="send-event-send"]'),
      empty: count('[data-test="send-event-no-event-type"]'),
      unsendable: count('[data-test="send-event-unsendable"]'),
      secretsRefused: count('[data-test="send-event-secrets-error"]'),
      instanceConfigError: count('[data-test="send-event-instance-config-error"]'),
      secretNotAccepted: count('[data-test="send-event-secret-not-accepted"]'),
      tutorialToken: count('[data-test="send-event-tutorial-token"]'),
    };
  });
}

/** Only the keys asked for, so an assertion says what it is about and nothing else. */
async function panelStateOf(page: Page, keys: readonly string[]): Promise<Hook0PanelState> {
  const whole = await panelState(page);
  const asked: Hook0PanelState = {};
  for (const key of keys) {
    asked[key] = whole[key];
  }
  return asked;
}

/** What names the one panel when no tab does. */
function panelSelfName(page: Page): Promise<string> {
  return page.evaluate(() => {
    const panel = document.getElementById("send-event-tabpanel");
    if (panel === null) {
      return "";
    }
    const named = panel.getAttribute("aria-label");
    return named === null ? "" : named;
  });
}

/** The languages the picker holds, read off the page rather than declared here. */
async function languagesBehindThePicker(page: Page): Promise<string[]> {
  const behind: string[] = [];
  for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
    if ((await page.locator(`[data-test="send-event-tab-${sdk.target}"]`).count()) === 0) {
      behind.push(sdk.target);
    }
  }
  expect(
    behind.length,
    "the picker must hold a language, or no panel names itself"
  ).toBeGreaterThan(0);
  return behind;
}

/**
 * The tutorial wizard from its first screen to the send step, walked on one SPA session so the use
 * case chosen at the intro is the one the send form is seeded from. A reload would put that use case
 * back to the neutral one, which fills the form with a preset the application may not have an event
 * type for. A caller whose send step needs a read to fail registers that route in
 * `beforeSubscriptionSubmit`, which runs once the subscription form is filled and before it is
 * submitted, so the wizard's own creations still go through.
 */
async function walkTutorialToSendScreen(
  page: Page,
  env: TestEnv,
  options: { eventTypeName: string; beforeSubscriptionSubmit?: () => Promise<void> }
): Promise<void> {
  const parts = options.eventTypeName.split(".");
  expect(parts.length, "an event type name is a service, a resource and a verb").toBe(3);
  const [service, resource, verb] = parts;

  await page.setViewportSize({ width: 1280, height: 1024 });

  await page.goto("/tutorial");
  await expect(page.locator('[data-test="tutorial-usecase-ecommerce"]')).toBeVisible({
    timeout: PANEL_TIMEOUT,
  });
  await page.locator('[data-test="tutorial-usecase-ecommerce"]').click();
  await page.locator('[data-test="tutorial-start-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/application/, { timeout: 15000 });
  const createAppRadio = page.locator('[data-test="tutorial-create-app-radio"]');
  if (await createAppRadio.isVisible({ timeout: 3000 }).catch(() => false)) {
    await createAppRadio.click();
  }
  const appName = page.locator('[data-test="application-name-input"]');
  await expect(appName).toBeVisible({ timeout: 20000 });
  await appName.fill(`Tutorial App ${env.timestamp}`);
  await page.locator('[data-test="application-submit-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/event_type/, { timeout: 15000 });
  await expect(page.locator('[data-test="event-type-service-input"]')).toBeVisible({
    timeout: PANEL_TIMEOUT,
  });
  await page.locator('[data-test="event-type-service-input"]').fill(service);
  await page.locator('[data-test="event-type-resource-input"]').fill(resource);
  await page.locator('[data-test="event-type-verb-input"]').fill(verb);
  await page.locator('[data-test="event-type-submit-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/subscription/, { timeout: 15000 });
  await page.locator('[data-test="subscription-description-input"]').fill("Tutorial webhook");
  await page.locator('[data-test="subscription-url-input"]').fill("https://example.com/webhook");
  const labels = page.locator('[data-test="subscription-labels"]');
  await expect(labels.locator('[data-test="kv-key-input-0"]')).toBeVisible({
    timeout: PANEL_TIMEOUT,
  });
  await labels.locator('[data-test="kv-key-input-0"]').fill("env");
  await labels.locator('[data-test="kv-value-input-0"]').fill("test");
  const eventTypeCheckbox = page.locator('[data-test="event-type-checkbox-0"]');
  await expect(eventTypeCheckbox).toBeVisible({ timeout: PANEL_TIMEOUT });
  if (!(await eventTypeCheckbox.isChecked())) {
    await eventTypeCheckbox.click();
  }

  const beforeSubmit = options.beforeSubscriptionSubmit;
  if (beforeSubmit !== undefined) {
    await beforeSubmit();
  }
  await page.locator('[data-test="subscription-submit-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/event/, { timeout: 15000 });
  await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
    timeout: PANEL_TIMEOUT,
  });
}

/** How long the event types are held back for, to read the panel while it is still a skeleton. */
const HELD_MS = 4000;

test.describe("Send an event — every SDK", () => {
  /**
   * One registration for the whole sweep rather than one per language.
   *
   * Eleven registrations would each cost a mailbox round trip for the verification link, and would
   * measure the sign-up path eleven times over rather than the screen this file is about.
   */
  async function onTheSendScreen(
    page: Page,
    request: import("@playwright/test").APIRequestContext,
    testId: string
  ): Promise<{ token: string }> {
    const env = await loginAndCreateAppWithEventType(page, request, testId);
    const token = await provisionedToken(request, env.email, env.password, env.applicationId);

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    return { token };
  }

  test("shows install, send and verify for every declared SDK, with no marker left", async ({
    page,
    request,
  }) => {
    // A guard on the artefact itself: an empty registry would make every assertion below vacuous,
    // and the suite would go green having checked nothing at all.
    expect(
      HOOK0_SDKS.length,
      "the generated artefact must declare at least one SDK"
    ).toBeGreaterThan(0);

    const { token } = await onTheSendScreen(page, request, "sdk-sweep");

    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);

        // The three blocks a reader needs, in the order they are used. Verify is the one the
        // onboarding used to stop short of, so its absence is a regression worth naming.
        await expect(panel.locator('[data-test="send-event-install"]')).toBeVisible();
        await expect(panel.locator('[data-test="send-event-send"]')).toBeVisible();
        await expect(panel.locator('[data-test="send-event-verify"]')).toBeVisible();

        // The application was created moments ago, so it has its default secret: the screen must
        // be showing snippets, not the warning that stands in for them.
        await expect(panel.locator('[data-test="send-event-no-secret"]')).toHaveCount(0);

        // What proves the whole snippet reached the DOM, and so what makes the marker assertion
        // below mean something.
        const tail = tailOf(sdk.send.body);
        expect(
          tail,
          `${sdk.target}: its send example must carry a substantial line free of markers`
        ).not.toBe("");
        await expect(panel.locator('[data-test="send-event-send"]')).toContainText(tail);

        // No marker may reach a reader. A snippet carrying one is not code, and pasting it fails
        // in a way that reads as Hook0 being broken.
        await expect(panel).not.toContainText("__HOOK0_");

        // The real credential, not a placeholder and not an empty one. `api-keys.spec.ts` pins the
        // same property for cURL, where an absent secret renders as `Bearer ` with nothing after
        // it; here the token stands on its own, so its presence is the assertion.
        await expect(panel.locator('[data-test="send-event-send"]')).toContainText(token);
      });
    }
  });

  test("copies exactly what it displays", async ({ page, request, browserName }) => {
    // Chromium alone: reading the clipboard back needs a permission Firefox and WebKit do not
    // grant to a test, and a page that faked `navigator.clipboard` would be measuring the fake.
    // The rest of this file runs everywhere; this one property is checked where it can be.
    test.skip(
      browserName !== "chromium",
      "reading the clipboard back requires a permission only Chromium grants"
    );
    await page.context().grantPermissions(["clipboard-read", "clipboard-write"]);

    const { token } = await onTheSendScreen(page, request, "sdk-copy");
    const first = HOOK0_SDKS[0] as Hook0Sdk;
    const panel = await openLanguage(page, first.target);

    const block = panel.locator('[data-test="send-event-send"]');
    await block.locator(".hook0-code-copy").click();

    const copied = await page.evaluate(() => navigator.clipboard.readText());

    // The clipboard carries the editor's whole value, which is what makes these three stronger
    // than the same assertions against the DOM: nothing here can be hidden below a fold.
    expect(copied, "the copy must carry the snippet, not an empty string").not.toBe("");
    expect(copied, "no marker may survive into what a reader pastes").not.toContain("__HOOK0_");
    expect(copied, "the copy must carry the real token").toContain(token);

    // And it must be the snippet on screen rather than some other one: every line the panel shows
    // has to appear in what was copied. Written this way round because the DOM may hold fewer
    // lines than the document, never more.
    const shown = (await block.locator(".cm-line").allInnerTexts())
      .map((line) => line.trim())
      .filter((line) => line.length > 0);
    expect(shown.length, "the panel must render the snippet it copied").toBeGreaterThan(0);
    for (const line of shown) {
      expect(copied, `the copy must contain the line the panel shows: ${line}`).toContain(line);
    }
  });

  test("keeps the fragments that were shared before this screen had twelve panels", async ({
    page,
    request,
  }) => {
    // `#js` and `#rust` sit in tickets, in emails and in the browser history of everyone who used
    // this screen before it was rebuilt. They are not derived from the registry — `#js` names no
    // target at all — so they are written out here, which is the only list this file carries and
    // the only one it should.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-hash");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(`${base}#js`);
    await expect(
      page.locator('[data-test="send-event-panel-typescript"]'),
      "#js opened the JavaScript snippet before the rename to the registry's target name"
    ).toBeVisible({ timeout: PANEL_TIMEOUT });

    await page.goto(`${base}#rust`);
    await expect(page.locator('[data-test="send-event-panel-rust"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    // An unknown fragment lands on the form rather than on a blank panel, which is what a stale
    // link to a language that has since been renamed would do.
    await page.goto(`${base}#no-such-language`);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
  });

  test("moves between tabs with the arrow keys, Home and End", async ({ page, request }) => {
    // Nothing covers this today. A tablist that cannot be driven from the keyboard is unusable
    // with a screen reader, and the failure is silent: the tabs still look right.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-keys");
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    // Read off the page rather than declared, so the featured languages can be reordered without
    // touching this test.
    const tabs = page.locator('[role="tab"]');
    const count = await tabs.count();
    expect(
      count,
      "the screen must offer more than one tab for arrows to mean anything"
    ).toBeGreaterThan(1);

    /** A tab bar says which tab is open through `aria-selected`, and that is what is asserted. */
    async function expectOpen(index: number, said: string) {
      await expect(tabs.nth(index), said).toHaveAttribute("aria-selected", "true");
      await expect(tabs.nth(index), `${said}, and focus follows it`).toBeFocused();
    }

    await tabs.first().focus();
    await page.keyboard.press("ArrowRight");
    await expectOpen(1, "ArrowRight opens the next tab");

    await page.keyboard.press("ArrowLeft");
    await expectOpen(0, "ArrowLeft opens the previous one");

    // Wrapping, in both directions: a tablist that stops at the ends makes the last tab reachable
    // only by going the long way round.
    await page.keyboard.press("ArrowLeft");
    await expectOpen(count - 1, "ArrowLeft from the first wraps to the last");

    await page.keyboard.press("ArrowRight");
    await expectOpen(0, "ArrowRight from the last wraps to the first");

    await page.keyboard.press("End");
    await expectOpen(count - 1, "End opens the last tab");

    await page.keyboard.press("Home");
    await expectOpen(0, "Home opens the first tab");
  });

  test("reports an opening and a copy to analytics, for every language", async ({
    page,
    request,
  }) => {
    // The order the languages are offered in is meant to be revisited on these figures, so a
    // language whose panel reports nothing is a language that will look unused however often it is
    // opened. The calls were there; nothing had ever run them.
    await captureTrackingQueue(page);
    await onTheSendScreen(page, request, "sdk-matomo");

    expect(HOOK0_SDKS.length, "the artefact must declare at least one SDK").toBeGreaterThan(0);

    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await panel.locator('[data-test="send-event-send"] [data-test="code-copy"]').click();
      });
    }

    const reported = await trackedEvents(page);
    const category = "send-event";
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      expect(
        reported.some((e) => e[1] === category && e[2] === "open" && e[3] === sdk.target),
        `opening ${sdk.target} must reach analytics`
      ).toBe(true);
      expect(
        reported.some((e) => e[1] === category && e[2] === "copy" && e[3] === `${sdk.target}:send`),
        `copying the ${sdk.target} send snippet must reach analytics, and say which block`
      ).toBe(true);
    }
  });

  test("says so, in every panel, when the application has no secret left", async ({
    page,
    request,
  }) => {
    // The state nothing had ever rendered. An application is provisioned with a secret when it is
    // created, and deleting the last one is not refused, so this is reachable by a reader doing
    // nothing unusual. What must not happen is a snippet built on a credential the screen does not
    // have: `Bearer ` with nothing after it reads as working code and fails on the first call.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-nosecret");

    const login = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email: env.email, password: env.password },
    });
    expect(login.status(), "the test user must be able to log in").toBeLessThan(400);
    const auth = { Authorization: `Bearer ${(await login.json()).access_token}` };

    const listed = await request.get(
      `${API_BASE_URL}/application_secrets?application_id=${env.applicationId}`,
      { headers: auth }
    );
    expect(listed.status(), "the application must expose its secrets").toBeLessThan(400);
    const secrets = (await listed.json()) as Array<{ token: string }>;
    expect(secrets.length, "an application is provisioned with a secret").toBeGreaterThan(0);

    for (const secret of secrets) {
      const revoked = await request.delete(
        `${API_BASE_URL}/application_secrets/${secret.token}?application_id=${env.applicationId}`,
        { headers: auth }
      );
      expect(revoked.status(), "deleting a secret must succeed").toBeLessThan(400);
    }

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    // cURL first, because it is the panel where an empty credential is visible as text rather than
    // as a missing argument.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl.locator('[data-test="send-event-no-secret"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(
      curl,
      "no command may be shown without the token it authenticates with"
    ).not.toContainText("Bearer");

    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await expect(panel.locator('[data-test="send-event-no-secret"]')).toBeVisible();
        await expect(
          panel.locator('[data-test="send-event-send"]'),
          `${sdk.target} must show no snippet when there is nothing to authenticate it with`
        ).toHaveCount(0);
      });
    }
  });

  test("colours every language it has a grammar for, and shows the others as they are", async ({
    page,
    request,
  }) => {
    // Every language meant to colour loads a CodeMirror grammar through an unguarded dynamic import,
    // and a grammar set to null or a renamed dependency drops it silently to plain text — which is
    // indistinguishable from the one language declared plain unless the paint itself is measured.
    // Nothing but this test measures it, and it did so for python alone; the other nine could each
    // have regressed with every assertion in this file still green. So each shown language is driven
    // and its highlight tokens counted, and zig — the one declared plain — is the negative control.
    await onTheSendScreen(page, request, "sdk-grammar");

    const tokensIn = async (target: string): Promise<number> => {
      const panel = await openLanguage(page, target);
      return panel.locator('[data-test="send-event-send"] .cm-line span').count();
    };

    // Which languages colour and which stay plain is read from the same declaration the component
    // builds its grammar map from, so a language moving between the two is not a change this test has
    // to be told about.
    let couldColour = 0;
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const tokens = await tokensIn(sdk.target);
        // The artefact types a target as a plain string, the same boundary the screen crosses with a
        // cast where it opens a panel; every shipped target is one of the block's languages.
        if (isPlainTextLanguage(sdk.target as Hook0CodeLanguage)) {
          expect(
            tokens,
            `${sdk.target} is declared plain, so its snippet carries no highlight token`
          ).toBe(0);
        } else {
          expect(
            tokens,
            `${sdk.target} has a grammar, so its snippet must render highlight tokens`
          ).toBeGreaterThan(0);
          couldColour += 1;
        }
      });
    }
    expect(
      couldColour,
      "the screen must colour at least one language, or this proves nothing"
    ).toBeGreaterThan(0);

    // A plain render is still a render: the fallback must show the snippet, not break into an empty
    // panel. Zig is the language declared plain, so it is where that is checked.
    const zigPanel = page.locator('[data-test="send-event-panel-zig"]');
    await expect(
      zigPanel.locator('[data-test="send-event-send"]'),
      "uncoloured still means shown"
    ).toContainText("hook0");
  });

  test("keeps exactly one tab focusable, whichever panel is open", async ({ page, request }) => {
    // A tablist is entered with Tab and walked from the inside with the arrow keys, which is why
    // exactly one of its tabs carries tabindex="0" at any moment. Tying that to the open panel
    // alone leaves none of them focusable while one of the languages behind the picker is open:
    // the strip drops out of the page's tab order altogether, and nothing inside it can put it
    // back, because the arrow handler only ever fires from a tab that already holds the focus. A
    // reader driving the keyboard opens a language and can no longer reach the form.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-roving");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const tabs = page.locator('[role="tab"]');
    const focusable = page.locator('[role="tab"][tabindex="0"]');

    // What keeps the counts below from being satisfied by a selector that matches nothing at all.
    expect(
      await tabs.count(),
      "the screen must offer a strip of tabs to begin with"
    ).toBeGreaterThan(1);
    await expect(focusable, "the form's own tab is open, so it is the focusable one").toHaveCount(
      1
    );

    // Which languages sit behind the picker is read off the page rather than declared, so moving
    // one of them into the strip is not a change this test has to be told about.
    const behindThePicker: string[] = [];
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      if ((await page.locator(`[data-test="send-event-tab-${sdk.target}"]`).count()) === 0) {
        behindThePicker.push(sdk.target);
      }
    }
    expect(
      behindThePicker.length,
      "the picker must hold a language for this property to be worth checking"
    ).toBeGreaterThan(0);

    for (const target of behindThePicker) {
      await test.step(target, async () => {
        await page.locator('[data-test="send-event-language-select"]').selectOption(target);
        await expect(page.locator(`[data-test="send-event-panel-${target}"]`)).toBeVisible({
          timeout: PANEL_TIMEOUT,
        });
        expect(await tabs.count(), `${target}: the strip is still on screen`).toBeGreaterThan(1);
        await expect(
          focusable,
          `${target} is open and no tab stands for it, so the strip must keep one focusable tab`
        ).toHaveCount(1);
      });
    }

    // The same state reached from a shared link instead of from the picker: a fragment opens the
    // language directly, and no tab is selected there either.
    const shared = behindThePicker[0];
    await page.goto(`${base}#${shared}`);
    await expect(page.locator(`[data-test="send-event-panel-${shared}"]`)).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(
      focusable,
      "a fragment opening a picker language must leave the strip one focusable tab"
    ).toHaveCount(1);
  });

  test("offers to define an event type rather than a form that cannot be filled", async ({
    page,
    request,
  }) => {
    // An application that has just been created has no event type, and the empty events list sends
    // a reader straight here from it. The form this screen used to show cannot be submitted: its
    // only select holds no option at all, and the button refuses with "fill all required fields"
    // while every field on screen is filled.
    const env = await loginAndCreateApp(page, request, "sdk-noeventtype");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const empty = page.locator('[data-test="send-event-no-event-type"]');
    await expect(empty, "an application with no event type must be told so").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(
      page.locator('[data-test="send-event-form"]'),
      "no form while its only select would hold nothing to choose"
    ).toHaveCount(0);

    // The way out has to lead somewhere, and it is the only way out this panel offers.
    await empty.locator('[data-test="send-event-create-event-type-button"]').click();
    await expect(page).toHaveURL(/\/event_types\/new$/, { timeout: PANEL_TIMEOUT });
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await page.locator('[data-test="event-type-submit-button"]').click();
    await expect(page).toHaveURL(/\/event_types$/, { timeout: PANEL_TIMEOUT });

    // Which is what makes the two counts above statements about this application rather than about
    // a pair of selectors that match nothing: with an event type defined, the same screen shows
    // the form, and the panel that stood in for it is gone.
    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(
      empty,
      "the panel belongs to the application with no event type, not to the screen"
    ).toHaveCount(0);
  });

  test("wires every tab to the panel it opens, in each state that panel has", async ({
    page,
    request,
  }) => {
    // The wiring is checked as a property rather than against a list of ids, because a list is the
    // thing that goes stale: a panel state added later would carry no `role`, no id and no name
    // back, and nothing written here would notice. Every state the panel has is driven below, so a
    // sixth one arriving unwired fails here rather than in front of a reader.
    const env = await loginAndCreateApp(page, request, "sdk-aria");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;
    const eventTypes = "**/api/v1/event_types*";

    /** The strip, held to the property, in whichever state the screen is in. */
    async function expectWired(state: string, tabs: number) {
      const strip = await tabStripOf(page);
      expect(strip.tabs, `${state}: the strip must still hold every tab`).toBe(tabs);
      expect(
        strip.tabs,
        `${state}: a wiring check over no tab at all checks nothing`
      ).toBeGreaterThan(0);
      expect(strip.faults, `${state}: every tab opens a tabpanel that names it back`).toEqual([]);
      expect(strip.focusableInTablist, `${state}: a tablist holds one focusable tab`).toBe(1);
    }

    // This application has no event type, so the panel opens on the state that says so.
    await page.goto(base);
    await expect(page.locator('[data-test="send-event-no-event-type"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    // Which languages carry a tab and which sit behind the picker is read off the page, so the
    // split can be revisited without this test hearing about it.
    const featured: string[] = [];
    const behindThePicker: string[] = [];
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      const hasTab = (await page.locator(`[data-test="send-event-tab-${sdk.target}"]`).count()) > 0;
      (hasTab ? featured : behindThePicker).push(sdk.target);
    }
    expect(
      behindThePicker.length,
      "the picker must hold a language, or the state where no tab is selected is unreachable"
    ).toBeGreaterThan(0);

    // The number, stated: the featured languages plus the two panels no SDK stands for — the form
    // and cURL, which `sendEventSnippets.ts` names `FORM_PANEL` and `CURL_PANEL`. Written out
    // rather than read back off the same query the check below runs, which would agree with itself
    // whatever the strip held.
    const NON_LANGUAGE_TABS = 2;
    const tabs = await page.locator('[role="tab"]').count();
    expect(tabs, "the strip carries the featured languages, the form and cURL").toBe(
      featured.length + NON_LANGUAGE_TABS
    );
    expect(tabs, "a wiring check over no tab at all checks nothing").toBeGreaterThan(0);
    await expectWired("the empty state", tabs);

    // The same state on a code tab, where a tab is selected and names the panel back. It reaches
    // every panel because every panel is written against the event types: the form offers them and
    // the examples name one, so an application that has none has nothing to show anywhere.
    const empty = page.locator('[data-test="send-event-no-event-type"]');
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await expect(empty, "cURL has no event type to name either").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the empty state under the cURL tab", tabs);

    // And on a language from the picker, where no tab is selected and the panel names itself.
    const picked = behindThePicker[0];
    await page.locator('[data-test="send-event-language-select"]').selectOption(picked);
    await expect(empty, `${picked} has no event type to name either`).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the empty state under a language the picker holds", tabs);

    // With an event type defined, the same panel becomes the form.
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await page.locator('[data-test="event-type-submit-button"]').click();
    await expect(page).toHaveURL(/\/event_types$/, { timeout: PANEL_TIMEOUT });

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the form", tabs);

    // A language from the picker: no tab is selected, so the panel has to name itself.
    await page.locator('[data-test="send-event-language-select"]').selectOption(picked);
    await expect(page.locator(`[data-test="send-event-panel-${picked}"]`)).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("a language opened from the picker", tabs);

    // cURL: a code panel a tab does open, so this one is named by that tab rather than by itself.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await expect(page.locator('[data-test="send-event-curl-panel"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the cURL panel", tabs);

    // The panel the fetch never came back for, on the form's tab and then on a code tab. The second
    // is a move between two fragments of one URL, so the app stays mounted and the fetch is not
    // made again: what is checked is that the same state carries across to a panel that used to
    // fall through it, and that the tab now open names it.
    await page.route(eventTypes, (route) =>
      route.fulfill({ status: 500, contentType: "application/json", body: "{}" })
    );
    await page.goto(base);
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: PANEL_TIMEOUT });
    await expectWired("the error card", tabs);

    await page.goto(`${base}#curl`);
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: PANEL_TIMEOUT });
    await expectWired("the error card under the cURL tab", tabs);
    await page.unroute(eventTypes);

    // And the panel while that fetch is still out. Held for a bounded stretch rather than
    // indefinitely, so the request always completes and the page is never left mid-flight.
    await page.route(eventTypes, async (route) => {
      await new Promise((held) => setTimeout(held, HELD_MS));
      await route.continue();
    });
    await page.goto(base);
    await expect(page.locator('[data-test="send-event-loading"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the loading skeleton", tabs);

    await page.goto(`${base}#curl`);
    await expect(page.locator('[data-test="send-event-loading"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expectWired("the loading skeleton under the cURL tab", tabs);
    await page.unroute(eventTypes);
  });

  test("lets the keyboard back out of the payload editor", async ({ page, request }) => {
    // The editor is the last field of the form and Cancel and Send sit right after it, so a Tab
    // that the editor keeps for itself is the end of the road for anyone not using a mouse. Which
    // of the two happens is a question about a key binding in a browser, so it is asked here
    // rather than answered from a manual.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-tabout");
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const editor = page.locator('[data-test="send-event-payload-input"] .cm-content');
    await editor.click();
    await expect(
      editor,
      "the payload editor must hold the focus, or pressing Tab measures something else"
    ).toBeFocused();

    await page.keyboard.press("Tab");
    await expect(
      editor,
      "Tab must leave the payload editor rather than indent inside it"
    ).not.toBeFocused();
    await expect(
      page.locator('[data-test="send-event-cancel-button"]'),
      "and it must land on the control that follows the editor"
    ).toBeFocused();

    // `Hook0Code` builds the snippets with the same binding turned on, and they are left that way.
    // This is the reason: they are built read-only, and a read-only CodeMirror is not somewhere the
    // focus can land at all, so the binding has nothing there to keep.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    const snippet = page.locator('[data-test="send-event-curl-panel"] .cm-content');
    await expect(snippet, "the cURL snippet must be on screen").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    const reachable = await snippet.evaluate((node) => ({
      editable: node.getAttribute("contenteditable"),
      tabIndex: (node as HTMLElement).tabIndex,
    }));
    expect(reachable, "a read-only snippet is not in the page's tab order").toEqual({
      editable: "false",
      tabIndex: -1,
    });
  });

  test("writes its examples against the origin the page is talking to", async ({
    page,
    request,
  }) => {
    // The dashboard reads an API endpoint off its own query string, validates it against an
    // allowlist and uses it for every call it makes; `self-hosting/bare-metal.md` documents that as
    // the way to work against another server. The examples were reading the value frozen into the
    // bundle instead, so under that override the Send button posted to one origin while all twelve
    // snippets printed another.
    //
    // The allowlist admits the origin this bundle was built with and nothing else, so the only
    // override this test can make is the same origin written with a trailing slash. Narrow, and
    // enough: the two readers disagree about the whole value, so anything that differs at all
    // tells them apart. A wider one would mean setting `VITE_ALLOWED_API_ORIGINS` in the build,
    // which `frontend/.gitlab-ci.yml` does not.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-endpoint");
    const overridden = `${API_BASE_URL}/`;

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send` +
        `?API_ENDPOINT=${encodeURIComponent(overridden)}#curl`
    );

    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });
    await expect(curl, "the panel must be showing a command at all").toContainText("curl -X POST");

    // The snippet writes `<base>/event`, and this base already ends in a slash — so the command
    // carries both. That doubled slash is the whole tell: the value frozen into the bundle has no
    // trailing slash, so a snippet built from it cannot contain this string.
    await expect(
      curl,
      "the example must name the endpoint this page is using, not the one it was built with"
    ).toContainText(`${overridden}/event`);
  });

  test("refuses a blank label rather than printing one it will not send", async ({
    page,
    request,
  }) => {
    // The examples were built from the rows as typed and the request from a trimmed copy of them,
    // so a label of nothing but spaces was printed in all twelve examples and then dropped on the
    // way out — and when it was the only label, the send was refused by a button that had never
    // looked disabled. What is on screen and what goes over the wire are one value now, and a row
    // that cannot be sent is named as such instead of vanishing.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-blank-label");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const value = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    const labelsError = page.locator('[data-test="send-event-labels-error"]');
    const submit = page.locator('[data-test="send-event-submit-button"]');

    /** What the cURL example says, which is the one panel where the labels are readable as text. */
    async function openCurl() {
      await page.locator('[data-test="send-event-tab-curl"]').click();
      await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });
    }

    async function openForm() {
      await page.locator('[data-test="send-event-tab-easy"]').click();
      await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
        timeout: PANEL_TIMEOUT,
      });
    }

    // The seeded label, in the example. Without this the two absences below would be satisfied by
    // a panel that never shows a label at all.
    await openCurl();
    await expect(curl, "the example carries the label the form was seeded with").toContainText(
      '"user_id": "1"'
    );

    // A value of nothing but spaces.
    await openForm();
    await value.fill("   ");
    await expect(labelsError, "the form must say what is wrong with the row").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(labelsError).toContainText("must not be blank");
    await expect(submit, "and it must not offer to send it").toBeDisabled();

    // No example at all rather than one printing an empty labels container. `LABELS_MIN_SIZE` is 1,
    // so a request with no label is a 400: the panel used to print exactly that, under a heading
    // saying "2. Send the event", to a reader who never sees the form's refusal.
    await openCurl();
    const withoutALabel = await panelStateOf(page, ["unsendable", "commands", "snippets"]);
    expect(withoutALabel, "no example may print a request the API would refuse").toEqual({
      unsendable: 1,
      commands: 0,
      snippets: 0,
    });
    await expect(
      curl.locator('[data-test="send-event-unsendable"]'),
      "and it must say what the API would refuse about it"
    ).toContainText("At least one label is required");

    // Padding around a real value is not the same thing: it is sent, trimmed, and the example has
    // to show what is sent rather than what was typed.
    await openForm();
    await value.fill("  1  ");
    await expect(labelsError, "a value that is blank only at its edges is not blank").toHaveCount(
      0
    );
    // A row that has been wrong once has to be able to stop being wrong. The form's value was the
    // editor's own rows, mutated under the validation by every keystroke, and a message raised
    // about one of them stayed up for good: measured, the Send button never came back however good
    // the value typed in afterwards.
    await expect(submit, "and the form is ready to send again").toBeEnabled();
    await openCurl();
    await expect(curl, "the example shows the value as it will be sent").toContainText(
      '"user_id": "1"'
    );

    await openForm();
    await submitEventWithLabels(page, { user_id: "1" });
  });

  test("names an event type the application has, whichever way the panel was reached", async ({
    page,
    request,
  }) => {
    // Reached by fragment rather than by clicking through, and that is the whole of it. Nothing
    // fills the form's event type but the select mounting and settling on its first option, so a
    // link straight to a code panel — the kind that gets pasted into a ticket — left the examples
    // with no event type to name. What stood in was a name invented for the occasion, and
    // `event_type` is a foreign key: the command a reader copied was refused by the API for naming
    // a type nobody had created, on an application whose real one was one tab away.
    //
    // Everything below is read from the fixture's own application, so the assertion is that the
    // screen names what this application has rather than that it names some particular string.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-eventtype");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(`${base}#curl`);
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });
    await expect(
      curl,
      "the command must post the event type this application declares"
    ).toContainText(`"event_type": "${env.eventTypeName}"`);

    // Every language, from that same arrival: `openLanguage` moves between code panels through the
    // tab strip and the picker, neither of which mounts the form, so the form's select stays out of
    // it for the whole sweep. Clicking through the form first would populate the value and hide the
    // defect — which is why the cover that walked this screen's states stayed green.
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await expect(
          panel.locator('[data-test="send-event-send"]'),
          `${sdk.target}: its example must name the event type this application declares`
        ).toContainText(env.eventTypeName);
      });
    }
  });

  test("shows the event types' own states in every panel, not only in the form", async ({
    page,
    request,
  }) => {
    // An application with no event type has nothing any panel can show: the form's select would
    // hold no option, and an example would have to name a type that does not exist. Held back to
    // the form's tab, that state let the code panels go on printing a name of their own.
    const env = await loginAndCreateApp(page, request, "sdk-eventtype-states");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;
    const eventTypes = "**/api/v1/event_types*";

    const empty = page.locator('[data-test="send-event-no-event-type"]');
    const loading = page.locator('[data-test="send-event-loading"]');
    const errorCard = page.locator('[data-test="error-card"]');

    /**
     * Lands on a panel the way a link pasted into a ticket does.
     *
     * Loaded again rather than only navigated to: two fragments of one URL are the same document,
     * so `goto` between them leaves the app mounted and its earlier fetch answered from cache. Every
     * state below is about what a fresh arrival renders, and one of them is a fetch that has not
     * come back — which cannot happen at all on a page that never asked again.
     */
    async function arriveOn(fragment: string) {
      await page.goto(`${base}#${fragment}`);
      await page.reload();
    }

    // Which languages sit behind the picker is read off the page, so the split can be revisited
    // without this test hearing about it.
    await page.goto(base);
    await expect(empty).toBeVisible({ timeout: PANEL_TIMEOUT });
    const behindThePicker: string[] = [];
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      if ((await page.locator(`[data-test="send-event-tab-${sdk.target}"]`).count()) === 0) {
        behindThePicker.push(sdk.target);
      }
    }
    expect(
      behindThePicker.length,
      "the picker must hold a language, or the fragment below opens nothing"
    ).toBeGreaterThan(0);

    // Every code panel a fragment opens, cURL and the languages alike. A fragment rather than a
    // click, because a click passes through a panel that has already answered for this state.
    const codePanels = ["curl", ...HOOK0_SDKS.map((sdk) => sdk.target)];
    for (const panel of codePanels) {
      await test.step(panel, async () => {
        await arriveOn(panel);
        await expect(
          empty,
          `#${panel}: an application with no event type must be told so here too`
        ).toBeVisible({ timeout: PANEL_TIMEOUT });
        await expect(
          page.locator('[data-test="send-event-send"]'),
          `#${panel}: no example may be shown while there is no event type to name`
        ).toHaveCount(0);
        await expect(
          page.locator('[data-test="send-event-curl-panel"]'),
          `#${panel}: and no command either`
        ).toHaveCount(0);
      });
    }

    // The refused fetch and the one still in flight, on a code tab. Both used to fall through to a
    // snippet built on whatever the form happened to hold, which was nothing.
    await page.route(eventTypes, (route) =>
      route.fulfill({ status: 500, contentType: "application/json", body: "{}" })
    );
    await arriveOn("curl");
    await expect(errorCard, "a refused fetch is said on a code tab as well").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(
      page.locator('[data-test="send-event-curl-panel"]'),
      "and no command is shown beside it"
    ).toHaveCount(0);
    await page.unroute(eventTypes);

    // Held for a bounded stretch rather than indefinitely, so the request always completes and the
    // page is never left mid-flight.
    await page.route(eventTypes, async (route) => {
      await new Promise((held) => setTimeout(held, HELD_MS));
      await route.continue();
    });
    await arriveOn("curl");
    await expect(loading, "a fetch still out is said on a code tab as well").toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.unroute(eventTypes);

    // Which is what makes the counts above statements about this application rather than about a
    // handful of selectors that match nothing: with an event type defined, the same fragment opens
    // the command it was hiding.
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await page.locator('[data-test="event-type-submit-button"]').click();
    await expect(page).toHaveURL(/\/event_types$/, { timeout: PANEL_TIMEOUT });

    await arriveOn("curl");
    await expect(page.locator('[data-test="send-event-curl-panel"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(empty, "the panel belongs to the application, not to the screen").toHaveCount(0);
  });

  test("says what is wrong with the instant rather than only refusing to send", async ({
    page,
    request,
  }) => {
    // The schema has always required it, and a datetime field is cleared as easily as any other.
    // The two fields under it say what is wrong with them; this one said nothing, and the only
    // sign was the Send button going disabled under a tooltip asking for fields that are all
    // filled — which is the same thing the payload used to do, for the same reason.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-occurred-at");
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`
    );
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const input = page.locator('[data-test="send-event-occurred-at-input"]');
    const message = page.locator("#send-event-occurred-at-error");
    const submit = page.locator('[data-test="send-event-submit-button"]');

    // The form seeds the present instant, so there is nothing to say to begin with. Without this
    // the assertions below would be satisfied by a field that always complains.
    await expect(message, "the seeded instant is fine, and nothing says otherwise").toHaveCount(0);
    await expect(submit, "and the form is ready to send").toBeEnabled();

    await input.fill("");
    await expect(message, "an empty instant has to say so").toBeVisible({ timeout: PANEL_TIMEOUT });
    await expect(message).toContainText("required");
    await expect(submit, "and the form must not offer to send it").toBeDisabled();

    // Said the way a field says it, so a screen reader hears it as belonging to this field rather
    // than as a sentence somewhere on the page.
    await expect(input).toHaveAttribute("aria-invalid", "true");
    await expect(input).toHaveAttribute("aria-describedby", "send-event-occurred-at-error");

    // And it goes away again: a message that outlives what raised it is worse than none.
    await input.fill("2026-01-02T03:04");
    await expect(message, "a filled instant is not wrong").toHaveCount(0);
    await expect(submit, "and the form is ready to send again").toBeEnabled();
  });

  test("keeps the picker showing the language that is in fact open", async ({ page, request }) => {
    // The picker's first entry is its own label rather than a language, and choosing it opens
    // nothing. Nothing is not the same as no change: the control moves to the label, the panel
    // stays where it was, and the strip goes on saying a language is open. A reader is then looking
    // at a Kotlin example under a control that reads "More languages…".
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-picker-placeholder");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-card"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const picker = page.locator('[data-test="send-event-language-select"]');

    // Read off the page rather than declared, so moving a language into the strip is not a change
    // this test has to be told about.
    const behindThePicker: string[] = [];
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      if ((await page.locator(`[data-test="send-event-tab-${sdk.target}"]`).count()) === 0) {
        behindThePicker.push(sdk.target);
      }
    }
    expect(
      behindThePicker.length,
      "the picker must hold a language for its placeholder to be reachable beside one"
    ).toBeGreaterThan(0);

    const opened = behindThePicker[0];
    await picker.selectOption(opened);
    await expect(page.locator(`[data-test="send-event-panel-${opened}"]`)).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect(picker, "the picker shows the language it just opened").toHaveValue(opened);

    // The label, chosen while that language is open.
    await picker.selectOption("");
    await expect(
      page.locator(`[data-test="send-event-panel-${opened}"]`),
      "choosing the picker's own label opens nothing, so the panel does not move"
    ).toBeVisible();
    await expect(
      picker,
      "and the picker goes on naming the language that is open, rather than its own label"
    ).toHaveValue(opened);
  });

  test("explains why the SDK examples do not carry the instant the form names", async ({
    page,
    request,
  }) => {
    // The API requires `occurred_at`, so the raw call has to state it and the command shows it. No
    // SDK example sets it, because every SDK dates the event itself when the caller leaves it
    // unset. Read one tab after the other, that difference looks like an omission in the examples —
    // so it is said, once, on the tabs where it applies.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-occurred-note");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;
    const note = '[data-test="send-event-send-occurred-at"]';

    await page.goto(`${base}#curl`);
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });
    await expect(
      curl,
      "the command states the instant, which is the half that is shown"
    ).toContainText('"occurred_at"');
    await expect(
      curl.locator(note),
      "and it needs no explaining on the tab that does carry it"
    ).toHaveCount(0);

    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await expect(
          panel.locator(note),
          `${sdk.target}: the tab that leaves it unset has to say why`
        ).toBeVisible();
      });
    }
  });
  test("keeps what it has when a refresh is refused, and says so when nothing ever arrived", async ({
    page,
    request,
  }) => {
    // Both halves of one property, because closing either alone opens the other. Queries here are
    // refetched when the window regains the focus once their data has gone stale, and retried once.
    // Guarded on the error alone, a refused refresh replaces the form a reader is filling in — and
    // the snippet they are half way through copying — over a request that changed nothing on
    // screen. Guarded on the data alone, a first fetch that was refused never gives up its
    // skeleton, because a refused fetch leaves the data undefined for good.
    test.setTimeout(180000);
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-refresh");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await expect(page.locator('[data-test="send-event-curl-panel"]')).toContainText(
      "curl -X POST",
      {
        timeout: PANEL_TIMEOUT,
      }
    );
    await page.locator('[data-test="send-event-tab-easy"]').click();
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible();

    // Counted, not merely installed: a refresh that never happened would leave every number below
    // satisfied by a page nothing had asked anything of.
    let refused = 0;
    const refuse = async (route: import("@playwright/test").Route) => {
      refused += 1;
      await route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
    };
    await page.route(EVENT_TYPES_URL, refuse);
    await page.route(SECRETS_URL, refuse);

    // Stale first, since a refetch on focus is only made for data that has gone stale.
    await page.waitForTimeout(STALE_MS + 1000);
    await page.evaluate(() => window.dispatchEvent(new Event("visibilitychange")));

    // Two queries, each retried once: four refusals is both of them having given up.
    await expect.poll(() => refused, { timeout: PANEL_TIMEOUT }).toBeGreaterThanOrEqual(4);

    const held = await panelStateOf(page, ["errorCards", "skeletons", "forms"]);
    expect(held, "a refused refresh must not take away the form the reader is filling in").toEqual({
      errorCards: 0,
      skeletons: 0,
      forms: 1,
    });

    await page.locator('[data-test="send-event-tab-curl"]').click();
    const onCurl = await panelStateOf(page, ["errorCards", "commands", "secretsRefused"]);
    expect(onCurl, "nor the example the reader was copying").toEqual({
      errorCards: 0,
      commands: 1,
      secretsRefused: 0,
    });

    // The other half, on the same refused calls: nothing has ever arrived here, so the panel has to
    // say so rather than sit on a skeleton it will never give up.
    await page.reload();
    await expect
      .poll(() => panelStateOf(page, ["errorCards", "skeletons", "forms"]), {
        message: "a first fetch that was refused must reach the error card",
        timeout: PANEL_TIMEOUT,
      })
      .toEqual({ errorCards: 1, skeletons: 0, forms: 0 });

    await page.unroute(EVENT_TYPES_URL);
    await page.unroute(SECRETS_URL);
  });

  test("says so in a code panel when the secrets never arrive, and offers the way back", async ({
    page,
    request,
  }) => {
    // The state nothing drove. The card is the dashboard's own, so it has to answer to the name the
    // rest of the dashboard reaches it by: naming it something else from here replaced that name
    // rather than adding to it, and this state became the one nothing could find.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-secrets-refused");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.route(SECRETS_URL, (route) =>
      route.fulfill({ status: 500, contentType: "application/json", body: "{}" })
    );
    await page.goto(`${base}#curl`);
    await page.reload();

    await expect(page.locator('[data-test="send-event-secrets-error"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    const refused = await panelStateOf(page, [
      "secretsRefused",
      "errorCards",
      "skeletons",
      "commands",
    ]);
    expect(refused, "the card the rest of the dashboard reaches must be the one shown").toEqual({
      secretsRefused: 1,
      errorCards: 1,
      skeletons: 0,
      commands: 0,
    });

    // And the way back works: with the call answering again, the retry brings the example.
    await page.unroute(SECRETS_URL);
    await page.locator('[data-test="error-card-retry"]').click();
    await expect(page.locator('[data-test="send-event-curl-panel"]')).toContainText(
      "curl -X POST",
      {
        timeout: PANEL_TIMEOUT,
      }
    );
    const recovered = await panelStateOf(page, ["secretsRefused", "errorCards", "commands"]);
    expect(recovered, "which is what makes the counts above about this state").toEqual({
      secretsRefused: 0,
      errorCards: 0,
      commands: 1,
    });
  });

  test("names a panel after what is in it, not after the language a picker happens to hold", async ({
    page,
    request,
  }) => {
    // A panel with no tab to name it carries its own name. Three of them are not code panels at
    // all, and named after the open language they announced "No event type yet" as "Rust".
    const env = await loginAndCreateApp(page, request, "sdk-panel-names");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;
    const languages = HOOK0_SDKS.map((sdk) => sdk.displayName);

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-no-event-type"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    const picked = (await languagesBehindThePicker(page))[0];
    const pickedName = HOOK0_SDKS.filter((sdk) => sdk.target === picked)[0].displayName;

    /** The panel, opened from the picker so that nothing but itself can name it. */
    async function nameUnderThePicker(said: string): Promise<string> {
      await page.locator('[data-test="send-event-language-select"]').selectOption(picked);
      const name = await panelSelfName(page);
      expect(name.length, `${said}: a panel no tab opens must still be named`).toBeGreaterThan(0);
      return name;
    }

    expect(
      languages,
      "the panel saying there is no event type is not an example of a language"
    ).not.toContain(await nameUnderThePicker("the empty state"));

    await page.route(EVENT_TYPES_URL, (route) =>
      route.fulfill({ status: 500, contentType: "application/json", body: "{}" })
    );
    await page.goto(`${base}#${picked}`);
    await page.reload();
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: PANEL_TIMEOUT });
    expect(languages, "nor is the panel saying the event types were refused").not.toContain(
      await panelSelfName(page)
    );
    await page.unroute(EVENT_TYPES_URL);

    await page.route(EVENT_TYPES_URL, async (route) => {
      await new Promise((held) => setTimeout(held, HELD_MS));
      await route.continue();
    });
    await page.goto(`${base}#${picked}`);
    await page.reload();
    await expect(page.locator('[data-test="send-event-loading"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    expect(languages, "nor is the panel waiting for them").not.toContain(await panelSelfName(page));
    await page.unroute(EVENT_TYPES_URL);

    // Which is what keeps the three above from being satisfied by a screen that names nothing: a
    // panel that really is an example of a language is still named after that language.
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await page.locator('[data-test="event-type-submit-button"]').click();
    await expect(page).toHaveURL(/\/event_types$/, { timeout: PANEL_TIMEOUT });

    await page.goto(`${base}#${picked}`);
    await expect(page.locator(`[data-test="send-event-panel-${picked}"]`)).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    expect(await panelSelfName(page), "a code panel is named after its language").toBe(pickedName);
  });

  test("counts a language as opened only where its example was on screen", async ({
    page,
    request,
  }) => {
    // The order the picker lists languages in is revisited on this figure, so what it counts is the
    // whole of its worth. Reported from the fragment alone it counted openings of panels that never
    // showed an example: on an application with no event type, every fragment scored one.
    await captureTrackingQueue(page);
    const env = await loginAndCreateApp(page, request, "sdk-open-count");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(`${base}#python`);
    await expect(page.locator('[data-test="send-event-no-event-type"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    const withNothingToShow = (await trackedEvents(page)).filter(
      (e) => e[1] === "send-event" && e[2] === "open"
    );
    expect(withNothingToShow, "no panel showed an example, so no panel was opened").toEqual([]);

    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
    );
    await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await page.locator('[data-test="event-type-service-input"]').fill("test");
    await page.locator('[data-test="event-type-resource-input"]').fill("entity");
    await page.locator('[data-test="event-type-verb-input"]').fill("created");
    await page.locator('[data-test="event-type-submit-button"]').click();
    await expect(page).toHaveURL(/\/event_types$/, { timeout: PANEL_TIMEOUT });

    // The same fragment, on the same screen, once there is something to show: which is what makes
    // the emptiness above a statement about the panel rather than about the tracking never running.
    await page.goto(`${base}#python`);
    await expect(page.locator('[data-test="send-event-panel-python"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await expect
      .poll(
        async () =>
          (await trackedEvents(page)).some(
            (e) => e[1] === "send-event" && e[2] === "open" && e[3] === "python"
          ),
        { timeout: PANEL_TIMEOUT }
      )
      .toBe(true);
  });

  test("offers a service token where an application secret authenticates nothing", async ({
    page,
    request,
  }) => {
    // `middleware_biscuit.rs` takes an application secret as a Bearer credential only under a
    // compatibility setting the API marks deprecated. With it off, every example built on one is
    // answered with a 401 and the navigation hides the page it is created on — so the twelve panels
    // printed a credential the server refuses, above a link to a page a reader could not reach. A
    // service token is a biscuit, which that middleware takes whatever the setting says.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-no-compat");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    let secretsAsked = 0;
    await page.route(SECRETS_URL, async (route) => {
      secretsAsked += 1;
      await route.continue();
    });
    await page.route("**/api/v1/instance*", async (route) => {
      const answered = await route.fetch();
      const config = (await answered.json()) as Record<string, unknown>;
      await route.fulfill({ json: { ...config, application_secret_compatibility: false } });
    });

    await page.goto(`${base}#curl`);
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });
    // Read as one state rather than waited for element by element: what the panel is showing is
    // the whole of the question, and "not found" would say neither what went wrong nor what a
    // reader saw in its place.
    await expect
      .poll(() => panelStateOf(page, ["secretNotAccepted", "commands", "skeletons"]), {
        message: "no command may carry a credential this instance refuses",
        timeout: PANEL_TIMEOUT,
      })
      .toEqual({ secretNotAccepted: 1, commands: 0, skeletons: 0 });
    await expect(curl, "and none may print a Bearer line at all").not.toContainText("Bearer");
    expect(secretsAsked, "and a list whose answer no panel can use must not be asked for").toBe(0);

    // Every language says the same, not only the panel assembled by hand.
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await expect(panel.locator('[data-test="send-event-secret-not-accepted"]')).toBeVisible();
        await expect(
          panel.locator('[data-test="send-event-send"]'),
          `${sdk.target}: no example while its credential would be refused`
        ).toHaveCount(0);
      });
    }

    // The way out leads to the credential this instance does take.
    await page.locator('[data-test="send-event-secret-not-accepted"] a').click();
    await expect(page).toHaveURL(/\/service_tokens$/, { timeout: PANEL_TIMEOUT });
  });

  test("prints no credential, and offers the call again, when the instance never says whether a secret is taken", async ({
    page,
    request,
  }) => {
    // `GET /instance` reports whether an application secret is taken as a Bearer credential. When
    // that call fails the setting is unknown — a third answer, neither accepted nor refused — and
    // any snippet then carries a credential chosen on a guess: a Bearer the server may answer with a
    // 401, or a blanket "use a service token" when secrets may well be taken. So the panel prints no
    // credential-bearing example and offers the call again, the way it does for a refused secrets
    // fetch. Nothing drove this state, and reading the failure as "secrets are accepted" is exactly
    // the guess the screen was rebuilt to stop making.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-instance-unknown");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    let secretsAsked = 0;
    await page.route(SECRETS_URL, async (route) => {
      secretsAsked += 1;
      await route.continue();
    });
    let instanceAsked = 0;
    let failInstance = true;
    await page.route("**/api/v1/instance*", async (route) => {
      instanceAsked += 1;
      if (failInstance) {
        await route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
        return;
      }
      await route.continue();
    });

    await captureTrackingQueue(page);
    await page.goto(`${base}#curl`);
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });

    // Read as one state rather than waited for element by element: the error card stands where the
    // command was, no command is printed, and no skeleton is left spinning. "Not found" would say
    // neither what went wrong nor what a reader saw in its place.
    await expect
      .poll(() => panelStateOf(page, ["instanceConfigError", "commands", "skeletons"]), {
        message: "an unknown instance setting must print no command and offer the call again",
        timeout: PANEL_TIMEOUT,
      })
      .toEqual({ instanceConfigError: 1, commands: 0, skeletons: 0 });
    await expect(
      curl,
      "and no Bearer line at all, which printing one on a guess would carry"
    ).not.toContainText("Bearer");
    expect(secretsAsked, "a secret whose acceptance is unknown has no list worth asking for").toBe(
      0
    );
    expect(
      instanceAsked,
      "and the instance was in fact asked, so the state is the failure's"
    ).toBeGreaterThan(0);

    // Every language says the same, not only the panel assembled by hand.
    for (const sdk of HOOK0_SDKS as readonly Hook0Sdk[]) {
      await test.step(`${sdk.displayName} (${sdk.target})`, async () => {
        const panel = await openLanguage(page, sdk.target);
        await expect(panel.locator('[data-test="send-event-instance-config-error"]')).toBeVisible();
        await expect(
          panel.locator('[data-test="send-event-send"]'),
          `${sdk.target}: no example while the credential to carry is unknown`
        ).toHaveCount(0);
      });
    }

    // An error card is not an opening. The panel shows no example here, so nothing was opened, and
    // the figure the picker order is read against must not gain a phantom open for a language a
    // reader only ever saw a failure in.
    const openedWhileFailed = (await trackedEvents(page)).filter(
      (e) => e[1] === "send-event" && e[2] === "open"
    );
    expect(openedWhileFailed, "a failed instance read reports no panel opening").toEqual([]);

    // The way back works: with the call answering again, the retry brings the example — which is what
    // makes the counts above statements about this state rather than about a screen that shows
    // nothing.
    failInstance = false;
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await page.locator('[data-test="error-card-retry"]').click();
    await expect(curl).toContainText("curl -X POST", { timeout: PANEL_TIMEOUT });
    const recovered = await panelStateOf(page, ["instanceConfigError", "commands"]);
    expect(recovered, "the retry recovers the example once the instance answers").toEqual({
      instanceConfigError: 0,
      commands: 1,
    });
  });

  test("names an event type the application still has, not the one the form settled on", async ({
    page,
    request,
  }) => {
    // The form's select holds the type the examples name, and it corrects itself whenever it is on
    // screen — but a code panel does not mount it. So once the list moves underneath a reader who
    // is looking at an example — a colleague deactivating a type in another tab, which is one
    // request — the examples went on naming a type the application no longer has, and `event_type`
    // is a foreign key: the command a reader copied was refused for naming it.
    test.setTimeout(180000);
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-stale-type");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    const login = await request.post(`${API_BASE_URL}/auth/login`, {
      data: { email: env.email, password: env.password },
    });
    expect(login.status(), "the test user must be able to log in").toBeLessThan(400);
    const auth = { Authorization: `Bearer ${(await login.json()).access_token}` };

    const survivor = "other.entity.created";
    const added = await request.post(`${API_BASE_URL}/event_types`, {
      headers: auth,
      data: {
        application_id: env.applicationId,
        service: "other",
        resource_type: "entity",
        verb: "created",
      },
    });
    expect(added.status(), "a second event type must be created").toBeLessThan(400);

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    // Whichever one the select settled on is the one to take away, read off the page rather than
    // assumed from the order the API lists them in.
    const settled = await page.locator('[data-test="send-event-type-select"]').inputValue();
    expect([survivor, env.eventTypeName], "the select settles on one of the two").toContain(
      settled
    );
    const remaining = settled === survivor ? env.eventTypeName : survivor;

    // On a code panel before the list moves, so the form's select is not mounted to correct it.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toContainText(`"event_type": "${settled}"`, { timeout: PANEL_TIMEOUT });

    const removed = await request.delete(
      `${API_BASE_URL}/event_types/${settled}?application_id=${env.applicationId}`,
      { headers: auth }
    );
    expect(removed.status(), "deactivating an event type must succeed").toBeLessThan(400);

    // The list is read again when the window comes back to a page whose data has gone stale.
    let listed = 0;
    await page.route(EVENT_TYPES_URL, async (route) => {
      listed += 1;
      await route.continue();
    });
    await page.waitForTimeout(STALE_MS + 1000);
    await page.evaluate(() => window.dispatchEvent(new Event("visibilitychange")));
    await expect.poll(() => listed, { timeout: PANEL_TIMEOUT }).toBeGreaterThan(0);

    await expect(curl, "the example must name a type the application still has").toContainText(
      `"event_type": "${remaining}"`,
      { timeout: PANEL_TIMEOUT }
    );
    await expect(
      curl,
      "and not the one it has just lost, which the API would refuse"
    ).not.toContainText(`"event_type": "${settled}"`);
    await page.unroute(EVENT_TYPES_URL);
  });

  test("prints no shell command an apostrophe would break", async ({ page, request }) => {
    // A single-quoted argument cannot hold the quote itself, and `JSON.stringify` does not escape
    // it: a payload as ordinary as an Irish surname ended the argument early, and the command a
    // reader pasted was answered by their own shell. The eleven SDK examples were already held to
    // this; the one panel assembled by hand was not.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-apostrophe");
    const base = `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send`;

    await page.goto(base);
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });

    const curl = page.locator('[data-test="send-event-curl-panel"]');
    const value = page.locator('[data-test="send-event-labels"] [data-test="kv-value-input-0"]');

    // The seeded label, in the command. Without it the assertion below would be satisfied by a
    // panel that never showed a label at all.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await expect(curl).toContainText('"user_id": "1"', { timeout: PANEL_TIMEOUT });

    await page.locator('[data-test="send-event-tab-easy"]').click();
    await expect(page.locator('[data-test="send-event-form"]')).toBeVisible({
      timeout: PANEL_TIMEOUT,
    });
    await value.fill("O'Brien");
    // The labels editor hands its rows up 150 ms after the last keystroke, and the tab below
    // unmounts it: leaving at once loses the row rather than showing it in the example.
    await page.waitForTimeout(KV_DEBOUNCE_MS);
    await page.locator('[data-test="send-event-tab-curl"]').click();
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });

    // The quote is spliced back outside the quoting, which is the one form a shell reads as a
    // quote inside a single-quoted argument.
    await expect(
      curl,
      "the apostrophe must be handed to the shell rather than ending the argument"
    ).toContainText("O'\\''Brien");
    await expect(curl, "and the command must still be a command").toContainText("curl -X POST");
  });

  test("takes the tutorial's own credential and asks for no other", async ({ page, request }) => {
    // The tutorial authenticates its examples with the dashboard session's own token, so the
    // application secrets are a list no panel there reads — and a refused fetch of it blanked all
    // twelve panels behind a Retry that changed nothing. The seeding read is the one that matters
    // in this step, and when it is refused the preset below is a guess rather than the subscription
    // the event has to satisfy.
    test.setTimeout(180000);
    const env = await loginAsNewUser(page, request, "sdk-tutorial");

    let secretsAsked = 0;
    await page.route(SECRETS_URL, async (route) => {
      secretsAsked += 1;
      await route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
    });

    // An event type of the reader's own rather than the one the use case suggests, which is what
    // makes the assertion below about the application rather than about the preset.
    const eventTypeName = "custom.thing.created";
    await walkTutorialToSendScreen(page, env, {
      eventTypeName,
      beforeSubscriptionSubmit: async () => {
        // Refused from here on, and only where it is read: the step's own creation still goes through.
        await page.route("**/api/v1/subscriptions*", async (route) => {
          if (route.request().method() === "GET") {
            await route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
            return;
          }
          await route.continue();
        });
        // Counted from here, so that a request some earlier step of the wizard makes is not read as
        // one this step made.
        secretsAsked = 0;
      },
    });

    // The read the seeding depends on was refused, so the form says the preset may not be what the
    // subscription listens for rather than letting the wizard end on a pipeline nothing travelled.
    await expect(
      page.locator('[data-test="send-event-tutorial-subscription-error"]'),
      "a refused read of the subscription must be said, not swallowed"
    ).toBeVisible({ timeout: PANEL_TIMEOUT });

    // Reached from the tab strip, so the form's select never corrects the preset.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });

    await expect
      .poll(() => panelStateOf(page, ["commands", "errorCards", "skeletons"]), {
        message: "a list no panel here reads must not be able to blank them all",
        timeout: PANEL_TIMEOUT,
      })
      .toEqual({ commands: 1, errorCards: 0, skeletons: 0 });
    expect(secretsAsked, "and it must not be asked for at all").toBe(0);

    const named = await panelStateOf(page, ["tutorialToken", "commands"]);
    expect(named, "the credential handed out here has to say what it is").toEqual({
      tutorialToken: 1,
      commands: 1,
    });

    // The instance here is the e2e default, which accepts application secrets, so the note names
    // that credential and points at its page. This is the common path a tutorial user takes, and
    // it must never show the neutral note or a link the instance would refuse.
    const noteLink = page.locator('[data-test="send-event-tutorial-token"] a');
    await expect(noteLink).toHaveText("Create an application secret");
    await expect(noteLink).toHaveAttribute("href", /\/application_secrets$/);

    await expect(
      curl,
      "an example must name an event type this application has, not the one a preset suggested"
    ).toContainText(`"event_type": "${eventTypeName}"`);
    await expect(curl, "and not the preset's").not.toContainText("store.order.created");
  });

  test("shows the tutorial's own example even when the instance config fails to load", async ({
    page,
    request,
  }) => {
    // The tutorial's example carries the dashboard session's own token, which the middleware takes
    // whatever the instance says, so a failed `/instance` read has no bearing on it. The error card
    // the durable-credential screen puts up for that failure would only withhold an example that
    // works, and its token note belongs above the example either way.
    test.setTimeout(180000);
    const env = await loginAsNewUser(page, request, "sdk-tutorial-instance-fail");

    let instanceAsked = 0;
    await page.route("**/api/v1/instance*", (route) => {
      instanceAsked += 1;
      return route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
    });

    await walkTutorialToSendScreen(page, env, { eventTypeName: "custom.thing.created" });

    // Reached from the tab strip, so the form's select never stands in for the panel.
    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });

    await expect
      .poll(
        () =>
          panelStateOf(page, [
            "commands",
            "instanceConfigError",
            "errorCards",
            "skeletons",
            "tutorialToken",
          ]),
        {
          message: "a failed instance read must not stand between the tutorial and its own example",
          timeout: PANEL_TIMEOUT,
        }
      )
      .toEqual({
        commands: 1,
        instanceConfigError: 0,
        errorCards: 0,
        skeletons: 0,
        tutorialToken: 1,
      });

    // The interception was in fact hit, so the screen is in the failed read's state rather than one
    // a route quietly passed through: without this, every assertion above would also hold if the
    // instance answered normally, since the tutorial ignores what it says.
    expect(instanceAsked, "the failed instance read must have reached the screen").toBeGreaterThan(
      0
    );

    // The instance has not said which durable credential it accepts, so the note names none. It
    // carries the neutral wording and holds no link a reader could follow to a credential the
    // instance may refuse.
    const note = page.locator('[data-test="send-event-tutorial-token"]');
    await expect(note).toContainText("durable credential of its own");
    await expect(
      note.locator("a"),
      "a note about an unconfirmed instance names no credential"
    ).toHaveCount(0);
  });

  test("shows the tutorial's own example even when the instance refuses application secrets", async ({
    page,
    request,
  }) => {
    // With `application_secret_compatibility` off the durable-credential screen offers a service
    // token where an example would be, since an application secret authenticates nothing there. The
    // tutorial's session token is a biscuit the middleware takes whatever that setting says, so the
    // example stands and the secret-not-accepted notice has no place above it.
    test.setTimeout(180000);
    const env = await loginAsNewUser(page, request, "sdk-tutorial-no-compat");

    await page.route("**/api/v1/instance*", async (route) => {
      const answered = await route.fetch();
      const config = (await answered.json()) as Record<string, unknown>;
      await route.fulfill({ json: { ...config, application_secret_compatibility: false } });
    });

    await walkTutorialToSendScreen(page, env, { eventTypeName: "custom.thing.created" });

    await page.locator('[data-test="send-event-tab-curl"]').click();
    const curl = page.locator('[data-test="send-event-curl-panel"]');
    await expect(curl).toBeVisible({ timeout: PANEL_TIMEOUT });

    await expect
      .poll(
        () => panelStateOf(page, ["commands", "secretNotAccepted", "skeletons", "tutorialToken"]),
        {
          message: "a refused application secret must not withhold the tutorial's own example",
          timeout: PANEL_TIMEOUT,
        }
      )
      .toEqual({ commands: 1, secretNotAccepted: 0, skeletons: 0, tutorialToken: 1 });

    // The refusal reached the screen: the note names the credential the instance does take, a
    // service token, and points at that page rather than at the application secret the instance
    // would answer with a 401. This also witnesses the compat:false answer, which the tutorial
    // otherwise ignores.
    const noteLink = page.locator('[data-test="send-event-tutorial-token"] a');
    await expect(noteLink).toHaveText("Create service token");
    await expect(noteLink).toHaveAttribute("href", /\/service_tokens$/);
  });

  test("says nothing about a session token where the credential is the application's own", async ({
    page,
    request,
  }) => {
    // What keeps the note above from being a sentence the screen always shows: outside the tutorial
    // the credential is the application's secret, which does not expire with the session.
    const env = await loginAndCreateAppWithEventType(page, request, "sdk-no-token-note");
    await page.goto(
      `/organizations/${env.organizationId}/applications/${env.applicationId}/events/send#curl`
    );
    await expect(page.locator('[data-test="send-event-curl-panel"]')).toContainText(
      "curl -X POST",
      {
        timeout: PANEL_TIMEOUT,
      }
    );
    const outsideTheTutorial = await panelStateOf(page, ["commands", "tutorialToken"]);
    expect(outsideTheTutorial, "the note belongs to the tutorial, not to the screen").toEqual({
      commands: 1,
      tutorialToken: 0,
    });
  });
});
