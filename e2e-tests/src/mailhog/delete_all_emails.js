import http from 'k6/http';
import { check } from 'k6';

/**
 * Delete all emails from Mailhog.
 *
 * @param {string} mailhogUrl - Mailhog API base URL (e.g., http://mailhog:8025)
 * @returns {boolean} True if deletion succeeded
 */
export default function deleteAllEmails(mailhogUrl) {
  const url = `${mailhogUrl}/api/v1/messages`;

  const res = http.del(url);

  if (
    !check(res, {
      'Mailhog delete all emails succeeded': (r) => r.status === 200,
    })
  ) {
    console.warn('Mailhog delete all emails failed:', res.status, res.body);
    return false;
  }

  return true;
}
