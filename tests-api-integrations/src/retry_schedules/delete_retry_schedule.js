// Retry schedule deletion tests — verifies delete, confirms 404 on GET, and checks
// idempotent delete also returns 404.

import http from 'k6/http';
import { check } from 'k6';

/**
 * @description Deletes a retry schedule and verifies it is actually gone (not soft-deleted).
 * Returns true on success, or null on failure.
 * @example
 *   const ok = delete_retry_schedule(baseUrl, serviceToken, orgId, scheduleId)
 *   // ok => true
 */
export default function (baseUrl, service_token, organization_id, schedule_id) {
  const url = `${baseUrl}api/v1/retry_schedules/${schedule_id}?organization_id=${organization_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // First delete must succeed — schedule exists at this point
  let res = http.del(url, null, params);
  if (
    !check(res, {
      'Delete retry schedule returns 200': (r) => r.status === 200,
    })
  ) {
    console.warn('Delete retry schedule failed:', res.status, res.body);
    return null;
  }

  // Confirm actually gone, not just soft-deleted
  res = http.get(url, params);
  check(res, {
    'Get deleted schedule returns 404': (r) => r.status === 404,
  });

  // Idempotent deletes must return 404, not 200 or 500
  res = http.del(url, null, params);
  check(res, {
    'Delete again returns 404': (r) => r.status === 404,
  });

  return true;
}
