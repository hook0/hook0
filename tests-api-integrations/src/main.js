import http from 'k6/http';
import { getEnvironmentVariables } from './config.js';
import create_application from './applications/create_application.js';
import create_event_type from './event_types/create_event_type.js';
import create_subscription from './subscriptions/create_subscription.js';
import create_subscription_legacy from './subscriptions/create_subscription_legacy.js';
import update_subscription from './subscriptions/update_subscription.js';
import delete_subscription from './subscriptions/delete_subscription.js';
import send_event from './events/send_event.js';
import list_request_attempt from './events/list_request_attempt.js';
import query_request_attempts from './database/query_request_attempts.js';
import delete_application from './applications/delete_application.js';
import get_quota from './unauthentified/quotas.js';
import get_environment_variables from './unauthentified/environment_variables.js';
import create_retry_schedule from './retry_schedules/create_retry_schedule.js';
import list_retry_schedules from './retry_schedules/list_retry_schedules.js';
import get_retry_schedule from './retry_schedules/get_retry_schedule.js';
import update_retry_schedule from './retry_schedules/update_retry_schedule.js';
import delete_retry_schedule from './retry_schedules/delete_retry_schedule.js';
import assign_to_subscription from './retry_schedules/assign_to_subscription.js';
import delete_all_retry_schedules from './retry_schedules/delete_all_retry_schedules.js';
import reactivation_tests from './health_monitor/reactivation_tests.js';
import {
  test_b1_failure_disables_subscription,
  test_b2_success_stays_enabled,
  test_b3_reenable_after_autodisable,
  test_b4_full_lifecycle,
  test_b5_adaptive_windowing,
  test_c1_user_disabled_not_evaluated,
  test_c2_below_min_sample_size,
  test_c3_independent_evaluation,
} from './health_monitor/health_monitor_tests.js';

export const config = getEnvironmentVariables();

export const options = {
  scenarios: {
    basic_workflow: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_basic_workflow',
    },
    subscription_deletion: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_subscription_deletion',
    },
    subscription_disable: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_subscription_disable',
    },
    retry_schedules: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_retry_schedules',
    },
    retry_schedule_subscription_assignment: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_retry_schedule_subscription_assignment',
    },
    health_monitor_reactivation: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_health_monitor_reactivation',
    },
    health_monitor_b1: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_b1',
    },
    health_monitor_b2: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_b2',
    },
    health_monitor_b3: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_b3',
    },
    health_monitor_b4: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_b4',
    },
    health_monitor_b5: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '3m',
      exec: 'scenario_b5',
    },
    health_monitor_c1: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_c1',
    },
    health_monitor_c2: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_c2',
    },
    health_monitor_c3: {
      executor: 'shared-iterations',
      vus: 1,
      iterations: 1,
      maxDuration: '2m',
      exec: 'scenario_c3',
    },
  },
  thresholds: {
    checks: ['rate>=1.0'],
  },
};

function isNotNull(value) {
  return value && value !== null;
}

