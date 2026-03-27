import { sleep } from 'k6';
import create_application from '../applications/create_application.js';
import delete_application from '../applications/delete_application.js';
import create_event_type from '../event_types/create_event_type.js';
import create_subscription from '../subscriptions/create_subscription.js';
import get_subscription from '../subscriptions/get_subscription.js';
import update_subscription from '../subscriptions/update_subscription.js';
import send_event from '../events/send_event.js';
import { get_health_events } from './helpers.js';

/**
 * @description Polls a condition function until it returns true or timeout is reached.
 * Returns true if condition was met, false on timeout.
 * @example
 *   const ok = wait_for_condition(() => get_sub().is_enabled === false, 30000, 2000)
 *   // ok => true (condition met) or false (timed out)
 */
function wait_for_condition(check_fn, timeout_ms = 30000, interval_ms = 2000) {
  const start = Date.now();
  while (Date.now() - start < timeout_ms) {
    if (check_fn()) return true;
    sleep(interval_ms / 1000);
  }
  return false;
}

/**
 * @description Checks value is non-null/undefined. Used for guard assertions.
 * @example
 *   isNotNull('abc') // => true
 *   isNotNull(null)  // => false
 */
function isNotNull(value) {
  return value !== null && value !== undefined;
}

/**
 * @description Sends N events of a given type with matching labels.
 * Returns array of event IDs.
 * @example
 *   const ids = send_n_events(token, url, appId, 'evt.test', { k: 'v' }, 5)
 *   // ids => ['uuid1', 'uuid2', ...]
 */
function send_n_events(service_token, base_url, application_id, event_type, labels, count) {
  const event_ids = [];
  for (let i = 0; i < count; i++) {
    const eid = send_event(service_token, base_url, application_id, event_type, labels);
    if (!isNotNull(eid)) {
      throw new Error(`Failed to send event ${i + 1}/${count}`);
    }
    event_ids.push(eid);
  }
  return event_ids;
}

/**
 * @description Creates a standard test harness: application, event type, subscription.
 * Returns { application_id, event_type, subscription } or throws.
 * @example
 *   const ctx = create_test_context(config, 'http://localhost:19999', { label: 'val' })
 *   // ctx => { application_id, event_type, subscription: { subscription_id, ... } }
 */
function create_test_context(config, target_url, labels) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  const application_id = create_application(h, o, s);
  if (!isNotNull(application_id)) {
    throw new Error('Failed to create application');
  }

  const event_type = create_event_type(h, s, application_id);
  if (!isNotNull(event_type)) {
    throw new Error('Failed to create event type');
  }

  const subscription = create_subscription(h, s, application_id, [event_type], target_url, labels);
  if (!isNotNull(subscription)) {
    throw new Error('Failed to create subscription');
  }

  return { application_id, event_type, subscription };
}

/**
 * @description Cleans up test application if keepTestApplication is false.
 * @example
 *   cleanup(config, 'app-uuid')
 */
function cleanup(config, application_id) {
  if (application_id && !config.keepTestApplication) {
    delete_application(config.apiOrigin, application_id, config.serviceToken);
  }
}

// ---------------------------------------------------------------------------
// B1: 100% failure -> subscription auto-disabled
// ---------------------------------------------------------------------------

/**
 * @description Tests that a subscription receiving only failures gets auto-disabled
 * by the health monitor cron, and that warning + disabled health events are created.
 * @example
 *   test_b1_failure_disables_subscription(config)
 */
export function test_b1_failure_disables_subscription(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrlFailing, { hm_test: 'b1' });
    application_id = ctx.application_id;

    // Send >= min_sample_size events (5) to the failing subscription
    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'b1' }, 6);

    // Poll until subscription is disabled (timeout 60s to allow cron cycles + delivery attempts)
    const sub_id = ctx.subscription.subscription_id;
    const disabled = wait_for_condition(() => {
      const sub = get_subscription(h, s, sub_id, ctx.application_id);
      return sub && sub.is_enabled === false;
    }, 30000, 2000);

    if (!disabled) {
      const sub = get_subscription(h, s, sub_id, ctx.application_id);
      throw new Error(
        `B1: subscription was not auto-disabled within timeout. is_enabled=${sub ? sub.is_enabled : 'null'}`
      );
    }

    // Verify health events contain warning and disabled entries from system
    const events = get_health_events(h, s, sub_id, o);
    if (!isNotNull(events)) {
      throw new Error('B1: Failed to fetch health events');
    }

    const has_warning = events.some((e) => e.status === 'warning' && e.source === 'system');
    const has_disabled = events.some((e) => e.status === 'disabled' && e.source === 'system');

    if (!has_warning) {
      throw new Error('B1: Expected a warning health event from system, found none');
    }
    if (!has_disabled) {
      throw new Error('B1: Expected a disabled health event from system, found none');
    }

    console.log('B1 PASSED: 100% failure -> subscription auto-disabled with warning + disabled events');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// B2: 100% success -> subscription stays enabled
