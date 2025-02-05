import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, organization_id) {
  const url = `${baseUrl}api/v1/service_token`;
  const payload = JSON.stringify({
    name: `test_k6_${Date.now()}`,
    organization_id: organization_id,
  });

  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  let res = http.post(url, payload, params);

  if (
    !check(res, {
      'Create service token is successful': (r) =>
        r.status === 201 &&
        r.body &&
        r.body.includes('token_id') &&
        r.body.includes('name') &&
        r.body.includes('biscuit') &&
        r.body.includes('created_at'),
    })
  ) {
    return null;
  }

  return JSON.parse(res.body).biscuit;
}
