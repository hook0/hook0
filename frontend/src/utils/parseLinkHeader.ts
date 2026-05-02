/**
 * Parses an RFC 8288 `Link` header value and returns the URLs for `rel="next"`
 * and `rel="prev"` if present.
 *
 * Example input:
 *   `<https://api.example.com/event_types?application_id=...&pagination_cursor=...>; rel="next", <...>; rel="prev"`
 *
 * Returns `{}` when the header is missing or empty.
 */
export type ParsedLinkHeader = {
  next?: string;
  prev?: string;
};

const ENTRY_REGEX = /<([^>]+)>\s*;\s*rel\s*=\s*"?(next|prev)"?/i;

export function parseLinkHeader(headerValue: string | null | undefined): ParsedLinkHeader {
  if (!headerValue) {
    return {};
  }

  const result: ParsedLinkHeader = {};

  // Split entries on `,` — Link header entries are comma-separated.
  // URLs in `<...>` are guaranteed not to contain raw commas per RFC 3986.
  const entries = headerValue.split(',');
  for (const entry of entries) {
    const match = ENTRY_REGEX.exec(entry.trim());
    if (!match) {
      continue;
    }
    const url = match[1].trim();
    const rel = match[2].toLowerCase();
    if (rel === 'next') {
      result.next = url;
    } else if (rel === 'prev') {
      result.prev = url;
    }
  }

  return result;
}
