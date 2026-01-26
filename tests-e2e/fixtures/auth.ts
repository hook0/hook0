import { test as base, expect, APIRequestContext } from "@playwright/test";

/**
 * Test fixtures for Hook0 E2E tests.
 *
 * Provides authenticated user context and API helpers.
 */

export interface TestUser {
  email: string;
  password: string;
  organizationId?: string;
}

export interface AuthFixtures {
  testUser: TestUser;
  authenticatedPage: Awaited<ReturnType<typeof base.extend>>["page"];
  apiContext: APIRequestContext;
}

/**
 * Create a new test user via API.
 */
async function createTestUser(request: APIRequestContext): Promise<TestUser> {
  const timestamp = Date.now();
  const email = `test-${timestamp}@hook0.local`;
  const password = `TestPassword123!${timestamp}`;

  const response = await request.post("/api/v1/register", {
    data: {
      email,
      password,
      password_confirmation: password,
    },
  });

  expect(response.status()).toBeLessThan(400);

  return { email, password };
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
    await page.locator('[data-test="email-input"]').fill(testUser.email);
    await page.locator('[data-test="password-input"]').fill(testUser.password);
    await page.locator('[data-test="login-button"]').click();

    await expect(page).toHaveURL(/\/dashboard|\/organizations/);
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
