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
  // from it. The document describes a label differently on each side, so this is also what that
  // difference costs a caller. Reading gives a free-form object, because a stored row is only held
  // to being an object and older rows were written when a value could be any JSON at all; sending
  // takes a record of string to string, because that is what the ingestion endpoint has accepted
  // since it was tightened. A forwarder therefore converts, and the conversion is the thing that
  // can fail on an event whose labels the ingestion endpoint did not write.
  const read: generated.Event = {
    event_id: '00000000-0000-0000-0000-000000000000',
    event_type_name: 'service.resource.verb',
    ip: '127.0.0.1',
    labels: { environment: 'test', tenant: 'acme' },
    occurred_at: '2026-01-01T00:00:00Z',
    payload_content_type: 'application/json',
    received_at: '2026-01-01T00:00:00Z',
  };

  /** What a forwarder has to do with what it read, since the two sides are typed differently. */
  const carried = (labels: unknown): Record<string, string> => {
    if (typeof labels !== 'object' || labels === null) {
      throw new Error(`labels read back as ${typeof labels}, which cannot be sent on`);
    }
    for (const [name, value] of Object.entries(labels)) {
      if (typeof value !== 'string') {
        throw new Error(
          `the label ${name} is a ${typeof value}, which the API refuses on the way in`
        );
      }
    }
    return labels as Record<string, string>;
  };

  test('labels read off an event fit the event posted back', () => {
    const posted: generated.EventPost = {
      application_id: '00000000-0000-0000-0000-000000000000',
      event_type: read.event_type_name,
      labels: carried(read.labels),
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
      carried(read.labels)
    );

    expect(forwarded.labels).toEqual(carried(read.labels));
  });
});
