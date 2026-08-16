---
title: "JavaScript & TypeScript webhook SDK — hook0-client"
description: "Send Hook0 events and verify webhook signatures from Node.js. Typed, ESM and CommonJS, idempotent event IDs, retries and payload bounds built in."
keywords: [JavaScript webhook SDK, TypeScript webhook client, hook0-client npm, verify webhook signature Node.js, Express webhook endpoint, send webhook event JavaScript]
sdkTarget: typescript
---

# JavaScript/TypeScript SDK

The official Hook0 SDK for JavaScript and TypeScript applications, providing a type-safe and idiomatic interface to the Hook0 API.

## Installation

```bash
npm install hook0-client
# or
yarn add hook0-client
# or
pnpm add hook0-client
```

## Quick Start

```typescript example=program
import { Hook0Client, Event } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081/api/v1',
  'app_1234567890', // Your application ID
  '{YOUR_TOKEN}'
);

// Send an event
const event = new Event(
  'user.account.created',
  JSON.stringify({
    user_id: 'user_123',
    email: 'john.doe@example.com'
  }),
  'application/json',
  { environment: 'production' }
);

const eventId = await hook0.sendEvent(event);
```

## Sending an Event Is Idempotent, and Retried

`sendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it
generates when the event carries none. **Passing no ID no longer means the ID comes from Hook0** —
the interface is unchanged, but the value now comes from the client, is sent with the request, and
is what `sendEvent` resolves to.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a
network failure or a server error ingests the event once rather than twice; without a
client-chosen ID, a repeated request would create a second event and deliver it to every
subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are
retried; a `Retry-After` header is honoured and clamped to what is left of the delay budget. An
answer Hook0 would repeat (a bad request, an exhausted daily quota) is reported as is. A retried
request Hook0 answers with `EventAlreadyIngested` resolves, because an earlier attempt of that same
send reached the API; the same answer to a *first* attempt is a genuine conflict and rejects.

Every send is bounded, and every bound is configurable:

```typescript example=program
import { Hook0Client, Hook0ClientOptions, RetryPolicy } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081/api/v1',
  'app_1234567890',
  '{YOUR_TOKEN}',
  false,
  new Hook0ClientOptions(
    new RetryPolicy(
      4,    // attempts, the first one included
      100,  // ceiling of the delay before the first retry, in milliseconds
      2000, // ceiling no single delay ever exceeds, in milliseconds
      5000  // budget all the delays of one send share, in milliseconds
    ),
    10000,       // longest one attempt is given, in milliseconds
    1024 * 1024  // largest event payload the client sends, in bytes
  )
);
```

Those are the defaults. `RetryPolicy.disabled()` sends each event exactly once. A payload above the
maximum is refused before any request is issued, so neither the round trip nor the retries after it
are spent on a request the API would refuse.

## Configuration

### Client Initialization

```typescript example=program
import { Hook0Client } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081/api/v1',     // API URL
  'app_1234567890',            // Your application ID
  '{YOUR_TOKEN}',   // Authentication token
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

```typescript example=program
import { Hook0Client, Event } from 'hook0-client';

const hook0 = new Hook0Client(
  'http://localhost:8081/api/v1',
  'app_1234567890',
  '{YOUR_TOKEN}'
);

const event = new Event(
  'order.checkout.completed',
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

Listing and querying events goes through the generated API groups rather than through `Hook0Client`. See [Calling the rest of the API](#calling-the-rest-of-the-api).

### Event Type Management

```typescript example=usingClient
// Upsert event types (creates if not exists)
const addedEventTypes = await hook0.upsertEventTypes([
  'user.account.created',
  'user.account.updated',
  'order.checkout.completed'
]);

console.log('Added event types:', addedEventTypes);
```

## Calling the rest of the API

`Hook0Client` covers sending events and declaring event types. Every other operation Hook0 declares is a method on a generated group, exported under the `generated` namespace:

```typescript example=restApiGroup
import { generated } from 'hook0-client';

const applications = new generated.ApplicationsApi(transport);
const application = await applications.get(applicationId);
```

The generated half declares the transport it issues requests through and does not implement one, so nothing in it carries a socket. Supply your own:

```typescript example=restApiTransport
import { generated } from 'hook0-client';

const transport: generated.Transport = {
  async request(asked) {
    const query = new URLSearchParams(asked.query as [string, string][]).toString();
    const answered = await fetch(
      `https://app.hook0.com${asked.path}${query ? `?${query}` : ''}`,
      {
        method: asked.method,
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'application/json',
          ...(asked.body === undefined ? {} : { 'Content-Type': 'application/json' }),
        },
        body: asked.body,
        signal: AbortSignal.timeout(10_000),
      }
    );

    return { status: answered.status, payload: await answered.text() };
  },
};
```

Every problem the API can report is its own subclass of `generated.ProblemError`, which carries the HTTP status and the parsed problem.

The names the API document declares live under `generated` rather than at the top level because the document itself declares an `Event` and an `EventType`, which are the API's resources and not the `Event` an emitter fills in.

## Advanced Features

### Webhook Verification

```typescript example=webhookHandlerFull
import { verifyWebhookSignature } from 'hook0-client';
import express from 'express';

const app = express();

