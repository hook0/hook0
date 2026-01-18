import encoding from 'k6/encoding';

/**
 * Decode a base64 string to UTF-8 text.
 *
 * @param {string} base64Str - Base64 encoded string
 * @returns {string} Decoded string
 */
function decodeBase64(base64Str) {
  try {
    // Remove any whitespace/line breaks before decoding
    const cleanBase64 = base64Str.replace(/\s/g, '');
    return encoding.b64decode(cleanBase64, 'std', 's');
  } catch (e) {
    console.warn('Failed to decode base64:', e);
    return base64Str;
  }
}

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
 * Parse MIME parts from a raw multipart email body.
 * This handles the case where Mailhog returns the raw MIME content.
 *
 * @param {string} rawBody - Raw multipart email body
 * @returns {Array<{contentType: string, encoding: string, body: string}>} Parsed parts
 */
function parseMimeParts(rawBody) {
  const parts = [];

  // Find the boundary from the first line (e.g., "--boundary")
  const boundaryMatch = rawBody.match(/^--([^\r\n]+)/m);
  if (!boundaryMatch) {
    // Not a multipart email, return the whole body
    return [{ contentType: 'text/plain', encoding: null, body: rawBody }];
  }

  const boundary = boundaryMatch[1];
  const boundaryRegex = new RegExp(`--${boundary.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`, 'g');

  // Split by boundary
  const rawParts = rawBody.split(boundaryRegex);

  for (const rawPart of rawParts) {
    if (!rawPart || rawPart.trim() === '' || rawPart.trim() === '--') {
      continue;
    }

    // Split headers from body (empty line separates them)
    const headerBodySplit = rawPart.split(/\r?\n\r?\n/);
    if (headerBodySplit.length < 2) {
      continue;
    }

    const headerSection = headerBodySplit[0];
    const bodySection = headerBodySplit.slice(1).join('\n\n');

    // Parse headers
    let contentType = 'text/plain';
    let transferEncoding = null;

    const contentTypeMatch = headerSection.match(/Content-Type:\s*([^\r\n;]+)/i);
    if (contentTypeMatch) {
      contentType = contentTypeMatch[1].trim();
    }

    const encodingMatch = headerSection.match(/Content-Transfer-Encoding:\s*([^\r\n]+)/i);
    if (encodingMatch) {
      transferEncoding = encodingMatch[1].trim().toLowerCase();
    }

    parts.push({
      contentType,
      encoding: transferEncoding,
      body: bodySection,
    });
  }

  return parts;
}

/**
 * Decode MIME part body based on its Content-Transfer-Encoding.
 *
 * @param {string} body - The raw body content
 * @param {string|null} transferEncoding - The Content-Transfer-Encoding value
 * @returns {string} Decoded body
 */
function decodePartBody(body, transferEncoding) {
  if (!body) {
    return '';
  }

  if (transferEncoding === 'base64') {
    return decodeBase64(body);
  }

  if (transferEncoding === 'quoted-printable') {
    return decodeQuotedPrintable(body);
  }

  // For 7bit, 8bit, or binary, return as-is
  return body;
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
  let fullDecodedBody = '';

  // Try to get the body from Content.Body (raw multipart content)
  if (email.Content && email.Content.Body) {
    const rawBody = email.Content.Body;

    // Parse MIME parts from the raw body
    const parts = parseMimeParts(rawBody);

    for (const part of parts) {
      // Decode and accumulate all text parts
      if (part.contentType.startsWith('text/')) {
        const decodedPart = decodePartBody(part.body, part.encoding);
        fullDecodedBody += decodedPart + '\n';
      }
    }
  }

  // Also check MIME.Parts structure (alternative Mailhog format)
  if (email.MIME && email.MIME.Parts) {
    for (const part of email.MIME.Parts) {
      if (part.Body) {
        // Check for nested multipart
        if (part.Body.includes('Content-Transfer-Encoding')) {
          const nestedParts = parseMimeParts(part.Body);
          for (const nested of nestedParts) {
            if (nested.contentType.startsWith('text/')) {
              fullDecodedBody += decodePartBody(nested.body, nested.encoding) + '\n';
            }
          }
        } else {
          // Get encoding from part headers
          let encoding = null;
          if (part.Headers && part.Headers['Content-Transfer-Encoding']) {
            encoding = part.Headers['Content-Transfer-Encoding'][0].toLowerCase();
          }
          fullDecodedBody += decodePartBody(part.Body, encoding) + '\n';
        }
      }

      // Check nested MIME parts
      if (part.MIME && part.MIME.Parts) {
        for (const subPart of part.MIME.Parts) {
          if (subPart.Body) {
            let encoding = null;
            if (subPart.Headers && subPart.Headers['Content-Transfer-Encoding']) {
              encoding = subPart.Headers['Content-Transfer-Encoding'][0].toLowerCase();
            }
            fullDecodedBody += decodePartBody(subPart.Body, encoding) + '\n';
          }
        }
      }
    }
  }

  if (!fullDecodedBody) {
    console.warn('Could not find email body');
    return null;
  }

  // Debug: log first 500 chars of decoded body
  console.log('Decoded email body (first 500 chars):', fullDecodedBody.substring(0, 500));

  // Look for the verify-email URL pattern
  // The URL contains the token as a query parameter
  const urlPattern = /https?:\/\/[^\s<>"]+verify-email[^\s<>"]+token=[^\s<>"]+/gi;
  const matches = fullDecodedBody.match(urlPattern);

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
  console.warn('Full body for debugging:', fullDecodedBody.substring(0, 2000));
  return null;
}
