import { getEnvironmentVariables} from "./config.js";
import create_user from "./users/create_user.js";
import create_application from "./applications/create_application.js";
import create_appliation_secret from "./applications/create_appliation_secret.js";
import create_event_type from "./event_types/create_event_type.js";
import create_subscription from "./subscriptions/create_subscription.js";
import create_event from "./events/create_event.js";
import list_requests_attempts from "./events/list_requests_attempts.js";

import http from 'k6/http';
import { check, sleep } from 'k6';

export const config = getEnvironmentVariables();

export const options = {
    vus: config.vus,
    duration: config.duration,
    maxDuration: config.max_duration,
}

function isNotNull(value) {
    return value && value !== null;
}

// Setup
export function setup() {
    let createUserBody = create_user(config.hostname);
    if (!isNotNull(createUserBody)) {
        console.error("Failed to create user");
        return;
    }

    let user_id = createUserBody.user_id;
    let organisation_id = createUserBody.organization_id;

    let application_id = create_application(config.hostname, organisation_id, config.masterApiKey);
    if (!isNotNull(application_id)) {
        console.error("Failed to create application");
        return;
    }

    let application_secret = create_appliation_secret(config.hostname, application_id, config.masterApiKey);
    if (!isNotNull(application_secret)) {
        console.error("Failed to create application secret");
        return;
    }

    let event_type_1 = create_event_type(config.hostname, application_secret, application_id);
    if (!isNotNull(event_type_1)) {
        console.error("Failed to create event type 1");
        return;
    }

    let event_type_2 = create_event_type(config.hostname, application_secret, application_id);
    if (!isNotNull(event_type_2)) {
        console.error("Failed to create event type 2");
        return;
    }

    let subscription_1 = create_subscription(config.hostname, application_secret, application_id, [event_type_1, event_type_2], config.targetUrl, "all", "yes");
    if (!isNotNull(subscription_1)) {
        console.error("Failed to create subscription");
        return;
    }
    let subscription_1_id = subscription_1.subscription_id;
    let label_key = subscription_1.label_key;
    let label_value = subscription_1.label_value;


    let subscription_2 = create_subscription(config.hostname, application_secret, application_id, [event_type_1], config.targetUrl, "all", "yes");
    if (!isNotNull(subscription_2)) {
        console.error("Failed to create subscription");
        return;
    }

    let event_1 = create_event(application_secret, config.hostname, application_id, event_type_1);
    if (!isNotNull(event_1)) {
        console.error("Failed to create event 1");
        return;
    }

    let event_2 = create_event(application_secret, config.hostname, application_id, event_type_2);
    if (!isNotNull(event_2)) {
        console.error("Failed to create event 2");
        return;
    }

    let request_attempts_1 = list_requests_attempts(config.hostname, application_secret, application_id, event_1);
    if (!isNotNull(request_attempts_1)) {
        console.error("Failed to list request attempts 1");
        return;
    }
    if (request_attempts_1.length !== 2) {
        console.error("Expected to find 2 request attempts for event 1 | Found: " + request_attempts_1.length);
        return;
    }

    let request_attempts_2 = list_requests_attempts(config.hostname, application_secret, application_id, event_2);
    if (!isNotNull(request_attempts_2)) {
        console.error("Failed to list request attempts 2");
        return;
    }
    if (request_attempts_2.length !== 1) {
        console.error("Expected to find 1 request attempts for event 2 | Found: " + request_attempts_2.length);
        return;
    }
}

export default function () {
}