import http from 'k6/http';
import { check } from 'k6';

export default function (base_url, service_token, application_id, event_id) {
  let res = http.get(
    `${base_url}api/v1/request_attempts/?application_id=${application_id}&event_id=${event_id}`,
    {
      headers: {
        Authorization: `Bearer ${service_token}`,
        'Content-Type': 'application/json',
      },
    }
  );

  if (
    !check(res, {
      'List request attempts': (r) => r.status === 200 && r.body,
    })
  ) {
    console.warn(res);
    return null;
  }

  return JSON.parse(res.body);
}
