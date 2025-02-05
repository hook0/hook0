import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, organization_id) {
  const url = `${baseUrl}api/v1/organizations/${organization_id}`;

  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  let res = http.get(url, params);

  if (
    !check(res, {
      'Get organization is successful': (r) =>
        r.status === 200 &&
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
