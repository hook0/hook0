import http from 'k6/http';
import { check } from 'k6';
import getEmails from '../mailhog/get_emails.js';
import extractVerificationLink from '../mailhog/extract_verification_link.js';

/**
 * Verify a user's email by fetching the verification email from Mailhog
 * and calling the verification API endpoint.
 *
 * @param {string} baseUrl - API base URL
 * @param {string} mailhogUrl - Mailhog API base URL
 * @param {string} email - User email address to verify
 * @returns {boolean} True if verification succeeded
 */
export default function verifyEmail(baseUrl, mailhogUrl, email) {
  // 1. Fetch the verification email from Mailhog
  const emails = getEmails(mailhogUrl, email);
  if (!emails || emails.length === 0) {
    console.warn(`No verification email found for ${email}`);
    return false;
  }

  // Use the most recent email
  const verificationEmail = emails[0];

  // 2. Extract the verification link
  const verificationLink = extractVerificationLink(verificationEmail);
  if (!verificationLink) {
    console.warn('Could not extract verification link from email');
    return false;
  }

  // 3. Extract the token from the URL query parameter
  const tokenMatch = verificationLink.match(/[?&]token=([^&]+)/);
  if (!tokenMatch) {
    console.warn('Could not extract token from verification link:', verificationLink);
    return false;
  }
  const token = decodeURIComponent(tokenMatch[1]);

  // 4. Call the verification API endpoint
  const url = `${baseUrl}api/v1/auth/verify-email`;
  const payload = JSON.stringify({ token: token });
  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  if (
    !check(res, {
      'Email verification succeeded': (r) => r.status === 204,
    })
  ) {
    console.warn('Email verification failed:', res.status, res.body);
    return false;
  }

  return true;
}
