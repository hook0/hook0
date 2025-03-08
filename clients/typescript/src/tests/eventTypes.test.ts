import { describe, expect, it } from '@jest/globals';

import { EventType, Hook0ClientError } from '../index';

describe('EventType', () => {
  it('should create an EventType instance', () => {
    const eventType = new EventType('billing', 'invoice', 'paid');
    expect(eventType).toBeInstanceOf(EventType);
    expect(eventType.service).toStrictEqual('billing');
    expect(eventType.resourceType).toStrictEqual('invoice');
    expect(eventType.verb).toStrictEqual('paid');
  });

  it('should create an EventType instance from a string', () => {
    const eventType = EventType.fromString('auth.user.create');
    expect(eventType).toBeInstanceOf(EventType);
    if (eventType instanceof EventType) {
      expect(eventType.service).toStrictEqual('auth');
      expect(eventType.resourceType).toStrictEqual('user');
      expect(eventType.verb).toStrictEqual('create');
    }
  });

  it('should return an error for an invalid event type string', () => {
    const eventType = EventType.fromString('an_invalid_event.type');
    expect(eventType).toBeInstanceOf(Hook0ClientError);
  });
});
