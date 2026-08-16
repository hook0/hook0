import { describe, expect, test } from '@jest/globals';

import { Event, generated } from '../src/index';

describe('Event', () => {
  test('should create an Event instance without optional parameters', () => {
    const event = new Event(
      'billing.invoice.paid',
      '{"user_id": "00000000-0000-0000-0000-000000000000", "amount": 100}',
      'application/json',
      { production: 'true' }
    );
    expect(event).toBeInstanceOf(Event);
    expect(event.eventType).toStrictEqual('billing.invoice.paid');
    expect(event.payload).toStrictEqual(
      '{"user_id": "00000000-0000-0000-0000-000000000000", "amount": 100}'
    );
    expect(event.payloadContentType).toStrictEqual('application/json');
    expect(event.labels).toEqual({ production: 'true' });
    expect(event.metadata).toBeUndefined();
    expect(event.occurredAt).toBeUndefined();
    expect(event.eventId).toBeUndefined();
  });

  test('should create an Event instance with optional parameters', () => {
    const event = new Event(
      'auth.user.create',
      '{"user_id": "00000000-0000-0000-0000-000000000000", "email": "test@example.com"}',
      'application/json',
      { production: 'true' },
      { production: 'true' },
      new Date(),
      '00000000-0000-0000-0000-000000000000'
    );
    expect(event).toBeInstanceOf(Event);
    expect(event.eventType).toStrictEqual('auth.user.create');
    expect(event.payload).toStrictEqual(
      '{"user_id": "00000000-0000-0000-0000-000000000000", "email": "test@example.com"}'
    );
    expect(event.payloadContentType).toStrictEqual('application/json');
    expect(event.labels).toEqual({ production: 'true' });
    expect(event.metadata).toEqual({ production: 'true' });
    expect(event.occurredAt).toBeInstanceOf(Date);
    expect(event.eventId).toStrictEqual('00000000-0000-0000-0000-000000000000');
  });
});

describe('Event labels round trip', () => {
  // The shape every forwarder, replayer and migration script has: read an event, build the next one
  // from it. It only works if both sides of the document agree on what a label is, so nothing below
  // casts, re-parses, or reaches for `as` — the record the read model hands over is the record the
  // write models take. Were the read side to go back to being free-form, this stops type-checking,
  // which is the failure worth having: a caller finds out at build time rather than by casting
  // until the compiler relents.
  const read: generated.Event = {
    event_id: '00000000-0000-0000-0000-000000000000',
    event_type_name: 'service.resource.verb',
    ip: '127.0.0.1',
    labels: { environment: 'test', tenant: 'acme' },
    occurred_at: '2026-01-01T00:00:00Z',
    payload_content_type: 'application/json',
    received_at: '2026-01-01T00:00:00Z',
  };

  test('labels read off an event fit the event posted back', () => {
    const posted: generated.EventPost = {
      application_id: '00000000-0000-0000-0000-000000000000',
      event_type: read.event_type_name,
      labels: read.labels,
      occurred_at: read.occurred_at,
      payload: '{"hello":"world"}',
      payload_content_type: read.payload_content_type,
    };

    expect(posted.labels).toEqual({ environment: 'test', tenant: 'acme' });
  });

  test('labels read off an event fit the event an emitter sends', () => {
    const forwarded = new Event(
      read.event_type_name,
      '{"hello":"world"}',
      read.payload_content_type,
      read.labels
    );

    expect(forwarded.labels).toEqual(read.labels);
  });
});
