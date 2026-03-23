import http from 'k6/http';
import { check } from 'k6';

export default function (baseUrl) {
  const url = `${baseUrl}api/v1/quotas`;

  const res = http.get(url);
  if (
    !check(res, {
      'Get quota': (r) => r.status === 200 && r.body,
    })
  ) {
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
      'quotas.enabled': () => typeof json.enabled === 'boolean',
      'quotas.global_members_per_organization_limit': () =>
        typeof json.limits.global_members_per_organization_limit === 'number' &&
        json.limits.global_members_per_organization_limit >= 1,
      'quotas.global_applications_per_organization_limit': () =>
        typeof json.limits.global_applications_per_organization_limit === 'number' &&
        json.limits.global_applications_per_organization_limit >= 1,
      'quotas.global_events_per_day_limit': () =>
        typeof json.limits.global_events_per_day_limit === 'number' &&
        json.limits.global_events_per_day_limit >= 1,
      'quotas.global_days_of_events_retention_limit': () =>
        typeof json.limits.global_days_of_events_retention_limit === 'number' &&
        json.limits.global_days_of_events_retention_limit >= 1,
      'quotas.global_subscriptions_per_application_limit': () =>
        typeof json.limits.global_subscriptions_per_application_limit === 'number' &&
        json.limits.global_subscriptions_per_application_limit >= 1,
      'quotas.global_event_types_per_application_limit': () =>
        typeof json.limits.global_event_types_per_application_limit === 'number' &&
        json.limits.global_event_types_per_application_limit >= 1,
    })
  ) {
    console.log(`Received quotas: ${JSON.stringify(json.limits, null, 2)}`);
    throw new Error(`Unexpected response: ${JSON.stringify(json)}`);
  }

  return true;
}
