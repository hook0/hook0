import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, qs) {
  let url = `${baseUrl}api/v1/subscriptions`;

  if (qs !== '' && qs !== null) {
    url = url + qs;
  }

  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  let res = http.get(url, params);

  if (
    !check(res, {
      'List subscriptions is successful': (r) => r.status === 200 && r.body,
    })
  ) {
    return null;
  }

  return JSON.parse(res.body);
}

export function list_subscriptions_fail(baseUrl, service_token, qs) {
  let url = `${baseUrl}api/v1/subscriptions`;

  if (qs !== '' && qs !== null) {
    url = url + qs;
  }

  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  let res = http.get(url, params);

  if (
    !check(res, {
      'List subscriptions is successfully failed (Unauthorized)': (r) => r.status === 403 && r.body,
    })
  ) {
    return null;
  }

  return JSON.parse(res.body);
}
