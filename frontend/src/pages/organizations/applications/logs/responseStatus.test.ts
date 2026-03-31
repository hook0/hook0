import { statusCodeClass } from './responseStatus';

describe('statusCodeClass', () => {
  test('returns unknown for null', () => {
    expect(statusCodeClass(null)).toBe('response-status--unknown');
  });

  test('returns unknown for undefined', () => {
    expect(statusCodeClass(undefined)).toBe('response-status--unknown');
  });

  test('returns error for 0 (falsy number, not null)', () => {
    expect(statusCodeClass(0)).toBe('response-status--error');
  });

  test('returns success for 2xx', () => {
    expect(statusCodeClass(200)).toBe('response-status--success');
    expect(statusCodeClass(201)).toBe('response-status--success');
    expect(statusCodeClass(299)).toBe('response-status--success');
  });

  test('returns warning for 3xx', () => {
    expect(statusCodeClass(300)).toBe('response-status--warning');
    expect(statusCodeClass(301)).toBe('response-status--warning');
    expect(statusCodeClass(399)).toBe('response-status--warning');
  });

  test('returns error for 4xx and 5xx', () => {
    expect(statusCodeClass(400)).toBe('response-status--error');
    expect(statusCodeClass(404)).toBe('response-status--error');
    expect(statusCodeClass(500)).toBe('response-status--error');
  });

  test('boundary: 199 is error', () => {
    expect(statusCodeClass(199)).toBe('response-status--error');
  });
});