export function scenario_basic_workflow() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  let application_id = null;

  try {
    application_id = create_application(h, o, s);
    if (!isNotNull(application_id)) {
      throw new Error('Failed to create application');
    }

    let event_type_1 = create_event_type(h, s, application_id);
    if (!isNotNull(event_type_1)) {
      throw new Error('Failed to create event type 1');
    }

    let event_type_2 = create_event_type(h, s, application_id);
    if (!isNotNull(event_type_2)) {
      throw new Error('Failed to create event type 2');
    }

    let subscription_1 = create_subscription_legacy(
      h,
      s,
      application_id,
      [event_type_1, event_type_2],
      config.targetUrl,
      'all',
      'yes'
    );
    if (!isNotNull(subscription_1)) {
      throw new Error('Failed to create subscription 1');
    }

    let subscription_2 = create_subscription_legacy(
      h,
      s,
      application_id,
      [event_type_1],
      config.targetUrl,
      'all',
      'yes'
    );
    if (!isNotNull(subscription_2)) {
      throw new Error('Failed to create subscription 2');
    }

    let subscription_3 = create_subscription(
      h,
      s,
      application_id,
      [event_type_1],
      config.targetUrl,
      { all: 'yes', other_label: '42' }
    );
    if (!isNotNull(subscription_3)) {
      throw new Error('Failed to create subscription 3');
    }

    let event_1 = send_event(s, h, application_id, event_type_1, {
      [subscription_1.label_key]: subscription_1.label_value,
    });
    if (!isNotNull(event_1)) {
      throw new Error('Failed to create event 1');
    }

    let event_2 = send_event(s, h, application_id, event_type_2, {
      [subscription_2.label_key]: subscription_2.label_value,
    });
    if (!isNotNull(event_2)) {
      throw new Error('Failed to create event 2');
    }

    let event_3 = send_event(s, h, application_id, event_type_1, { test: 'test' });
    if (!isNotNull(event_3)) {
      throw new Error('Failed to create event 3');
    }

    let event_4 = send_event(s, h, application_id, event_type_1, {
      ...subscription_3.labels,
      unused_label: 'test',
    });
    if (!isNotNull(event_4)) {
      throw new Error('Failed to create event 4');
    }

    let request_attempts_1 = list_request_attempt(h, s, application_id, event_1);
    if (!isNotNull(request_attempts_1) || request_attempts_1.length !== 2) {
      throw new Error(
        'Expected to find 2 request attempts for event 1 | Found: ' + request_attempts_1.length
      );
    }

    let request_attempts_2 = list_request_attempt(h, s, application_id, event_2);
    if (!isNotNull(request_attempts_2) || request_attempts_2.length !== 1) {
      throw new Error(
        'Expected to find 1 request attempts for event 2 | Found: ' + request_attempts_2.length
      );
    }

    let request_attempts_3 = list_request_attempt(h, s, application_id, event_3);
    if (!isNotNull(request_attempts_3) || request_attempts_3.length !== 0) {
      throw new Error(
        'Expected to find 0 request attempts for event 3 | Found: ' + request_attempts_3.length
      );
    }

    let request_attempts_4 = list_request_attempt(h, s, application_id, event_4);
    if (!isNotNull(request_attempts_4) || request_attempts_4.length !== 3) {
      throw new Error(
        'Expected to find 3 request attempts for event 4 | Found: ' + request_attempts_4.length
      );
    }

    let validation_quota = get_quota(h);
    if (!validation_quota) {
      throw new Error('Failed to verify quota response');
    }

    let validation_environment_variables = get_environment_variables(h);
    if (!validation_environment_variables) {
      throw new Error('Failed to verify environment_variables response');
    }

    console.log('✓ Basic workflow test passed');
  } finally {
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  }
}

export function scenario_subscription_deletion() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  let application_id = null;

  try {
    // 1. Setup
    application_id = create_application(h, o, s);
    if (!isNotNull(application_id)) {
      throw new Error('Failed to create application');
    }

    const event_type = create_event_type(h, s, application_id);
    if (!isNotNull(event_type)) {
      throw new Error('Failed to create event type');
    }

    const subscription = create_subscription(h, s, application_id, [event_type], config.targetUrl, {
      test_label: 'test_value',
    });
    if (!isNotNull(subscription)) {
      throw new Error('Failed to create subscription');
    }

    // 2. Send event to create pending request attempts
    const event_id = send_event(s, h, application_id, event_type, {
      test_label: 'test_value',
    });
    if (!isNotNull(event_id)) {
      throw new Error('Failed to create event');
    }

    // 3. Verify we have pending attempts
    let attempts_before = list_request_attempt(h, s, application_id, event_id);
    if (!isNotNull(attempts_before) || attempts_before.length === 0) {
      throw new Error(
        'Expected to find at least 1 request attempt before deletion | Found: ' +
          (attempts_before ? attempts_before.length : 0)
      );
    }

    // Find pending attempts (no failed_at, no succeeded_at)
    const pending_before = attempts_before.filter((a) => !a.failed_at && !a.succeeded_at);
    if (pending_before.length === 0) {
      console.log(
        'No pending attempts found (they may have been processed already), skipping test'
      );
      return;
    }

    // 5. Record timestamp before deletion
    const timestamp_before_delete = new Date().toISOString();

    // 6. Delete subscription
    const delete_result = delete_subscription(h, s, subscription.subscription_id, application_id);
    if (!isNotNull(delete_result)) {
      throw new Error('Failed to delete subscription');
    }

    // 7. Verify pending attempts now have failed_at set
    let attempts_after = query_request_attempts(
      h,
      s,
      application_id,
      subscription.subscription_id,
      event_id
    );

    // Filter pending attempts that should now be marked as failed
    const failed_attempts = attempts_after.filter((a) => {
      return a.failed_at !== null && a.failed_at !== undefined;
    });

    if (failed_attempts.length < pending_before.length) {
      throw new Error(
        `Expected at least ${pending_before.length} attempts to be marked as failed | Found: ${failed_attempts.length}`
      );
    }

    // Verify failed_at timestamps are reasonable (after deletion timestamp)
    for (const attempt of failed_attempts) {
      const failed_at = new Date(attempt.failed_at);
      const before_delete = new Date(timestamp_before_delete);
      if (failed_at < before_delete) {
        throw new Error(
          `failed_at timestamp (${attempt.failed_at}) should be after deletion timestamp (${timestamp_before_delete})`
        );
      }
    }

    console.log(
      `✓ Subscription deletion test passed: ${failed_attempts.length} attempts marked as failed`
    );
  } finally {
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  }
}

