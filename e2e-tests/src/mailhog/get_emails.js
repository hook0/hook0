import http from 'k6/http';
import { check, sleep } from 'k6';

/**
 * Get all emails from Mailhog for a specific recipient.
 *
 * @param {string} mailhogUrl - Mailhog API base URL (e.g., http://mailhog:8025)
 * @param {string} recipientEmail - Email address to search for
 * @param {number} maxAttempts - Maximum number of attempts to find the email (default: 10)
 * @param {number} delayMs - Delay between attempts in milliseconds (default: 500)
 * @returns {Array|null} Array of matching emails or null if none found
 */
export default function getEmails(mailhogUrl, recipientEmail, maxAttempts = 10, delayMs = 500) {
  const url = `${mailhogUrl}/api/v2/search?kind=to&query=${encodeURIComponent(recipientEmail)}`;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    const res = http.get(url);

    if (
      !check(res, {
        'Mailhog API responded': (r) => r.status === 200,
      })
    ) {
      console.warn(`Mailhog API error (attempt ${attempt}):`, res.status, res.body);
      sleep(delayMs / 1000);
      continue;
    }

    const data = JSON.parse(res.body);

    if (data.total > 0) {
      return data.items;
    }

    if (attempt < maxAttempts) {
      sleep(delayMs / 1000);
    }
  }

  console.warn(`No emails found for ${recipientEmail} after ${maxAttempts} attempts`);
  return null;
}