// ---------------------------------------------------------------------------

/**
 * @description Tests that a subscription receiving only successes stays enabled
 * and produces no health events.
 * @example
 *   test_b2_success_stays_enabled(config)
 */
export function test_b2_success_stays_enabled(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrl, { hm_test: 'b2' });
    application_id = ctx.application_id;

    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'b2' }, 6);

    // Wait a few cron ticks (15s) to give health monitor time to evaluate
    sleep(15);

    const sub = get_subscription(h, s, ctx.subscription.subscription_id, ctx.application_id);
    if (!isNotNull(sub)) {
      throw new Error('B2: Failed to fetch subscription');
    }
    if (sub.is_enabled !== true) {
      throw new Error(`B2: subscription should remain enabled, got is_enabled=${sub.is_enabled}`);
    }

    const events = get_health_events(h, s, ctx.subscription.subscription_id, o);
    if (!isNotNull(events)) {
      throw new Error('B2: Failed to fetch health events');
    }
    if (events.length !== 0) {
      throw new Error(`B2: Expected 0 health events for healthy subscription, got ${events.length}`);
    }

    console.log('B2 PASSED: 100% success -> subscription stays enabled, no health events');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// B3: Re-enable after auto-disable
// ---------------------------------------------------------------------------

/**
 * @description Tests that a user can re-enable an auto-disabled subscription
 * and that a resolved health event with source=user is created.
 * @example
 *   test_b3_reenable_after_autodisable(config)
 */
export function test_b3_reenable_after_autodisable(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    // Reuse B1 setup: create subscription to failing target, send events, wait for disable
    const ctx = create_test_context(config, config.targetUrlFailing, { hm_test: 'b3' });
    application_id = ctx.application_id;
    const sub_id = ctx.subscription.subscription_id;

    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'b3' }, 6);

    const disabled = wait_for_condition(() => {
      const sub = get_subscription(h, s, sub_id, ctx.application_id);
      return sub && sub.is_enabled === false;
    }, 30000, 2000);

    if (!disabled) {
      throw new Error('B3: subscription was not auto-disabled within timeout (prerequisite for re-enable test)');
    }

    // Re-enable the subscription
    const update_payload = {
      application_id: ctx.application_id,
      is_enabled: true,
      event_types: [ctx.event_type],
      target: ctx.subscription.target,
      description: ctx.subscription.description,
      metadata: ctx.subscription.metadata || {},
      labels: ctx.subscription.labels,
    };

    const updated = update_subscription(h, s, sub_id, ctx.application_id, update_payload);
    if (!isNotNull(updated)) {
      throw new Error('B3: Failed to re-enable subscription');
    }

    const sub_after = get_subscription(h, s, sub_id, ctx.application_id);
    if (!isNotNull(sub_after) || sub_after.is_enabled !== true) {
      throw new Error(`B3: subscription should be re-enabled, got is_enabled=${sub_after ? sub_after.is_enabled : 'null'}`);
    }

    // Verify last health event is resolved from user
    const events = get_health_events(h, s, sub_id, o);
    if (!isNotNull(events) || events.length === 0) {
      throw new Error('B3: Expected health events after re-enable');
    }

    // Events are ordered by created_at DESC; events[0] is the most recent
    const latest_event = events[0];
    if (latest_event.status !== 'resolved' || latest_event.source !== 'user') {
      throw new Error(
        `B3: Expected latest health event to be status=resolved source=user, got status=${latest_event.status} source=${latest_event.source}`
      );
    }

    console.log('B3 PASSED: re-enable after auto-disable creates resolved/user health event');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// B4: Full lifecycle: healthy -> disabled -> re-enable -> healthy
// ---------------------------------------------------------------------------

