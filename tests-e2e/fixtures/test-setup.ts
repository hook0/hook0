import { expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "./email-verification";

export interface TestEnv {
  email: string;
  password: string;
  organizationId: string;
  timestamp: number;
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

export { loginAsNewUser, API_BASE_URL };
export { test, expect } from "@playwright/test";
