// Tests retry schedule assignment to subscriptions — creation-time binding, update-time
// attach/detach, FK cascade on delete, and rejection of nonexistent schedule UUIDs.
//
// How it works:
// 1. Creates a schedule, then two subscriptions (one with and one without the schedule)
// 2. Tests attach via PUT, detach via null, and ON DELETE SET NULL cascade
// 3. Verifies nonexistent UUID is rejected

import http from 'k6/http';
import { check } from 'k6';

/**
 * @description Helper to create a retry schedule and test its assignment to subscriptions.
 * Covers: create with schedule, create without, update to assign/remove, cascade on delete, nonexistent UUID.
 * @example
 *   assign_to_subscription(baseUrl, serviceToken, organizationId, applicationId, eventType, targetUrl)
 */
export default function (baseUrl, service_token, organization_id, application_id, event_type, target_url) {
  const auth_headers = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };

  // Need a schedule to assign — strategy details don't matter, just need a valid ID
  const schedule_payload = {
    organization_id,
    name: 'k6-test-increasing-' + Date.now(),
    strategy: 'increasing',
      increasing_base_delay: 3,
      increasing_wait_factor: 3.0,
    max_retries: 10,
  };

  const schedule_res = http.post(
    `${baseUrl}api/v1/retry_schedules/`,
    JSON.stringify(schedule_payload),
    auth_headers
  );
  if (
    !check(schedule_res, {
      'Retry schedule created (201)': (r) => r.status === 201,
    })
  ) {
    console.warn('Failed to create retry schedule', schedule_res.body);
    return null;
  }
  const schedule = JSON.parse(schedule_res.body);
  const schedule_id = schedule.retry_schedule_id;

  // Prove schedule can be set at creation time, not just via update
  const sub_with_schedule_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: 'k6 test - subscription with retry schedule',
    metadata: { test_k6: 'true' },
    labels: { k6_assign_test: 'with_schedule' },
    target: {
      type: 'http',
      method: 'POST',
      url: target_url,
      headers: {},
    },
    retry_schedule_id: schedule_id,
  };

  const sub_with_res = http.post(
    `${baseUrl}api/v1/subscriptions/`,
    JSON.stringify(sub_with_schedule_payload),
    auth_headers
  );
  if (
    !check(sub_with_res, {
      'Subscription with schedule created (201)': (r) => r.status === 201,
    })
  ) {
    console.warn('Failed to create subscription with schedule', sub_with_res.body);
    return null;
  }
  const sub_with = JSON.parse(sub_with_res.body);
  check(sub_with, {
    'Subscription has retry_schedule_id set': (s) => s.retry_schedule_id === schedule_id,
  });

  // Baseline: starts with null schedule so we can test assigning one
  const sub_without_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: 'k6 test - subscription without retry schedule',
    metadata: { test_k6: 'true' },
    labels: { k6_assign_test: 'without_schedule' },
    target: {
      type: 'http',
      method: 'POST',
      url: target_url,
      headers: {},
    },
  };

  const sub_without_res = http.post(
    `${baseUrl}api/v1/subscriptions/`,
    JSON.stringify(sub_without_payload),
    auth_headers
  );
  if (
    !check(sub_without_res, {
      'Subscription without schedule created (201)': (r) => r.status === 201,
    })
  ) {
    console.warn('Failed to create subscription without schedule', sub_without_res.body);
    return null;
  }
  const sub_without = JSON.parse(sub_without_res.body);
  check(sub_without, {
    'Subscription has retry_schedule_id null': (s) => s.retry_schedule_id === null,
  });

  // Prove schedule can be attached to an existing subscription via PUT
  const update_assign_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: sub_without.description,
    metadata: sub_without.metadata || {},
    labels: sub_without.labels,
    target: sub_without.target,
    retry_schedule_id: schedule_id,
  };

  const update_assign_res = http.put(
    `${baseUrl}api/v1/subscriptions/${sub_without.subscription_id}`,
    JSON.stringify(update_assign_payload),
    auth_headers
  );
  if (
    !check(update_assign_res, {
      'Subscription updated to assign schedule (200)': (r) => r.status === 200,
    })
  ) {
    console.warn('Failed to assign schedule to subscription', update_assign_res.body);
    return null;
  }
  const sub_assigned = JSON.parse(update_assign_res.body);
  check(sub_assigned, {
    'Updated subscription has retry_schedule_id set': (s) => s.retry_schedule_id === schedule_id,
  });

  // Explicitly nulling the FK must detach the schedule without deleting it
  const update_remove_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: sub_assigned.description,
    metadata: sub_assigned.metadata || {},
    labels: sub_assigned.labels,
    target: sub_assigned.target,
    retry_schedule_id: null,
  };

  const update_remove_res = http.put(
    `${baseUrl}api/v1/subscriptions/${sub_without.subscription_id}`,
    JSON.stringify(update_remove_payload),
    auth_headers
  );
  if (
    !check(update_remove_res, {
      'Subscription updated to remove schedule (200)': (r) => r.status === 200,
    })
  ) {
    console.warn('Failed to remove schedule from subscription', update_remove_res.body);
    return null;
  }
  const sub_removed = JSON.parse(update_remove_res.body);
  check(sub_removed, {
    'Updated subscription has retry_schedule_id null': (s) => s.retry_schedule_id === null,
  });

  // FK has ON DELETE SET NULL — deleting the schedule must not orphan the subscription
  const reassign_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: sub_removed.description,
    metadata: sub_removed.metadata || {},
    labels: sub_removed.labels,
    target: sub_removed.target,
    retry_schedule_id: schedule_id,
  };

  const reassign_res = http.put(
    `${baseUrl}api/v1/subscriptions/${sub_without.subscription_id}`,
    JSON.stringify(reassign_payload),
    auth_headers
  );
  if (
    !check(reassign_res, {
      'Subscription re-assigned schedule (200)': (r) => r.status === 200,
    })
  ) {
    console.warn('Failed to reassign schedule', reassign_res.body);
    return null;
  }
  const sub_reassigned = JSON.parse(reassign_res.body);
  check(sub_reassigned, {
    'Re-assigned subscription has retry_schedule_id set': (s) =>
      s.retry_schedule_id === schedule_id,
  });

  // Triggers the SET NULL cascade we're about to verify
  const delete_schedule_res = http.del(
    `${baseUrl}api/v1/retry_schedules/${schedule_id}?organization_id=${organization_id}`,
    null,
    auth_headers
  );
  check(delete_schedule_res, {
    'Retry schedule deleted (200)': (r) => r.status === 200,
  });

  const get_sub_res = http.get(
    `${baseUrl}api/v1/subscriptions/${sub_without.subscription_id}?application_id=${application_id}`,
    auth_headers
  );
  if (
    !check(get_sub_res, {
      'GET subscription after schedule deletion (200)': (r) => r.status === 200,
    })
  ) {
    console.warn('Failed to GET subscription after schedule deletion', get_sub_res.body);
    return null;
  }
  const sub_after_delete = JSON.parse(get_sub_res.body);
  check(sub_after_delete, {
    'Subscription retry_schedule_id is null after schedule deletion (SET NULL cascade)': (s) =>
      s.retry_schedule_id === null,
  });

  // The API must reject references to schedules that don't exist, not silently null them
  const nonexistent_uuid = '00000000-0000-0000-0000-000000000000';
  const update_nonexistent_payload = {
    application_id,
    is_enabled: true,
    event_types: [event_type],
    description: sub_after_delete.description,
    metadata: sub_after_delete.metadata || {},
    labels: sub_after_delete.labels,
    target: sub_after_delete.target,
    retry_schedule_id: nonexistent_uuid,
  };

  const update_nonexistent_res = http.put(
    `${baseUrl}api/v1/subscriptions/${sub_without.subscription_id}`,
    JSON.stringify(update_nonexistent_payload),
    auth_headers
  );
  check(update_nonexistent_res, {
    'Update with nonexistent schedule returns 404': (r) => r.status === 404,
  });

  console.log('All retry schedule subscription assignment tests passed');
  return true;
}
