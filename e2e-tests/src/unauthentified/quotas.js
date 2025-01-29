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
      'quotas.global_members_per_organization_limit': () => typeof json.global_members_per_organization_limit === 'number' && json.global_members_per_organization_limit > 0,
      'quotas.global_applications_per_organization_limit': () => typeof json.global_applications_per_organization_limit === 'number' && json.global_applications_per_organization_limit > 0,
      'quotas.global_events_per_day_limit': () => typeof json.global_events_per_day_limit === 'number' && json.global_events_per_day_limit > 0,
      'quotas.global_days_of_events_retention_limit': () => typeof json.global_days_of_events_retention_limit === 'number' && json.global_days_of_events_retention_limit > 0,
      'quotas.global_subscriptions_per_application_limit': () => typeof json.global_subscriptions_per_application_limit === 'number' && json.global_subscriptions_per_application_limit > 0,
      'quotas.global_event_types_per_application_limit': () => typeof json.global_event_types_per_application_limit === 'number' && json.global_event_types_per_application_limit > 0,
    })
  ) {
    throw new Error(
      `Unexpected response: ${JSON.stringify(json)}`
    );
  }

  return true;
}
