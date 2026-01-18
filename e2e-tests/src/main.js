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
import login from './me/login.js';
import register from './me/register.js';
import verify_email from './me/verify_email.js';
import get_deletion_status from './me/get_deletion_status.js';
import request_deletion from './me/request_deletion.js';
import cancel_deletion from './me/cancel_deletion.js';
import account_isolation_test from './me/account_isolation_test.js';
import deleteAllEmails from './mailhog/delete_all_emails.js';

export const config = getEnvironmentVariables();

export const options = {
  vus: config.vus,
  iterations: config.iterations,
  duration: config.maxDuration,
  thresholds: {
    // the rate of successful checks should be at 100%
    checks: ['rate>=1.0'],
  },
};

function isNotNull(value) {
  return value && value !== null;
}

function scenario_1() {
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

    if (application_id && !config.keepTestApplication) {
      // TODO: wait for the request attempts to be successful instead of waiting 3s
      setTimeout(() => {
        delete_application(h, application_id, s);
      }, 3000);
    }
  } catch (error) {
    console.error(error.message);
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
    throw error;
  }
}

function scenario_subscription_deletion() {
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

    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  } catch (error) {
    console.error('Subscription deletion test failed:', error.message);
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
    throw error;
  }
}

function scenario_subscription_disable() {
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

    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
  } catch (error) {
    console.error('Subscription disable test failed:', error.message);
    if (application_id && !config.keepTestApplication) {
      delete_application(h, application_id, s);
    }
    throw error;
  }
}

function scenario_account_deletion() {
  const h = config.apiOrigin;
  const m = config.mailhogUrl;

  try {
    // 0. Clear any existing emails in Mailhog
    deleteAllEmails(m);

    // 1. Register the test user (if not already existing)
    const regResult = register(h, config.testUserEmail, config.testUserPassword);
    if (!regResult) {
      throw new Error('Failed to register test user');
    }

    // 2. Verify the user's email (only if user was newly registered)
    if (!regResult.already_exists) {
      const verifyResult = verify_email(h, m, config.testUserEmail);
      if (!verifyResult) {
        throw new Error('Failed to verify test user email');
      }
    }

    // 3. Login to get user access token
    const loginResponse = login(h, config.testUserEmail, config.testUserPassword);
    if (!loginResponse || !loginResponse.access_token) {
      throw new Error('Failed to login');
    }
    const accessToken = loginResponse.access_token;

    // 4. Get initial deletion status (should be false)
    const initialStatus = get_deletion_status(h, accessToken);
    if (initialStatus === null) {
      throw new Error('Failed to get initial deletion status');
    }
    if (initialStatus.deletion_requested !== false) {
      throw new Error('Expected initial deletion_requested to be false');
    }

    // 5. Request account deletion
    const requestResult = request_deletion(h, accessToken);
    if (!requestResult) {
      throw new Error('Failed to request account deletion');
    }

    // 6. Verify deletion status is now true
    const statusAfterRequest = get_deletion_status(h, accessToken);
    if (statusAfterRequest === null) {
      throw new Error('Failed to get deletion status after request');
    }
    if (statusAfterRequest.deletion_requested !== true) {
      throw new Error('Expected deletion_requested to be true after request');
    }

    // 7. Cancel deletion request
    const cancelResult = cancel_deletion(h, accessToken);
    if (!cancelResult) {
      throw new Error('Failed to cancel account deletion');
    }

    // 8. Verify deletion status is back to false
    const finalStatus = get_deletion_status(h, accessToken);
    if (finalStatus === null) {
      throw new Error('Failed to get final deletion status');
    }
    if (finalStatus.deletion_requested !== false) {
      throw new Error('Expected deletion_requested to be false after cancellation');
    }

    console.log('✓ Account deletion test passed');
  } catch (error) {
    console.error('Account deletion test failed:', error.message);
    throw error;
  }
}

function scenario_account_isolation() {
  account_isolation_test(config);
}

export default function () {
  scenario_1();
  scenario_subscription_deletion();
  scenario_subscription_disable();
  scenario_account_deletion();
  scenario_account_isolation();
}
