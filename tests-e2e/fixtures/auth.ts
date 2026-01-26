import { test as base, expect, APIRequestContext, Page } from "@playwright/test";

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
 * Verify user email via Mailpit API.
 * Fetches the latest email sent to the user and extracts the verification link.
 */
async function verifyEmailViaMailpit(request: APIRequestContext, email: string): Promise<void> {
  const mailpitUrl = process.env.MAILPIT_URL || "http://localhost:8025";

  // Wait a bit for the email to arrive
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Search for emails sent to this address
  const searchResponse = await request.get(
    `${mailpitUrl}/api/v1/search?query=to:${encodeURIComponent(email)}`,
    { timeout: 10000 }
  );

  if (!searchResponse.ok()) {
    console.warn(`Mailpit search failed: ${searchResponse.status()}`);
    // Try alternative: get all messages
    const messagesResponse = await request.get(`${mailpitUrl}/api/v1/messages`, {
      timeout: 10000,
    });
    if (!messagesResponse.ok()) {
      throw new Error(`Failed to fetch emails from Mailpit`);
    }
    const messages = await messagesResponse.json();
    const userEmail = messages.messages?.find((m: { To: Array<{ Address: string }> }) =>
      m.To?.some((t) => t.Address === email)
    );
    if (!userEmail) {
      throw new Error(`No email found for ${email}`);
    }
    // Get the full message
    const messageResponse = await request.get(`${mailpitUrl}/api/v1/message/${userEmail.ID}`, {
      timeout: 10000,
    });
    const message = await messageResponse.json();
    const verificationLink = extractVerificationLink(message.Text || message.HTML);
    if (verificationLink) {
      await request.get(verificationLink, { timeout: 10000 });
    }
    return;
  }

  const searchResult = await searchResponse.json();
  if (!searchResult.messages || searchResult.messages.length === 0) {
    throw new Error(`No emails found for ${email}`);
  }

  // Get the latest email
  const latestEmail = searchResult.messages[0];
  const messageResponse = await request.get(`${mailpitUrl}/api/v1/message/${latestEmail.ID}`, {
    timeout: 10000,
  });

  if (!messageResponse.ok()) {
    throw new Error(`Failed to fetch email content`);
  }

  const message = await messageResponse.json();
  const verificationLink = extractVerificationLink(message.Text || message.HTML);

  if (!verificationLink) {
    throw new Error(`No verification link found in email`);
  }

  // Visit the verification link
  await request.get(verificationLink, { timeout: 10000 });
}

/**
 * Extract verification link from email content.
 */
function extractVerificationLink(content: string): string | null {
  // Look for verification URLs in the email
  // Common patterns: /verify-email?token=xxx, /email-verification?token=xxx
  const patterns = [
    /https?:\/\/[^\s<>"]+\/verify-email[^\s<>"]+/i,
    /https?:\/\/[^\s<>"]+\/email-verification[^\s<>"]+/i,
    /https?:\/\/[^\s<>"]+token=[^\s<>"]+/i,
  ];

  for (const pattern of patterns) {
    const match = content.match(pattern);
    if (match) {
      return match[0];
    }
  }

  return null;
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
