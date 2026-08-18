// The TypeScript client against a Hook0 that is really running.
//
// Two things happen here, and the second is the reason the first is worth having.
//
// The control: whether an application secret the API minted is accepted, whether a second send
// under an identifier already ingested is reported as the conflict it is, and whether a signature
// the output worker computed verifies. Those are the three questions no loopback suite can ask
// itself, because a suite that signs and verifies with the same module only proves the module
// agrees with itself.
//
// The surface: every operation the API document declares, driven through the generated layer
// against the same instance, and every model type it decodes out of a real answer.
// `clients/typescript/tests/generated.test.ts` already drives all of them — against an API the
// suite itself writes, out of the same document the client was generated from. That proves the
// client matches the document. It cannot prove the document matches Hook0.
//
// Reached as `require('hook0-client')`, which is one of the two ways the package promises to be
// reachable and the only one that goes through its `exports` map. The generated half arrives under
// `generated` rather than flattened beside the rest, because the document declares an `Event` and
// an `EventType` of its own that would otherwise collide with the emitter's.

const fs = require('fs');
const path = require('path');
const { Event, Hook0Client, verifyWebhookSignature, generated } = require('hook0-client');

// The conflict the API answers a duplicated ingestion with.
const ALREADY_INGESTED = 'EventAlreadyIngested';

// What this smoke labels everything it creates with, so that the subscription it makes and the
// event it sends find each other.
const LANGUAGE = 'typescript';

// Where the subscription this smoke creates points. Nothing listens there, deliberately: what a
// delivery proves is proved once, by the webhook the harness catches and every language verifies.
const NOWHERE = 'http://127.0.0.1:1/';

// What a paced instance answers, the most times one request is sent again after it, and the
// shortest and longest this waits in between.
const TOO_MANY_REQUESTS = 429;
const PACED_AGAIN = 8;
const SHORTEST_PAUSE_MS = 200;
const LONGEST_PAUSE_MS = 10000;

function setting(name) {
  const value = process.env[name];
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error(`${name} is not set`);
  }
  return value;
}

// The instance without the path the hand-written half is built with.
//
// The generated half composes paths that already carry `/api/v1`, since the API document's own
// server URL is the bare origin. Handing it the whole of `HOOK0_API_URL` reaches `/api/v1/api/v1`
// and answers 404 for every operation.
function originOf(apiUrl) {
  const parsed = new URL(apiUrl);
  return `${parsed.protocol}//${parsed.host}`;
}

function waited(ms) {
  return new Promise((woken) => setTimeout(woken, ms));
}

// How long the answer says to wait, held between a floor and a ceiling of this smoke's own.
//
// The floor is there because the header counts in whole seconds and the delay being waited out is a
// fraction of one, so a truthful `Retry-After: 0` would otherwise mean sending the same request
// again immediately, forever. The ceiling is there because a header is written by a server this
// smoke does not control.
function pause(answer) {
  const said = Number(answer.headers.get('retry-after'));
  const asked = Number.isFinite(said) ? said * 1000 : SHORTEST_PAUSE_MS;
  return Math.min(Math.max(asked, SHORTEST_PAUSE_MS), LONGEST_PAUSE_MS);
}

// What every generated method is issued through: one `fetch`, pointed at the instance, carrying one
// credential.
//
// The generated half is handed a transport and nothing else — reaching the network is the
// application's business — so this is the piece a consumer writes, and writing it here is what
// makes this smoke the same shape as their code. Waiting out a paced instance belongs here too, so
// that every call site stays a call site: Hook0 paces callers per credential, and a flow driving
// three dozen operations one after another is exactly what that is for.
class Reaching {
  constructor(origin, token) {
    this.origin = origin.replace(/\/+$/, '');
    this.token = token;
  }

  request(asked) {
    // The path is written whole by the generated method, escaping included, so it is carried as it
    // was built rather than parsed apart and put back together.
    const query = asked.query
      .map(([name, value]) => `${encodeURIComponent(name)}=${encodeURIComponent(value)}`)
      .join('&');
    const target = `${this.origin}${asked.path}${query === '' ? '' : `?${query}`}`;

    const headers = { Authorization: `Bearer ${this.token}`, Accept: 'application/json' };
    if (asked.body !== undefined) {
      headers['Content-Type'] = 'application/json';
    }

    const send = (sent) =>
      fetch(target, { method: asked.method, headers, body: asked.body }).then((answer) => {
        if (answer.status === TOO_MANY_REQUESTS && sent < PACED_AGAIN) {
          return waited(pause(answer)).then(() => send(sent + 1));
        }
        return answer.text().then((payload) => ({ status: answer.status, payload }));
      });

    return send(1);
  }
}

