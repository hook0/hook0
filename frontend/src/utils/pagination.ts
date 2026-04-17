/**
 * Generic cursor pagination utilities.
 *
 * The cursor is opaque to the client — direction is baked into the blob
 * by the server. Callers just follow the `Link: <url>; rel="next|prev"`
 * headers and pass the cursor back as-is.
 *
 * - `parseLinkHeader` extracts next/prev cursors from RFC 8288 Link.
 * - `parseCursorFromQuery` safely reads a cursor from a URL query value.
 * - `CursorPage<T>` wraps API list responses.
 */

import type { LocationQueryValue } from 'vue-router';
import type { operations } from '@/types';

// Any paginated endpoint as a reference. Fails compilation if the backend
// renames the pagination convention.
type PaginatedQuery = operations['requestAttempts.read']['parameters']['query'];
type PaginationQueryParam = Extract<keyof PaginatedQuery, `pagination_${string}`>;

export const PARAM_CURSOR = 'pagination_cursor' satisfies PaginationQueryParam;

export type CursorPage<T> = {
  data: T[];
  nextCursor: string | null;
  prevCursor: string | null;
};

export type ParsedLinkCursors = {
  next: string | null;
  prev: string | null;
};

/**
 * Extracts next/prev cursors from an RFC 8288 Link header.
 *
 * @example
 * parseLinkHeader('<https://api.hook0.com/ep?pagination_cursor=abc>; rel="next"')
 * // => { next: 'abc', prev: null }
 *
 * @example
 * parseLinkHeader(null)
 * // => { next: null, prev: null }
 */
export function parseLinkHeader(linkHeader: string | null): ParsedLinkCursors {
  const parsed: ParsedLinkCursors = { next: null, prev: null };

  if (!linkHeader) {
    return parsed;
  }

  for (const part of linkHeader.split(',')) {
    const urlStart = part.indexOf('<');
    const urlEnd = part.indexOf('>', urlStart);
    if (urlStart === -1 || urlEnd === -1) continue;

    const urlString = part.slice(urlStart + 1, urlEnd);

    // RFC 8288: rel can be quoted or unquoted, may appear in any order.
    const relMatch = part.match(/;\s*rel\s*=\s*"?([\w-]+)"?/);
    if (!relMatch) continue;

    const rel = relMatch[1];

    try {
      // Dummy base: Link header may contain relative URLs
      const cursor = new URL(urlString, 'http://x').searchParams.get(PARAM_CURSOR);
      if (cursor === null) continue;

      if (rel === 'next') {
        parsed.next = cursor;
      } else if (rel === 'prev') {
        parsed.prev = cursor;
      }
    } catch {
      console.error('Failed to parse Link header URL:', urlString);
    }
  }

  return parsed;
}

/**
 * Reads a cursor from a URL query value. Rejects arrays and non-strings.
 *
 * @example
 * parseCursorFromQuery('abc123')  // => 'abc123'
 * parseCursorFromQuery(['a', 'b']) // => null
 * parseCursorFromQuery(undefined) // => null
 */
export function parseCursorFromQuery(
  value: LocationQueryValue | LocationQueryValue[]
): string | null {
  return typeof value === 'string' ? value : null;
}
