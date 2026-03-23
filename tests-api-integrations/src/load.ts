import { fail } from 'k6';
import { expect } from 'https://jslib.k6.io/k6-testing/0.5.0/index.js';
import create_application from './applications/create_application.js';
import create_event_type from './event_types/create_event_type.js';
import create_subscription from './subscriptions/create_subscription.js';
import send_event from './events/send_event.js';

const config = {
  vus: parseInt(__ENV.VUS) ?? 1,
  iterations: parseInt(__ENV.ITERATIONS) ?? 1,
  duration: __ENV.DURATION ?? null,
  apiOrigin: __ENV.API_ORIGIN,
  targetUrl: __ENV.TARGET_URL,
  serviceToken: __ENV.SERVICE_TOKEN,
  organizationId: __ENV.ORGANIZATION_ID,
  nEventTypes: parseInt(__ENV.N_EVENT_TYPES) ?? 10,
  nSubscriptions: parseInt(__ENV.N_SUBSCRIPTIONS) ?? 10,
  nLabels: parseInt(__ENV.N_LABELS) ?? 2,
};

if (!config.apiOrigin || !config.targetUrl || !config.serviceToken || !config.organizationId) {
  fail('Missing environment variables API_ORIGIN, TARGET_URL, SERVICE_TOKEN, ORGANIZATION_ID');
}

export const options = {
  vus: config.vus,
  iterations: config.iterations,
  duration: config.duration,
  thresholds: {
    checks: ['rate>=0.5'],
  },
};

interface SetupData {
  applicationId: string;
  eventTypes: string[];
  labelKey: string;
  labelValues: string[];
}

function getMultipleRandom<T>(arr: T[], num: number): T[] {
  const shuffled = [...arr].sort(() => 0.5 - Math.random());

  return shuffled.slice(0, num);
}

function randomIntFromInterval(min: number, max: number): number {
  // min and max included
  return Math.floor(Math.random() * (max - min + 1) + min);
}

export function setup(): SetupData {
  const applicationId = create_application(
    config.apiOrigin,
    config.organizationId,
    config.serviceToken,
    `load_k6_${new Date().toISOString()}`
  );
  expect(applicationId).not.toBeNull();

  const eventTypes = [];
  for (let i = 0; i < config.nEventTypes; i++) {
    const et = create_event_type(config.apiOrigin, config.serviceToken, applicationId);
    expect(et).not.toBeNull();
    eventTypes.push(et);
  }

  const labelKey = 'label_key';
  const labelValues = [];
  for (let i = 0; i < config.nLabels; i++) {
    labelValues.push(`label_value_${i}`);
  }

  for (let i = 0; i < config.nSubscriptions; i++) {
    const et = getMultipleRandom(eventTypes, randomIntFromInterval(1, eventTypes.length));
    const lv = getMultipleRandom(labelValues, 1)[0];
    const s = create_subscription(
      config.apiOrigin,
      config.serviceToken,
      applicationId,
      et,
      config.targetUrl,
      { [labelKey]: lv }
    );
    expect(s).not.toBeNull();
  }

  return {
    applicationId,
    eventTypes,
    labelKey,
    labelValues,
  };
}

export default function (setupData: SetupData): void {
  const et = getMultipleRandom(setupData.eventTypes, 1)[0];
  const lv = getMultipleRandom(setupData.labelValues, 1)[0];
  const e = send_event(config.serviceToken, config.apiOrigin, setupData.applicationId, et, {
    [setupData.labelKey]: lv,
    otherLabel1: 'otherValue1',
    otherLabel2: 'otherValue2',
  });
  expect(e).not.toBeNull();
}

export function teardown(setupData: SetupData): void {
  console.log(`Run the following SQL query to see if delivery has ended:\n
select count(ra.request_attempt__id)
from webhook.request_attempt as ra
inner join event.event as e on e.event__id = ra.event__id
where ra.succeeded_at is null and ra.failed_at is null and e.application__id = '${setupData.applicationId}';\n\n\n`);

  console.log(`Run the following SQL query to see delivery stats:\n
with events as (
  select application__id, count(event__id) as nb_events
  from event.event
  group by application__id
), request_attempts as (
  select
    e.application__id,
    count(ra.request_attempt__id) as nb_request_attempt,
    avg(ra.picked_at - coalesce(ra.delay_until, ra.created_at)) as avg_pickup_time,
    min(ra.picked_at - coalesce(ra.delay_until, ra.created_at)) as min_pickup_time,
    max(ra.picked_at - coalesce(ra.delay_until, ra.created_at)) as max_pickup_time,
    avg(coalesce(ra.succeeded_at, ra.failed_at) - ra.picked_at) as avg_target_response_time,
    max(coalesce(ra.succeeded_at, ra.failed_at) - ra.picked_at) as max_target_response_time,
    min(coalesce(ra.succeeded_at, ra.failed_at) - ra.picked_at) as min_target_response_time
  from webhook.request_attempt as ra
  inner join event.event as e on e.event__id = ra.event__id
  group by e.application__id
)
select
    e.nb_events,
    ra.nb_request_attempt,
    ra.nb_request_attempt::float / nb_events::float as ratio_request_attempts_per_event,
    ra.avg_pickup_time,
    ra.min_pickup_time,
    ra.max_pickup_time,
    ra.avg_target_response_time,
    ra.min_target_response_time,
    ra.max_target_response_time
from events as e
inner join request_attempts as ra on ra.application__id = e.application__id
where e.application__id = '${setupData.applicationId}';\n\n\n`)
}
