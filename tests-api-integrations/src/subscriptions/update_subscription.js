import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, subscription_id, application_id, payload) {
  const url = `${baseUrl}api/v1/subscriptions/${subscription_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.put(url, JSON.stringify(payload), params);
  if (
    !check(res, {
      'Subscription updated': (r) =>
        r.status === 200 &&
        r.body &&
        r.body.includes('subscription_id') &&
        r.body.includes('is_enabled'),
    })
  ) {
    console.warn(res);
    return null;
  }

  return JSON.parse(res.body);
}
