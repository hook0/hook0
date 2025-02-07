# Hook0 TypeScript/JavaScript Client

This is the TypeScript/JavaScript SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.

## Getting Started

To add the Hook0 client in your TS/JS project, install it via npm:

```bash
npm install hook0-client
```

## Usage Examples

### Initializing the Hook0Client
```typescript
import { Hook0Client } from 'hook0-client';

const client = new Hook0Client(
  'https://api.hook0.com', // API URL
  'application-id', // Application ID
  'token-auth', // Authentication Token
  true // Debug mode (optional)
);
```

### Creating an EventType Normally and From String
```typescript
import { EventType } from 'hook0-client';

const eventType = new EventType('billing', 'invoice', 'paid');
console.log(eventType);

try {
  const eventTypeFromString = EventType.fromString('auth.user.create');
  console.log(eventTypeFromString);
} catch (error) {
  console.error('Failed to create EventType from string:', error);
}
```

### Upserting Event Types
```typescript
try {
  const eventTypes = ['auth.user.create', 'billing.invoice.paid'];
  const addedTypes = await client.upsertEventTypes(eventTypes);
  console.log('Upserted event types:', addedTypes);
} catch (error) {
  console.error('Failed to upsert event types:', error);
}
```

### Sending an Event with Error Handling
```typescript
import { Event } from 'hook0-client';

const event = new Event(
  'billing.invoice.paid',
  '{"user_id": "00000000-0000-0000-0000-000000000000", "amount": 100}',
  'application/json',
  { production: 'true' }
);

try {
  const eventId = await client.sendEvent(event);
  console.log('Event sent successfully with ID:', eventId);
} catch (error) {
  console.error('Failed to send event:', error);
}
```

### Verifying Webhook Signature with Current Time
```typescript
import { verifyWebhookSignatureWithCurrentTime } from 'hook0-client';

const signature = 't=1636936200,v0=abc';
const payload = Buffer.from('hello !');
const secret = 'my_secret';
const currentTime = Date.now();
const tolerance = 300;

try {
  const isValid = verifyWebhookSignatureWithCurrentTime(signature, payload, secret, tolerance, currentTime);
  console.log('Webhook signature valid:', isValid);
} catch (error) {
  console.error('Webhook signature verification failed:', error);
}
```

### Verifying Webhook Signature
```typescript
import { verifyWebhookSignature } from 'hook0-client';

const signature = 't=1636936200,v0=abc';
const payload = Buffer.from('hello !');
const secret = 'my_secret';
const tolerance = 300;

try {
  const isValid = verifyWebhookSignature(signature, payload, secret, tolerance);
  console.log('Webhook signature valid:', isValid);
} catch (error) {
  console.error('Webhook signature verification failed:', error);
}
```

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)
- Thomas Tartrau - [thomas@tartrau.fr](mailto:thomas@tartrau.fr)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com).

