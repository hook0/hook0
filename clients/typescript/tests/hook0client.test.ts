import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import * as http from 'http';

import { Hook0Client, Event, Hook0ClientOptions, RetryPolicy } from '../src/index';

/**
 * The client is exercised against a Hook0 API listening on a loopback port, so every case below
 * goes over a real socket: the request the client builds, the headers it sets and the way it reads
 * the answer are all the real ones.
 */

/** What the API answers to one request, in the order the case scripted it. */
interface ScriptedResponse {
  status: number;
  body: unknown;
}

/** The same, plus how long the API sits on it before writing anything. */
interface HeldResponse extends ScriptedResponse {
  heldForMs: number;
}

/** The shape a UUID has, whichever version it carries. */
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/**
 * A retry schedule short enough that a case spends its time on requests rather than on waiting,
 * and whose budget is far above what its delays add up to, so the number of attempts a case
 * observes is the one its policy asked for.
 */
function promptRetries(maxAttempts: number): RetryPolicy {
  return new RetryPolicy(maxAttempts, 5, 5, 1_000);
}

function withRetries(policy: RetryPolicy, requestTimeoutMs = 5_000): Hook0ClientOptions {
  return new Hook0ClientOptions(policy, requestTimeoutMs);
}

function anEvent(): Event {
  return new Event('auth.user.create', '{"email": "test@example.com"}', 'application/json', {
    environment: 'production',
  });
}

function alreadyIngested(): ScriptedResponse {
  return {
    status: 409,
    body: {
      id: 'EventAlreadyIngested',
      title: 'Event already Ingested',
      detail: 'This event was previously ingested and recorded inside Hook0 service.',
      status: 409,
    },
  };
}

function serverError(): ScriptedResponse {
  return { status: 500, body: { id: 'InternalServerError', status: 500 } };
}

function ingested(eventId: string): ScriptedResponse {
  return {
    status: 201,
    body: {
      application_id: 'app-123',
      event_id: eventId,
      received_at: new Date().toISOString(),
    },
  };
}

/** The event ID request number `index` carried, as the API read it. */
function eventIdOf(api: FakeHook0Api, index: number): string {
  const request = api.received[index];
  if (request === undefined) {
    throw new Error(`Expected at least ${index + 1} requests, got ${api.received.length}`);
  }
  const body: unknown = JSON.parse(request.body);
  if (typeof body === 'object' && body !== null && 'event_id' in body) {
    const eventId = (body as { event_id: unknown }).event_id;
    if (typeof eventId === 'string') {
      return eventId;
    }
  }
  throw new Error(`Request ${index} carries no event_id: ${request.body}`);
}

/** A request the API received, in the order it received it. */
interface ReceivedRequest {
  target: string;
  body: string;
  headers: http.IncomingHttpHeaders;
}

/** What a request asked for, which a server always reads off the request line. */
function targetOf(request: http.IncomingMessage): string {
  const target = request.url;
  if (typeof target !== 'string') {
    throw new Error('A request arrived carrying no target');
  }
  return target;
}

/** No request here is anywhere near this large; the cap bounds what one connection can buffer. */
const MAX_REQUEST_BODY_BYTES = 64 * 1024;

/** Every case talks to a loopback socket, so none of them has any reason to take this long. */
const TEST_TIMEOUT_MS = 10_000;

/** Resolves after `delayMs`, so an answer can be withheld from the client for a while. */
function hold(delayMs: number): Promise<void> {
  if (delayMs === 0) {
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    setTimeout(resolve, delayMs);
  });
}

function readBody(request: http.IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    const chunks: Buffer[] = [];
    let size = 0;

    request.on('data', (chunk: Buffer) => {
      size += chunk.length;
      if (size > MAX_REQUEST_BODY_BYTES) {
        request.destroy();
        reject(new Error(`Request body is larger than ${MAX_REQUEST_BODY_BYTES} bytes`));
        return;
      }
      chunks.push(chunk);
    });
    request.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
    request.on('error', reject);
  });
}

/** A Hook0 API listening on a loopback port for the lifetime of one test. */
class FakeHook0Api {
  readonly received: ReceivedRequest[] = [];
  private readonly responses: HeldResponse[] = [];
  private answered = 0;
  private readonly server: http.Server;

