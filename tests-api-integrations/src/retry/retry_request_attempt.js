import http from 'k6/http';
import { check, sleep } from 'k6';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';
import create_application from '../applications/create_application.js';
import delete_application from '../applications/delete_application.js';
import create_event_type from '../event_types/create_event_type.js';
import create_subscription from '../subscriptions/create_subscription.js';
import update_subscription from '../subscriptions/update_subscription.js';
import send_event from '../events/send_event.js';
import list_request_attempt from '../events/list_request_attempt.js';

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
 * @description Polls a condition function until it returns a truthy value or timeout is reached.
 * Returns the truthy value on success, null on timeout.
 * @example
 *   const result = poll_until(() => { const a = getAttempts(); return a.find(x => x.failed_at) || null; }, 30000, 2000)
 */
function poll_until(check_fn, timeout_ms = 30000, interval_ms = 2000) {
  const start = Date.now();
  while (Date.now() - start < timeout_ms) {
    const result = check_fn();
    if (result) return result;
    sleep(interval_ms / 1000);
  }
  return null;
}

/**
 * @description Creates a standard test harness: application, event type, subscription.
 * Returns { application_id, event_type, subscription } or throws.
 * @example
 *   const ctx = create_test_context(config, 'http://localhost:19999', { label: 'val' })
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

/**
 * @description POSTs to the retry endpoint for a given request_attempt_id.
 * Returns the parsed response body (does not assert status).
 * @example
 *   const res = retry_attempt(baseUrl, serviceToken, 'attempt-uuid')
 *   // res => { status, body }
 */
function retry_attempt_raw(base_url, service_token, request_attempt_id) {
  const url = `${base_url}api/v1/request_attempts/${request_attempt_id}/retry`;
  const params = {
    headers: {
      Authorization: `Bearer ${service_token}`,
      'Content-Type': 'application/json',
    },
  };
  return http.post(url, null, params);
}

// ---------------------------------------------------------------------------
// Test 1: Retry a failed attempt
// ---------------------------------------------------------------------------

/**
 * @description Creates subscription to failing URL, sends event, waits for failure,
 * then retries the attempt and asserts 201 with source=user.
 * @example
 *   test_retry_failed_attempt(config)
 */
export function test_retry_failed_attempt(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrlFailing, { retry_test: 't1' });
    application_id = ctx.application_id;

    const event_id = send_event(s, h, ctx.application_id, ctx.event_type, { retry_test: 't1' });
    if (!isNotNull(event_id)) {
      throw new Error('T1: Failed to send event');
    }

    // Poll until at least one attempt has failed_at set
    const failed_attempt = poll_until(() => {
      const attempts = list_request_attempt(h, s, ctx.application_id, event_id);
      if (!isNotNull(attempts)) return null;
      return attempts.find((a) => a.failed_at !== null && a.failed_at !== undefined) || null;
    }, 60000, 2000);

    if (!failed_attempt) {
      throw new Error('T1: No failed attempt found within timeout');
    }

    const res = retry_attempt_raw(h, s, failed_attempt.request_attempt_id);
    check(res, {
      'T1: retry failed attempt returns 201': (r) => r.status === 201,
    });

    if (res.status !== 201) {
      throw new Error(`T1: Expected 201, got ${res.status}: ${res.body}`);
    }

    const body = JSON.parse(res.body);
    check(body, {
      'T1: retry response source is user': (b) => b.source === 'user',
      'T1: retry response retry_count is 0': (b) => b.retry_count === 0,
      'T1: retry response has request_attempt_id': (b) => isNotNull(b.request_attempt_id),
      'T1: retry response has event_id': (b) => b.event_id === event_id,
    });

    console.log('T1 PASSED: retry failed attempt');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// Test 2: Retry a succeeded attempt
// ---------------------------------------------------------------------------

/**
 * @description Creates subscription to healthy URL, sends event, waits for success,
 * then retries the attempt and asserts 201 (retrying succeeded attempts is allowed).
 * @example
 *   test_retry_succeeded_attempt(config)
 */
export function test_retry_succeeded_attempt(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrl, { retry_test: 't2' });
    application_id = ctx.application_id;

    const event_id = send_event(s, h, ctx.application_id, ctx.event_type, { retry_test: 't2' });
    if (!isNotNull(event_id)) {
      throw new Error('T2: Failed to send event');
    }

    // Poll until at least one attempt has succeeded_at set
    const succeeded_attempt = poll_until(() => {
      const attempts = list_request_attempt(h, s, ctx.application_id, event_id);
      if (!isNotNull(attempts)) return null;
      return attempts.find((a) => a.succeeded_at !== null && a.succeeded_at !== undefined) || null;
    }, 60000, 2000);

    if (!succeeded_attempt) {
      throw new Error('T2: No succeeded attempt found within timeout');
    }

    const res = retry_attempt_raw(h, s, succeeded_attempt.request_attempt_id);
    check(res, {
      'T2: retry succeeded attempt returns 201': (r) => r.status === 201,
    });

    if (res.status !== 201) {
      throw new Error(`T2: Expected 201, got ${res.status}: ${res.body}`);
    }

    const body = JSON.parse(res.body);
    check(body, {
      'T2: retry response source is user': (b) => b.source === 'user',
    });

    console.log('T2 PASSED: retry succeeded attempt');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// Test 3: Retry nonexistent attempt
