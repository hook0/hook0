import type { AxiosResponse, InternalAxiosRequestConfig, AxiosRequestHeaders } from 'axios';
import { followAllPages, readLinkHeader, unwrapCursorPage } from './cursorPagination';

const FAKE_CONFIG = {
  headers: {} as AxiosRequestHeaders,
} as InternalAxiosRequestConfig;

function makeResponse<T>(data: T, headers: Record<string, string> = {}): AxiosResponse<T> {
  return {
    data,
    status: 200,
    statusText: 'OK',
    headers,
    config: FAKE_CONFIG,
  };
}

describe('readLinkHeader', () => {
  test('reads lower-case link header', () => {
    const res = makeResponse([], { link: '<https://api/x>; rel="next"' });
    expect(readLinkHeader(res)).toBe('<https://api/x>; rel="next"');
  });

  test('falls back to capitalised Link header', () => {
    const res = makeResponse([], { Link: '<https://api/y>; rel="next"' });
    expect(readLinkHeader(res)).toBe('<https://api/y>; rel="next"');
  });

  test('returns undefined when neither variant is present', () => {
    const res = makeResponse([], { 'content-type': 'application/json' });
    expect(readLinkHeader(res)).toBeUndefined();
  });

  test('returns undefined when headers is missing entirely', () => {
    const res = {
      data: [],
      status: 200,
      statusText: 'OK',
      config: FAKE_CONFIG,
    } as unknown as AxiosResponse<unknown>;
    expect(readLinkHeader(res)).toBeUndefined();
  });
});

describe('unwrapCursorPage', () => {
  test('extracts items + next/prev cursors from Link header', async () => {
    const res = makeResponse([{ id: '1' }, { id: '2' }], {
      link: '<https://api/x?cursor=NEXT&limit=2>; rel="next", <https://api/x?cursor=PREV&limit=2>; rel="prev"',
    });
    const page = await unwrapCursorPage(Promise.resolve(res));
    expect(page.items).toEqual([{ id: '1' }, { id: '2' }]);
    expect(page.nextCursor).toBe('https://api/x?cursor=NEXT&limit=2');
    expect(page.prevCursor).toBe('https://api/x?cursor=PREV&limit=2');
  });

  test('emits undefined cursors when Link header is missing', async () => {
    const res = makeResponse([{ id: '1' }]);
    const page = await unwrapCursorPage(Promise.resolve(res));
    expect(page.items).toEqual([{ id: '1' }]);
    expect(page.nextCursor).toBeUndefined();
    expect(page.prevCursor).toBeUndefined();
  });
});

describe('followAllPages', () => {
  test('follows next cursors until exhausted', async () => {
    const first = jest.fn(() =>
      Promise.resolve({
        items: [1, 2],
        nextCursor: 'https://api/x?cursor=P2',
        prevCursor: undefined,
      })
    );
    const byUrl = jest.fn((url: string) => {
      if (url === 'https://api/x?cursor=P2') {
        return Promise.resolve({
          items: [3, 4],
          nextCursor: 'https://api/x?cursor=P3',
          prevCursor: undefined,
        });
      }
      return Promise.resolve({
        items: [5],
        nextCursor: undefined,
        prevCursor: undefined,
      });
    });

    const all = await followAllPages<number>(first, byUrl);
    expect(all).toEqual([1, 2, 3, 4, 5]);
    expect(first).toHaveBeenCalledTimes(1);
    expect(byUrl).toHaveBeenCalledTimes(2);
    expect(byUrl).toHaveBeenNthCalledWith(1, 'https://api/x?cursor=P2');
    expect(byUrl).toHaveBeenNthCalledWith(2, 'https://api/x?cursor=P3');
  });

  test('returns the first page when no nextCursor is present', async () => {
    const first = jest.fn(() =>
      Promise.resolve({ items: [42], nextCursor: undefined, prevCursor: undefined })
    );
    const byUrl = jest.fn();
    const all = await followAllPages<number>(first, byUrl);
    expect(all).toEqual([42]);
    expect(byUrl).not.toHaveBeenCalled();
  });
});
