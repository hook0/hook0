import type { AxiosResponse } from 'axios';
import { parseLinkHeader } from '@/utils/parseLinkHeader';
import type { CursorPage } from '@/composables/useCursorInfiniteQuery';

/**
 * Reads the (case-insensitive) `Link` header from an axios response.
 *
 * Axios lower-cases response header keys when consumers go through `res.headers[k]`,
 * but some HTTP stacks preserve the original casing. Check both to be safe.
 */
export function readLinkHeader(response: AxiosResponse<unknown>): string | undefined {
  const headers = response.headers as Record<string, unknown> | undefined;
  if (!headers) {
    return undefined;
  }
  const lower = headers['link'];
  if (typeof lower === 'string') {
    return lower;
  }
  const upper = headers['Link'];
  if (typeof upper === 'string') {
    return upper;
  }
  return undefined;
}

/**
 * Wraps an axios response promise into a {@link CursorPage}.
 * Pulls items from `response.data` and parses next/prev cursors from the
 * `Link` header. The cursors carried in the page are the FULL URLs from the
 * Link header — they can be passed straight back into the page fetcher.
 *
 * Errors propagate as-is — the underlying axios interceptor at `@/http`
 * already converts them into the canonical `Problem` shape, and the
 * `unwrapResponse` helper used elsewhere keeps the same contract.
 */
export function unwrapCursorPage<T>(promise: Promise<AxiosResponse<T[]>>): Promise<CursorPage<T>> {
  return promise.then((res) => {
    const links = parseLinkHeader(readLinkHeader(res));
    return {
      items: res.data,
      nextCursor: links.next,
      prevCursor: links.prev,
    };
  });
}

/**
 * Follows `Link: rel="next"` until exhausted, returning the concatenated
 * list of items. Used by the legacy `list()` shims that still need a flat
 * array (e.g. dropdowns).
 *
 * @param firstPage - Fetcher for the initial page (no cursor).
 * @param byUrl - Fetcher for subsequent pages, given the absolute URL pulled
 *                from the `Link` header.
 */
export function followAllPages<T>(
  firstPage: () => Promise<CursorPage<T>>,
  byUrl: (url: string) => Promise<CursorPage<T>>
): Promise<T[]> {
  const all: T[] = [];

  function consume(page: CursorPage<T>): Promise<T[]> {
    all.push(...page.items);
    if (!page.nextCursor) {
      return Promise.resolve(all);
    }
    return byUrl(page.nextCursor).then(consume);
  }

  return firstPage().then(consume);
}
