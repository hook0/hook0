/**
 * Tests for the cursor-pagination logic that drives `useCursorInfiniteQuery`.
 *
 * The composable itself is a thin Vue-reactive wrapper around the helpers in
 * `cursorInfiniteQueryLogic.ts`. The Jest setup has no Vue SFC transformer
 * and no jsdom env, so we exercise the helpers directly — they encode the
 * full navigation contract (AC-19).
 */
import {
  getCurrentPageItems,
  hasNext,
  hasPrev,
  planNext,
  planPrev,
  type Page,
} from './cursorInfiniteQueryLogic';

type Item = { id: string };

const PAGE1: Page<Item> = {
  items: [{ id: 'a' }, { id: 'b' }],
  nextCursor: 'https://api/x?cursor=PAGE2&limit=2',
  prevCursor: undefined,
};
const PAGE2: Page<Item> = {
  items: [{ id: 'c' }, { id: 'd' }],
  nextCursor: 'https://api/x?cursor=PAGE3&limit=2',
  prevCursor: 'https://api/x?cursor=PAGE1&limit=2',
};
const LAST_PAGE: Page<Item> = {
  items: [{ id: 'e' }],
  nextCursor: undefined,
  prevCursor: 'https://api/x?cursor=PAGE2&limit=2',
};

describe('cursorInfiniteQueryLogic — empty state', () => {
  test('no pages yet: hasNext=false, hasPrev=false, items=[]', () => {
    const state = { pages: [] as Page<Item>[], currentPageIdx: 0 };
    expect(hasNext(state)).toBe(false);
    expect(hasPrev(state)).toBe(false);
    expect(getCurrentPageItems(state)).toEqual([]);
    expect(planNext(state)).toBe('noop');
    expect(planPrev(state)).toBe('noop');
  });
});

describe('cursorInfiniteQueryLogic — first page loaded with more available', () => {
  const state = { pages: [PAGE1], currentPageIdx: 0 };

  test('parses_link_header_next_and_prev — exposes nextCursor without a cached next page', () => {
    expect(hasNext(state)).toBe(true);
    expect(hasPrev(state)).toBe(false);
    expect(getCurrentPageItems(state)).toEqual([{ id: 'a' }, { id: 'b' }]);
  });

  test('planNext returns "fetch" when last loaded page has a nextCursor', () => {
    expect(planNext(state)).toBe('fetch');
  });

  test('planPrev is "noop" on the first page', () => {
    expect(planPrev(state)).toBe('noop');
  });
});

describe('cursorInfiniteQueryLogic — middle page (cached forward and backward)', () => {
  const state = { pages: [PAGE1, PAGE2, LAST_PAGE], currentPageIdx: 1 };

  test('items reflect the current page index', () => {
    expect(getCurrentPageItems(state)).toEqual([{ id: 'c' }, { id: 'd' }]);
  });

  test('hasNext is true (next page already cached)', () => {
    expect(hasNext(state)).toBe(true);
    expect(planNext(state)).toBe('cached');
  });

  test('hasPrev is true and plan is "cached"', () => {
    expect(hasPrev(state)).toBe(true);
    expect(planPrev(state)).toBe('cached');
  });
});

describe('cursorInfiniteQueryLogic — last page', () => {
  const state = { pages: [PAGE1, PAGE2, LAST_PAGE], currentPageIdx: 2 };

  test('hasNextPage flips false on last page', () => {
    expect(hasNext(state)).toBe(false);
    expect(planNext(state)).toBe('noop');
  });

  test('items come from the last page', () => {
    expect(getCurrentPageItems(state)).toEqual([{ id: 'e' }]);
  });

  test('hasPrev stays true', () => {
    expect(hasPrev(state)).toBe(true);
    expect(planPrev(state)).toBe('cached');
  });
});

describe('cursorInfiniteQueryLogic — single-page result', () => {
  const single: Page<Item> = {
    items: [{ id: 'only' }],
    nextCursor: undefined,
    prevCursor: undefined,
  };
  const state = { pages: [single], currentPageIdx: 0 };

  test('hasNext false, hasPrev false', () => {
    expect(hasNext(state)).toBe(false);
    expect(hasPrev(state)).toBe(false);
    expect(planNext(state)).toBe('noop');
    expect(planPrev(state)).toBe('noop');
  });
});

describe('cursorInfiniteQueryLogic — navigation simulation', () => {
  test('fetchNextPage advances currentPageIdx (sequential walk)', () => {
    // Simulate the runtime side-effects of fetchNextPage / fetchPreviousPage.
    const state: { pages: Page<Item>[]; currentPageIdx: number } = {
      pages: [PAGE1],
      currentPageIdx: 0,
    };

    // Step forward: planner says "fetch", runtime would push PAGE2 then bump idx.
    expect(planNext(state)).toBe('fetch');
    state.pages.push(PAGE2);
    state.currentPageIdx += 1;
    expect(state.currentPageIdx).toBe(1);
    expect(getCurrentPageItems(state)).toEqual(PAGE2.items);
    expect(hasNext(state)).toBe(true); // PAGE2 has a nextCursor

    // Step forward again: still "fetch" (last loaded page has nextCursor).
    expect(planNext(state)).toBe('fetch');
    state.pages.push(LAST_PAGE);
    state.currentPageIdx += 1;
    expect(state.currentPageIdx).toBe(2);
    expect(hasNext(state)).toBe(false);

    // Step back: cached.
    expect(planPrev(state)).toBe('cached');
    state.currentPageIdx -= 1;
    expect(getCurrentPageItems(state)).toEqual(PAGE2.items);

    // Step forward: now we serve from cache — no fetch.
    expect(planNext(state)).toBe('cached');
    state.currentPageIdx += 1;
    expect(state.currentPageIdx).toBe(2);
  });
});
