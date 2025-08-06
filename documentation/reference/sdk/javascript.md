# JavaScript/TypeScript SDK

The official Hook0 SDK for JavaScript and TypeScript applications, providing a type-safe and idiomatic interface to the Hook0 API.

## Installation

```bash
npm install @hook0/sdk
# or
yarn add @hook0/sdk
# or
pnpm add @hook0/sdk
```

## Quick Start

```typescript
import { Hook0 } from '@hook0/sdk';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  'app_1234567890', // Your application ID
  'biscuit:YOUR_TOKEN_HERE'
);

// Send an event
const event = new Event(
  'user.created',
  JSON.stringify({
    user_id: 'user_123',
    email: 'john.doe@example.com'
  }),
  'application/json',
  { environment: 'production' }
);

const eventId = await hook0.sendEvent(event);
```

## Configuration

### Client Initialization

```typescript
import { Hook0Client } from '@hook0/sdk';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',     // API URL
  'app_1234567890',            // Your application ID
  'biscuit:YOUR_TOKEN_HERE',   // Authentication token
  false                        // Debug mode (optional)
);
```

### Environment Variables

:::note Environment Variable Configuration
The current TypeScript SDK implementation requires explicit configuration and does not automatically read from environment variables.
:::

## Core Features

### Event Management

#### Send Single Event

```typescript
import { Hook0Client, Event } from '@hook0/sdk';

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  'app_1234567890',
  'biscuit:YOUR_TOKEN_HERE'
);

const event = new Event(
  'order.placed',
  JSON.stringify({
    order_id: 'ord_123',
    customer_id: 'cust_456',
    total: 99.99,
    items: [
      { product_id: 'prod_789', quantity: 2 }
    ]
  }),
  'application/json',
  {
    environment: 'production',
    region: 'us-west'
  }
);

const eventId = await hook0.sendEvent(event);
console.log('Event ID:', eventId);
```

:::note Batch Events Not Available
The batch events functionality is not currently implemented. Please send events individually using the single event method above.
:::

:::note Event Query Not Available
The event listing and querying functionality is not available in the current SDK implementation. Use the REST API directly for these operations.
:::

### Event Type Management

```typescript
// Upsert event types (creates if not exists)
const addedEventTypes = await hook0.upsertEventTypes([
  'user.created',
  'user.updated',
  'order.placed'
]);

console.log('Added event types:', addedEventTypes);
```

:::note Limited Management Features
The current SDK implementation provides basic event sending and event type management. For full application and subscription management, please use the REST API directly.
:::

## Advanced Features

### Webhook Verification

```typescript
import { verifyWebhookSignature } from '@hook0/sdk';
import express from 'express';

const app = express();

app.post('/webhook', express.raw({ type: 'application/json' }), (req, res) => {
  const signature = req.headers['hook0-signature'] as string;
  const secret = process.env.WEBHOOK_SECRET!;
  
  try {
    // Verify the signature with headers
    const headers = new Headers();
    Object.entries(req.headers).forEach(([key, value]) => {
      if (typeof value === 'string') {
        headers.set(key, value);
      }
    });
    
    const isValid = verifyWebhookSignature(
      signature,
      req.body,
      headers,
      secret,
      300 // 5-minute tolerance
    );
    
    if (!isValid) {
      return res.status(401).json({ error: 'Invalid signature' });
    }
    
    // Parse and process the webhook
    const payload = JSON.parse(req.body.toString());
    console.log('Webhook received:', payload);
    
    // Process the webhook
    processWebhook(payload);
    
    res.json({ status: 'processed' });
  } catch (error) {
    console.error('Webhook processing error:', error);
    res.status(500).json({ error: 'Processing failed' });
  }
});
```

### Error Handling

```typescript
import { Hook0ClientError } from '@hook0/sdk';

try {
  const event = new Event(
    'user.created',
    JSON.stringify({ user_id: 'user_123' }),
    'application/json',
    { source: 'api' }
  );
  
  const eventId = await hook0.sendEvent(event);
} catch (error) {
  if (error instanceof Hook0ClientError) {
    console.error('Hook0 error:', error.message);
    
    // Handle specific error types
    if (error.message.includes('Invalid event type')) {
      console.error('Event type format is invalid');
    } else if (error.message.includes('Sending event') && error.message.includes('failed')) {
      console.error('Failed to send event, retry later');
    }
  } else {
    console.error('Unexpected error:', error);
  }
}
```

