import { APIRequestContext } from "@playwright/test";
import { Client, QueryResult } from "pg";

/**
 * Email verification helper for E2E tests.
 *
 * In CI, we use direct database verification for reliability.
 * Locally with docker-compose, Mailpit can also be used.
 */

const MAILPIT_URL = process.env.MAILPIT_URL || "http://localhost:8025";
const DATABASE_URL =
  process.env.DATABASE_URL || "postgres://postgres:postgres@localhost:5432/hook0";

/**
 * API base URL for direct API calls in tests.
 * The API always runs on port 8081 (both locally and in CI).
 * The frontend is served on port 8001 but doesn't proxy API requests.
 */
export const API_BASE_URL = "http://localhost:8081/api/v1";

/**
 * Result of database verification including organization ID.
 */
export interface VerificationResult {
  organizationId: string | null;
}

/**
 * Verify user email directly via PostgreSQL and return the organization ID.
 * This is the most reliable method in CI where SMTP delivery may be slow/unreliable.
 */
export async function verifyEmailViaDatabase(email: string): Promise<VerificationResult> {
  console.log(`[DB Verify] Attempting database verification for ${email}`);
  console.log(`[DB Verify] DATABASE_URL: ${DATABASE_URL}`);

  const client = new Client({
    connectionString: DATABASE_URL,
  });

  let organizationId: string | null = null;

  return client
    .connect()
    .then(() => {
      console.log(`[DB Verify] Connected to database`);
      return client.query(
        "UPDATE iam.user SET email_verified_at = NOW() WHERE email = $1 AND email_verified_at IS NULL RETURNING user__id",
        [email]
      );
    })
    .then((result: QueryResult) => {
      console.log(`[DB Verify] Update result rowCount: ${result.rowCount}`);
      if (result.rowCount === 0) {
        console.warn(`[DB Verify] No user found with email ${email} or already verified`);
        // Try to get user ID anyway
        return client.query("SELECT user__id FROM iam.user WHERE email = $1", [email]);
      }
      return result;
    })
    .then((result: QueryResult) => {
      if (result.rows.length > 0) {
        const userId = result.rows[0].user__id;
        console.log(`[DB Verify] User ID: ${userId}`);
        // Get the organization ID for this user
        return client.query(
          "SELECT organization__id FROM iam.user__organization WHERE user__id = $1 LIMIT 1",
          [userId]
        );
      }
      return { rows: [], command: "", rowCount: 0, oid: 0, fields: [] } as QueryResult;
    })
    .then((result: QueryResult) => {
      if (result.rows.length > 0) {
        organizationId = result.rows[0].organization__id;
        console.log(`[DB Verify] Organization ID: ${organizationId}`);
      }
      return { organizationId };
    })
    .catch((error: Error) => {
      console.error(`[DB Verify] Database verification failed for ${email}:`, error);
      throw new Error(`Failed to verify email via database for ${email}`);
    })
    .finally(() => {
      return client.end();
    });
}

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
  const tokenMatch = content.match(/token=([a-zA-Z0-9_-]+)/i);
  return tokenMatch ? tokenMatch[1] : null;
}

/**
 * Verify user email via Mailpit (email-based verification).
 * In CI, uses database verification directly for reliability.
 * Locally, tries Mailpit first, then falls back to database.
 * Returns the organization ID for the user.
 */
