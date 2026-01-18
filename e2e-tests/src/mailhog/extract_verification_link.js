/**
 * Extract the email verification link from a Mailhog email.
 *
 * The verification link is in the format: http://.../?verify-email?token=...
 *
 * @param {object} email - Mailhog email object
 * @returns {string|null} The verification URL or null if not found
 */
export default function extractVerificationLink(email) {
  // The email body can be in different places depending on MIME structure
  let body = '';

  // Try to get the body from different locations
  if (email.Content && email.Content.Body) {
    body = email.Content.Body;
  } else if (email.MIME && email.MIME.Parts) {
    // For multipart emails, search through parts
    for (const part of email.MIME.Parts) {
      if (part.Body) {
        body += part.Body;
      }
      if (part.MIME && part.MIME.Parts) {
        for (const subPart of part.MIME.Parts) {
          if (subPart.Body) {
            body += subPart.Body;
          }
        }
      }
    }
  }

  if (!body) {
    console.warn('Could not find email body');
    return null;
  }

  // Look for the verify-email URL pattern
  // The URL contains the token as a query parameter
  const urlPattern = /https?:\/\/[^\s<>"]+verify-email[^\s<>"]+token=[^\s<>"]+/gi;
  const matches = body.match(urlPattern);

  if (matches && matches.length > 0) {
    // Clean up the URL (remove any trailing characters that might have been captured)
    let url = matches[0];
    // Remove common trailing artifacts
    url = url.replace(/[&;]$/, '');
    // Decode HTML entities if present
    url = url.replace(/&amp;/g, '&');
    return url;
  }

  console.warn('Could not find verification link in email body');
  return null;
}
