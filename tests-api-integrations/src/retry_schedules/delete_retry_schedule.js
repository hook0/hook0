import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, organization_id, schedule_id) {
  const url = `${baseUrl}api/v1/retry_schedules/${schedule_id}?organization_id=${organization_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // --- Delete existing schedule ---
  let res = http.del(url, null, params);
  if (
    !check(res, {
      'Delete retry schedule returns 200': (r) => r.status === 200,
    })
  ) {
    console.warn('Delete retry schedule failed:', res.status, res.body);
    return null;
  }

  // --- Get deleted schedule -> 404 ---
  res = http.get(url, params);
  check(res, {
    'Get deleted schedule returns 404': (r) => r.status === 404,
  });

  // --- Delete again -> 404 ---
  res = http.del(url, null, params);
  check(res, {
    'Delete again returns 404': (r) => r.status === 404,
  });

  return true;
}
