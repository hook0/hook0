import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, service_token, application_id, label_key, label_value) {
  const url = `${baseUrl}api/v1/service_token/attenuate/dashboard?application_id=${application_id}&label_key=${label_key}&label_value=${label_value}`;

  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  let res = http.put(url, null, params);

  if (
    !check(res, {
      'Attenuate service token dashboard is successful': (r) =>
        r.status === 200 && r.body && r.body.includes('biscuit'),
    })
  ) {
    return null;
  }

  return JSON.parse(res.body).biscuit;
}
