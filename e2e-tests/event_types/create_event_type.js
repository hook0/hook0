import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

export default function (baseUrl, service_token, application_id) {
  const url = `${baseUrl}api/v1/event_types/`;
  const payload = JSON.stringify({
    application_id: application_id,
    service: 'test_k6_' + uuidv4(),
    resource_type: 'rst_k6_' + uuidv4(),
    verb: 'vrb_k6_' + uuidv4(),
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${service_token}`,
    },
  };

  let res = http.post(url, payload, params);

  if (
    !check(res, {
      'Create event type is successful': (r) =>
        r.status === 201 &&
        r.body &&
        r.body.includes('resource_type_name') &&
        r.body.includes('service_name') &&
        r.body.includes('verb_name') &&
        r.body.includes('event_type_name'),
    })
  ) {
    return null;
  }

  return JSON.parse(res.body).event_type_name;
}
