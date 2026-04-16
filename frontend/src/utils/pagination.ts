/**
 * Generic bidirectional cursor pagination utilities.
 *
 * Parses RFC 8288 Link headers emitted by the API and provides
 * shared types for any paginated endpoint.
 */

export const PARAM_NEXT_CURSOR = 'pagination_cursor';
export const PARAM_PREV_CURSOR = 'pagination_before_cursor';

export type PaginationDirection = 'forward' | 'backward';

export type CursorPage<T> = {
  data: T[];
  nextCursor: string | null;
  prevCursor: string | null;
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
export function parseLinkHeader(linkHeader: string | null): {
  next: string | null;
  prev: string | null;
} {
  const result = { next: null as string | null, prev: null as string | null };

  if (!linkHeader) {
    return result;
  }

  for (const part of linkHeader.split(',')) {
    const match = part.match(/<([^>]+)>;\s*rel="(\w+)"/);
    if (!match) continue;

    const [, urlString, rel] = match;

    try {
      // Dummy base: Link header may contain relative URLs
      const params = new URL(urlString, 'http://x').searchParams;

      if (rel === 'next') {
        result.next = params.get(PARAM_NEXT_CURSOR);
      } else if (rel === 'prev') {
        result.prev = params.get(PARAM_PREV_CURSOR);
      }
    } catch {
      // Malformed URL — skip rather than crash
      console.error('Failed to parse Link header URL:', urlString);
    }
  }

  return result;
}
