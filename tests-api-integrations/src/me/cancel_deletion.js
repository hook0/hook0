import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, userAccessToken) {
  const url = `${baseUrl}api/v1/account/cancel-deletion`;

  const params = {
    headers: {
      Authorization: `Bearer ${userAccessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, null, params);
  if (
    !check(res, {
      'Cancel deletion succeeded': (r) => r.status === 204,
    })
  ) {
    console.warn('cancel_deletion response:', res.status, res.body);
    return false;
  }

  return true;
}
