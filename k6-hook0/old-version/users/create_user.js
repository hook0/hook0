import http from 'k6/http';
import { check } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const DEFAULT_EMAIL = uuidv4() + '@test.com';
const DEFAULT_FIRST_NAME = 'test_k6';
const DEFAULT_LAST_NAME = 'test_k6';
const DEFAULT_ORGANIZATION_NAME = 'test_k6';
const DEFAULT_PASSWORD = 'testtest24';

export default function(BASE_URL) {

  const url = `${BASE_URL}api/v1/register/`;
  const payload = JSON.stringify({
    email: DEFAULT_EMAIL,
    first_name: DEFAULT_FIRST_NAME,
    last_name: DEFAULT_LAST_NAME,
    organization_name: DEFAULT_ORGANIZATION_NAME,
    password: DEFAULT_PASSWORD
    });

    const params = {
        headers: {
            'Content-Type': 'application/json',
        },
    };

    const res = http.post(url, payload, params);
    if(!check(res, {
        'User and default organisation created with success': (r) => r.status === 201 && r.body && r.body.includes('organization_id') && r.body.includes('user_id'),
    })) {
        return null;
    }

    return JSON.parse(res.body);
}