import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl) {
  const url = `${baseUrl}api/v1/environments`;

  const res = http.get(url);
  if (
    !check(res, {
      'Get environments': (r) => r.status === 200 && r.body,
    })
  ) {
    console.warn(res);
    return false;
  }

  let json;
  try {
    json = JSON.parse(res.body);
  } catch (e) {
    throw new Error(`Failed to parse JSON: ${res.body}`);
  }

  if (
    !check(json, {
      'environments is array': () => Array.isArray(json),
      'environments has items': () => json.length > 0,
    })
  ) {
    throw new Error(`Unexpected response: ${JSON.stringify(json)}`);
  }

  // Validate structure of first item
  const first = json[0];
  if (
    !check(first, {
      'env.name is string': () => typeof first.name === 'string' && first.name.length > 0,
      'env.env_var is string': () => typeof first.env_var === 'string' && first.env_var.length > 0,
      'env.description is string or null': () =>
        first.description === null || typeof first.description === 'string',
      'env.default is string or null': () =>
        first.default === null || typeof first.default === 'string',
      'env.sensitive is boolean': () => typeof first.sensitive === 'boolean',
      'env.required is boolean': () => typeof first.required === 'boolean',
      'env.group is string or null': () => first.group === null || typeof first.group === 'string',
    })
  ) {
    console.log(`Received environment: ${JSON.stringify(first, null, 2)}`);
    throw new Error(`Invalid environment structure: ${JSON.stringify(first)}`);
  }

  // Verify known environment variables exist
  const envNames = json.map((e) => e.env_var);
  if (
    !check(envNames, {
      'contains IP': () => envNames.includes('IP'),
      'contains PORT': () => envNames.includes('PORT'),
      'contains DATABASE_URL': () => envNames.includes('DATABASE_URL'),
    })
  ) {
    console.log(`Env vars found: ${envNames.join(', ')}`);
    throw new Error('Missing expected environment variables');
  }

  return true;
}
