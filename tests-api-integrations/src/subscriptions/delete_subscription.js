import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, subscription_id, application_id) {
  const url = `${baseUrl}api/v1/subscriptions/${subscription_id}?application_id=${application_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.del(url, null, params);
  if (
    !check(res, {
      'Subscription deleted': (r) => r.status === 204,
    })
  ) {
    console.warn(res);
    return null;
  }

  return true;
}
