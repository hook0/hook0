// Fetches a single subscription by ID. Used by health monitor tests to poll subscription
// state during async cron evaluation.

import http from 'k6/http';
import { check } from 'k6';

/**
 * @description Fetches a single subscription by ID via GET /api/v1/subscriptions/{id}.
 * Returns parsed subscription object on success, or null on failure.
 * @example
 *   const sub = get_subscription(baseUrl, serviceToken, subId, appId)
 *   // sub => { subscription_id, is_enabled, event_types, target, ... }
 */
export default function (baseUrl, service_token, subscription_id, application_id) {
  const url = `${baseUrl}api/v1/subscriptions/${subscription_id}?application_id=${application_id}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.get(url, params);
  if (
    !check(res, {
      'GET subscription returns 200': (r) => r.status === 200,
    })
  ) {
    console.warn('GET subscription failed:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}
