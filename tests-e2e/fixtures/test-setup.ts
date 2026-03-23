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

  // Register via API
  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    data: { email, first_name: "Test", last_name: "User", password },
  });
  expect(registerResponse.status()).toBeLessThan(400);

  // Verify email and get org ID
  const verificationResult = await verifyEmailViaMailpit(request, email);
  const organizationId = verificationResult.organizationId;
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

export { loginAsNewUser, loginAndCreateApp, loginAndCreateAppWithEventType, expectToast, API_BASE_URL };
export { test, expect } from "@playwright/test";
