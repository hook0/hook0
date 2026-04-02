import { filterSensitiveHeaders } from './responseHeaders';

describe('filterSensitiveHeaders', () => {
  test('removes all sensitive headers', () => {
    const headers = {
      cookie: 'session=abc123',
      'set-cookie': 'token=xyz; HttpOnly',
      authorization: 'Bearer secret',
      'www-authenticate': 'Basic realm="test"',
      'proxy-authorization': 'Basic creds',
      'proxy-authenticate': 'Negotiate',
      'content-type': 'application/json',
    };

    const result = filterSensitiveHeaders(headers);
    expect(result).toEqual({ 'content-type': 'application/json' });
  });

  test('preserves non-sensitive headers', () => {
    const headers = {
      'content-type': 'text/html',
      'x-request-id': 'abc-123',
      'x-ratelimit-limit': '120',
      server: 'cloudflare',
    };

    const result = filterSensitiveHeaders(headers);
    expect(result).toEqual(headers);
  });

  test('filters case-insensitively', () => {
    const headers = {
      Authorization: 'Bearer token',
      'SET-COOKIE': 'id=1',
      'Content-Type': 'text/plain',
      'Proxy-Authorization': 'Basic abc',
    };

    const result = filterSensitiveHeaders(headers);
    expect(result).toEqual({ 'Content-Type': 'text/plain' });
  });

  test('returns null for undefined input', () => {
    expect(filterSensitiveHeaders(undefined)).toBeNull();
  });

  test('returns null for null input', () => {
    expect(filterSensitiveHeaders(null)).toBeNull();
  });

  test('returns null for empty object', () => {
    expect(filterSensitiveHeaders({})).toBeNull();
  });

  test('returns null when all headers are sensitive', () => {
    const headers = {
      cookie: 'session=abc',
      'set-cookie': 'token=xyz',
      authorization: 'Bearer secret',
      'www-authenticate': 'Basic',
      'proxy-authorization': 'Basic creds',
      'proxy-authenticate': 'Negotiate',
    };

    expect(filterSensitiveHeaders(headers)).toBeNull();
  });
});