// One operation the flow goes on to use the answer of, which therefore has to be a success.
function read(operation, answering) {
  return answering.then(
    (answered) => {
      console.log(`exercised ${operation} accepted`);
      return answered;
    },
    (refused) => {
      throw new Error(
        `${operation}: the flow needs what it answers, and it answered ${refused.message}`
      );
    }
  );
}

// One operation driven for its own sake, reported whichever way the instance answered it.
//
// A success and a problem are both complete round trips through the generated layer: the request
// was composed, the instance answered, and this client read the answer. What is neither — the API
// not reached, a body this client cannot read, a problem it does not know — stops the smoke,
// because none of those say the client and the instance agree on anything.
function exercised(operation, answering) {
  return answering.then(
    () => {
      console.log(`exercised ${operation} accepted`);
    },
    (refused) => {
      if (!(refused instanceof generated.ProblemError) || refused.kind === undefined) {
        throw new Error(`${operation}: ${refused.message}`);
      }
      console.log(`exercised ${operation} refused:${refused.kind}`);
    }
  );
}

// Reports one generated model type as decoded out of a real answer.
//
// The value is taken and handed back rather than only named, so the line cannot outlive what it is
// about: a member that stops being part of an answer stops resolving here.
function decoded(model, value) {
  if (value === undefined) {
    throw new Error(`${model} is not part of what the instance answered`);
  }
  console.log(`decoded ${model}`);
  return value;
}

// The event both control sends carry, under the identifier the caller names.
function event(eventType, eventId) {
  return new Event(
    eventType,
    '{"from":"the typescript smoke"}',
    'application/json',
    { language: LANGUAGE },
    undefined,
    undefined,
    eventId
  );
}

// Verifies what the output worker really delivered, with this client's own verification.
function verify(delivery) {
  const read = (part) => fs.readFileSync(path.join(delivery, part), 'utf8');
  const headers = new Headers();
  for (const line of read('headers').split('\n')) {
    const at = line.indexOf(': ');
    if (at > 0) {
      headers.append(line.slice(0, at), line.slice(at + 2));
    }
  }
  verifyWebhookSignature(
    read('signature').trim(),
    fs.readFileSync(path.join(delivery, 'body')),
    headers,
    read('secret').trim(),
    Number(read('tolerance').trim())
  );
}

// The three questions the loopback suite cannot ask, against the instance that can answer them.
function control(apiUrl, applicationId, token, eventType) {
  const client = new Hook0Client(apiUrl, applicationId, token);

  return client
    .sendEvent(event(eventType, undefined))
    .catch((refused) => {
      throw new Error(
        `the instance refused the first send, with ${apiUrl} as the API URL: ${refused.message}`
      );
    })
    .then((sent) => {
      console.log(`ingested ${sent}`);
      return client.sendEvent(event(eventType, sent)).then(
        () => {
          throw new Error('sending the same event twice was accepted twice');
        },
        (refused) => {
          const said = String(refused && refused.message ? refused.message : refused);
          if (!said.includes(ALREADY_INGESTED)) {
            throw new Error(`the second send failed without naming ${ALREADY_INGESTED}: ${said}`);
          }
          console.log(`the second send reported ${ALREADY_INGESTED}`);
        }
      );
    });
}

