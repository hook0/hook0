import { describe, expect, test, beforeEach, afterEach, jest } from '@jest/globals';

import { Hook0Client, Event, paginatedFetch } from '../index';

function makeFakeFetchResponse(res: Partial<Response>): Response {
  return Object.assign(
    {},
    {
      headers: new Headers(),
      ok: false,
      redirected: false,
      status: 0,
      statusText: '',
      type: 'basic',
      url: '',
      clone: function (): Response {
        throw new Error('Function not implemented.');
      },
      body: null,
      bodyUsed: false,
      arrayBuffer: function (): Promise<ArrayBuffer> {
        throw new Error('Function not implemented.');
      },
      blob: function (): Promise<Blob> {
        throw new Error('Function not implemented.');
      },
      bytes: function (): Promise<Uint8Array> {
        throw new Error('Function not implemented.');
      },
      formData: function (): Promise<FormData> {
        throw new Error('Function not implemented.');
      },
      json: function (): Promise<unknown> {
        throw new Error('Function not implemented.');
      },
      text: function (): Promise<string> {
        throw new Error('Function not implemented.');
      },
    },
    res
  );
}

describe('Hook0Client', () => {
  let client: Hook0Client;
  let mockFetch: jest.Mock<typeof global.fetch>;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;

    client = new Hook0Client('https://api.example.com', 'app-123', 'token-xyz');
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  test('should send an event with client-provided eventId (201 Created)', async () => {
    mockFetch.mockResolvedValue(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          application_id: 'app-123',
          event_id: '00000000-0000-0000-0000-000000000000',
          received_at: new Date().toISOString(),
        }),
      })
    );

    const event = new Event(
      'auth.user.create',
      '{"email": "test@example.com"}',
      'application/json',
      { environment: 'production' }
    );
    event.eventId = '00000000-0000-0000-0000-000000000000';

    const eventId = await client.sendEvent(event);

    expect(eventId).toStrictEqual('00000000-0000-0000-0000-000000000000');
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  test('should send an event without eventId and return server-generated id (201 Created)', async () => {
    const serverGeneratedId = '01961234-5678-7abc-8def-0123456789ab';
    mockFetch.mockResolvedValue(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          application_id: 'app-123',
          event_id: serverGeneratedId,
          received_at: new Date().toISOString(),
        }),
      })
    );

    const event = new Event(
      'auth.user.create',
      '{"email": "test@example.com"}',
      'application/json',
      { environment: 'production' }
    );

    const eventId = await client.sendEvent(event);

    expect(eventId).toStrictEqual(serverGeneratedId);
    expect(mockFetch).toHaveBeenCalledTimes(1);

    // Verify the request body does not contain event_id
    const callArgs = mockFetch.mock.calls[0] as [string, RequestInit];
    const requestBody = JSON.parse(callArgs[1].body as string);
    expect(requestBody.event_id).toBeUndefined();
  });

  test('should fail when too many events are sent (429 Too Many Requests)', async () => {
    mockFetch.mockResolvedValue(
      makeFakeFetchResponse({
        ok: false,
        status: 429,
        json: async () => ({
          error: 'TooManyEventsToday',
          limit: 1000,
        }),
      })
    );

    const event = new Event(
      'auth.user.create',
      '{"email": "test@example.com"}',
      'application/json',
      { environment: 'production' }
    );

    await expect(client.sendEvent(event)).rejects.toThrow('Sending event');
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  test('should upsert a new event type successfully', async () => {
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => [],
      })
    );

    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'auth',
          resource_type_name: 'user',
          verb_name: 'create',
          event_type_name: 'auth.user.create',
        }),
      })
    );

    const eventTypes = ['auth.user.create'];
    const result = await client.upsertEventTypes(eventTypes);

    expect(result).toEqual(['auth.user.create']);
    expect(mockFetch).toHaveBeenCalledTimes(2);
  });

  test('should upsert 5 new event types successfully', async () => {
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => [],
      })
    );

    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'auth',
          resource_type_name: 'user',
          verb_name: 'create',
          event_type_name: 'auth.user.create',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'auth',
          resource_type_name: 'user',
          verb_name: 'delete',
          event_type_name: 'auth.user.delete',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'billing',
          resource_type_name: 'invoice',
          verb_name: 'paid',
          event_type_name: 'billing.invoice.paid',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'billing',
          resource_type_name: 'invoice',
          verb_name: 'failed',
          event_type_name: 'billing.invoice.failed',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'order',
          resource_type_name: 'product',
          verb_name: 'shipped',
          event_type_name: 'order.product.shipped',
        }),
      })
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
    expect(mockFetch).toHaveBeenCalledTimes(6);
  });

  test('should upsert 3 new event types and ignore 2 existing ones', async () => {
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => [
          { event_type_name: 'auth.user.create' },
          { event_type_name: 'auth.user.delete' },
        ],
      })
    );

    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'order',
          resource_type_name: 'product',
          verb_name: 'shipped',
          event_type_name: 'order.product.shipped',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'order',
          resource_type_name: 'product',
          verb_name: 'delivered',
          event_type_name: 'order.product.delivered',
        }),
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        json: async () => ({
          service_name: 'billing',
          resource_type_name: 'invoice',
          verb_name: 'paid',
          event_type_name: 'billing.invoice.paid',
        }),
      })
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
    expect(mockFetch).toHaveBeenCalledTimes(4);
  });

  test('should return empty array when upserting an empty list', async () => {
    const result = await client.upsertEventTypes([]);

    expect(result).toEqual([]);
    expect(mockFetch).not.toHaveBeenCalled();
  });

  // paginatedFetch must follow `Link: rel="next"` until exhausted.
  test('paginated_fetch_follows_link_until_done', async () => {
    const baseUrl = 'https://api.example.com/event_types?application_id=app-123';
    const cursor1 = 'CURSOR_PAGE_2';
    const cursor2 = 'CURSOR_PAGE_3';
    const url2 = `${baseUrl}&pagination_cursor=${cursor1}&limit=100`;
    const url3 = `${baseUrl}&pagination_cursor=${cursor2}&limit=100`;

    const page1: Array<{ event_type_name: string }> = Array.from({ length: 100 }, (_, i) => ({
      event_type_name: `svc.res.v${i}`,
    }));
    const page2: Array<{ event_type_name: string }> = Array.from({ length: 100 }, (_, i) => ({
      event_type_name: `svc.res.v${100 + i}`,
    }));
    const page3: Array<{ event_type_name: string }> = Array.from({ length: 50 }, (_, i) => ({
      event_type_name: `svc.res.v${200 + i}`,
    }));

    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        headers: new Headers({ Link: `<${url2}>; rel="next"` }),
        json: async () => page1,
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        headers: new Headers({
          Link: `<${url3}>; rel="next", <${baseUrl}>; rel="prev"`,
        }),
        json: async () => page2,
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        // Last page: only `prev`, no `next` → loop stops.
        headers: new Headers({ Link: `<${url2}>; rel="prev"` }),
        json: async () => page3,
      })
    );

    const result = await paginatedFetch<{ event_type_name: string }>(baseUrl, {
      Authorization: 'Bearer token-xyz',
    });

    expect(result).toHaveLength(250);
    expect(mockFetch).toHaveBeenCalledTimes(3);

    // Verify the cursor chain was actually followed (URLs from the Link headers,
    // not just the initial URL repeated).
    const calledUrls = mockFetch.mock.calls.map((call) => (call as [string, RequestInit])[0]);
    expect(calledUrls).toEqual([baseUrl, url2, url3]);

    // No row dropped, no row duplicated.
    const names = result.map((et) => et.event_type_name);
    expect(new Set(names).size).toBe(250);
    expect(names[0]).toBe('svc.res.v0');
    expect(names[249]).toBe('svc.res.v249');
  });

  // upsertEventTypes wires paginatedFetch and sees ALL 250 server-side event
  // types; nothing is silently dropped, and no duplicate POST is issued for
  // an existing event type that lived on page 2 or 3.
  test('upsertEventTypes_with_250_items_fetches_all', async () => {
    const baseUrl = 'https://api.example.com/event_types?application_id=app-123';
    const url2 = `${baseUrl}&pagination_cursor=PAGE2&limit=100`;
    const url3 = `${baseUrl}&pagination_cursor=PAGE3&limit=100`;

    // Server already has 250 event types spread across 3 pages of 100/100/50.
    // The "duplicate" we want to avoid re-creating lives on page 3.
    const page1 = Array.from({ length: 100 }, (_, i) => ({
      event_type_name: `svc.res.v${i}`,
    }));
    const page2 = Array.from({ length: 100 }, (_, i) => ({
      event_type_name: `svc.res.v${100 + i}`,
    }));
    const page3 = [
      ...Array.from({ length: 49 }, (_, i) => ({
        event_type_name: `svc.res.v${200 + i}`,
      })),
      // The event type that the caller will try to "create" — it already exists,
      // but on page 3, so a non-paginating SDK would silently re-POST it.
      { event_type_name: 'auth.user.create' },
    ];

    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        headers: new Headers({ Link: `<${url2}>; rel="next"` }),
        json: async () => page1,
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        headers: new Headers({ Link: `<${url3}>; rel="next"` }),
        json: async () => page2,
      })
    );
    mockFetch.mockResolvedValueOnce(
      makeFakeFetchResponse({
        ok: true,
        // Last page: no `Link: rel="next"` — loop terminates.
        headers: new Headers(),
        json: async () => page3,
      })
    );

    // Caller asks to upsert one event type that already exists on page 3.
    // Expectation: zero POSTs (no row created), and no extra HTTP call beyond
    // the 3 GET pages.
    const result = await client.upsertEventTypes(['auth.user.create']);

    expect(result).toEqual([]);
    expect(mockFetch).toHaveBeenCalledTimes(3);

    const calls = mockFetch.mock.calls.map((call) => {
      const [url, init] = call as [string, RequestInit];
      return { url, method: init.method ?? 'GET' };
    });
    // All three calls were GETs walking the cursor chain — no spurious POST.
    expect(calls.every((c) => c.method === 'GET')).toBe(true);
  });
});
