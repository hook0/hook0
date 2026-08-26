import { expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "./email-verification";

/** Shared UUID pattern for extracting IDs from URLs. */
export const UUID_PATTERN =
  /([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;

export interface TestEnv {
  email: string;
  password: string;
  organizationId: string;
  timestamp: number;
}

export interface TestEnvWithApp extends TestEnv {
  applicationId: string;
}

export interface TestEnvWithAppAndEventType extends TestEnvWithApp {
  eventTypeName: string;
}

/**
 * Login a new test user via UI. Returns user credentials and organizationId.
 */
async function loginAsNewUser(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
  testId: string
): Promise<TestEnv> {
  const timestamp = Date.now();
  const email = `test-${testId}-${timestamp}@hook0.local`;
  const password = `TestPassword123!${timestamp}`;

  // Register via API — response contains organization_id
  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    headers: fromItsOwnAddress(),
    data: { email, first_name: "Test", last_name: "User", password },
  });
  expect(registerResponse.status()).toBeLessThan(400);
  const registerData = await registerResponse.json();
  const organizationId = registerData.organization_id;

  // Verify email (pass org ID so we don't need DB access)
  await verifyEmailViaMailpit(request, email, organizationId);
  expect(organizationId).toBeTruthy();

  // Login via UI
  await page.goto("/login");
  await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
  await page.locator('[data-test="login-email-input"]').fill(email);
  await page.locator('[data-test="login-password-input"]').fill(password);
  await page.locator('[data-test="login-submit-button"]').click();
  await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

  return { email, password, organizationId: organizationId!, timestamp };
}

/**
 * Login + create an application. Returns env with applicationId.
 */
async function loginAndCreateApp(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
  testId: string
): Promise<TestEnvWithApp> {
  const env = await loginAsNewUser(page, request, testId);

  await page.goto(`/organizations/${env.organizationId}/applications/new`);
  await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
  await page.locator('[data-test="application-name-input"]').fill(`App ${env.timestamp}`);

  const createAppResponse = page.waitForResponse(
    (response) =>
      response.url().includes("/api/v1/applications") && response.request().method() === "POST",
    { timeout: 15000 }
  );
  await page.locator('[data-test="application-submit-button"]').click();
  const appResponse = await createAppResponse;
  expect(appResponse.status()).toBeLessThan(400);

  // Extract application ID from URL after redirect
  const uuidPattern =
    /\/applications\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/i;
  await expect(page).toHaveURL(uuidPattern, { timeout: 15000 });
  const match = page.url().match(uuidPattern);
  expect(match).toBeTruthy();
  const applicationId = match![1];

  return { ...env, applicationId };
}

/**
 * Login + create an application + create an event type. Returns full env.
 */
async function loginAndCreateAppWithEventType(
  page: import("@playwright/test").Page,
  request: import("@playwright/test").APIRequestContext,
  testId: string,
  eventType = { service: "test", resource: "entity", verb: "created" }
): Promise<TestEnvWithAppAndEventType> {
  const env = await loginAndCreateApp(page, request, testId);

  await page.goto(
    `/organizations/${env.organizationId}/applications/${env.applicationId}/event_types/new`
  );
  await expect(page.locator('[data-test="event-type-form"]')).toBeVisible({ timeout: 10000 });
  await page.locator('[data-test="event-type-service-input"]').fill(eventType.service);
  await page.locator('[data-test="event-type-resource-input"]').fill(eventType.resource);
  await page.locator('[data-test="event-type-verb-input"]').fill(eventType.verb);

  const createETResponse = page.waitForResponse(
    (response) =>
      response.url().includes("/api/v1/event_types") && response.request().method() === "POST",
    { timeout: 15000 }
  );
  await page.locator('[data-test="event-type-submit-button"]').click();
  const etResponse = await createETResponse;
  expect(etResponse.status()).toBeLessThan(400);
  await expect(page).toHaveURL(/\/event_types$/, { timeout: 10000 });

  const eventTypeName = `${eventType.service}.${eventType.resource}.${eventType.verb}`;
  return { ...env, eventTypeName };
}

/**
 * Headers that make a call look like it comes from its own source address.
 *
 * The endpoints that put a message in a mailbox are capped per source address.
 * A test suite drives them from one machine, so without this the whole run
 * shares a single allowance: a test that legitimately makes a dozen calls
 * starves every test after it, and the failure lands on whichever test happened
 * to run next. In production those same calls would be a dozen different
 * people, which is what this reproduces.
 *
 * The API reads the header only from a peer it has been told to trust — the
 * real reverse proxy in production, the loopback range in the e2e stack — so
 * nothing here changes what the header is allowed to mean.
 *
 * Addresses come from 198.18.0.0/15, reserved for benchmarking by RFC 2544, so
 * one can never be confused with a real client. Only calls made straight to the
 * API can carry it: a request issued by the page is refused at the CORS
 * preflight, because the API allows a fixed short list of request headers and
 * this is not one of them.
 */
function fromItsOwnAddress(): Record<string, string> {
  const octet = () => Math.floor(Math.random() * 256);
  return { "X-Forwarded-For": `198.18.${octet()}.${octet()}` };
}

/**
 * Assert a toast notification is visible. Centralizes the vue-sonner selector.
 */
async function expectToast(
  page: import("@playwright/test").Page,
  options: { type?: 'success' | 'error'; contains?: string; timeout?: number } | number = 10000
) {
  const opts = typeof options === 'number' ? { timeout: options } : options;
  const { type, contains, timeout = 10000 } = opts;
  const selector = type
    ? `[data-sonner-toast][data-type="${type}"]`
    : "[data-sonner-toast]";
  const toast = page.locator(selector).first();
  await expect(toast).toBeVisible({ timeout });
  if (contains) {
    await expect(toast).toContainText(contains);
  }
}

/**
 * Tick an event type on the subscription form and make sure the choice survived.
 *
 * The list does not own its selection: the box is bound to an array the parent
 * rebuilds from whatever the event-types request last returned, so a click that
 * lands while that request is still in flight is dropped with nothing to show
 * for it. The subscription is then saved matching no event type, and nothing
 * says so — an event sent afterwards matches nothing, no attempt is recorded,
 * and the first sign of it is another test waiting a minute for a log row that
 * was never going to come.
 *
 * So click until the box is ticked rather than once, and leave it ticked.
 */
async function selectEventType(checkbox: import("@playwright/test").Locator): Promise<void> {
  await expect(checkbox).toBeVisible({ timeout: 15000 });
  await expect(async () => {
    if (!(await checkbox.isChecked())) {
      await checkbox.click();
    }
    await expect(checkbox).toBeChecked({ timeout: 1000 });
  }).toPass({ timeout: 20000, intervals: [250, 500, 1000] });
}

/**
 * Submit a form carrying a key/value labels editor, and prove the request went
 * out with the labels that were typed into it.
 *
 * The editor waits 150ms after the last keystroke before telling the
 * surrounding form about an edit, so the form is not rebuilt on every
 * character, and the form sends whatever its own copy holds when the button is
 * clicked. Both of these forms start out holding user_id=1, a complete pair
 * that satisfies every validation on the way out, so a click landing inside
 * that window is accepted everywhere and quietly produces a subscription, or
 * an event, that matches nothing. Reading what went on the wire says so at the
 * submit, instead of a minute later as a logs table that stayed empty.
 */
async function submitWithLabels(
  page: import("@playwright/test").Page,
  form: { button: string; matches: (url: string) => boolean; what: string },
  expectedLabels: Record<string, string>
): Promise<import("@playwright/test").Response> {
  await page.waitForTimeout(400);

  const submitted = page.waitForResponse(
    (response) => form.matches(response.url()) && response.request().method() === "POST",
    { timeout: 15000 }
  );
  await page.locator(`[data-test="${form.button}"]`).click();
  const response = await submitted;

  const payload = response.request().postDataJSON() as { labels: Record<string, string> };
  expect(payload, `${form.what} submit carried no JSON body`).toBeTruthy();
  expect(
    payload.labels,
    `${form.what} went out with the form's own labels rather than the typed ones`
  ).toEqual(expectedLabels);

  // Read defensively: once the page navigates away the browser drops the body,
  // and reading it unconditionally fails the assertion below for a reason that
  // has nothing to do with what it is checking.
  const body = await response.text().catch(() => "(body no longer available)");
  expect(
    response.status(),
    `sending ${form.what} answered ${response.status()}: ${body}`
  ).toBeLessThan(400);

  return response;
}

function submitEventWithLabels(
  page: import("@playwright/test").Page,
  expectedLabels: Record<string, string>
): Promise<import("@playwright/test").Response> {
  return submitWithLabels(
    page,
    {
      button: "send-event-submit-button",
      matches: (url) => url.includes("/api/v1/event") && !url.includes("/api/v1/event_types"),
      what: "the event",
    },
    expectedLabels
  );
}

function submitSubscriptionWithLabels(
  page: import("@playwright/test").Page,
  expectedLabels: Record<string, string>
): Promise<import("@playwright/test").Response> {
  return submitWithLabels(
    page,
    {
      button: "subscription-submit-button",
      matches: (url) => url.includes("/api/v1/subscriptions"),
      what: "the subscription",
    },
    expectedLabels
  );
}

export { loginAsNewUser, loginAndCreateApp, loginAndCreateAppWithEventType, expectToast, fromItsOwnAddress, selectEventType, submitEventWithLabels, submitSubscriptionWithLabels, API_BASE_URL };
export { test, expect } from "@playwright/test";
