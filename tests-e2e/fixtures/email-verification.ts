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
 * Verify user email directly via PostgreSQL.
 * This is the most reliable method in CI where SMTP delivery may be slow/unreliable.
 */
export async function verifyEmailViaDatabase(email: string): Promise<void> {
  console.log(`Attempting database verification for ${email}`);
  console.log(`DATABASE_URL: ${DATABASE_URL}`);

  const client = new Client({
    connectionString: DATABASE_URL,
  });

  return client
    .connect()
    .then(() => {
      console.log(`Connected to database successfully`);
      // First, check if user exists
      return client.query("SELECT user__id, email, email_verified_at FROM iam.user WHERE email = $1", [
        email,
      ]);
    })
    .then((selectResult) => {
      console.log(`User lookup result: ${JSON.stringify(selectResult.rows)}`);
      if (selectResult.rows.length === 0) {
        console.warn(`No user found with email ${email} in database`);
        // List all users for debugging
        return client
          .query("SELECT email FROM iam.user LIMIT 10")
          .then((allUsers) => {
            console.log(`Users in database: ${JSON.stringify(allUsers.rows)}`);
            return { rowCount: 0 };
          });
      }
      // User exists, update email_verified_at
      return client.query(
        "UPDATE iam.user SET email_verified_at = NOW() WHERE email = $1 AND email_verified_at IS NULL",
        [email]
      );
    })
    .then((result) => {
      if (result.rowCount === 0) {
        console.warn(`No user found with email ${email} or already verified`);
      } else {
        console.log(`Successfully verified email for ${email}`);
      }
    })
    .catch((error) => {
      console.error(`Database verification failed for ${email}:`, error);
      throw new Error(`Failed to verify email via database for ${email}: ${error.message}`);
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
 * Falls back to database verification if email is not found.
 */
export async function verifyEmailViaMailpit(
  request: APIRequestContext,
  email: string,
  maxWaitMs = 10000
): Promise<void> {
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

          const verificationLink = extractVerificationLink(content);
          if (verificationLink) {
            await request.get(verificationLink, {
              timeout: 10000,
              failOnStatusCode: false,
            });
            return;
          }

          const token = extractVerificationToken(content);
          if (token) {
            await request.post("/api/v1/verify-email", {
              data: { token },
              timeout: 10000,
              failOnStatusCode: false,
            });
            return;
          }
        }
      }
    }

    await sleep(500);
  }

  // Fallback: verify directly via database
  console.log(`Email not found in Mailpit, falling back to database verification for ${email}`);
  await verifyEmailViaDatabase(email);
}

/**
 * Register a user and verify their email.
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
