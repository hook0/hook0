import { computed, ref, type ComputedRef, type Ref } from 'vue';
import { useInfiniteQuery, type InfiniteData } from '@tanstack/vue-query';
import type { Problem } from '@/http';

import {
  getCurrentPageItems,
  hasNext,
  hasPrev,
  planNext,
  planPrev,
  type Page,
} from './cursorInfiniteQueryLogic';

/**
 * Shape returned by a single page fetch.
 *
 * - `items`: rows for this page.
 * - `nextCursor`: opaque cursor string OR a fully-qualified URL to fetch the
 *   next page; `undefined` when this is the last page.
 * - `prevCursor`: opaque cursor string OR a fully-qualified URL to fetch the
 *   previous page; `undefined` when this is the first page.
 *
 * Services typically extract these by parsing the `Link` header on the axios
 * response (see {@link parseLinkHeader}).
 */
export type CursorPage<T> = Page<T>;

export type UseCursorInfiniteQueryOptions = {
  enabled?: () => boolean;
};

export type UseCursorInfiniteQueryResult<T> = {
  /** All loaded pages, in fetch order. */
  pages: ComputedRef<CursorPage<T>[]>;
  /** Zero-based index of the currently displayed page. */
  currentPageIdx: Ref<number>;
  /** Items of the current page (empty array while loading). */
  currentPageItems: ComputedRef<T[]>;
  /** Total number of pages the user has navigated through (max idx reached + 1). */
  totalPagesSeen: ComputedRef<number>;
  /** Advance to the next page; fetches when no cached page is available. */
  fetchNextPage: () => Promise<void>;
  /** Step back to the previous page (always served from cache). */
  fetchPreviousPage: () => Promise<void>;
  /** Whether a next page exists (i.e. last loaded page has a `nextCursor`). */
  hasNextPage: ComputedRef<boolean>;
  /** Whether the user can navigate backward (i.e. `currentPageIdx > 0`). */
  hasPreviousPage: ComputedRef<boolean>;
  /** Whether the underlying query is currently loading or fetching. */
  isLoading: ComputedRef<boolean>;
  /** Error from the underlying query, if any. */
  error: ComputedRef<Problem | Error | null>;
  /** Force a refetch, resetting the page cursor. */
  refetch: () => Promise<void>;
};

/**
 * Cursor-based infinite query composable.
 *
 * Wraps TanStack Query's `useInfiniteQuery` with cursor parsing logic. Tracks
 * the user's current page locally so the UI can render a classical paged
 * interface (prev/next + "Page N") on top of cursor pagination.
 *
 * @param queryKey - Reactive query key factory.
 * @param fetchPage - Page fetcher; receives the cursor (or `undefined` for the
 *                    first page) and returns the page payload + neighbor cursors.
 * @param options.enabled - Optional reactive predicate disabling the query.
 */
export function useCursorInfiniteQuery<T>(
  queryKey: () => unknown[],
  fetchPage: (cursor: string | undefined) => Promise<CursorPage<T>>,
  options?: UseCursorInfiniteQueryOptions
): UseCursorInfiniteQueryResult<T> {
  const currentPageIdx = ref(0);

  const query = useInfiniteQuery<
    CursorPage<T>,
    unknown,
    InfiniteData<CursorPage<T>>,
    unknown[],
    string | undefined
  >({
    queryKey: computed(queryKey),
    queryFn: ({ pageParam }) => fetchPage(pageParam),
    initialPageParam: undefined,
    getNextPageParam: (lastPage) => lastPage.nextCursor,
    getPreviousPageParam: (firstPage) => firstPage.prevCursor,
    enabled: computed(() => (options?.enabled ? options.enabled() : true)),
  });

  const pages = computed<CursorPage<T>[]>(() => query.data.value?.pages ?? []);

  const navState = () => ({ pages: pages.value, currentPageIdx: currentPageIdx.value });

  const currentPageItems = computed<T[]>(() => getCurrentPageItems(navState()));
  const totalPagesSeen = computed(() => pages.value.length);
  const hasNextPage = computed<boolean>(() => hasNext(navState()));
  const hasPreviousPage = computed<boolean>(() => hasPrev(navState()));

  const isLoading = computed<boolean>(
    () =>
      query.isLoading.value || query.isFetchingNextPage.value || query.isFetchingPreviousPage.value
  );

  const error = computed<Problem | Error | null>(() => {
    const e = query.error.value;
    if (e === null || e === undefined) {
      return null;
    }
    // The underlying axios error is propagated as `Error`; the cursor pagination
    // wrappers don't transform it into a `Problem`. `Hook0ErrorCard` accepts
    // both shapes — return whichever we have so the caller can type-narrow.
    return e as Problem | Error;
  });

  function fetchNextPage(): Promise<void> {
    const plan = planNext(navState());
    if (plan === 'cached') {
      currentPageIdx.value += 1;
      return Promise.resolve();
    }
    if (plan === 'noop') {
      return Promise.resolve();
    }
    return query.fetchNextPage().then(() => {
      currentPageIdx.value += 1;
    });
  }

  function fetchPreviousPage(): Promise<void> {
    if (planPrev(navState()) === 'noop') {
      return Promise.resolve();
    }
    currentPageIdx.value -= 1;
    return Promise.resolve();
  }

  function refetch(): Promise<void> {
    currentPageIdx.value = 0;
    return query.refetch().then(() => undefined);
  }

  return {
    pages,
    currentPageIdx,
    currentPageItems,
    totalPagesSeen,
    fetchNextPage,
    fetchPreviousPage,
    hasNextPage,
    hasPreviousPage,
    isLoading,
    error,
    refetch,
  };
}