  constructor() {
    this.server = http.createServer((request, response) => {
      const target = targetOf(request);
      readBody(request)
        .then((body) => {
          this.received.push({ target, body, headers: request.headers });
          const scripted = this.nextResponse();
          return hold(scripted.heldForMs).then(() => {
            response.writeHead(scripted.status, { 'Content-Type': 'application/json' });
            response.end(JSON.stringify(scripted.body));
          });
        })
        .catch((error: Error) => {
          response.writeHead(500, { 'Content-Type': 'application/json' });
          response.end(JSON.stringify({ error: error.message }));
        });
    });
  }

  /** Queues the answers the case expects the client to draw, in order. */
  willAnswer(...responses: ScriptedResponse[]): void {
    this.responses.push(...responses.map((response) => ({ ...response, heldForMs: 0 })));
  }

  /** Queues one answer the API withholds long enough for a client to give up waiting for it. */
  willAnswerAfter(heldForMs: number, response: ScriptedResponse): void {
    this.responses.push({ ...response, heldForMs });
  }

  private nextResponse(): HeldResponse {
    if (this.answered >= this.responses.length) {
      return {
        status: 500,
        body: { error: 'The test scripted no answer for this request' },
        heldForMs: 0,
      };
    }
    const scripted = this.responses[this.answered];
    this.answered += 1;
    return scripted;
  }

  listen(): Promise<void> {
    return new Promise((resolve) => {
      this.server.listen(0, '127.0.0.1', () => resolve());
    });
  }

  get baseUrl(): string {
    const address = this.server.address();
    if (address === null || typeof address === 'string') {
      throw new Error('The fake Hook0 API is not listening on a TCP port');
    }
    return `http://127.0.0.1:${address.port}`;
  }

  close(): Promise<void> {
    return new Promise((resolve, reject) => {
      // `fetch` keeps its connections alive, and a live connection holds `close` open forever.
      this.server.closeAllConnections();
      this.server.close((error) => {
        if (error === undefined) {
          resolve();
        } else {
          reject(error);
        }
      });
    });
  }
}

