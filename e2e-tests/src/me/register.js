import http from 'k6/http';
import { check } from 'k6';

/**
 * Register a new user account.
 *
 * @param {string} baseUrl - API base URL
 * @param {string} email - User email
 * @param {string} password - User password
 * @param {string} firstName - User first name (optional, defaults to 'E2E')
 * @param {string} lastName - User last name (optional, defaults to 'Test')
 * @returns {object|null} Registration response or null if failed
 */
export default function register(baseUrl, email, password, firstName = 'E2E', lastName = 'Test') {
  const url = `${baseUrl}api/v1/registrations`;

  const payload = JSON.stringify({
    email: email,
    password: password,
    first_name: firstName,
    last_name: lastName,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  const res = http.post(url, payload, params);
  if (
    !check(res, {
      'Registration succeeded': (r) => r.status === 201 || r.status === 409, // 409 = user already exists (that's OK for our tests)
    })
  ) {
    console.warn('register response:', res.status, res.body);
    return null;
  }

  // If user already exists (409), that's OK - we can proceed with login
  if (res.status === 409) {
    console.log('User already exists, proceeding with login...');
    return { already_exists: true };
  }

  return JSON.parse(res.body);
}
