// Retry schedule creation tests — validates all rejection cases before creating a valid schedule.
//
// How it works:
// 1. Fires invalid payloads (bad strategy, cross-field conflicts, boundary values) expecting 400/422
// 2. Creates one valid "increasing" schedule that downstream tests depend on
// 3. Proves "linear" and "custom" variants work end-to-end, then deletes them
// 4. Verifies the unique name constraint rejects duplicates

import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

/**
 * @description Creates a valid retry schedule after running validation tests for all error cases.
 * Returns the full schedule object (with retry_schedule_id) on success, or null on failure.
 * @example
 *   const schedule = create_retry_schedule(baseUrl, serviceToken, organizationId)
 *   // schedule.retry_schedule_id => "uuid"
 */
export default function (baseUrl, service_token, organization_id) {
  const url = `${baseUrl}api/v1/retry_schedules/`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // The API must reject strategies it doesn't recognize, not silently default
  let res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_invalid_strategy_' + uuidv4(),
      strategy: 'unknown',
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Invalid strategy returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // custom_intervals only valid for "custom" strategy — mixing is a user error the API must catch
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_exp_with_intervals_' + uuidv4(),
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 3,
      custom_intervals: [10, 20, 30],
    }),
    params
  );
  check(res, {
    'Increasing with custom_intervals returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // linear strategy requires linear_delay — omitting must not fall back to a default
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_linear_no_delay_' + uuidv4(),
      strategy: 'linear',
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Linear without linear_delay returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // custom_intervals length must equal max_retries — a mismatch means the user miscounted
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_custom_mismatch_' + uuidv4(),
      strategy: 'custom',
      max_retries: 3,
      custom_intervals: [10, 20],
    }),
    params
  );
  check(res, {
    'Custom with length mismatch returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // Zero-second intervals would hammer the target immediately — API must enforce minimum
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_custom_zero_' + uuidv4(),
      strategy: 'custom',
      max_retries: 3,
      custom_intervals: [0, 30, 300],
    }),
    params
  );
  check(res, {
    'Custom with interval=0 returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // Zero retries means "never retry" — defeats the purpose of a retry schedule
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_zero_retries_' + uuidv4(),
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 0,
    }),
    params
  );
  check(res, {
    'max_retries=0 returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // Server caps retries to prevent runaway delivery loops
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_101_retries_' + uuidv4(),
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 101,
    }),
    params
  );
  check(res, {
    'max_retries=101 returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // Names appear in the UI — empty string would be confusing
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: '',
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Empty name returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // Minimum 2 chars — single-char names are likely typos
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'x',
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Name too short returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // This schedule is returned to the caller — all downstream tests depend on it
  const schedule_name = 'k6_increasing_' + uuidv4();
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: schedule_name,
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 5,
    }),
    params
  );
  if (
    !check(res, {
      'Create retry schedule returns 201': (r) => r.status === 201,
      'Create retry schedule has all fields': (r) =>
        r.body &&
        r.body.includes('retry_schedule_id') &&
        r.body.includes('organization_id') &&
        r.body.includes('name') &&
        r.body.includes('strategy') &&
        r.body.includes('max_retries') &&
        r.body.includes('created_at') &&
        r.body.includes('updated_at'),
    })
  ) {
    console.warn('Create retry schedule failed:', res.status, res.body);
    return null;
  }

  const schedule = JSON.parse(res.body);

  // Prove linear variant works end-to-end, then delete so it doesn't pollute the org
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_linear_' + uuidv4(),
      strategy: 'linear',
      max_retries: 3,
      linear_delay: 300,
    }),
    params
  );
  check(res, {
    'Create linear schedule returns 201': (r) => r.status === 201,
  });
  if (res.status === 201) {
    const linear_id = JSON.parse(res.body).retry_schedule_id;
    http.del(
      `${baseUrl}api/v1/retry_schedules/${linear_id}?organization_id=${organization_id}`,
      null,
      params
    );
  }

  // Prove custom variant works end-to-end, then delete to avoid pollution
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_custom_' + uuidv4(),
      strategy: 'custom',
      max_retries: 3,
      custom_intervals: [3, 30, 300],
    }),
    params
  );
  check(res, {
    'Create custom schedule returns 201': (r) => r.status === 201,
  });
  if (res.status === 201) {
    const custom_id = JSON.parse(res.body).retry_schedule_id;
    http.del(
      `${baseUrl}api/v1/retry_schedules/${custom_id}?organization_id=${organization_id}`,
      null,
      params
    );
  }

  // Names are unique per org — reusing the name from step 1 must conflict
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: schedule_name,
      strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Duplicate name returns 409 or 422': (r) => r.status === 409 || r.status === 422,
  });

  return schedule;
}
