import { Event } from '../lib';

describe('Event', () => {
  test('should create an Event instance without optional parameters', () => {
    const event = new Event(
      'auth.user.create',
      '{"email": "test@example.com", "password": "test"}',
      'application/json',
      { production: 'true' }
    );
    expect(event).toBeInstanceOf(Event);
    expect(event.eventType).toBe('auth.user.create');
    expect(event.payload).toBe('{"email": "test@example.com", "password": "test"}');
    expect(event.payloadContentType).toBe('application/json');
    expect(event.labels).toEqual({ production: 'true' });
    expect(event.metadata).toBeUndefined();
    expect(event.occurredAt).toBeUndefined();
    expect(event.eventId).toBeUndefined();
  });

  test('should create an Event instance with optional parameters', () => {
    const event = new Event(
      'auth.user.create',
      '{"email": "test@example.com", "password": "test"}',
      'application/json',
      { production: 'true' },
      { production: 'true' },
      new Date(),
      '00000000-0000-0000-0000-000000000000'
    );
    expect(event).toBeInstanceOf(Event);
    expect(event.eventType).toBe('auth.user.create');
    expect(event.payload).toBe('{"email": "test@example.com", "password": "test"}');
    expect(event.payloadContentType).toBe('application/json');
    expect(event.labels).toEqual({ production: 'true' });
    expect(event.metadata).toEqual({ production: 'true' });
    expect(event.occurredAt).toBeInstanceOf(Date);
    expect(event.eventId).toBe('00000000-0000-0000-0000-000000000000');
  });
});
