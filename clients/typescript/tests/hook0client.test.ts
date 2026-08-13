import { describe, expect, test, beforeEach, afterEach } from '@jest/globals';
import * as http from 'http';

import { Hook0Client, Event } from '../src/index';

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

/** A request the API received, in the order it received it. */
interface ReceivedRequest {
  body: string;
}

/** No request here is anywhere near this large; the cap bounds what one connection can buffer. */
const MAX_REQUEST_BODY_BYTES = 64 * 1024;

/** Every case talks to a loopback socket, so none of them has any reason to take this long. */
const TEST_TIMEOUT_MS = 10_000;

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
  private readonly responses: ScriptedResponse[] = [];
  private answered = 0;
  private readonly server: http.Server;

  constructor() {
    this.server = http.createServer((request, response) => {
      readBody(request)
        .then((body) => {
          this.received.push({ body });
          const scripted = this.nextResponse();
          response.writeHead(scripted.status, { 'Content-Type': 'application/json' });
          response.end(JSON.stringify(scripted.body));
        })
        .catch((error: Error) => {
          response.writeHead(500, { 'Content-Type': 'application/json' });
          response.end(JSON.stringify({ error: error.message }));
        });
    });
  }

  /** Queues the answers the case expects the client to draw, in order. */
  willAnswer(...responses: ScriptedResponse[]): void {
    this.responses.push(...responses);
  }

  private nextResponse(): ScriptedResponse {
    if (this.answered >= this.responses.length) {
      return {
        status: 500,
        body: { error: 'The test scripted no answer for this request' },
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
    'should send an event without eventId and return server-generated id (201 Created)',
    async () => {
      const serverGeneratedId = '01961234-5678-7abc-8def-0123456789ab';
      api.willAnswer({
        status: 201,
        body: {
          application_id: 'app-123',
          event_id: serverGeneratedId,
          received_at: new Date().toISOString(),
        },
      });

      const event = new Event(
        'auth.user.create',
        '{"email": "test@example.com"}',
        'application/json',
        { environment: 'production' }
      );

      const eventId = await client.sendEvent(event);

      expect(eventId).toStrictEqual(serverGeneratedId);
      expect(api.received).toHaveLength(1);

      // Verify the request body does not contain event_id
      const requestBody = JSON.parse(api.received[0].body);
      expect(requestBody.event_id).toBeUndefined();
    },
    TEST_TIMEOUT_MS
  );

  test(
    'should fail when too many events are sent (429 Too Many Requests)',
    async () => {
      api.willAnswer({
        status: 429,
        body: {
          error: 'TooManyEventsToday',
          limit: 1000,
        },
      });

      const event = new Event(
        'auth.user.create',
        '{"email": "test@example.com"}',
        'application/json',
        { environment: 'production' }
      );

      await expect(client.sendEvent(event)).rejects.toThrow('Sending event');
      expect(api.received).toHaveLength(1);
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