:::note Advanced Features Not Available
Middleware system and event streaming are not available in the current SDK implementation. These features may be added in future versions.
:::

## TypeScript Support

The SDK is written in TypeScript and provides type definitions:

```typescript
import { Hook0Client, Event, EventType, Hook0ClientError } from '@hook0/sdk';

// Type-safe event creation
const event = new Event(
  'user.created',
  JSON.stringify({
    user_id: 'user_123',
    email: 'john@example.com'
  }),
  'application/json',
  { source: 'api' }
);

// EventType helper for parsing event types
const eventType = EventType.fromString('auth.user.create');
if (eventType instanceof Hook0ClientError) {
  console.error('Invalid event type format');
} else {
  console.log(`Service: ${eventType.service}`);
  console.log(`Resource: ${eventType.resourceType}`);
  console.log(`Verb: ${eventType.verb}`);
}
```

## Testing

### Testing

```typescript
import { Hook0Client, Event } from '@hook0/sdk';
import { jest } from '@jest/globals';

describe('Event Handler', () => {
  test('should send user created event', async () => {
    // Mock the fetch function
    global.fetch = jest.fn().mockResolvedValueOnce({
      ok: true,
      text: async () => '',
    });
    
    const client = new Hook0Client(
      'https://app.hook0.com/api/v1',
      'app_test',
      'biscuit:test_token'
    );
    
    const event = new Event(
      'user.created',
      JSON.stringify({ email: 'test@example.com' }),
      'application/json',
      {}
    );
    
    const eventId = await client.sendEvent(event);
    
    // Verify fetch was called correctly
    expect(fetch).toHaveBeenCalledWith(
      'https://app.hook0.com/api/v1/event',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          'Authorization': 'Bearer biscuit:test_token',
          'Content-Type': 'application/json'
        })
      })
    );
  });
});
```

## Best Practices

### 1. Use Environment Variables

```typescript
// Bad
const hook0 = new Hook0({
  token: 'biscuit:hardcoded_token_here'
});

// Good
const hook0 = new Hook0({
  token: process.env.HOOK0_TOKEN!
});
```

### 2. Implement Proper Error Handling

```typescript
// Bad
await hook0.events.send(event);

// Good
try {
  await hook0.events.send(event);
} catch (error) {
  if (error instanceof Hook0Error) {
    logger.error('Failed to send event', {
      code: error.code,
      message: error.message,
      event
    });
    // Implement retry or fallback logic
  }
  throw error;
}
```

### 3. Efficient Event Sending

```typescript
// When sending multiple events, consider using Promise.all for parallelization
const eventPromises = users.map(user => {
  const event = new Event(
    'user.created',
    JSON.stringify(user),
    'application/json',
    { source: 'bulk_import' }
  );
  return hook0.sendEvent(event);
});

const eventIds = await Promise.all(eventPromises);
console.log(`Sent ${eventIds.length} events`);
```

### 4. Use Unique Event IDs

```typescript
// Provide your own event ID for idempotency
const event = new Event(
  'payment.processed',
  JSON.stringify({ amount: 100.00 }),
  'application/json',
  { transaction_id },
  undefined, // metadata
  new Date(), // occurredAt
  `payment-${transaction_id}` // Your own event ID
);

const eventId = await hook0.sendEvent(event);
```

## Troubleshooting

### Common Issues

**Authentication Errors**
```typescript
// Ensure token includes 'biscuit:' prefix
const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  'app_1234567890',
  'biscuit:YOUR_TOKEN_HERE' // ✓ Correct
  // 'YOUR_TOKEN_HERE' // ✗ Wrong - missing 'biscuit:' prefix
);
```

**CORS Issues in Browser**
```typescript
// The SDK uses fetch() which handles CORS automatically
// Ensure your Hook0 application is configured to accept
// requests from your domain
```

**Network Errors**
```typescript
// Implement retry logic for network failures
async function sendEventWithRetry(client: Hook0Client, event: Event, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await client.sendEvent(event);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * Math.pow(2, i)));
    }
  }
}
```

## Support

- **Documentation**: [Hook0 API Docs](https://app.hook0.com/api/v1/docs)
- **GitHub Issues**: [Report Issues](https://github.com/hook0/hook0/issues)
- **Discord**: [Join Community](https://www.hook0.com/community)
- **NPM Package**: @hook0/sdk