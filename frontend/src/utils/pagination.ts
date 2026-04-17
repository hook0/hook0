/**
 * Bidirectional cursor pagination shared across endpoints.
 *
 * Consumers get cursor extraction from Link headers, typed direction
 * enum, and safe parsers for URL query params.
 *
 * - `parseLinkHeader` reads RFC 8288 `<url>; rel="next|prev"`.
 * - `parseCursorFromQuery` / `parseDirectionFromQuery` coerce raw
 *   `LocationQueryValue` into typed values, rejecting arrays.
 * - `CursorPage<T>` wraps API list responses.
 */

import type { LocationQueryValue } from 'vue-router';

export const PARAM_NEXT_CURSOR = 'pagination_cursor';
export const PARAM_PREV_CURSOR = 'pagination_before_cursor';

export type PaginationDirection = 'forward' | 'backward';

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
    const match = part.match(/<([^>]+)>;\s*rel="(\w+)"/);
    if (!match) continue;

    const [, urlString, rel] = match;

    try {
      // Dummy base: Link header may contain relative URLs
      const params = new URL(urlString, 'http://x').searchParams;

      if (rel === 'next') {
        parsed.next = params.get(PARAM_NEXT_CURSOR);
      } else if (rel === 'prev') {
        parsed.prev = params.get(PARAM_PREV_CURSOR);
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

/**
 * Reads a direction from a URL query value. Defaults to 'forward' on anything unexpected.
 *
 * @example
 * parseDirectionFromQuery('backward') // => 'backward'
 * parseDirectionFromQuery('forward')  // => 'forward'
 * parseDirectionFromQuery(['x'])      // => 'forward'
 * parseDirectionFromQuery(undefined)  // => 'forward'
 */
export function parseDirectionFromQuery(
  value: LocationQueryValue | LocationQueryValue[]
): PaginationDirection {
  return value === 'backward' ? 'backward' : 'forward';
}
