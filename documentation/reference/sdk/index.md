---
title: "Hook0 SDKs — 11 official webhook client libraries"
description: "Official Hook0 clients for JavaScript, TypeScript, Rust, Python, Go, Ruby, PHP, C#, Java, Kotlin, Lua and Zig. Each sends events, verifies webhook signatures, and retries under bounds you set."
keywords: [Hook0 SDK, webhook SDK, webhook client library, JavaScript webhook library, Python webhook client, Rust webhook SDK, Go webhook client, Java webhook SDK, C# webhook client, verify webhook signature]
sdkTarget: none
---

# SDKs & client libraries

Hook0 has eleven official clients. They share the same behaviour and the same defaults, and each one is written in the idiom of its language rather than a translation of another.

Every client does four things: it sends events, it declares the event types your application uses, it verifies the signature of an incoming webhook, and it exposes the rest of the API as generated, typed operations.

## The clients

| Language | Page | Package | Install from | Surfaces |
|----------|------|---------|--------------|----------|
| JavaScript / TypeScript | [JavaScript SDK](javascript.md) | `hook0-client` | npm | promises |
| Rust | [Rust SDK](rust.md) | `hook0-client` | crates.io | async |
| Python | [Python SDK](python.md) | `hook0-client` | PyPI | blocking and `asyncio` |
| Ruby | [Ruby SDK](ruby.md) | `hook0-client` | RubyGems | blocking |
| C# / .NET | [C# SDK](csharp.md) | `Hook0.Client` | NuGet | blocking and `Task` |
| Go | [Go SDK](go.md) | `github.com/hook0/hook0-go` | Go module proxy | blocking |
| PHP | [PHP SDK](php.md) | `hook0/client` | Packagist | blocking |
| Java | [Java SDK](java.md) | `com.hook0:hook0-client` | source | blocking and `CompletableFuture` |
| Kotlin | [Kotlin SDK](kotlin.md) | `com.hook0:hook0-client-kotlin` | source | blocking and suspending |
| Lua | [Lua SDK](lua.md) | `hook0-client` | source | blocking |
| Zig | [Zig SDK](zig.md) | `hook0_client` | tagged archive | blocking |

Three of the clients — Java, Kotlin and Lua — are not on their language's registry yet. Nothing is wrong with the code; each one is waiting on something a pipeline cannot supply on its own, such as a namespace to claim. Each page says what stands in the way and how to depend on the client today. The rest install with one command.

Go, PHP and Zig install from a repository rather than from a registry, because that is what those three ecosystems fetch from. Each client is pushed to a read-only mirror of its own on GitHub — `github.com/hook0/hook0-<language>` — tagged `vX.Y.Z` on every SDK release. The mirrors are derived from this monorepo, which stays the one place anything is changed.

## Set up environment variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081 locally)

# Set your application ID (shown in dashboard URL or application details)
export APP_ID="YOUR_APPLICATION_ID_HERE"
```

Save these values:
```bash
# Save to .env file for later use
cat > .env <<EOF
HOOK0_TOKEN=$HOOK0_TOKEN
HOOK0_API=$HOOK0_API
APP_ID=$APP_ID
EOF
```

The token goes to the client without a `Bearer` prefix. Every client adds it.

## What every client does without being asked

Read this before you write anything around a client, because three of these will change what you write.

### It mints the event ID, so a retry cannot duplicate an event

A send goes out under an ID the client knows: the one you set on the event, or a UUIDv7 it generates when you set none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, travels with the request, and is what the send returns.

Hook0 keys events on that ID, so a request repeated after a failure ingests the event once rather than twice. Set an ID yourself only when the *same* event can be produced more than once by your own application, a payment webhook replayed by your provider or a job that can run twice, by deriving a stable UUID from your domain key.

### It already retries

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A second retry loop wrapped around a client is a loop around a loop. Change the client's policy instead, or disable it and own the retry yourself.

| Bound | Default |
|-------|---------|
| Attempts, the first one included | 4, capped at 16 |
| Delay before the first retry | 100 ms |
| Ceiling no single delay exceeds | 2 s |
| Budget all delays of one send share | 5 s |
| Timeout, per attempt | 10 s |
| Largest event payload sent | 1 MiB |
| Largest response body read | 8 MiB |

Every client exposes a disabled policy that sends each event exactly once.

### It refuses what the API would refuse

A payload above the maximum fails before any request goes out, so neither the round trip nor the retries after it are spent on a request the API would reject. A response body above the ceiling is refused rather than read into memory, and so are an oversized header, too many headers, and an oversized head.

## Sending an event

```typescript example=usingClient
// JavaScript/TypeScript
const event = new Event(
  'order.checkout.completed',
  JSON.stringify({
    order_id: 'ord_123',
    total: 99.99
  }),
  'application/json',
  {
    environment: 'production',
    region: 'us-west'
  }
);

await hook0.sendEvent(event);
```

```bash
# Using cURL
curl -X POST $HOOK0_API/event \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "order.checkout.completed",
    "payload": {
      "order_id": "ord_123",
      "total": 99.99
    },
    "labels": {
      "environment": "production",
      "region": "us-west"
    }
  }'
