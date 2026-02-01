import { test as base, expect, APIRequestContext, Page } from "@playwright/test";
import { verifyEmailViaMailpit } from "./email-verification";

/**
 * Test fixtures for Hook0 E2E tests.
 *
 * Provides authenticated user context and API helpers.
 * Uses data-test selectors for stability.
 */

export interface TestUser {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  organizationId?: string;
}

export interface AuthFixtures {
  testUser: TestUser;
  authenticatedPage: Page;
  apiContext: APIRequestContext;
}

/**
 * Create a new test user via API and verify their email.
 */
async function createTestUser(request: APIRequestContext): Promise<TestUser> {
  const timestamp = Date.now();
  const email = `test-${timestamp}@hook0.local`;
  const password = `TestPassword123!${timestamp}`;
  const firstName = "Test";
  const lastName = "User";

  const response = await request.post("/api/v1/register", {
    data: {
      email,
      first_name: firstName,
      last_name: lastName,
      password,
    },
  });

  expect(response.status()).toBeLessThan(400);

  // Verify email via Mailpit
  try {
    await verifyEmailViaMailpit(request, email);
  } catch (e) {
    console.warn(`Email verification failed: ${e}. Tests may fail if verification is required.`);
  }

  return { email, password, firstName, lastName };
}

/**
 * Extended test with authentication fixtures.
 */
export const authTest = base.extend<AuthFixtures>({
  testUser: async ({ request }, use) => {
    const user = await createTestUser(request);
    await use(user);
  },

  authenticatedPage: async ({ page, testUser }, use) => {
    await page.goto("/login");

    // Wait for login form to be visible
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });

    // Fill login form using data-test selectors
    await page.locator('[data-test="login-email-input"]').fill(testUser.email);
    await page.locator('[data-test="login-password-input"]').fill(testUser.password);

    // Submit and wait for API response
    const responsePromise = page.waitForResponse(
      (response) =>
        (response.url().includes("/api/v1/auth/login") || response.url().includes("/auth/login")) &&
        response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="login-submit-button"]').click();

    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    // Wait for redirect to authenticated area
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, {
      timeout: 15000,
    });

    await use(page);
  },

  apiContext: async ({ playwright }, use) => {
    const context = await playwright.request.newContext({
      baseURL: process.env.API_URL || "http://localhost:8081",
    });
    await use(context);
    await context.dispose();
  },
});

export { expect } from "@playwright/test";
