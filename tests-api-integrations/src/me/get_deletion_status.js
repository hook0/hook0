import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, userAccessToken) {
  const url = `${baseUrl}api/v1/account/deletion-status`;

  const params = {
    headers: {
      Authorization: `Bearer ${userAccessToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.get(url, params);
  if (
    !check(res, {
      'Get deletion status succeeded': (r) =>
        r.status === 200 && r.body && r.body.includes('deletion_requested'),
    })
  ) {
    console.warn('get_deletion_status response:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}
