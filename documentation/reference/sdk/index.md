---
title: "Hook0 SDKs: 11 official webhook client libraries"
description: "Official Hook0 clients for JavaScript, TypeScript, Rust, Python, Go, Ruby, PHP, C#, Java, Kotlin, Lua and Zig, plus an MCP server. Each sends events, verifies webhook signatures, and retries under bounds you set."
keywords: [Hook0 SDK, webhook SDK, webhook client library, JavaScript webhook library, Python webhook client, Rust webhook SDK, Go webhook client, Java webhook SDK, C# webhook client, verify webhook signature, Hook0 MCP server]
sdkTarget: none
---

# SDKs & client libraries

Hook0 has eleven official clients. They share the same behaviour and the same defaults, and each one is written in the idiom of its language rather than a translation of another.

Every client does four things. It sends events, it declares the event types your application uses, it verifies the signature of an incoming webhook, and it exposes the rest of the API as generated, typed operations.

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

Java, Kotlin and Lua are not on their language's registry yet. Nothing is wrong with the code; each one is waiting on something a pipeline cannot supply on its own, such as a namespace to claim. Each page says what stands in the way and how to depend on the client today. The rest install with one command.

Go, PHP and Zig install from a repository rather than from a registry, because that is what those three ecosystems fetch from. Each client is pushed to a read-only mirror of its own on GitHub, `github.com/hook0/hook0-<language>`, tagged `vX.Y.Z` on every SDK release. The mirrors are derived from this monorepo, which stays the one place anything is changed.

## Not a library: the MCP server

The generator writes a twelfth thing from the same API document, and it is not a client library. [`hook0-mcp`](mcp.md) is a Model Context Protocol server. An assistant starts it, talks to it over stdio, and calls one of twenty-three generated tools on it. Nothing imports it to send an event. An assistant calls `events.ingest`.

It shares the headers every client sends and none of the rest. It does not retry, verifies no signature, and applies none of the bounds below. Its page says why.

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

Read this before you write anything around a client. The first three will change what you write; the fourth is what an instance sees you doing.

### It mints the event ID, so a retry cannot duplicate an event

A send goes out under an ID the client knows, either the one you set on the event or a UUIDv7 it generates when you set none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, travels with the request, and is what the send returns.

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

### It says which client it is, and what it will repeat

Past the credential and the media types it declares, every request carries two more headers, and both are written for whoever is reading an instance's logs rather than for your code:

```
User-Agent: hook0-client-python/1.1.0 (CPython 3.13.1; Linux x86_64)
Hook0-Client-Options: attempts=4,backoff=100,ceiling=2000,budget=5000
```

The first names the SDK, its version, the runtime and the machine. Without it an instance cannot tell a deprecation that has reached everybody from one that has reached nobody, and there is no way to go back and ask a client that shipped two years ago.

The second is the retry policy behind the request, in milliseconds, in that fixed order. It is the only client setting the API can see the consequences of without being told. A burst of identical requests arriving inside a few seconds is one send being repeated, and without this header it is indistinguishable from an application in a loop. It states the policy **in force** rather than the one asked for, so a policy that asked for a thousand attempts states the sixteen the client will actually make, and it states it on every request, including the one that succeeded first time.

Both are composed the same way in every client, from parts cut to a fixed length so that neither can grow with whatever the platform feels like saying about itself.

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

Two rules hold in all eleven. Verify against the **raw** request body, since a body that has been parsed and re-serialised no longer hashes to what was signed. And keep the tolerance bilateral, which every client does, so a delivery dated too far ahead is refused exactly like one dated too far behind.

```typescript example=webhookHandlerUsingApp
// JavaScript/TypeScript
import { verifyWebhookSignature } from 'hook0-client';

// `verify` is the only place Express hands over the bytes before it parses them, and the bytes are
// what was signed: a body already turned back into a string no longer hashes to it.
app.post('/webhook', express.json({
  verify: (req, _res, buf) => { (req as unknown as { rawBody: Buffer }).rawBody = buf; }
}), (req, res) => {
  // Express lowercases every header name it received.
  const headers = new Headers();
  for (const [name, value] of Object.entries(req.headers)) {
    if (typeof value === 'string') headers.set(name, value);
  }

  try {
    verifyWebhookSignature(
      req.headers['x-hook0-signature'] as string,
      (req as unknown as { rawBody: Buffer }).rawBody,
      headers,
      process.env.WEBHOOK_SECRET!,
      300 // 5-minute tolerance, in seconds
    );
  } catch {
    res.status(401).json({ error: 'invalid signature' });
    return;
  }

  // act on the delivery, already parsed into req.body
  res.json({ status: 'processed' });
});
```

How a refusal reaches you differs by language, and each page says which. Rust and Go return a result; Python, Ruby, PHP, C#, Java, Kotlin and TypeScript raise; Lua raises a table you match with `Hook0.is`; Zig answers a closed error set. TypeScript carries one exception its page covers, `EventType.fromString`, which hands the failure back rather than throwing it.

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

Every client reports what it refused as a type, never as a message you are expected to read. Match on the type; a message is prose, and prose is rewritten.

```typescript example=usingEvent
// JavaScript/TypeScript
import { Hook0ClientError } from 'hook0-client';

try {
  const eventId = await hook0.sendEvent(event);
  console.log(`ingested as ${eventId}`);
} catch (refused) {
  if (refused instanceof Hook0ClientError) {
    console.error(`event not sent: ${refused.message}`);
  } else {
    throw refused;
  }
}
```

Every problem the API can report reaches you as something you can name, but not as the same shape everywhere. C#, Java, Kotlin, PHP, Python and Ruby give each problem a type of its own under a common base, so a handler names one or catches all of them. Lua does the same with error kinds. Rust, TypeScript and Go carry one type for every problem with the identifier beside it, either a `kind` you compare or a sentinel you match with `errors.Is`, and Zig answers one member of a closed error set. The identifiers themselves come from the API document and are the same everywhere; how strictly a client holds you to that list is its own page's business, and C# holds you to it least.

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
