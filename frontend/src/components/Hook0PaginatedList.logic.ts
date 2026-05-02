/**
 * Pure helpers used by `Hook0PaginatedList.vue`.
 *
 * Extracted into a standalone module so they can be unit-tested without
 * mounting the SFC (the Jest setup currently has no Vue SFC transformer).
 *
 * The tests for these helpers double as the AC-20 / AC-26 contract for the
 * paginated-list component: they pin the disable rules, the page indicator,
 * and the i18n keys we depend on.
 */

export type PaginatedListState = {
  /** Currently active page index (0-based). */
  currentPageIdx: number;
  /** Whether a next page exists. */
  hasNextPage: boolean;
  /** Whether a previous page exists. */
  hasPreviousPage: boolean;
  /** Whether a fetch is in flight. */
  isLoading: boolean;
};

/** Page indicator shown in the UI; pages are 1-indexed for humans. */
export function pageIndicator(state: Pick<PaginatedListState, 'currentPageIdx'>): number {
  return state.currentPageIdx + 1;
}

/** Whether the prev button should be disabled. */
export function isPrevDisabled(
  state: Pick<PaginatedListState, 'isLoading' | 'hasPreviousPage'>
): boolean {
  return state.isLoading || !state.hasPreviousPage;
}

/** Whether the next button should be disabled. */
export function isNextDisabled(
  state: Pick<PaginatedListState, 'isLoading' | 'hasNextPage'>
): boolean {
  return state.isLoading || !state.hasNextPage;
}

/**
 * The set of i18n keys consumed by `Hook0PaginatedList.vue`.
 * The component template is the source of truth, but we keep this list here
 * so the test suite can pin them down — a missing key in `en.json` would cause
 * raw `pagination.*` strings to leak into the UI (regression of AC-26).
 */
export const PAGINATION_I18N_KEYS = [
  'pagination.previous',
  'pagination.next',
  'pagination.currentPage',
  'pagination.loading',
] as const;
