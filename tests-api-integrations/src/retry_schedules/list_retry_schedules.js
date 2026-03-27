import http from 'k6/http';
import { check } from 'k6';

/**
 * @description Lists all retry schedules for an organization.
 * Returns the parsed array on success, or null on failure.
 * @example
 *   const schedules = list_retry_schedules(baseUrl, serviceToken, organizationId)
 *   // schedules => [{ retry_schedule_id, name, ... }, ...]
 */
export default function (baseUrl, service_token, organization_id) {
  const url = `${baseUrl}api/v1/retry_schedules/?organization_id=${organization_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.get(url, params);
  if (
    !check(res, {
      'List retry schedules returns 200': (r) => r.status === 200,
      'List retry schedules returns array': (r) => {
        const body = JSON.parse(r.body);
        return Array.isArray(body);
      },
    })
  ) {
    console.warn('List retry schedules failed:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}
