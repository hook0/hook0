import { getEnvironmentVariables } from './config.js';
import create_application from './applications/create_application.js';
import create_event_type from './event_types/create_event_type.js';
import create_subscription from './subscriptions/create_subscription.js';
import send_event from './events/send_event.js';
import list_request_attempt from './events/list_request_attempt.js';
import delete_application from './applications/delete_application.js';

export const config = getEnvironmentVariables();

export const options = {
  vus: config.vus,
  duration: config.duration,
  //maxDuration: config.maxDuration,
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

  let application_id = create_application(h, o, s);
  if (!isNotNull(application_id)) {
    throw new Error('Failed to create application');
  }

  let event_type_1 = create_event_type(h, s, application_id);
  if (!isNotNull(event_type_1)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create event type 1');
  }

  let event_type_2 = create_event_type(h, s, application_id);
  if (!isNotNull(event_type_2)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
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
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create subscription');
  }
  let subscription_1_id = subscription_1.subscription_id;
  let label_1_key = subscription_1.label_key;
  let label_1_value = subscription_1.label_value;

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
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create subscription');
  }
  let subscription_2_id = subscription_2.subscription_id;
  let label_2_key = subscription_2.label_key;
  let label_2_value = subscription_2.label_value;

  let event_1 = send_event(s, h, application_id, event_type_1, { [label_1_key]: label_1_value });
  if (!isNotNull(event_1)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create event 1');
  }

  let event_2 = send_event(s, h, application_id, event_type_2, { [label_2_key]: label_2_value });
  if (!isNotNull(event_2)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create event 2');
  }

  // Event 3 shouldn't be delivered
  let event_3 = send_event(s, h, application_id, event_type_1, { test: 'test' });
  if (!isNotNull(event_3)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to create event 3');
  }

  let request_attempts_1 = list_request_attempt(h, s, application_id, event_1);
  if (!isNotNull(request_attempts_1)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to list request attempts 1');
  }
  if (request_attempts_1.length !== 2) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error(
      'Expected to find 2 request attempts for event 1 | Found: ' + request_attempts_1.length
    );
  }

  let request_attempts_2 = list_request_attempt(h, s, application_id, event_2);
  if (!isNotNull(request_attempts_2)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to list request attempts 2');
  }
  if (request_attempts_2.length !== 1) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error(
      'Expected to find 1 request attempts for event 2 | Found: ' + request_attempts_2.length
    );
  }

  // Event 3 shouldn't be received by any subscription so the length should be 0
  let request_attempts_3 = list_request_attempt(h, s, application_id, event_3);
  if (!isNotNull(request_attempts_3)) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error('Failed to list request attempts 3');
  }
  if (request_attempts_3.length !== 0) {
    if (config.deleteOnFail) {
      delete_application(h, application_id, s);
    }
    throw new Error(
      'Expected to find 0 request attempts for event 3 | Found: ' + request_attempts_3.length
    );
  }
}

export default function () {
  scenario_1();
}
