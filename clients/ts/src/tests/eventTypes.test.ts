import { EventType, Hook0ClientError } from '../lib';

describe('EventType', () => {
  it('should create an EventType instance', () => {
    const eventType = new EventType('billing', 'invoice', 'paid');
    expect(eventType).toBeInstanceOf(EventType);
  });

  it('should create an EventType instance from a string', () => {
    const eventType = EventType.fromString('auth.user.create');
    expect(eventType).toBeInstanceOf(EventType);
  });

  it('should return an error for an invalid event type string', () => {
    const eventType = EventType.fromString('an_invalid_event.type');
    expect(eventType).toBeInstanceOf(Hook0ClientError);
  });
});
