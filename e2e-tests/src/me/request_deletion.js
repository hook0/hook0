import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, userAccessToken) {
  const url = `${baseUrl}api/v1/account`;

  const params = {
    headers: {
      Authorization: `Bearer ${userAccessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.del(url, null, params);
  if (
    !check(res, {
      'Request deletion succeeded': (r) => r.status === 204,
    })
  ) {
    console.warn('request_deletion response:', res.status, res.body);
    return false;
  }

  return true;
}