```

## Verifying a webhook

This is the half most readers need, and the half most often got wrong by hand. Every client ships it.

Two rules hold in all eleven. Verify against the **raw** request body: a body that has been parsed and re-serialised no longer hashes to what was signed. And keep the tolerance bilateral, which every client does, so a delivery dated too far ahead is refused exactly like one dated too far behind.

```typescript example=webhookHandlerUsingApp
// JavaScript/TypeScript
import { verifyWebhookSignature } from 'hook0-client';

// Note: Express.js normalizes all header names to lowercase
// Capture raw body for signature verification
app.post('/webhook', express.json({
  verify: (req, res, buf) => { (req as any).rawBody = buf; }
}), (req, res) => {
  const signature = req.headers['x-hook0-signature'] as string;
  // Verify against the raw bytes, not a stringified copy: verifyWebhookSignature hashes the body it
  // is handed, so a body already turned into a string would no longer match what Hook0 signed.
  const rawBody = (req as any).rawBody as Buffer;
  const headers = new Headers();
  Object.entries(req.headers).forEach(([key, value]) => {
    if (typeof value === 'string') headers.set(key, value);
  });
  const secret = process.env.WEBHOOK_SECRET!;

  try {
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
    // Process webhook (already parsed via req.body)...
  } catch (error) {
    return res.status(401).json({ error: 'Invalid signature' });
  }
});
```

How a refusal reaches you differs by language, and each page says which. Rust and Go return a result; Python, Ruby, PHP, C#, Java and Kotlin raise; Lua raises a table you match with `Hook0.is`; Zig answers a closed error set.

## Managing subscriptions

```bash
# Using the REST API
curl -X POST $HOOK0_API/subscriptions \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Order Events",
    "event_types": ["order.checkout.completed", "order.shipped"],
    "target": {
      "type": "http",
      "url": "https://api.example.com/webhooks",
      "method": "POST"
    }
  }'
```

Every client also reaches this through its generated API groups, one group per entity and one method per operation, so you do not have to drop to `curl`. See the "Calling the rest of the API" section of each page.

## Error handling

SDKs return typed errors you can match on:

```typescript example=usingEvent
// JavaScript/TypeScript
import { Hook0ClientError } from 'hook0-client';

try {
  const eventId = await hook0.sendEvent(event);
} catch (error) {
  if (error instanceof Hook0ClientError) {
    console.error('Hook0 error:', error.message);

    if (error.message.includes('Invalid event type')) {
      // Handle invalid event type format
    } else if (error.message.includes('failed')) {
      // Retry logic
    }
  }
}
```

Every problem the API can report is its own type in the generated half of each client, under a common base, so a handler may name one problem or catch all of them.

## Authentication and security

- Authentication via Biscuit tokens (user sessions) and Service tokens (programmatic access)
- Webhook signature verification, `v0` over the body and `v1` over the covered headers and the body
- TLS for every request
- The token is never logged, and never exposed by a client's accessors

## Contributing an SDK

### Requirements checklist

- [ ] All essential endpoints implemented
- [ ] Biscuit token and Service token support
- [ ] Typed error messages
- [ ] >80% test coverage
- [ ] API docs with examples
- [ ] Working example applications
- [ ] Automated testing and publishing

### Best practices

1. Write idiomatic code for your language
2. Provide type definitions where possible
3. Implement async/await or promises
4. Handle connection pooling and cleanup
5. Follow semantic versioning
6. Maintain a changelog

## Getting help

### Documentation
- [API Reference](../../openapi/intro) - REST API documentation
- [Tutorials](../../tutorials/) - Step-by-step guides
- [How-to Guides](../../how-to-guides/) - Problem-solving guides

### Support channels
- GitHub Issues - SDK-specific issue tracking
- [Discord](https://www.hook0.com/community) - Community support
- [Stack Overflow](https://stackoverflow.com/questions/tagged/hook0) - #hook0 tag
- support@hook0.com - For critical issues