// Every operation the API document declares, driven against the instance in the order a consumer
// would: what it needs is created, read and listed, updated, and destroyed last.
//
// Two credentials, because the API takes two and one of them cannot do everything. An application
// secret is scoped to the application it belongs to; what belongs to the organization — listing its
// applications, everything about service tokens, its per-day counts — needs the organization-scoped
// token beside it.
function surface() {
  const origin = originOf(setting('HOOK0_API_URL'));
  const application = setting('HOOK0_APPLICATION_ID');
  const organization = setting('HOOK0_ORGANIZATION_ID');
  const seeded = setting('HOOK0_SEEDED_APPLICATION_ID');
  const attempt = setting('HOOK0_REQUEST_ATTEMPT_ID');
  const response = setting('HOOK0_RESPONSE_ID');

  const held = new Reaching(origin, setting('HOOK0_TOKEN'));
  const organizationWide = new Reaching(origin, setting('HOOK0_SERVICE_TOKEN'));

  const applications = new generated.ApplicationsApi(held);
  const secrets = new generated.ApplicationSecretsApi(held);
  const eventTypes = new generated.EventTypesApi(held);
  const subscriptions = new generated.SubscriptionsApi(held);
  const events = new generated.EventsApi(held);
  const eventsPerDay = new generated.EventsPerDayApi(held);
  const instance = new generated.InstanceApi(held);
  const quotas = new generated.QuotasApi(held);
  const payloadContentTypes = new generated.PayloadContentTypesApi(held);
  const errors = new generated.ErrorsApi(held);

  const organizationApplications = new generated.ApplicationsApi(organizationWide);
  const organizationEventsPerDay = new generated.EventsPerDayApi(organizationWide);
  const requestAttempts = new generated.RequestAttemptsApi(organizationWide);
  const responses = new generated.ResponseApi(organizationWide);
  const serviceTokens = new generated.ServiceTokenApi(organizationWide);

  const labels = { language: LANGUAGE };
  const target = { headers: {}, method: 'POST', type: 'http', url: NOWHERE };

  // What one step of the flow hands the next. A chain of `then`s has no scope that spans it, and
  // this package's rule is chains rather than `await`, so what later steps need is put here as it
  // is read rather than nested twenty callbacks deep.
  const carried = {};

  // What the instance says about itself, which is what an application asks before it has anything
  // of its own: how it is configured, what it will let this account do, what a payload may be, and
  // every problem it can report.
  return read('instance.get', instance.get())
    .then((configured) => decoded('InstanceConfig', configured))
    .then(() => read('quotas.get', quotas.get()))
    .then((allowed) => {
      decoded('QuotasResponseLimits', allowed.limits);
      decoded('QuotasResponse', allowed);
      return exercised('payload_content_types.list', payloadContentTypes.list());
    })
    .then(() => read('errors.list', errors.list()))
    .then((catalogue) => {
      const problem = decoded('Problem', catalogue[0]);
      decoded('ProblemId', problem.id);

      // The application this smoke owns. One per language, so that the three deletions at the end
      // of this flow are real deletions rather than something eleven other smokes live with.
      return read('applications.get', applications.get(application));
    })
    .then((info) => {
      decoded('ApplicationInfoConsumption', info.consumption);
      decoded('ApplicationInfoQuotas', info.quotas);
      decoded('ApplicationInfoOnboardingStepsEvent', info.onboarding_steps.event);
      decoded('ApplicationInfoOnboardingStepsEventType', info.onboarding_steps.event_type);
      decoded('ApplicationInfoOnboardingStepsSubscription', info.onboarding_steps.subscription);
      decoded('ApplicationInfoOnboardingSteps', info.onboarding_steps);
      decoded('ApplicationInfo', info);

      return read(
        'applications.update',
        applications.update(application, {
          name: 'the application the typescript smoke drives',
          organization_id: organization,
        })
      );
    })
    .then((renamed) => {
      decoded('Application', renamed);

      // The organization's, so the organization credential. Listing what an account has is the
      // first thing a console does.
      return exercised('applications.list', organizationApplications.list(organization));
    })
    // This one is driven with the *application* secret on purpose, and it is the flow's one
    // refusal. Creating an application is the organization's business and an application secret is
    // not the organization's, so the instance answers a problem document and this client reads it —
    // which is the half of the client that nothing else here would exercise.
    .then(() =>
      exercised(
        'applications.create',
        applications.create({
          name: "an application the typescript smoke's application secret may not create",
          organization_id: organization,
        })
      )
    )
    // A second secret, so that the one this smoke is authenticating with is never the one it
    // revokes. Deleting that one succeeds and then locks the flow out of everything below.
    .then(() =>
      read(
        'applicationSecrets.create',
        secrets.create({
          application_id: application,
          name: 'a secret the typescript smoke minted',
        })
      )
    )
    .then((minted) => {
      decoded('ApplicationSecret', minted);
      carried.minted = minted.token;
      return exercised('applicationSecrets.list', secrets.list(application));
    })
    .then(() =>
      exercised(
        'applicationSecrets.update',
        secrets.update(carried.minted, {
          application_id: application,
          name: 'a secret the typescript smoke renamed',
        })
      )
    )
    .then(() =>
      exercised('applicationSecrets.delete', secrets.delete(carried.minted, application))
    )
    // An event type of this smoke's own, rather than the one the harness declared: what is created
    // here is what is subscribed to, sent, replayed and deleted below.
    .then(() =>
      read(
        'eventTypes.create',
        eventTypes.create({
          application_id: application,
          resource_type: 'smoke',
          service: LANGUAGE,
          verb: 'ran',
        })
      )
    )
    .then((declared) => {
      decoded('EventType', declared);
      carried.eventType = declared.event_type_name;
      return exercised('eventTypes.get', eventTypes.get(carried.eventType, application));
    })
    .then(() => exercised('eventTypes.list', eventTypes.list(application)))
    .then(() =>
      read(
        'subscriptions.create',
        subscriptions.create({
          application_id: application,
          description: 'what the typescript smoke subscribes to its own events with',
          event_types: [carried.eventType],
          is_enabled: true,
          labels,
          target,
        })
      )
    )
    .then((subscription) => {
      decoded('SubscriptionTarget', subscription.target);
      decoded('Subscription', subscription);
      carried.subscription = subscription.subscription_id;
      return exercised('subscriptions.get', subscriptions.get(carried.subscription));
    })
    .then(() => exercised('subscriptions.list', subscriptions.list(application)))
    .then(() =>
      exercised(
        'subscriptions.update',
        subscriptions.update(carried.subscription, {
          application_id: application,
          description: 'what the typescript smoke renamed it to',
          event_types: [carried.eventType],
          is_enabled: true,
          labels,
          target,
        })
      )
    )
    // The event the subscription above selects, sent through the generated layer rather than
    // through `sendEvent`: the hand-written half has its own three questions above, and this is the
    // operation the document declares.
    .then(() =>
      read(
        'events.ingest',
        events.ingest({
          application_id: application,
          event_id: crypto.randomUUID(),
          event_type: carried.eventType,
          labels,
          occurred_at: new Date().toISOString(),
          payload: '{"from":"the typescript smoke"}',
          payload_content_type: 'application/json',
        })
      )
    )
    .then((ingested) => {
      decoded('IngestedEvent', ingested);
      carried.event = ingested.event_id;
      return read('events.get', events.get(carried.event, application));
    })
    .then((whole) => {
      decoded('EventWithPayload', whole);
      return read('events.list', events.list(application));
    })
    .then((listed) => {
      decoded('Event', listed[0]);
      return exercised('events.replay', events.replay(carried.event, { application_id: application }));
    })
    // This application was created a moment ago and the counts come out of a view the instance
    // refreshes on a cycle of its own, so this answers a list with nothing in it — which is an
    // answer, and one a client has to be able to read.
    .then(() =>
      exercised(
        'events_per_day.list_for_application',
        eventsPerDay.listForApplication(application)
      )
    )
    // The organization's counts do have something in them: the harness waited for the instance to
    // refresh them before running any of this, precisely so that the type they are answered with is
    // one a client decodes rather than one nothing ever produces.
    .then(() =>
      read(
        'events_per_day.list_for_organization',
        organizationEventsPerDay.listForOrganization(organization)
      )
    )
    .then((perDay) => {
      decoded('EventsPerDayEntry', perDay[0]);

      // An attempt and a response exist only once the output worker has finished a delivery. The
      // harness waited for one, in the application it caught the shared delivery from, and handed
      // the ids on — so this reads them back with the organization credential.
      return exercised('requestAttempts.list', requestAttempts.list(seeded));
    })
    .then(() => read('requestAttempts.get', requestAttempts.get(attempt, seeded)))
    .then((attempted) => {
      decoded('RequestAttemptEvent', attempted.event);
      decoded('RequestAttemptSubscription', attempted.subscription);
      decoded('RequestAttemptStatusType', attempted.status.type);
      decoded('RequestAttemptStatus', attempted.status);
      decoded('RequestAttempt', attempted);
      return read('response.get', responses.get(response, seeded));
    })
    .then((answered) => {
      decoded('Response', answered);

      // Service tokens belong to the organization, so they are minted, read and revoked with the
      // organization credential. The one revoked below is the one minted here — never the one this
      // half of the flow is authenticating with.
      return read(
        'serviceToken.create',
        serviceTokens.create({
          name: 'a token the typescript smoke minted',
          organization_id: organization,
        })
      );
    })
    .then((token) => {
      decoded('ServiceToken', token);
      carried.token = token.token_id;
      return exercised('serviceToken.list', serviceTokens.list(organization));
    })
    .then(() => exercised('serviceToken.get', serviceTokens.get(carried.token, organization)))
    .then(() =>
      exercised(
        'serviceToken.update',
        serviceTokens.update(carried.token, {
          name: 'a token the typescript smoke renamed',
          organization_id: organization,
        })
      )
    )
    .then(() => exercised('serviceToken.delete', serviceTokens.delete(carried.token, organization)))
    // Destroyed in the order the instance can accept: the subscription that references the event
    // type, then the event type, then the application — which is last because the secret this whole
    // flow authenticates with stops authenticating the moment its application is gone.
    .then(() =>
      exercised('subscriptions.delete', subscriptions.delete(carried.subscription, application))
    )
    .then(() =>
      exercised('eventTypes.delete', eventTypes.delete(carried.eventType, application))
    )
    .then(() => exercised('applications.delete', applications.delete(application)));
}

control(
  setting('HOOK0_API_URL'),
  setting('HOOK0_APPLICATION_ID'),
  setting('HOOK0_TOKEN'),
  setting('HOOK0_EVENT_TYPE')
)
  .then(() => surface())
  .then(() => {
    // Last, and on purpose: it needs no instance at all, so it still answers after the flow above
    // has deleted the application it was run against.
    verify(setting('HOOK0_DELIVERY'));
    console.log('the signature the instance produced verifies');
  })
  .catch((refused) => {
    console.error(String(refused && refused.stack ? refused.stack : refused));
    process.exitCode = 1;
  });
