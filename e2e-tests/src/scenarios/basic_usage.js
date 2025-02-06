import create_application from '../services/applications/create_application.js';
import create_event_type from '../services/event_types/create_event_type.js';
import create_subscription from '../services/subscriptions/create_subscription.js';
import send_event from '../services/events/send_event.js';
import list_request_attempt from '../services/events/list_request_attempt.js';
import delete_application from '../services/applications/delete_application.js';
import get_quota from '../services/unauthentified/quotas.js';
import { isNotNull } from '../utils/function.js';

export function basic_usage(config) {
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

    let subscription_1 = create_subscription(
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

    let subscription_2 = create_subscription(
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

    let validation_quota = get_quota(h);
    if (!validation_quota) {
      throw new Error('Failed to verify quota response');
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
