import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, organizationId, serviceToken, name) {
  const url = `${baseUrl}api/v1/applications/`;
  const currentDateTime = new Date().toISOString();

  const payload = JSON.stringify({
    name: name ?? `test_k6_${currentDateTime}`,
    organization_id: organizationId, // Ensure this value is not undefined or null
  });

  const params = {
    headers: {
      Authorization: `Bearer ${serviceToken}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  if (
    !check(res, {
      'Application created': (r) =>
        r.status === 201 &&
        r.body &&
        r.body.includes('organization_id') &&
        r.body.includes('name') &&
        r.body.includes('application_id'),
    })
  ) {
    console.warn(res);
    return null;
  }

  return JSON.parse(res.body).application_id;
}
