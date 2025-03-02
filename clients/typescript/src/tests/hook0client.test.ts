import { describe, expect, test, beforeEach, afterEach, jest } from '@jest/globals';

import { Hook0Client, Event } from '../index';

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

  test('should send an event successfully (201 Created)', async () => {
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

    expect(eventId).toBeDefined();
    expect(eventId).toBe('00000000-0000-0000-0000-000000000000');
    expect(mockFetch).toHaveBeenCalledTimes(1);
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
});
