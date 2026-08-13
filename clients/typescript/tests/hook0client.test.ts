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
  body: string;
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
      readBody(request)
        .then((body) => {
          this.received.push({ body });
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
});
