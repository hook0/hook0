import { APIRequestContext } from "@playwright/test";

/**
 * Email verification helper using Mailpit API.
 *
 * In CI, Mailpit captures all emails sent by the API.
 * This helper fetches verification emails and extracts verification links.
 */

const MAILPIT_URL = process.env.MAILPIT_URL || "http://localhost:8025";

interface MailpitMessage {
  ID: string;
  To: Array<{ Address: string }>;
  Text?: string;
  HTML?: string;
}

interface MailpitSearchResult {
  messages?: MailpitMessage[];
  total?: number;
}

/**
 * Extract verification link from email content.
 */
function extractVerificationLink(content: string): string | null {
  // Look for verification URLs in the email
  // The API sends emails with links like: APP_URL/verify-email?token=xxx
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
 * Extract verification token from email content.
 */
function extractVerificationToken(content: string): string | null {
  // Look for token parameter in URLs
  const tokenMatch = content.match(/token=([a-zA-Z0-9_-]+)/i);
  return tokenMatch ? tokenMatch[1] : null;
}

/**
 * Wait for and verify email for a given user.
 *
 * @param request - Playwright API request context
 * @param email - Email address to check
 * @param maxWaitMs - Maximum time to wait for email (default 5000ms)
 */
export async function verifyEmailViaMailpit(
  request: APIRequestContext,
  email: string,
  maxWaitMs = 5000
): Promise<void> {
  const startTime = Date.now();

  while (Date.now() - startTime < maxWaitMs) {
    // Try to find the verification email
    const messagesResponse = await request.get(`${MAILPIT_URL}/api/v1/messages`, {
      timeout: 5000,
    });

    if (!messagesResponse.ok()) {
      console.warn(`Mailpit API returned ${messagesResponse.status()}`);
      await sleep(500);
      continue;
    }

    const result: MailpitSearchResult = await messagesResponse.json();
    const messages = result.messages || [];

    // Find email sent to the user
    const userEmail = messages.find((m) =>
      m.To?.some((t) => t.Address.toLowerCase() === email.toLowerCase())
    );

    if (userEmail) {
      // Get full message content
      const messageResponse = await request.get(
        `${MAILPIT_URL}/api/v1/message/${userEmail.ID}`,
        { timeout: 5000 }
      );

      if (messageResponse.ok()) {
        const message: MailpitMessage = await messageResponse.json();
        const content = message.Text || message.HTML || "";

        // Extract and follow verification link
        const verificationLink = extractVerificationLink(content);
        if (verificationLink) {
          // Follow the verification link
          await request.get(verificationLink, {
            timeout: 10000,
            failOnStatusCode: false, // Don't fail - the link might redirect
          });
          return;
        }

        // If no link found, try to extract token and call API directly
        const token = extractVerificationToken(content);
        if (token) {
          await request.post("/api/v1/auth/verify-email", {
            data: { token },
            timeout: 10000,
            failOnStatusCode: false,
          });
          return;
        }
      }
    }

    await sleep(500);
  }

  throw new Error(`No verification email found for ${email} within ${maxWaitMs}ms`);
}

/**
 * Register a user and verify their email.
 *
 * @param request - Playwright API request context
 * @param userData - User registration data
 * @returns User credentials
 */
export async function registerAndVerifyUser(
  request: APIRequestContext,
  userData: {
    email: string;
    password: string;
    firstName: string;
    lastName: string;
  }
): Promise<typeof userData> {
  // Register the user
  const registerResponse = await request.post("/api/v1/register", {
    data: {
      email: userData.email,
      first_name: userData.firstName,
      last_name: userData.lastName,
      password: userData.password,
    },
  });

  if (!registerResponse.ok()) {
    throw new Error(
      `Registration failed: ${registerResponse.status()} ${await registerResponse.text()}`
    );
  }

  // Verify email
  await verifyEmailViaMailpit(request, userData.email);

  return userData;
}

/**
 * Generate unique test user credentials.
 */
export function generateTestUser() {
  const timestamp = Date.now();
  return {
    email: `test-${timestamp}@hook0.local`,
    password: `TestPassword123!${timestamp}`,
    firstName: "Test",
    lastName: "User",
  };
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
