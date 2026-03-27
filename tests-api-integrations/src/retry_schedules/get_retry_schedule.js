import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

/**
 * @description Gets a retry schedule by ID and verifies a nonexistent UUID returns 404.
 * Returns the parsed schedule object on success, or null on failure.
 * @example
 *   const schedule = get_retry_schedule(baseUrl, serviceToken, organizationId, scheduleId)
 *   // schedule => { retry_schedule_id, name, strategy, ... }
 */
export default function (baseUrl, service_token, organization_id, schedule_id) {
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // --- Get existing schedule ---
  const url = `${baseUrl}api/v1/retry_schedules/${schedule_id}?organization_id=${organization_id}`;
  let res = http.get(url, params);
  if (
    !check(res, {
      'Get retry schedule returns 200': (r) => r.status === 200,
      'Get retry schedule has all fields': (r) =>
        r.body &&
        r.body.includes('retry_schedule_id') &&
        r.body.includes('organization_id') &&
        r.body.includes('name') &&
        r.body.includes('strategy') &&
        r.body.includes('max_retries') &&
        r.body.includes('created_at') &&
        r.body.includes('updated_at'),
      'Get retry schedule returns correct id': (r) => {
        const body = JSON.parse(r.body);
        return body.retry_schedule_id === schedule_id;
      },
    })
  ) {
    console.warn('Get retry schedule failed:', res.status, res.body);
    return null;
  }

  // --- Get nonexistent UUID ---
  const fake_id = uuidv4();
  const not_found_url = `${baseUrl}api/v1/retry_schedules/${fake_id}?organization_id=${organization_id}`;
  res = http.get(not_found_url, params);
  check(res, {
    'Get nonexistent schedule returns 404': (r) => r.status === 404,
  });

  return JSON.parse(http.get(url, params).body);
}