/**
 * @description Tests the full lifecycle: subscription auto-disabled, re-enabled by user,
 * target changed to healthy, events sent, subscription stays enabled.
 * @example
 *   test_b4_full_lifecycle(config)
 */
export function test_b4_full_lifecycle(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    // Phase 1: subscription to failing target -> auto-disabled
    const ctx = create_test_context(config, config.targetUrlFailing, { hm_test: 'b4' });
    application_id = ctx.application_id;
    const sub_id = ctx.subscription.subscription_id;

    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'b4' }, 6);

    const disabled = wait_for_condition(() => {
      const sub = get_subscription(h, s, sub_id, ctx.application_id);
      return sub && sub.is_enabled === false;
    }, 30000, 2000);

    if (!disabled) {
      throw new Error('B4: subscription was not auto-disabled within timeout');
    }

    // Phase 2: re-enable with healthy target
    const update_payload = {
      application_id: ctx.application_id,
      is_enabled: true,
      event_types: [ctx.event_type],
      target: {
        type: 'http',
        method: 'POST',
        url: config.targetUrl,
        headers: {},
      },
      description: ctx.subscription.description,
      metadata: ctx.subscription.metadata || {},
      labels: ctx.subscription.labels,
    };

    const updated = update_subscription(h, s, sub_id, ctx.application_id, update_payload);
    if (!isNotNull(updated)) {
      throw new Error('B4: Failed to re-enable subscription with healthy target');
    }

    // Phase 3: send events to the now-healthy subscription
    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'b4' }, 6);

    // Wait for cron ticks to evaluate the subscription again
    sleep(15);

    // Phase 4: verify subscription is still enabled
    const sub_final = get_subscription(h, s, sub_id, ctx.application_id);
    if (!isNotNull(sub_final)) {
      throw new Error('B4: Failed to fetch subscription after healthy events');
    }
    if (sub_final.is_enabled !== true) {
      throw new Error(`B4: subscription should stay enabled after healthy events, got is_enabled=${sub_final.is_enabled}`);
    }

    console.log('B4 PASSED: full lifecycle healthy -> disabled -> re-enable -> healthy');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// C1: User-disabled subscription not evaluated by cron
// ---------------------------------------------------------------------------

/**
 * @description Tests that a subscription manually disabled by the user is not
 * evaluated by the health monitor cron (no system disabled event).
 * @example
 *   test_c1_user_disabled_not_evaluated(config)
 */
export function test_c1_user_disabled_not_evaluated(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrlFailing, { hm_test: 'c1' });
    application_id = ctx.application_id;
    const sub_id = ctx.subscription.subscription_id;

    // Disable BEFORE sending events
    const update_payload = {
      application_id: ctx.application_id,
      is_enabled: false,
      event_types: [ctx.event_type],
      target: ctx.subscription.target,
      description: ctx.subscription.description,
      metadata: ctx.subscription.metadata || {},
      labels: ctx.subscription.labels,
    };

    const updated = update_subscription(h, s, sub_id, ctx.application_id, update_payload);
    if (!isNotNull(updated) || updated.is_enabled !== false) {
      throw new Error('C1: Failed to disable subscription before sending events');
    }

    // Send events (they won't be delivered since sub is disabled)
    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'c1' }, 6);

    // Wait for cron ticks
    sleep(15);

    // Verify no system disabled event was created
    const events = get_health_events(h, s, sub_id, o);
    if (!isNotNull(events)) {
      throw new Error('C1: Failed to fetch health events');
    }

    const system_disabled = events.filter((e) => e.status === 'disabled' && e.source === 'system');
    if (system_disabled.length > 0) {
      throw new Error(`C1: Expected no system disabled events for user-disabled subscription, found ${system_disabled.length}`);
    }

    console.log('C1 PASSED: user-disabled subscription not evaluated by cron');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// C2: Fewer than min_sample_size -> no evaluation
// ---------------------------------------------------------------------------

/**
 * @description Tests that a subscription with fewer events than min_sample_size (5)
 * is not evaluated by the health monitor.
 * @example
 *   test_c2_below_min_sample_size(config)
 */
