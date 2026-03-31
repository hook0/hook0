import { formatBody } from './formatBody';

describe('formatBody', () => {
  test('returns null for undefined', () => {
    expect(formatBody(undefined)).toBeNull();
  });

  test('returns null for null', () => {
    expect(formatBody(null)).toBeNull();
  });

  test('returns null for empty string', () => {
    expect(formatBody('')).toBeNull();
  });

  test('pretty-prints valid JSON', () => {
    expect(formatBody('{"a":1}')).toBe('{\n    "a": 1\n}');
  });

  test('returns raw string for invalid JSON', () => {
    expect(formatBody('not json')).toBe('not json');
  });

  test('returns raw HTML for HTML body', () => {
    const html = '<html><body>Error</body></html>';
    expect(formatBody(html)).toBe(html);
  });
});
