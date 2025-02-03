import { Event } from '../lib';

describe('Event', () => {
  test('should create an Event instance without optional parameters', () => {
    const event = new Event(
      'billing.invoice.paid',
      '{"user_id": "00000000-0000-0000-0000-000000000000", "amount": 100}',
      'application/json',
      { production: 'true' }
    );
    expect(event).toBeInstanceOf(Event);
    expect(event.eventType).toBe('billing.invoice.paid');
    expect(event.payload).toBe(
      '{"user_id": "00000000-0000-0000-0000-000000000000", "amount": 100}'
    );
    expect(event.payloadContentType).toBe('application/json');
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
    expect(event.eventType).toBe('auth.user.create');
    expect(event.payload).toBe(
      '{"user_id": "00000000-0000-0000-0000-000000000000", "email": "test@example.com"}'
    );
    expect(event.payloadContentType).toBe('application/json');
    expect(event.labels).toEqual({ production: 'true' });
    expect(event.metadata).toEqual({ production: 'true' });
    expect(event.occurredAt).toBeInstanceOf(Date);
    expect(event.eventId).toBe('00000000-0000-0000-0000-000000000000');
  });
});
