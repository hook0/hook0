/**
 * Unit tests for `Hook0PaginatedList`. Exercises the pure helpers that drive
 * the prev/next disable rules and the page indicator, plus pins the i18n
 * keys expected by the SFC against `en.json` so missing-key warnings can't
 * silently leak into the UI.
 */
// Use require so the test works regardless of esModuleInterop setting in
// tsconfig (the production build sets it via Vite). ts-jest compiles to
// CommonJS where the default export of a JSON module is the object itself.
// eslint-disable-next-line @typescript-eslint/no-var-requires
const en = require('@/locales/en.json') as Record<string, unknown>;
import {
  isNextDisabled,
  isPrevDisabled,
  pageIndicator,
  PAGINATION_I18N_KEYS,
} from './Hook0PaginatedList.logic';

describe('Hook0PaginatedList — page indicator', () => {
  test('page_indicator_updates_on_navigation', () => {
    expect(pageIndicator({ currentPageIdx: 0 })).toBe(1);
    expect(pageIndicator({ currentPageIdx: 1 })).toBe(2);
    expect(pageIndicator({ currentPageIdx: 7 })).toBe(8);
  });
});

describe('Hook0PaginatedList — button disable rules', () => {
  test('prev_disabled_on_first_page', () => {
    expect(isPrevDisabled({ isLoading: false, hasPreviousPage: false })).toBe(true);
  });

  test('prev enabled when on a later page and not loading', () => {
    expect(isPrevDisabled({ isLoading: false, hasPreviousPage: true })).toBe(false);
  });

  test('prev disabled while loading even if previous page exists', () => {
    expect(isPrevDisabled({ isLoading: true, hasPreviousPage: true })).toBe(true);
  });

  test('next_disabled_on_last_page', () => {
    expect(isNextDisabled({ isLoading: false, hasNextPage: false })).toBe(true);
  });

  test('next enabled when more pages exist and not loading', () => {
    expect(isNextDisabled({ isLoading: false, hasNextPage: true })).toBe(false);
  });

  test('next disabled while loading even if next page exists', () => {
    expect(isNextDisabled({ isLoading: true, hasNextPage: true })).toBe(true);
  });
});

describe('Hook0PaginatedList — i18n strings render correctly', () => {
  function deepGet(obj: unknown, path: string): unknown {
    return path.split('.').reduce<unknown>((acc, key) => {
      if (acc !== null && typeof acc === 'object' && key in (acc as Record<string, unknown>)) {
        return (acc as Record<string, unknown>)[key];
      }
      return undefined;
    }, obj);
  }

  test('i18n_strings_render_correctly — every pagination key exists in en.json', () => {
    for (const key of PAGINATION_I18N_KEYS) {
      const value = deepGet(en, key);
      expect(typeof value).toBe('string');
      // Catch the obvious "raw key leaked into UI" regression.
      expect(value).not.toBe(key);
      expect((value as string).length).toBeGreaterThan(0);
    }
  });

  test('pagination.currentPage uses the {page} interpolation token', () => {
    const pagination = (en as { pagination: { currentPage: string } }).pagination;
    expect(pagination.currentPage).toContain('{page}');
  });
});
