import http from 'k6/http';
import { check } from 'k6';
import create_application from '../applications/create_application.js';
import create_event_type from '../event_types/create_event_type.js';
import create_subscription from '../subscriptions/create_subscription.js';
import update_subscription from '../subscriptions/update_subscription.js';
import delete_application from '../applications/delete_application.js';
import { get_health_events } from './helpers.js';

/**
 * @description Builds the PUT payload for a subscription update, preserving all existing fields.
 * @example
 *   const payload = build_update_payload(subscription, { is_enabled: false })
 *   // payload => { application_id, is_enabled, event_types, target, description, metadata, labels }
 */
function build_update_payload(subscription, overrides) {
  return {
    application_id: subscription.application_id,
    is_enabled: subscription.is_enabled,
    event_types: subscription.event_types,
    target: subscription.target,
    description: subscription.description,
    metadata: subscription.metadata || {},
    labels: subscription.labels,
    ...overrides,
  };
}

/**
 * @description Test A1: Re-enable a disabled subscription produces a 'resolved' health event from 'user' source.
 * @example
 *   test_a1_reenable_disabled(baseUrl, serviceToken, orgId, appId, eventType, targetUrl)
 */
function test_a1_reenable_disabled(baseUrl, service_token, organization_id, application_id, event_type, target_url) {
  // 1. Create subscription (enabled by default)
  const subscription = create_subscription(baseUrl, service_token, application_id, [event_type], target_url, {
    a1_test: 'reenable',
  });
  if (!subscription) {
    console.warn('A1: Failed to create subscription');
    return false;
  }

  check(subscription, {
    'A1: subscription created enabled': (s) => s.is_enabled === true,
  });

  // 2. Disable the subscription
  const disable_payload = build_update_payload(subscription, { is_enabled: false });
  const disabled = update_subscription(baseUrl, service_token, subscription.subscription_id, application_id, disable_payload);
  if (!disabled) {
    console.warn('A1: Failed to disable subscription');
    return false;
  }

  check(disabled, {
    'A1: subscription disabled (200, is_enabled=false)': (s) => s.is_enabled === false,
  });

  // 3. Health events should be empty — user-initiated disable does not go through health monitor
  const events_after_disable = get_health_events(baseUrl, service_token, subscription.subscription_id, organization_id);
  if (events_after_disable === null) {
    console.warn('A1: Failed to get health events after disable');
    return false;
  }

  check(events_after_disable, {
    'A1: no health events after user disable': (e) => e.length === 0,
  });

  // 4. Re-enable the subscription
  const enable_payload = build_update_payload(disabled, { is_enabled: true });
  const reenabled = update_subscription(baseUrl, service_token, subscription.subscription_id, application_id, enable_payload);
  if (!reenabled) {
    console.warn('A1: Failed to re-enable subscription');
    return false;
  }

  check(reenabled, {
    'A1: subscription re-enabled (200, is_enabled=true)': (s) => s.is_enabled === true,
  });

  // 5. Health events should contain exactly 1 event: status='resolved', source='user'
  const events_after_enable = get_health_events(baseUrl, service_token, subscription.subscription_id, organization_id);
  if (events_after_enable === null) {
    console.warn('A1: Failed to get health events after re-enable');
    return false;
  }

  check(events_after_enable, {
    'A1: 1 health event after re-enable': (e) => e.length === 1,
    'A1: health event status is resolved': (e) => e.length > 0 && e[0].status === 'resolved',
    'A1: health event source is user': (e) => e.length > 0 && e[0].source === 'user',
  });

  console.log('A1: Re-enable disabled subscription test passed');
  return true;
}

/**
 * @description Test A2: Re-enabling an already enabled subscription is idempotent (no health events created).
 * @example
 *   test_a2_reenable_already_enabled(baseUrl, serviceToken, orgId, appId, eventType, targetUrl)
 */
function test_a2_reenable_already_enabled(baseUrl, service_token, organization_id, application_id, event_type, target_url) {
  // 1. Create subscription (enabled by default)
  const subscription = create_subscription(baseUrl, service_token, application_id, [event_type], target_url, {
    a2_test: 'idempotent',
  });
  if (!subscription) {
    console.warn('A2: Failed to create subscription');
    return false;
  }

  check(subscription, {
    'A2: subscription created enabled': (s) => s.is_enabled === true,
  });

  // 2. PUT with is_enabled: true (no-op transition)
  const enable_payload = build_update_payload(subscription, { is_enabled: true });
  const updated = update_subscription(baseUrl, service_token, subscription.subscription_id, application_id, enable_payload);
  if (!updated) {
    console.warn('A2: Failed to update subscription');
    return false;
  }

  check(updated, {
    'A2: subscription still enabled after idempotent update': (s) => s.is_enabled === true,
  });

  // 3. Health events should be empty — no transition happened
  const events = get_health_events(baseUrl, service_token, subscription.subscription_id, organization_id);
  if (events === null) {
    console.warn('A2: Failed to get health events');
    return false;
  }

  check(events, {
    'A2: no health events for idempotent enable': (e) => e.length === 0,
  });

  console.log('A2: Idempotent re-enable test passed');
  return true;
}

