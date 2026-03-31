import { statusCodeClass } from './responseStatus';

describe('statusCodeClass', () => {
  test('returns unknown for null', () => {
    expect(statusCodeClass(null)).toBe('log-detail__status-badge--unknown');
  });

  test('returns unknown for undefined', () => {
    expect(statusCodeClass(undefined)).toBe('log-detail__status-badge--unknown');
  });

  test('returns error for 0 (falsy number, not null)', () => {
    expect(statusCodeClass(0)).toBe('log-detail__status-badge--error');
  });

  test('returns success for 2xx', () => {
    expect(statusCodeClass(200)).toBe('log-detail__status-badge--success');
    expect(statusCodeClass(201)).toBe('log-detail__status-badge--success');
    expect(statusCodeClass(299)).toBe('log-detail__status-badge--success');
  });

  test('returns warning for 3xx', () => {
    expect(statusCodeClass(300)).toBe('log-detail__status-badge--warning');
    expect(statusCodeClass(301)).toBe('log-detail__status-badge--warning');
    expect(statusCodeClass(399)).toBe('log-detail__status-badge--warning');
  });

  test('returns error for 4xx and 5xx', () => {
    expect(statusCodeClass(400)).toBe('log-detail__status-badge--error');
    expect(statusCodeClass(404)).toBe('log-detail__status-badge--error');
    expect(statusCodeClass(500)).toBe('log-detail__status-badge--error');
  });

  test('boundary: 199 is error', () => {
    expect(statusCodeClass(199)).toBe('log-detail__status-badge--error');
  });
});