// Note: Express.js normalizes all header names to lowercase
// Capture raw body for signature verification
app.post('/webhook', express.json({
  verify: (req, res, buf) => { (req as any).rawBody = buf; }
}), (req, res) => {
  const signature = req.headers['x-hook0-signature'] as string;
  const secret = process.env.WEBHOOK_SECRET!;
  // Verify against the raw bytes, not a stringified copy: verifyWebhookSignature hashes the body it
  // is handed, so a body already turned into a string would no longer match what Hook0 signed.
  const rawBody = (req as any).rawBody as Buffer;

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
      rawBody,
      headers,
      secret,
      300 // 5-minute tolerance
    );

    if (!isValid) {
      return res.status(401).json({ error: 'Invalid signature' });
    }

    // Process the webhook (already parsed via req.body)
    console.log('Webhook received:', req.body);
    processWebhook(req.body);

    res.json({ status: 'processed' });
  } catch (error) {
    console.error('Webhook processing error:', error);
    res.status(500).json({ error: 'Processing failed' });
  }
});
```

### Error Handling

```typescript example=usingClient
import { Hook0ClientError } from 'hook0-client';

try {
  const event = new Event(
    'user.account.created',
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

```typescript example=program
import { Hook0Client, Event, EventType, Hook0ClientError } from 'hook0-client';

// Type-safe event creation
const event = new Event(
  'user.account.created',
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

```typescript example=program
import { Hook0Client, Event } from 'hook0-client';
import { describe, expect, jest, test } from '@jest/globals';

describe('Event Handler', () => {
  test('should send user created event', async () => {
    // Mock the fetch function; typing the mock as `typeof fetch` is what lets
    // mockResolvedValueOnce take a Response-shaped value instead of rejecting the call outright.
    global.fetch = jest.fn<typeof fetch>().mockResolvedValueOnce({
      ok: true,
      text: async () => '',
    } as Response);

    const client = new Hook0Client(
      'http://localhost:8081/api/v1',
      'app_test',
      'test_token'
    );

    const event = new Event(
      'user.account.created',
      JSON.stringify({ email: 'test@example.com' }),
      'application/json',
      {}
    );

    const eventId = await client.sendEvent(event);

    // Verify fetch was called correctly
    expect(fetch).toHaveBeenCalledWith(
      'http://localhost:8081/api/v1/event',
      expect.objectContaining({
        method: 'POST',
        headers: expect.objectContaining({
          'Authorization': 'Bearer test_token',
          'Content-Type': 'application/json'
        })
      })
    );
  });
});
```

## Best Practices

### 1. Use Environment Variables

```typescript example=hook0ClientImport
{
  // Bad - hardcoded credentials
  const hook0 = new Hook0Client(
    'http://localhost:8081/api/v1',
    'app_1234567890',
    'hardcoded_token_here'
  );
}

// Good - use environment variables
const hook0 = new Hook0Client(
  process.env.HOOK0_API_URL!,
  process.env.HOOK0_APP_ID!,
  process.env.HOOK0_TOKEN!
);
```

### 2. Implement Proper Error Handling

```typescript example=usingClientAndEvent
// Bad
await hook0.sendEvent(event);

// Good
try {
  await hook0.sendEvent(event);
} catch (error) {
  if (error instanceof Hook0ClientError) {
    logger.error('Failed to send event', {
      message: error.message,
      event
    });
    // Implement retry or fallback logic
  }
  throw error;
}
```

### 3. Efficient Event Sending

```typescript example=usingClient
// When sending multiple events, consider using Promise.all for parallelization
const eventPromises = users.map(user => {
  const event = new Event(
    'user.account.created',
    JSON.stringify(user),
    'application/json',
    { source: 'bulk_import' }
  );
  return hook0.sendEvent(event);
});

const eventIds = await Promise.all(eventPromises);
console.log(`Sent ${eventIds.length} events`);
```

### 4. Use Unique Event IDs to Deduplicate Across Emitters

```typescript example=usingClient
import { v5 as uuidv5 } from 'uuid';

// The client already sends an id of its own, so retries are idempotent without you doing anything.
// Set an id yourself when the *same* event can be produced more than once by your application — a
// payment webhook replayed by your provider, a job that can run twice.
// Hook0 requires event_id to be a UUID, so derive a stable one from your domain key.
const event = new Event(
  'payment.transaction.processed',
  JSON.stringify({ amount: 100.00 }),
  'application/json',
  { transaction_id },
  undefined, // metadata
  new Date(), // occurredAt
  uuidv5(transaction_id, uuidv5.URL) // Your own event ID (stable UUID derived from transaction_id)
);

const eventId = await hook0.sendEvent(event);
```

## Troubleshooting

### Common Issues

**Authentication Errors**
```typescript example=hook0ClientImport
// Ensure token is passed correctly (without Bearer prefix - SDK adds it)
const hook0 = new Hook0Client(
  'http://localhost:8081/api/v1',
  'app_1234567890',
  '{YOUR_TOKEN}' // ✓ Correct - just the token, SDK adds "Bearer " automatically
);
```

**CORS Issues in Browser**
```typescript example=program
// The SDK uses fetch() which handles CORS automatically
// Ensure your Hook0 application is configured to accept
// requests from your domain
```

**Network Errors**
```typescript example=retryTuning
// The client already retries network failures and server errors on its own, under the same event
// id, so a retry cannot ingest the event twice. Change how it does so rather than wrapping it:
import { Hook0Client, Hook0ClientOptions, RetryPolicy } from 'hook0-client';

const patient = new Hook0Client(apiUrl, applicationId, token, false, new Hook0ClientOptions(
  new RetryPolicy(6, 200, 5000, 20000)
));

// A `sendEvent` that rejects with "gave up after N attempts" has already exhausted them.
```

## Support

- **Documentation**: [Hook0 API Docs](/api)
- **Getting Started**: [Tutorial](/tutorials/getting-started)
- **GitHub Issues**: [Report Issues](https://github.com/hook0/hook0/issues)
- **Discord**: [Join Community](https://www.hook0.com/community)
- **NPM Package**: hook0-client