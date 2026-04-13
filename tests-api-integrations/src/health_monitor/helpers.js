// Shared helpers for health monitor integration tests — used by both reactivation and health monitor test suites.

import http from 'k6/http';
import { check } from 'k6';

/**
 * @description Fetches health events for a subscription via GET /api/v1/subscriptions/{id}/health_events.
 * Returns parsed array on success, or null on failure.
 * @example
 *   const events = get_health_events(baseUrl, serviceToken, subId, orgId, 10)
 *   // events => [{ health_event_id, subscription_id, status, source, user_id, created_at }]
 */
export function get_health_events(baseUrl, service_token, subscription_id, organization_id, limit = 50) {
  const url = `${baseUrl}api/v1/subscriptions/${subscription_id}/health_events?organization_id=${organization_id}&limit=${limit}`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  const res = http.get(url, params);
  if (
    !check(res, {
      'GET health_events returns 200': (r) => r.status === 200,
    })
  ) {
    console.warn('GET health_events failed:', res.status, res.body);
    return null;
  }

  return JSON.parse(res.body);
}