describe('Hook0Client', () => {
  let api: FakeHook0Api;
  let client: Hook0Client;

  beforeEach(async () => {
    api = new FakeHook0Api();
    await api.listen();

    client = new Hook0Client(api.baseUrl, 'app-123', 'token-xyz');
  });

  afterEach(async () => {
    await api.close();
  });

  test(
    'should send an event with client-provided eventId (201 Created)',
    async () => {
      api.willAnswer({
        status: 201,
        body: {
          application_id: 'app-123',
          event_id: '00000000-0000-0000-0000-000000000000',
          received_at: new Date().toISOString(),
        },
      });

      const event = new Event(
        'auth.user.create',
        '{"email": "test@example.com"}',
        'application/json',
        { environment: 'production' }
      );
      event.eventId = '00000000-0000-0000-0000-000000000000';

      const eventId = await client.sendEvent(event);

      expect(eventId).toStrictEqual('00000000-0000-0000-0000-000000000000');
      expect(api.received).toHaveLength(1);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'sends an event the caller gave no id under an id it generated itself',
    async () => {
      const ingestedId = '01961234-5678-7abc-8def-0123456789ab';
      api.willAnswer(ingested(ingestedId));

      const eventId = await client.sendEvent(anEvent());

      expect(eventId).toStrictEqual(ingestedId);
      expect(api.received).toHaveLength(1);

      // The request must carry an id: without one, a replayed request makes Hook0 mint a second
      // one, and the event is ingested and delivered twice.
      expect(eventIdOf(api, 0)).toMatch(UUID);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'retries an attempt that ran out of time under the same event id',
    async () => {
      const ingestedId = '01961234-5678-7abc-8def-0123456789ab';
      api.willAnswerAfter(400, ingested(ingestedId));
      api.willAnswer(ingested(ingestedId));

      const patient = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(promptRetries(3), 100)
      );

      const eventId = await patient.sendEvent(anEvent());

      expect(eventId).toStrictEqual(ingestedId);
      expect(api.received).toHaveLength(2);
      // The retry must repeat the id of the attempt it repeats, or Hook0 ingests the event twice.
      expect(eventIdOf(api, 1)).toStrictEqual(eventIdOf(api, 0));
      expect(eventIdOf(api, 0)).toMatch(UUID);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'stops retrying server errors at the configured number of attempts',
    async () => {
      api.willAnswer(serverError(), serverError(), serverError(), serverError());

      const stubborn = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(promptRetries(3))
      );

      await expect(stubborn.sendEvent(anEvent())).rejects.toThrow('gave up after 3 attempts');
      expect(api.received).toHaveLength(3);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'does not retry an answer the API would repeat (429 Too Many Requests)',
    async () => {
      api.willAnswer({
        status: 429,
        body: {
          error: 'TooManyEventsToday',
          limit: 1000,
        },
      });

      const stubborn = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(promptRetries(4))
      );

      // A quota that is exhausted for the day cannot clear itself between two attempts.
      await expect(stubborn.sendEvent(anEvent())).rejects.toThrow('Sending event');
      expect(api.received).toHaveLength(1);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'reports success when a retry is answered that the event was already ingested',
    async () => {
      api.willAnswer(serverError(), alreadyIngested());

      const stubborn = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(promptRetries(3))
      );

      const eventId = await stubborn.sendEvent(anEvent());

      // The conflict is the mark of the attempt this one repeats having reached the API.
      expect(eventId).toStrictEqual(eventIdOf(api, 0));
      expect(api.received).toHaveLength(2);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'reports the conflict when a first attempt is answered that the event was already ingested',
    async () => {
      api.willAnswer(alreadyIngested());

      const stubborn = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(promptRetries(3))
      );

      // Nothing this send did can explain the conflict, so the caller has to hear about it.
      await expect(stubborn.sendEvent(anEvent())).rejects.toThrow('EventAlreadyIngested');
      expect(api.received).toHaveLength(1);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'issues a single request when retrying is switched off',
    async () => {
      api.willAnswer(serverError(), serverError(), serverError());

      const once = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(RetryPolicy.disabled())
      );

      await expect(once.sendEvent(anEvent())).rejects.toThrow('Sending event');
      expect(api.received).toHaveLength(1);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'reaches the same endpoint whether or not the API URL ends in a slash',
    async () => {
      api.willAnswer(
        ingested('01961234-5678-7abc-8def-0123456789ab'),
        ingested('01961234-5678-7abc-8def-0123456789ac')
      );

      // A Hook0 is reached at an API URL carrying the path it is served under — `/api/v1` — and
      // the two spellings of that URL are the same instance. Resolving an endpoint against the
      // one written without the slash is what replaces `v1` with the endpoint and posts to a path
      // no instance serves.
      const written = new Hook0Client(`${api.baseUrl}/api/v1`, 'app-123', 'token-xyz');
      const spelt = new Hook0Client(`${api.baseUrl}/api/v1/`, 'app-123', 'token-xyz');
      await written.sendEvent(anEvent());
      await spelt.sendEvent(anEvent());

      expect(api.received.map((request) => request.target)).toStrictEqual([
        '/api/v1/event',
        '/api/v1/event',
      ]);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'names the status when a refused send gives it nothing else to go on',
    async () => {
      // What a request that missed the path an instance is served under draws: a status, and no
      // body to read a reason out of.
      api.willAnswer({ status: 404, body: undefined });

      await expect(client.sendEvent(anEvent())).rejects.toThrow(
        'the API answered 404 with no body'
      );
      expect(api.received).toHaveLength(1);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'names the problem and what it said when a send is refused with one',
    async () => {
      api.willAnswer({
        status: 403,
        body: {
          id: 'AuthInvalidApplicationSecret',
          title: 'Invalid application secret',
          detail: 'The provided application secret does not exist.',
          status: 403,
        },
      });

      // The identifier is what tells one refusal from another without reading prose.
      await expect(client.sendEvent(anEvent())).rejects.toThrow(
        'the API answered 403 AuthInvalidApplicationSecret: The provided application secret does not exist.'
      );
    },
    TEST_TIMEOUT_MS
  );

  test(
    'refuses a payload above the maximum before any request is issued',
    async () => {
      const maximum = 16;
      api.willAnswer(ingested('01961234-5678-7abc-8def-0123456789ab'));

      const strict = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        new Hook0ClientOptions(new RetryPolicy(), 5_000, maximum)
      );
      const event = new Event('auth.user.create', 'x'.repeat(maximum + 1), 'application/json', {});

      await expect(strict.sendEvent(event)).rejects.toThrow(
        `${maximum} bytes this client sends at most`
      );
      expect(api.received).toHaveLength(0);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'keeps the delays of one send inside the configured budget',
    async () => {
      // Nine retries of up to 300 ms each would run for seconds; the budget below lets 300 ms of
      // them through in total.
      const attempts = 10;
      const budgetMs = 300;
      /** What ten requests to a loopback socket, and the work around them, may cost on top. */
      const roundTripAllowanceMs = 400;

      const budgeted = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(new RetryPolicy(attempts, budgetMs, budgetMs, budgetMs))
      );

      const started = Date.now();
      await expect(budgeted.sendEvent(anEvent())).rejects.toThrow('gave up after');
      const elapsed = Date.now() - started;

      expect(elapsed).toBeLessThan(budgetMs + roundTripAllowanceMs);
      expect(api.received.length).toBeGreaterThanOrEqual(2);
      expect(api.received.length).toBeLessThanOrEqual(attempts);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'should upsert a new event type successfully',
    async () => {
      api.willAnswer(
        { status: 200, body: [] },
        {
          status: 201,
          body: {
            service_name: 'auth',
            resource_type_name: 'user',
            verb_name: 'create',
            event_type_name: 'auth.user.create',
          },
        }
      );

      const eventTypes = ['auth.user.create'];
      const result = await client.upsertEventTypes(eventTypes);

      expect(result).toEqual(['auth.user.create']);
      expect(api.received).toHaveLength(2);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'should upsert 5 new event types successfully',
    async () => {
      api.willAnswer(
        { status: 200, body: [] },
        {
          status: 201,
          body: {
            service_name: 'auth',
            resource_type_name: 'user',
            verb_name: 'create',
            event_type_name: 'auth.user.create',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'auth',
            resource_type_name: 'user',
            verb_name: 'delete',
            event_type_name: 'auth.user.delete',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'billing',
            resource_type_name: 'invoice',
            verb_name: 'paid',
            event_type_name: 'billing.invoice.paid',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'billing',
            resource_type_name: 'invoice',
            verb_name: 'failed',
            event_type_name: 'billing.invoice.failed',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'order',
            resource_type_name: 'product',
            verb_name: 'shipped',
            event_type_name: 'order.product.shipped',
          },
        }
      );

      const eventTypes = [
        'auth.user.create',
        'auth.user.delete',
        'billing.invoice.paid',
        'billing.invoice.failed',
        'order.product.shipped',
      ];

      const result = await client.upsertEventTypes(eventTypes);

      expect(result).toEqual([
        'auth.user.create',
        'auth.user.delete',
        'billing.invoice.paid',
        'billing.invoice.failed',
        'order.product.shipped',
      ]);
      expect(api.received).toHaveLength(6);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'should upsert 3 new event types and ignore 2 existing ones',
    async () => {
      api.willAnswer(
        {
          status: 200,
          body: [{ event_type_name: 'auth.user.create' }, { event_type_name: 'auth.user.delete' }],
        },
        {
          status: 201,
          body: {
            service_name: 'order',
            resource_type_name: 'product',
            verb_name: 'shipped',
            event_type_name: 'order.product.shipped',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'order',
            resource_type_name: 'product',
            verb_name: 'delivered',
            event_type_name: 'order.product.delivered',
          },
        },
        {
          status: 201,
          body: {
            service_name: 'billing',
            resource_type_name: 'invoice',
            verb_name: 'paid',
            event_type_name: 'billing.invoice.paid',
          },
        }
      );

      const eventTypes = [
        'auth.user.create',
        'auth.user.delete',
        'order.product.shipped',
        'order.product.delivered',
        'billing.invoice.paid',
      ];

      const result = await client.upsertEventTypes(eventTypes);

      expect(result).toEqual([
        'order.product.shipped',
        'order.product.delivered',
        'billing.invoice.paid',
      ]);
      expect(api.received).toHaveLength(4);
    },
    TEST_TIMEOUT_MS
  );

  test(
    'should return empty array when upserting an empty list',
    async () => {
      const result = await client.upsertEventTypes([]);

      expect(result).toEqual([]);
      expect(api.received).toHaveLength(0);
    },
    TEST_TIMEOUT_MS
  );

  test.each([
    ['infinite', Infinity],
    ['unreadable', NaN],
    ['past what a whole number reaches', 1e300],
    ['backwards', -1],
  ])(
    'a policy holding a %s delay still states four integers',
    async (_delay, milliseconds) => {
      // A `number` holds delays a whole number of milliseconds does not: infinity and `NaN` write
      // themselves out by name, and anything past the safe range writes itself in exponent form.
      // None of the three is an integer, and a reader cutting the value apart gets a word where a
      // number belongs. What has to hold is that the value stays four integers whatever a caller
      // configured.
      api.willAnswer(ingested('01961234-5678-7abc-8def-0123456789ab'));
      const edges = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        withRetries(new RetryPolicy(1e9, milliseconds, milliseconds, milliseconds))
      );

      await edges.sendEvent(anEvent());

      const stated = api.received[0].headers['hook0-client-options'];
      expect(typeof stated).toBe('string');
      for (const part of String(stated).split(',')) {
        const [name, ...written] = part.split('=');
        expect({
          name,
          written: written.join('='),
          whole: /^\d+$/.test(written.join('=')),
        }).toEqual({ name, written: written.join('='), whole: true });
      }
    },
    TEST_TIMEOUT_MS
  );

  const NON_FINITE: [string, number][] = [
    ['infinite', Infinity],
    ['negatively infinite', -Infinity],
    ['unreadable', NaN],
  ];
  const DURATIONS: [string, (policy: RetryPolicy, held: number) => RetryPolicy][] = [
    [
      'initialBackoffMs',
      (p, held) => new RetryPolicy(p.maxAttempts, held, p.maxBackoffMs, p.maxTotalDelayMs),
    ],
    [
      'maxBackoffMs',
      (p, held) => new RetryPolicy(p.maxAttempts, p.initialBackoffMs, held, p.maxTotalDelayMs),
    ],
    [
      'maxTotalDelayMs',
      (p, held) => new RetryPolicy(p.maxAttempts, p.initialBackoffMs, p.maxBackoffMs, held),
    ],
  ];

  const cases = DURATIONS.flatMap(([duration, holding]) =>
    NON_FINITE.map(([shape, value]): [string, string, (p: RetryPolicy) => RetryPolicy, number] => [
      duration,
      shape,
      (p: RetryPolicy) => holding(p, value),
      value,
    ])
  );

  test.each(cases)(
    'a %s that is %s is read as that field default',
    async (duration, _shape, holding, value) => {
      // A duration that is not a number says nothing about how long to wait, so the default stands.
      // Reading it as zero would delete the spacing between attempts and turn a broken policy into
      // a burst; reading it as unbounded is what makes a client wait for ever. Both halves are held
      // here: what the header states, read off the socket, and what the client would actually wait.
      // A header stating a schedule the client does not keep is worse than no header — this client
      // used to hand `NaN` to its timer, which fires in a millisecond, while stating a real delay.
      const defaults = new RetryPolicy();
      const policy = holding(defaults);
      api.willAnswer(ingested('01961234-5678-7abc-8def-0123456789ab'));

      const client = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        new Hook0ClientOptions(policy)
      );
      await client.sendEvent(anEvent());

      const stated = api.received[0].headers['hook0-client-options'];
      const expected =
        `attempts=${defaults.attempts()},` +
        `backoff=${defaults.initialBackoffMs},` +
        `ceiling=${defaults.maxBackoffMs},` +
        `budget=${defaults.maxTotalDelayMs}`;
      expect({ duration, value: String(value), stated }).toEqual({
        duration,
        value: String(value),
        stated: expected,
      });

      // The schedule the client would keep, at both ends of the draw range and inside it. This is
      // the half a header-only case would miss: before this rule the delays came back `[NaN, NaN,
      // NaN]`, which `setTimeout` fires in a millisecond, so the client stated a wait it skipped.
      for (const draws of [
        [1, 1, 1],
        [0, 0, 0],
        [0.5, 0.25, 0.75],
      ]) {
        expect({ duration, draws, kept: policy.delaysMs(draws) }).toEqual({
          duration,
          draws,
          kept: defaults.delaysMs(draws),
        });
      }
    },
    TEST_TIMEOUT_MS
  );

  test(
    'a wait the policy schedules is actually waited out',
    async () => {
      // The delay of one retry is drawn anywhere between zero and its ceiling, so no single wait
      // has a floor worth asserting. What does have one is the ceiling the client hands its timer:
      // a client that skips its waits reaches the second attempt at once, whatever it drew.
      const defaults = new RetryPolicy();
      const whole = defaults.delaysMs([1, 1, 1])[0];
      api.willAnswer(serverError(), ingested('01961234-5678-7abc-8def-0123456789ab'));

      const client = new Hook0Client(
        api.baseUrl,
        'app-123',
        'token-xyz',
        false,
        // A budget that is not a number: the field falls back to its default, so the retry below
        // is scheduled and paid for rather than skipped.
        new Hook0ClientOptions(new RetryPolicy(2, whole, whole, NaN))
      );

      const started = Date.now();
      await client.sendEvent(anEvent());
      const took = Date.now() - started;

      expect({ requests: api.received.length, within: took <= whole + 2_000 }).toEqual({
        requests: 2,
        within: true,
      });
    },
    TEST_TIMEOUT_MS
  );
});