export function test_c2_below_min_sample_size(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrlFailing, { hm_test: 'c2' });
    application_id = ctx.application_id;
    const sub_id = ctx.subscription.subscription_id;

    // Send only 1 event — with retries, this produces ~3 attempts in 15s,
    // staying below min_sample_size of 5 (which counts attempts, not events)
    send_n_events(s, h, ctx.application_id, ctx.event_type, { hm_test: 'c2' }, 1);

    // Wait for cron ticks
    sleep(15);

    const sub = get_subscription(h, s, sub_id, ctx.application_id);
    if (!isNotNull(sub)) {
      throw new Error('C2: Failed to fetch subscription');
    }
    if (sub.is_enabled !== true) {
      throw new Error(`C2: subscription should stay enabled with < min_sample_size events, got is_enabled=${sub.is_enabled}`);
    }

    const events = get_health_events(h, s, sub_id, o);
    if (!isNotNull(events)) {
      throw new Error('C2: Failed to fetch health events');
    }
    if (events.length !== 0) {
      throw new Error(`C2: Expected 0 health events for under-sampled subscription, got ${events.length}`);
    }

    console.log('C2 PASSED: below min_sample_size -> no evaluation');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// C3: Subscriptions evaluated independently
// ---------------------------------------------------------------------------

/**
 * @description Tests that two subscriptions with different event types are
 * evaluated independently: a failing sub gets disabled while a healthy sub stays enabled.
 * @example
 *   test_c3_independent_evaluation(config)
 */
export function test_c3_independent_evaluation(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;
  let application_id = null;

  try {
    const application_id_val = create_application(h, o, s);
    if (!isNotNull(application_id_val)) {
      throw new Error('C3: Failed to create application');
    }
    application_id = application_id_val;

    // Create two distinct event types
    const event_type_1 = create_event_type(h, s, application_id);
    if (!isNotNull(event_type_1)) {
      throw new Error('C3: Failed to create event type 1');
    }

    const event_type_2 = create_event_type(h, s, application_id);
    if (!isNotNull(event_type_2)) {
      throw new Error('C3: Failed to create event type 2');
    }

    // Sub A: failing target, listens to event_type_1
    const sub_a = create_subscription(h, s, application_id, [event_type_1], config.targetUrlFailing, {
      hm_test: 'c3_a',
    });
    if (!isNotNull(sub_a)) {
      throw new Error('C3: Failed to create subscription A (failing)');
    }

    // Sub B: healthy target, listens to event_type_2
    const sub_b = create_subscription(h, s, application_id, [event_type_2], config.targetUrl, {
      hm_test: 'c3_b',
    });
    if (!isNotNull(sub_b)) {
      throw new Error('C3: Failed to create subscription B (healthy)');
    }

    // Send events for both types
    send_n_events(s, h, application_id, event_type_1, { hm_test: 'c3_a' }, 6);
    send_n_events(s, h, application_id, event_type_2, { hm_test: 'c3_b' }, 6);

    // Poll until Sub A is disabled
    const sub_a_disabled = wait_for_condition(() => {
      const sub = get_subscription(h, s, sub_a.subscription_id, application_id);
      return sub && sub.is_enabled === false;
    }, 30000, 2000);

    if (!sub_a_disabled) {
      throw new Error('C3: subscription A was not auto-disabled within timeout');
    }

    // Sub B should still be enabled
    const sub_b_after = get_subscription(h, s, sub_b.subscription_id, application_id);
    if (!isNotNull(sub_b_after)) {
      throw new Error('C3: Failed to fetch subscription B');
    }
    if (sub_b_after.is_enabled !== true) {
      throw new Error(`C3: subscription B should remain enabled, got is_enabled=${sub_b_after.is_enabled}`);
    }

    // Verify health events for Sub A contain disabled event
    const events_a = get_health_events(h, s, sub_a.subscription_id, o);
    if (!isNotNull(events_a)) {
      throw new Error('C3: Failed to fetch health events for Sub A');
    }
    const has_disabled_a = events_a.some((e) => e.status === 'disabled' && e.source === 'system');
    if (!has_disabled_a) {
      throw new Error('C3: Expected disabled health event for Sub A');
    }

    // Verify health events for Sub B are empty
    const events_b = get_health_events(h, s, sub_b.subscription_id, o);
    if (!isNotNull(events_b)) {
      throw new Error('C3: Failed to fetch health events for Sub B');
    }
    if (events_b.length !== 0) {
      throw new Error(`C3: Expected 0 health events for healthy Sub B, got ${events_b.length}`);
    }

    console.log('C3 PASSED: subscriptions evaluated independently');
  } finally {
    cleanup(config, application_id);
  }
}
