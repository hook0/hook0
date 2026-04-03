// Bulk-deletes all retry schedules for an org. Safety net before test runs to prevent
// leftover schedules from causing duplicate-name conflicts.

import http from 'k6/http';

/**
 * @description Lists all retry schedules for an org and deletes each one.
 * Used as a safety net to clean up leaked schedules from previous test runs.
 * @example
 *   delete_all_retry_schedules(baseUrl, serviceToken, organizationId)
 */
export default function (baseUrl, service_token, organization_id) {
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const list_url = `${baseUrl}api/v1/retry_schedules/?organization_id=${organization_id}`;
  const res = http.get(list_url, params);
  if (res.status !== 200) {
    // Best-effort cleanup — don't fail the test suite if listing fails
    console.warn('delete_all_retry_schedules: failed to list schedules:', res.status, res.body);
    return;
  }

  const schedules = JSON.parse(res.body);
  for (const schedule of schedules) {
    const del_url = `${baseUrl}api/v1/retry_schedules/${schedule.retry_schedule_id}?organization_id=${organization_id}`;
    const del_res = http.del(del_url, null, params);
    if (del_res.status !== 200) {
      console.warn(
        `delete_all_retry_schedules: failed to delete ${schedule.retry_schedule_id}:`,
        del_res.status
      );
    }
  }
}