// ---------------------------------------------------------------------------

/**
 * @description POSTs retry with a random UUID and asserts 404.
 * @example
 *   test_retry_nonexistent_attempt(config)
 */
export function test_retry_nonexistent_attempt(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;

  const fake_id = uuidv4();
  const res = retry_attempt_raw(h, s, fake_id);
  check(res, {
    'T3: retry nonexistent attempt returns 404': (r) => r.status === 404,
  });

  if (res.status !== 404) {
    throw new Error(`T3: Expected 404, got ${res.status}: ${res.body}`);
  }

  console.log('T3 PASSED: retry nonexistent attempt returns 404');
}

// ---------------------------------------------------------------------------
// Test 4: Retry on disabled subscription
// ---------------------------------------------------------------------------

/**
 * @description Creates subscription, sends event, waits for attempt, disables subscription,
 * then retries the attempt and asserts 201 (bypass is_enabled).
 * @example
 *   test_retry_on_disabled_subscription(config)
 */
export function test_retry_on_disabled_subscription(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrl, { retry_test: 't4' });
    application_id = ctx.application_id;
    const sub_id = ctx.subscription.subscription_id;

    const event_id = send_event(s, h, ctx.application_id, ctx.event_type, { retry_test: 't4' });
    if (!isNotNull(event_id)) {
      throw new Error('T4: Failed to send event');
    }

    // Wait for at least one attempt to exist
    const attempt = poll_until(() => {
      const attempts = list_request_attempt(h, s, ctx.application_id, event_id);
      if (!isNotNull(attempts) || attempts.length === 0) return null;
      return attempts[0];
    }, 60000, 2000);

    if (!attempt) {
      throw new Error('T4: No attempt found within timeout');
    }

    // Disable the subscription
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
      throw new Error('T4: Failed to disable subscription');
    }

    // Retry the attempt — should succeed despite disabled subscription
    const res = retry_attempt_raw(h, s, attempt.request_attempt_id);
    check(res, {
      'T4: retry on disabled subscription returns 201': (r) => r.status === 201,
    });

    if (res.status !== 201) {
      throw new Error(`T4: Expected 201, got ${res.status}: ${res.body}`);
    }

    const body = JSON.parse(res.body);
    check(body, {
      'T4: retry response source is user': (b) => b.source === 'user',
    });

    console.log('T4 PASSED: retry on disabled subscription');
  } finally {
    cleanup(config, application_id);
  }
}

// ---------------------------------------------------------------------------
// Test 5: Verify source field in list response after retry
// ---------------------------------------------------------------------------

/**
 * @description After a retry, lists request_attempts and asserts that the new attempt
 * has source=user and the original has source=system.
 * @example
 *   test_retry_source_in_list_response(config)
 */
export function test_retry_source_in_list_response(config) {
  const h = config.apiOrigin;
  const s = config.serviceToken;
  let application_id = null;

  try {
    const ctx = create_test_context(config, config.targetUrl, { retry_test: 't5' });
    application_id = ctx.application_id;

    const event_id = send_event(s, h, ctx.application_id, ctx.event_type, { retry_test: 't5' });
    if (!isNotNull(event_id)) {
      throw new Error('T5: Failed to send event');
    }

    // Wait for the original attempt to be delivered
    const original_attempt = poll_until(() => {
      const attempts = list_request_attempt(h, s, ctx.application_id, event_id);
      if (!isNotNull(attempts) || attempts.length === 0) return null;
      // Wait for it to be completed (succeeded or failed)
      const completed = attempts.find(
        (a) =>
          (a.succeeded_at !== null && a.succeeded_at !== undefined) ||
          (a.failed_at !== null && a.failed_at !== undefined)
      );
      return completed || null;
    }, 60000, 2000);

    if (!original_attempt) {
      throw new Error('T5: No completed attempt found within timeout');
    }

    // Retry the attempt
    const res = retry_attempt_raw(h, s, original_attempt.request_attempt_id);
    if (res.status !== 201) {
      throw new Error(`T5: Retry failed with status ${res.status}: ${res.body}`);
    }

    // Poll until at least 2 attempts appear in the list (retry has been dispatched)
    const all_attempts = poll_until(() => {
      const attempts = list_request_attempt(h, s, ctx.application_id, event_id);
      if (!isNotNull(attempts) || attempts.length < 2) return null;
      return attempts;
    }, 30000, 2000);

    if (!all_attempts) {
      throw new Error('T5: Expected at least 2 attempts after retry within timeout');
    }

    const system_attempts = all_attempts.filter((a) => a.source === 'system');
    const user_attempts = all_attempts.filter((a) => a.source === 'user');

    check(null, {
      'T5: at least one system source attempt': () => system_attempts.length >= 1,
      'T5: at least one user source attempt': () => user_attempts.length >= 1,
    });

    if (system_attempts.length < 1) {
      throw new Error('T5: Expected at least one attempt with source=system');
    }
    if (user_attempts.length < 1) {
      throw new Error('T5: Expected at least one attempt with source=user');
    }

    console.log('T5 PASSED: retry source visible in list response');
  } finally {
    cleanup(config, application_id);
  }
}
