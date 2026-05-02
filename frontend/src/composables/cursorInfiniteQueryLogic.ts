/**
 * Pure helpers for {@link useCursorInfiniteQuery}, extracted so they can be
 * unit-tested without booting a Vue app + TanStack QueryClient.
 *
 * The composable is responsible for wiring these helpers into reactive refs;
 * the helpers themselves know nothing about Vue.
 */

export type Page<T> = {
  items: T[];
  nextCursor?: string;
  prevCursor?: string;
};

export type NavState<T> = {
  pages: Page<T>[];
  currentPageIdx: number;
};

/** Items currently visible to the user. */
export function getCurrentPageItems<T>(state: NavState<T>): T[] {
  if (state.pages.length === 0) {
    return [];
  }
  const idx = Math.min(state.currentPageIdx, state.pages.length - 1);
  return state.pages[idx]?.items ?? [];
}

/**
 * `true` when the user can move forward — either there's a cached page
 * after the current one, or the last loaded page carries a `nextCursor`.
 */
export function hasNext<T>(state: NavState<T>): boolean {
  if (state.pages.length === 0) {
    return false;
  }
  if (state.currentPageIdx < state.pages.length - 1) {
    return true;
  }
  return state.pages[state.pages.length - 1]?.nextCursor !== undefined;
}

/** `true` when the user is past the first page. */
export function hasPrev<T>(state: NavState<T>): boolean {
  return state.currentPageIdx > 0;
}

/**
 * Plan for advancing to the next page.
 * - `cached`: the next page is already in `pages`; just bump the index.
 * - `fetch`: the user is on the last loaded page and a `nextCursor` exists.
 * - `noop`: there is no further page to load.
 */
export type NextPagePlan = 'cached' | 'fetch' | 'noop';

export function planNext<T>(state: NavState<T>): NextPagePlan {
  if (state.pages.length === 0) {
    return 'noop';
  }
  if (state.currentPageIdx < state.pages.length - 1) {
    return 'cached';
  }
  return state.pages[state.pages.length - 1]?.nextCursor !== undefined ? 'fetch' : 'noop';
}

export type PrevPagePlan = 'cached' | 'noop';

export function planPrev<T>(state: NavState<T>): PrevPagePlan {
  return state.currentPageIdx > 0 ? 'cached' : 'noop';
}
