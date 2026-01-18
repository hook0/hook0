import encoding from 'k6/encoding';

/**
 * Decode a quoted-printable encoded string.
 * Handles soft line breaks (=\r\n or =\n) and hex-encoded characters (=XX).
 *
 * @param {string} str - Quoted-printable encoded string
 * @returns {string} Decoded string
 */
function decodeQuotedPrintable(str) {
  // Remove soft line breaks (= followed by line ending)
  let decoded = str.replace(/=\r?\n/g, '');

  // Decode hex-encoded characters (=XX where XX is a hex value)
  decoded = decoded.replace(/=([0-9A-Fa-f]{2})/g, (match, hex) => {
    return String.fromCharCode(parseInt(hex, 16));
  });

  return decoded;
}

/**
 * Get the Content-Transfer-Encoding from a MIME part's headers.
 *
 * @param {object} part - MIME part object from Mailhog
 * @returns {string|null} The encoding type or null
 */
function getContentTransferEncoding(part) {
  if (!part || !part.Headers) {
    return null;
  }

  // Headers in Mailhog are arrays
  const cte = part.Headers['Content-Transfer-Encoding'];
  if (cte && cte.length > 0) {
    return cte[0].toLowerCase();
  }

  return null;
}

/**
 * Decode a MIME part body based on its Content-Transfer-Encoding.
 *
 * @param {string} body - The raw body content
 * @param {string|null} transferEncoding - The Content-Transfer-Encoding value
 * @returns {string} Decoded body
 */
function decodeBody(body, transferEncoding) {
  if (!body) {
    return '';
  }

  if (transferEncoding === 'base64') {
    try {
      // Remove any whitespace/line breaks before decoding
      const cleanBase64 = body.replace(/\s/g, '');
      return encoding.b64decode(cleanBase64, 'std', 's');
    } catch (e) {
      console.warn('Failed to decode base64 body:', e);
      return body;
    }
  }

  if (transferEncoding === 'quoted-printable') {
    return decodeQuotedPrintable(body);
  }

  // For 7bit, 8bit, or binary, return as-is
  return body;
}

/**
 * Extract body content from a MIME part, handling encoding.
 *
 * @param {object} part - MIME part object
 * @returns {string} Decoded body content
 */
function extractPartBody(part) {
  if (!part || !part.Body) {
    return '';
  }

  const encoding = getContentTransferEncoding(part);
  return decodeBody(part.Body, encoding);
}

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
    const contentEncoding = getContentTransferEncoding(email.Content);
    body = decodeBody(email.Content.Body, contentEncoding);
  }

  if (email.MIME && email.MIME.Parts) {
    // For multipart emails, search through parts
    for (const part of email.MIME.Parts) {
      body += extractPartBody(part);

      if (part.MIME && part.MIME.Parts) {
        for (const subPart of part.MIME.Parts) {
          body += extractPartBody(subPart);
        }
      }
    }
  }

  if (!body) {
    console.warn('Could not find email body');
    return null;
  }

  // Debug: log first 500 chars of decoded body
  console.log('Decoded email body (first 500 chars):', body.substring(0, 500));

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

    console.log('Extracted verification URL:', url);
    return url;
  }

  console.warn('Could not find verification link in email body');
  console.warn('Full body for debugging:', body);
  return null;
}