export function scenario_subscription_disable() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  let application_id = null;

  try {
    // 1. Setup
    application_id = create_application(h, o, s);
    if (!isNotNull(application_id)) {
      throw new Error('Failed to create application');
    }

    const event_type = create_event_type(h, s, application_id);
    if (!isNotNull(event_type)) {
      throw new Error('Failed to create event type');
    }

    const subscription = create_subscription(h, s, application_id, [event_type], config.targetUrl, {
      test_label: 'test_value',
    });
    if (!isNotNull(subscription)) {
      throw new Error('Failed to create subscription');
    }

    // Verify subscription is enabled
    if (!subscription.is_enabled) {
      throw new Error('Subscription should be enabled by default');
    }

    // 2. Send event to create pending request attempts
    const event_id = send_event(s, h, application_id, event_type, {
      test_label: 'test_value',
    });
    if (!isNotNull(event_id)) {
      throw new Error('Failed to create event');
    }

    // 3. Verify we have pending attempts
    let attempts_before = list_request_attempt(h, s, application_id, event_id);
    if (!isNotNull(attempts_before) || attempts_before.length === 0) {
      throw new Error(
        'Expected to find at least 1 request attempt before disable | Found: ' +
          (attempts_before ? attempts_before.length : 0)
      );
    }

    // Find pending attempts (no failed_at, no succeeded_at)
    const pending_before = attempts_before.filter((a) => !a.failed_at && !a.succeeded_at);
    if (pending_before.length === 0) {
      console.log(
        'No pending attempts found (they may have been processed already), skipping test'
      );
      return;
    }

    // 5. Record timestamp before disable
    const timestamp_before_disable = new Date().toISOString();

    // 6. Disable subscription
    const subscription_to_update = {
      application_id: application_id,
      is_enabled: false,
      event_types: [event_type],
      target: subscription.target,
      description: subscription.description,
      metadata: subscription.metadata || {},
      labels: subscription.labels,
    };

    const updated = update_subscription(
      h,
      s,
      subscription.subscription_id,
      application_id,
      subscription_to_update
    );
    if (!isNotNull(updated)) {
      throw new Error('Failed to disable subscription');
    }

    if (updated.is_enabled !== false) {
      throw new Error('Subscription should be disabled after update');
    }

    // 7. Verify pending attempts now have failed_at set
    let attempts_after = query_request_attempts(
      h,
      s,
      application_id,
      subscription.subscription_id,
      event_id
    );

    // Filter pending attempts that should now be marked as failed
    const failed_attempts = attempts_after.filter((a) => {
      return a.failed_at !== null && a.failed_at !== undefined;
    });

    if (failed_attempts.length < pending_before.length) {
      throw new Error(
        `Expected at least ${pending_before.length} attempts to be marked as failed | Found: ${failed_attempts.length}`
      );
    }

    // Verify failed_at timestamps are reasonable (after disable timestamp)
    for (const attempt of failed_attempts) {
      const failed_at = new Date(attempt.failed_at);
      const before_disable = new Date(timestamp_before_disable);
      if (failed_at < before_disable) {
        throw new Error(
          `failed_at timestamp (${attempt.failed_at}) should be after disable timestamp (${timestamp_before_disable})`
        );
      }
    }

    console.log(
      `✓ Subscription disable test passed: ${failed_attempts.length} attempts marked as failed`
    );
  } finally {
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  }
}