/**
 * @description Test A3: Disable/re-enable preserves all subscription data (description, labels, metadata, event_types, target).
 * @example
 *   test_a3_preserves_data(baseUrl, serviceToken, orgId, appId, eventType, targetUrl)
 */
function test_a3_preserves_data(baseUrl, service_token, organization_id, application_id, event_type, target_url) {
  // 1. Create subscription with specific data
  const subscription = create_subscription(baseUrl, service_token, application_id, [event_type], target_url, {
    a3_test: 'preserve_data',
    environment: 'k6',
  });
  if (!subscription) {
    console.warn('A3: Failed to create subscription');
    return false;
  }

  // Capture original values for comparison
  const original_description = subscription.description;
  const original_labels = subscription.labels;
  const original_metadata = subscription.metadata;
  const original_event_types = subscription.event_types;
  const original_target = subscription.target;

  // 2. Disable the subscription
  const disable_payload = build_update_payload(subscription, { is_enabled: false });
  const disabled = update_subscription(baseUrl, service_token, subscription.subscription_id, application_id, disable_payload);
  if (!disabled) {
    console.warn('A3: Failed to disable subscription');
    return false;
  }

  // 3. Re-enable the subscription
  const enable_payload = build_update_payload(disabled, { is_enabled: true });
  const reenabled = update_subscription(baseUrl, service_token, subscription.subscription_id, application_id, enable_payload);
  if (!reenabled) {
    console.warn('A3: Failed to re-enable subscription');
    return false;
  }

  // 4. GET subscription and verify all fields unchanged
  const get_res = http.get(
    `${baseUrl}api/v1/subscriptions/${subscription.subscription_id}?application_id=${application_id}`,
    {
      headers: {
        Authorization: `Bearer ${service_token}`,
        'Content-Type': 'application/json',
      },
    }
  );

  if (
    !check(get_res, {
      'A3: GET subscription returns 200': (r) => r.status === 200,
    })
  ) {
    console.warn('A3: Failed to GET subscription', get_res.status, get_res.body);
    return false;
  }

  const final_sub = JSON.parse(get_res.body);

  check(final_sub, {
    'A3: is_enabled restored to true': (s) => s.is_enabled === true,
    'A3: description unchanged': (s) => s.description === original_description,
    'A3: labels unchanged': (s) => {
        const keys_a = Object.keys(s.labels || {}).sort();
        const keys_b = Object.keys(original_labels || {}).sort();
        return keys_a.length === keys_b.length && keys_a.every((k, i) => k === keys_b[i] && s.labels[k] === original_labels[k]);
      },
    'A3: metadata unchanged': (s) => JSON.stringify(s.metadata) === JSON.stringify(original_metadata),
    'A3: event_types unchanged': (s) => JSON.stringify(s.event_types) === JSON.stringify(original_event_types),
    'A3: target unchanged': (s) => JSON.stringify(s.target) === JSON.stringify(original_target),
  });

  console.log('A3: Disable/re-enable preserves data test passed');
  return true;
}

/**
 * @description Runs all Group A reactivation tests within a single application context.
 * Creates application + event type, runs A1/A2/A3, cleans up in finally.
 * @example
 *   reactivation_tests(baseUrl, serviceToken, organizationId)
 */
export default function (baseUrl, service_token, organization_id, target_url) {
  let application_id = null;

  try {
    // Setup shared application and event type
    application_id = create_application(baseUrl, organization_id, service_token);
    if (!application_id) {
      throw new Error('Health monitor reactivation: failed to create application');
    }

    const event_type = create_event_type(baseUrl, service_token, application_id);
    if (!event_type) {
      throw new Error('Health monitor reactivation: failed to create event type');
    }

    // Run all Group A tests
    test_a1_reenable_disabled(baseUrl, service_token, organization_id, application_id, event_type, target_url);
    test_a2_reenable_already_enabled(baseUrl, service_token, organization_id, application_id, event_type, target_url);
    test_a3_preserves_data(baseUrl, service_token, organization_id, application_id, event_type, target_url);

    console.log('All health monitor reactivation tests (Group A) passed');
  } finally {
    if (application_id) {
      delete_application(baseUrl, application_id, service_token);
    }
  }
}
