import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token) {
  const url = `${baseUrl}api/v1/organizations`;
  const payload = JSON.stringify({
    name: `test_k6_${Date.now()}`,
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
      'Create organization is successful': (r) =>
        r.status === 201 &&
        r.body &&
        r.body.includes('organization_id') &&
        r.body.includes('name') &&
        r.body.includes('plan') &&
        r.body.includes('users') &&
        r.body.includes('quotas') &&
        r.body.includes('consumption') &&
        r.body.includes('onboarding_steps'),
    })
  ) {
    return null;
  }

  return JSON.parse(res.body).organization_id;
}