export function scenario_retry_schedules() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  // Safety net: delete any leftover schedules from previous runs
  delete_all_retry_schedules(h, s, o);

  let application_id = null;
  const created_schedule_ids = [];

  try {
    // 1. Setup — create an application and subscription to test against
    application_id = create_application(h, o, s);
    if (!isNotNull(application_id)) {
      throw new Error('Failed to create application');
    }

    const event_type = create_event_type(h, s, application_id);
    if (!isNotNull(event_type)) {
      throw new Error('Failed to create event type');
    }

    const subscription = create_subscription(h, s, application_id, [event_type], config.targetUrl, {
      test_label: 'test_value',
    });
    if (!isNotNull(subscription)) {
      throw new Error('Failed to create subscription');
    }

    // 2. Create retry schedules
    const schedule_1 = create_retry_schedule(h, s, o);
    if (!isNotNull(schedule_1)) {
      throw new Error('Failed to create retry schedule 1');
    }
    created_schedule_ids.push(schedule_1.retry_schedule_id);

    const schedule_2 = create_retry_schedule(h, s, o);
    if (!isNotNull(schedule_2)) {
      throw new Error('Failed to create retry schedule 2');
    }
    created_schedule_ids.push(schedule_2.retry_schedule_id);

    // 3. List retry schedules
    const schedules = list_retry_schedules(h, s, o);
    if (!isNotNull(schedules) || schedules.length < 2) {
      throw new Error(
        'Expected at least 2 retry schedules | Found: ' + (schedules ? schedules.length : 0)
      );
    }

    // 4. Get individual retry schedules
    const fetched_1 = get_retry_schedule(h, s, o, schedule_1.retry_schedule_id);
    if (!isNotNull(fetched_1)) {
      throw new Error('Failed to get retry schedule 1');
    }

    const fetched_2 = get_retry_schedule(h, s, o, schedule_2.retry_schedule_id);
    if (!isNotNull(fetched_2)) {
      throw new Error('Failed to get retry schedule 2');
    }

    // 5. Update retry schedule
    const updated = update_retry_schedule(h, s, o, schedule_1.retry_schedule_id);
    if (!isNotNull(updated)) {
      throw new Error('Failed to update retry schedule 1');
    }

    // 6. Delete retry schedules (delete schedule_2 which is not assigned)
    const deleted = delete_retry_schedule(h, s, o, schedule_2.retry_schedule_id);
    if (!isNotNull(deleted)) {
      throw new Error('Failed to delete retry schedule 2');
    }

    console.log('✓ Retry schedules test passed');
  } finally {
    // Some schedules were already deleted during the test; ignore 404s from redundant deletes
    for (const sid of created_schedule_ids) {
      try {
        http.del(`${h}api/v1/retry_schedules/${sid}?organization_id=${o}`, null, {
          headers: { Authorization: `Bearer ${s}` },
        });
      } catch (_) {} // Silently ignore — schedule may already be deleted by the test
    }
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  }
}

export function scenario_retry_schedule_subscription_assignment() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  let application_id = null;

  try {
    application_id = create_application(h, o, s);
    if (!isNotNull(application_id)) {
      throw new Error('Failed to create application');
    }

    const event_type = create_event_type(h, s, application_id);
    if (!isNotNull(event_type)) {
      throw new Error('Failed to create event type');
    }

    const result = assign_to_subscription(h, s, o, application_id, event_type, config.targetUrl);
    if (!isNotNull(result)) {
      throw new Error('Retry schedule subscription assignment tests failed');
    }

    console.log('✓ Retry schedule subscription assignment scenario passed');
  } finally {
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  }
}

export function scenario_health_monitor_reactivation() {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  const o = config.organizationId;

  reactivation_tests(h, s, o, config.targetUrl);
}

export function scenario_b1() { test_b1_failure_disables_subscription(config); }
export function scenario_b2() { test_b2_success_stays_enabled(config); }
export function scenario_b3() { test_b3_reenable_after_autodisable(config); }
export function scenario_b4() { test_b4_full_lifecycle(config); }
export function scenario_b5() { test_b5_adaptive_windowing(config); }
export function scenario_c1() { test_c1_user_disabled_not_evaluated(config); }
export function scenario_c2() { test_c2_below_min_sample_size(config); }
export function scenario_c3() { test_c3_independent_evaluation(config); }
