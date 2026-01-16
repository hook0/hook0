import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl, email, password) {
  const url = `${baseUrl}api/v1/auth/login`;

  const payload = JSON.stringify({
    email: email,
    password: password,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  if (
    !check(res, {
      'Login succeeded': (r) => r.status === 201 && r.body && r.body.includes('access_token'),
    })
  ) {
    console.warn('login response:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}
