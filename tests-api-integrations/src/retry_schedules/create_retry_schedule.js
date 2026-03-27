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

  // --- Invalid: unknown strategy ---
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

  // --- Invalid: exponential with custom_intervals ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_exp_with_intervals_' + uuidv4(),
      strategy: 'exponential',
      max_retries: 3,
      custom_intervals: [10, 20, 30],
    }),
    params
  );
  check(res, {
    'Exponential with custom_intervals returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // --- Invalid: linear without linear_delay ---
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

  // --- Invalid: custom with length mismatch ---
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

  // --- Invalid: custom with interval=0 ---
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

  // --- Invalid: max_retries=0 ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_zero_retries_' + uuidv4(),
      strategy: 'exponential',
      max_retries: 0,
    }),
    params
  );
  check(res, {
    'max_retries=0 returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // --- Invalid: max_retries=101 ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'k6_101_retries_' + uuidv4(),
      strategy: 'exponential',
      max_retries: 101,
    }),
    params
  );
  check(res, {
    'max_retries=101 returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // --- Invalid: empty name ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: '',
      strategy: 'exponential',
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Empty name returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // --- Invalid: name too short (1 char) ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: 'x',
      strategy: 'exponential',
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Name too short returns 400': (r) => r.status === 400 || r.status === 422,
  });

  // --- Valid: create exponential schedule ---
  const schedule_name = 'k6_exponential_' + uuidv4();
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: schedule_name,
      strategy: 'exponential',
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

  // --- Valid: linear schedule (verify variant works) ---
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

  // --- Valid: custom schedule (verify variant works) ---
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

  // --- Invalid: duplicate name same org ---
  res = http.post(
    url,
    JSON.stringify({
      organization_id,
      name: schedule_name,
      strategy: 'exponential',
      max_retries: 3,
    }),
    params
  );
  check(res, {
    'Duplicate name returns 409 or 422': (r) => r.status === 409 || r.status === 422,
  });

  return schedule;
}
