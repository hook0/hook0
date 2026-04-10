/**
 * Integration tests for POST /api/v1/request_attempts/{id}/retry
 *
 * Covers: happy-path 202, cooldown 429, and non-existent attempt 404.
 * These are k6 test functions called from the main test harness.
 */

import http from 'k6/http';
import { check } from 'k6';

const headers = (token) => ({
  Authorization: `Bearer ${token}`,
  'Content-Type': 'application/json',
});

/**
 * Verify that a valid retry request creates a new, independent attempt (202)
 * with a different ID than the source.
 */
export function retryHappyPath(base_url, service_token, application_id, request_attempt_id) {
  const res = http.post(
    `${base_url}api/v1/request_attempts/${request_attempt_id}/retry?application_id=${application_id}`,
    null,
    { headers: headers(service_token) }
  );

  if (
    !check(res, {
      'Retry happy path: 202 Accepted': (r) => r.status === 202,
      'Retry happy path: returns request_attempt_id': (r) => {
        const body = JSON.parse(r.body);
        return body.request_attempt_id && body.request_attempt_id !== request_attempt_id;
      },
    })
  ) {
    console.warn('Retry happy path failed:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body).request_attempt_id;
}

/**
 * Verify the server enforces a per-event cooldown: the second retry within the
 * window must be rejected (429) to prevent click-storm abuse.
 */
export function retryCooldown(base_url, service_token, application_id, request_attempt_id) {
  // We need at least one successful retry to put the event into cooldown.
  // If another test already triggered a retry for this event, this call
  // may itself return 429 — that's acceptable; the important assertion
  // is on res2 below.
  const res1 = http.post(
    `${base_url}api/v1/request_attempts/${request_attempt_id}/retry?application_id=${application_id}`,
    null,
    { headers: headers(service_token) }
  );

  // Second retry immediately — must hit the cooldown window.
  const res2 = http.post(
    `${base_url}api/v1/request_attempts/${request_attempt_id}/retry?application_id=${application_id}`,
    null,
    { headers: headers(service_token) }
  );

  check(res2, {
    'Retry cooldown: 429 Too Many Requests': (r) => r.status === 429,
  });
}

/**
 * Verify that a completely fabricated attempt ID returns 404, not 500.
 */
export function retryNonExistent(base_url, service_token, application_id) {
  const fake_attempt_id = '00000000-0000-0000-0000-000000000000';
  const res = http.post(
    `${base_url}api/v1/request_attempts/${fake_attempt_id}/retry?application_id=${application_id}`,
    null,
    { headers: headers(service_token) }
  );

  check(res, {
    'Retry non-existent: 404 Not Found': (r) => r.status === 404,
  });
}
