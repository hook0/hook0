import { APIRequestContext } from "@playwright/test";
import { Client } from "pg";

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

  const client = new Client({ connectionString: DATABASE_URL });

  try {
    await client.connect();
    console.log(`[DB Verify] Connected to database`);

    let result = await client.query(
      "UPDATE iam.user SET email_verified_at = NOW() WHERE email = $1 AND email_verified_at IS NULL RETURNING user__id",
      [email]
    );
    console.log(`[DB Verify] Update result rowCount: ${result.rowCount}`);

    if (result.rowCount === 0) {
      console.warn(`[DB Verify] No user found with email ${email} or already verified`);
      result = await client.query("SELECT user__id FROM iam.user WHERE email = $1", [email]);
    }

    if (result.rows.length === 0) {
      return { organizationId: null };
    }

    const userId = result.rows[0].user__id;
    console.log(`[DB Verify] User ID: ${userId}`);

    const orgResult = await client.query(
      "SELECT organization__id FROM iam.user__organization WHERE user__id = $1 LIMIT 1",
      [userId]
    );

    const organizationId = orgResult.rows.length > 0 ? orgResult.rows[0].organization__id : null;
    console.log(`[DB Verify] Organization ID: ${organizationId}`);

    return { organizationId };
  } catch (error) {
    console.error(`[DB Verify] Database verification failed for ${email}:`, error);
    throw new Error(`Failed to verify email via database for ${email}`);
  } finally {
    await client.end();
  }
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
 * Extract verification token from email content.
 */
function extractVerificationToken(content: string): string | null {
  const tokenMatch = content.match(/token=([a-zA-Z0-9_%+/=-]+)/i);
  return tokenMatch ? decodeURIComponent(tokenMatch[1]) : null;
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
  organizationId?: string,
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
      .get(`${MAILPIT_URL}/api/v1/search?query=to:${encodeURIComponent(email)}`, {
        timeout: 5000,
      })
      .catch(() => null);

    if (messagesResponse && messagesResponse.ok()) {
      const result: MailpitSearchResult = await messagesResponse.json();
      const messages = result.messages || [];

      // Take the most recent message (first in search results)
      const userEmail = messages[0];

      if (userEmail) {
        const messageResponse = await request
          .get(`${MAILPIT_URL}/api/v1/message/${userEmail.ID}`, { timeout: 5000 })
          .catch(() => null);

        if (messageResponse && messageResponse.ok()) {
          const message: MailpitMessage = await messageResponse.json();
          const content = message.HTML || message.Text || "";

          // Extract token and call API directly
          const token = extractVerificationToken(content);
          if (token) {
            const verifyResponse = await request.post(`${API_BASE_URL}/auth/verify-email`, {
              data: { token },
              timeout: 10000,
              failOnStatusCode: false,
            });
            if (verifyResponse.ok()) {
              return { organizationId: organizationId ?? null };
            }
            console.log(`[Mailpit] verify-email API returned ${verifyResponse.status()}: ${await verifyResponse.text()}`);
          } else {
            console.log(`[Mailpit] No token found in email content (length=${content.length})`);
          }
        }
      } else {
        console.log(`[Mailpit] No email found for ${email} among ${messages.length} messages`);
      }
    }

    await sleep(500);
  }

  // Fallback: verify directly via database
  console.log(`Email verification via Mailpit failed, falling back to database for ${email}`);
  return verifyEmailViaDatabase(email);
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
          const content = message.HTML || message.Text || "";

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
