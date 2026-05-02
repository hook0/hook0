import { parseLinkHeader } from './parseLinkHeader';

describe('parseLinkHeader', () => {
  test('returns empty object for null/undefined/empty input', () => {
    expect(parseLinkHeader(null)).toEqual({});
    expect(parseLinkHeader(undefined)).toEqual({});
    expect(parseLinkHeader('')).toEqual({});
  });

  test('parses a single rel="next" entry', () => {
    const header =
      '<https://api.example.com/event_types?application_id=abc&pagination_cursor=xyz%3D&limit=100>; rel="next"';
    expect(parseLinkHeader(header)).toEqual({
      next: 'https://api.example.com/event_types?application_id=abc&pagination_cursor=xyz%3D&limit=100',
    });
  });

  test('parses a single rel="prev" entry', () => {
    expect(parseLinkHeader('<https://api/x?cursor=abc>; rel="prev"')).toEqual({
      prev: 'https://api/x?cursor=abc',
    });
  });

  test('parses both next and prev entries in one header', () => {
    const header =
      '<https://api.example.com/p?cursor=NEXT&limit=50>; rel="next", <https://api.example.com/p?cursor=PREV&limit=50>; rel="prev"';
    expect(parseLinkHeader(header)).toEqual({
      next: 'https://api.example.com/p?cursor=NEXT&limit=50',
      prev: 'https://api.example.com/p?cursor=PREV&limit=50',
    });
  });

  test('handles unquoted rel values', () => {
    expect(parseLinkHeader('<https://api/x>; rel=next')).toEqual({ next: 'https://api/x' });
  });

  test('case-insensitive rel matching', () => {
    expect(parseLinkHeader('<https://api/x>; REL="NEXT"')).toEqual({ next: 'https://api/x' });
  });

  test('ignores entries with unknown rel', () => {
    expect(parseLinkHeader('<https://api/x>; rel="first", <https://api/y>; rel="next"')).toEqual({
      next: 'https://api/y',
    });
  });

  test('handles whitespace around delimiters', () => {
    expect(parseLinkHeader(' <https://api/x>  ;  rel="next"  ')).toEqual({
      next: 'https://api/x',
    });
  });

  test('returns empty object for malformed input (no angle brackets)', () => {
    expect(parseLinkHeader('https://api/x; rel="next"')).toEqual({});
  });

  test('preserves URL with encoded query params (cursor with = padding)', () => {
    const header =
      '<https://api/x?application_id=app&pagination_cursor=eyJkYXRlIjoi%3D&limit=100>; rel="next"';
    const result = parseLinkHeader(header);
    expect(result.next).toContain('pagination_cursor=eyJkYXRlIjoi%3D');
    expect(result.next).toContain('limit=100');
  });
});
