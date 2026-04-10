import http from 'k6/http';
import { check, sleep } from 'k6';

const headers = (token) => ({
  Authorization: `Bearer ${token}`,
  'Content-Type': 'application/json',
});

/**
 * Test 1: Happy path — retry a request attempt → 202
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
 * Test 2: Cooldown — retry same event twice within cooldown → 429
 */
export function retryCooldown(base_url, service_token, application_id, request_attempt_id) {
  // First retry should succeed (or may 429 if previous test already triggered one)
  const res1 = http.post(
    `${base_url}api/v1/request_attempts/${request_attempt_id}/retry?application_id=${application_id}`,
    null,
    { headers: headers(service_token) }
  );

  // Second retry immediately — should hit cooldown
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
 * Test 3: Wrong application — retry attempt from another app → 404
 */
export function retryWrongApp(base_url, service_token, request_attempt_id) {
  const fake_app_id = '00000000-0000-0000-0000-000000000000';
  const res = http.post(
    `${base_url}api/v1/request_attempts/${request_attempt_id}/retry?application_id=${fake_app_id}`,
    null,
    { headers: headers(service_token) }
  );

  check(res, {
    'Retry wrong app: 404 Not Found': (r) => r.status === 404,
  });
}

/**
 * Test 4: Non-existent attempt → 404
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