export async function verifyEmailViaMailpit(
  request: APIRequestContext,
  email: string,
  maxWaitMs = 10000
): Promise<VerificationResult> {
  // In CI, use database verification directly - it's the most reliable method
  // Mailpit token verification can fail due to timing/token format issues
  if (process.env.CI) {
    return verifyEmailViaDatabase(email);
  }

  // Locally, try Mailpit first
  const startTime = Date.now();

  while (Date.now() - startTime < maxWaitMs) {
    const messagesResponse = await request
      .get(`${MAILPIT_URL}/api/v1/messages`, {
        timeout: 5000,
      })
      .catch(() => null);

    if (messagesResponse && messagesResponse.ok()) {
      const result: MailpitSearchResult = await messagesResponse.json();
      const messages = result.messages || [];

      const userEmail = messages.find((m) =>
        m.To?.some((t) => t.Address.toLowerCase() === email.toLowerCase())
      );

      if (userEmail) {
        const messageResponse = await request
          .get(`${MAILPIT_URL}/api/v1/message/${userEmail.ID}`, { timeout: 5000 })
          .catch(() => null);

        if (messageResponse && messageResponse.ok()) {
          const message: MailpitMessage = await messageResponse.json();
          const content = message.Text || message.HTML || "";

          // Extract token and call API directly
          const token = extractVerificationToken(content);
          if (token) {
            const verifyResponse = await request.post(`${API_BASE_URL}/auth/verify-email`, {
              data: { token },
              timeout: 10000,
              failOnStatusCode: false,
            });
            if (verifyResponse.ok()) {
              // Still need to get org ID from database
              return verifyEmailViaDatabase(email);
            }
            // If API verification failed, fall through to database
          }
        }
      }
    }

    await sleep(500);
  }

  // Fallback: verify directly via database
  console.log(`Email verification via Mailpit failed, falling back to database for ${email}`);
  return verifyEmailViaDatabase(email);
}

/**
 * User data with organization ID.
 */
export interface RegisteredUser {
  email: string;
  password: string;
  firstName: string;
  lastName: string;
  organizationId: string;
}

/**
 * Register a user and verify their email.
 * Returns user data including the auto-created organization ID.
 */
export async function registerAndVerifyUser(
  request: APIRequestContext,
  userData: {
    email: string;
    password: string;
    firstName: string;
    lastName: string;
  }
): Promise<RegisteredUser> {
  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
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

  const verificationResult = await verifyEmailViaMailpit(request, userData.email);

  if (!verificationResult.organizationId) {
    throw new Error(`No organization found for user ${userData.email}`);
  }

  return {
    ...userData,
    organizationId: verificationResult.organizationId,
  };
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

/**
 * Extract password reset token from email sent to Mailpit.
 * Password reset tokens are Biscuit cryptographic tokens sent via email,
 * not stored in the database, so we must extract them from the email content.
 *
 * This function specifically looks for emails containing "reset-password" links
 * to distinguish from verification emails.
 *
 * Note: The token in text emails may span multiple lines, so we need to
 * remove line breaks before extracting.
 */
export async function getPasswordResetTokenFromMailpit(
  request: APIRequestContext,
  email: string,
  maxWaitMs = 15000
): Promise<string> {
  const startTime = Date.now();

  while (Date.now() - startTime < maxWaitMs) {
    const messagesResponse = await request
      .get(`${MAILPIT_URL}/api/v1/messages`, { timeout: 5000 })
      .catch(() => null);

    if (messagesResponse && messagesResponse.ok()) {
      const result: MailpitSearchResult = await messagesResponse.json();
      const messages = result.messages || [];

      // Find all emails for this user
      const userEmails = messages.filter((m) =>
        m.To?.some((t) => t.Address.toLowerCase() === email.toLowerCase())
      );

      // Check each email to find the password reset one
      for (const userEmail of userEmails) {
        const messageResponse = await request
          .get(`${MAILPIT_URL}/api/v1/message/${userEmail.ID}`, { timeout: 5000 })
          .catch(() => null);

        if (messageResponse && messageResponse.ok()) {
          const message: MailpitMessage = await messageResponse.json();
          const content = message.Text || message.HTML || "";

          // Only consider emails that contain reset-password link (not verify-email)
          if (content.includes("reset-password")) {
            // Remove line breaks to handle multi-line tokens
            const cleanedContent = content.replace(/[\r\n]+/g, "");

            const tokenMatch = cleanedContent.match(
              /reset-password\?token=([A-Za-z0-9_\-+/=]+)/i
            );
            if (tokenMatch) {
              return tokenMatch[1];
            }
          }
        }
      }
    }

    await sleep(500);
  }

  throw new Error(`Password reset email not found for ${email} within ${maxWaitMs}ms`);
}
