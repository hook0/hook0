---
title: "JavaScript & TypeScript webhook SDK — hook0-client"
description: "Send Hook0 events and verify webhook signatures from Node.js. Typed, ESM and CommonJS, idempotent event IDs, retries and payload bounds built in."
keywords: [JavaScript webhook SDK, TypeScript webhook client, hook0-client npm, verify webhook signature Node.js, Express webhook endpoint, send webhook event JavaScript]
sdkTarget: typescript
---

# JavaScript / TypeScript SDK

The Hook0 SDK for JavaScript and TypeScript sends events and verifies webhook signatures. It is written in TypeScript and ships its own declarations, so a JavaScript consumer gets the same completions a TypeScript one does.

Every method that reaches the network returns a promise. The package declares no runtime dependencies: sending goes through `fetch` and `AbortSignal.timeout`, and signatures are computed with Node's own `crypto`.

## Installation

```bash
npm install hook0-client
```

It publishes both module systems from one package, so `import` and `require` each resolve to the build meant for them:

```typescript example=program
import { Event, Hook0Client } from 'hook0-client';
// or: const { Event, Hook0Client } = require('hook0-client');

const hook0 = new Hook0Client(
  'https://app.hook0.com/api/v1',
  '0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21',
  process.env.HOOK0_TOKEN!
);

const eventId = await hook0.sendEvent(
  new Event(
    'billing.invoice.paid',
    JSON.stringify({ invoice: 'in_123' }),
    'application/json',
    { environment: 'production' }
  )
);
```

Node is what it is written for. The module imports `node:url`, and verification takes a `Buffer`, so a browser or an edge runtime needs both of those resolved before anything here loads — the client does allow for it, naming neither the runtime nor the machine in its `User-Agent` when there is no `process` to read them off.

## The event

`Event` takes four required arguments and three optional ones, positionally:

```typescript example=event
new Event(
  'billing.invoice.paid',
  JSON.stringify({ invoice: 'in_123' }),
  'application/json',
  { environment: 'production' },
  { emitter: 'billing-worker' }, // metadata
  new Date(), // occurredAt; the current moment when absent
  eventId // the client mints a UUIDv7 when absent
);
```

`labels` is required and `metadata` is not, which is the one place the positional order is worth reading twice. Labels are what a subscription routes on, and the API declares the field required — pass `{}` for an event nothing filters on, never skip the argument.

The token goes in without a `Bearer` prefix; the client adds it.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `sendEvent` resolves to.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` resolves, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and rejects.

## Bounds, and how to change them

`Hook0ClientOptions` is the fifth argument, after the debug flag:

```typescript example=options
import { Hook0Client, Hook0ClientOptions, RetryPolicy } from 'hook0-client';

const hook0 = new Hook0Client(apiUrl, applicationId, token, false, new Hook0ClientOptions(
  new RetryPolicy(
    4,    // attempts, the first one included
    100,  // ceiling of the delay before the first retry, in milliseconds
    2000, // ceiling no single delay ever exceeds, in milliseconds
    5000  // budget all the delays of one send share, in milliseconds
  ),
  10_000,          // longest one attempt is given, in milliseconds
  1024 * 1024,     // largest event payload the client sends, in bytes
  8 * 1024 * 1024  // largest answer it reads off the socket, in bytes
));
```

Those are the defaults, and every argument of both constructors has one, so `new Hook0ClientOptions()` and `new RetryPolicy()` are the table below.

| Bound | Default |
|-------|---------|
| `maxAttempts` (the first attempt included) | `4`, capped at `MAX_ATTEMPTS_CAP` = 16 |
| `initialBackoffMs` | 100 |
| `maxBackoffMs` | 2 000 |
| `maxTotalDelayMs`, the budget all delays of one send share | 5 000 |
| `DEFAULT_REQUEST_TIMEOUT_MS`, per attempt | 10 000 |
| `DEFAULT_MAX_PAYLOAD_BYTES` | 1 MiB |
| `DEFAULT_MAX_RESPONSE_BYTES` | 8 MiB |
| `MAX_RESPONSE_HEADERS` | 64 |
| `MAX_HEADER_BYTES` | 64 KiB |
| `MAX_HEAD_BYTES` | 16 KiB |

The last three are exported but not configurable. They bound the head of an answer, which is written by whatever is on the other end: a line count and a size per line multiply, so the whole head is capped as well as each line of it.

`RetryPolicy.disabled()` sends each event exactly once. A payload above the maximum rejects before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

A duration that is not a finite number falls back to that field's own default rather than to zero: a zero would delete the spacing between attempts and turn a mistyped policy into a burst.

## Verify a webhook signature

```typescript example=verify
import { verifyWebhookSignature } from 'hook0-client';

try {
  verifyWebhookSignature(signature, rawBody, headers, subscriptionSecret, 300);
} catch (refused) {
  // answer 400, and do not act on the delivery
}
```

:::caution The return type is wider than the behaviour
`verifyWebhookSignature` is declared `boolean | Hook0ClientError`, but it returns `true` and nothing else. Every refusal is **thrown** as a `Hook0ClientError` — it never returns `false`, and it never returns an error value. A handler shaped `if (!isValid) { … }` therefore has a branch that cannot be reached, and one without a `try` treats every forged delivery as a crash.
:::

`rawBody` is a `Buffer` of the bytes that arrived. A body that has been parsed and re-serialised no longer hashes to what was signed. `headers` is a `Headers`, not the plain object a framework hands you, so build one from what arrived. `tolerance` is a number of **seconds**.

The clock window is bilateral: a delivery dated too far ahead is refused exactly like one dated too far behind, since a window that only looked backwards is one a sender widens by dating its own delivery ahead. A header the signature covers but the request did not carry is refused before any code is computed.

`verifyWebhookSignatureWithCurrentTime` takes the same arguments followed by a `Date`, for holding a signature against a moment you choose.

### Express

```typescript example=webhookHandlerFull
import { verifyWebhookSignature } from 'hook0-client';
import express from 'express';

const app = express();

// `verify` is the only place Express hands over the bytes before it parses them.
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
      300
    );
  } catch {
    res.status(400).json({ error: 'invalid signature' });
    return;
  }

  processWebhook(req.body);
  res.json({ status: 'processed' });
});
```

## Upsert event types

An event whose type the application does not declare is refused. `upsertEventTypes` creates the ones that are missing and resolves to only those it created:

```typescript example=usingClient
const created = await hook0.upsertEventTypes([
  'billing.invoice.paid',
  'billing.invoice.voided',
]);
```

An event type is written `service.resource_type.verb`. `EventType.fromString` reads one and — unlike every refusal in the signature half — *returns* the failure rather than throwing it, so the result is a union you have to narrow:

```typescript example=eventType
import { EventType, Hook0ClientError } from 'hook0-client';

const parsed = EventType.fromString('billing.invoice.paid');

if (parsed instanceof Hook0ClientError) {
  throw parsed;
}

console.log(parsed.service, parsed.resourceType, parsed.verb);
```

## Calling the rest of the API

`Hook0Client` covers sending events and declaring event types. Every other operation Hook0 declares is a method on a generated group under the `generated` namespace — `ApplicationsApi`, `SubscriptionsApi`, `EventsApi`, `RequestAttemptsApi` and nine more, one per entity, one method per operation.

They sit under `generated` rather than at the top level because the API document declares its own `Event` and `EventType`, which are the API's resources and not the `Event` an emitter fills in.

The generated half declares the transport it issues requests through and implements none, so nothing in it carries a socket. Nine of the eleven clients hand you one anyway, built from the client you already have; this one and the Rust one do not, so supply your own:

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

With one in hand, every group is one line:

```typescript example=restApiGroup
import { generated } from 'hook0-client';

const applications = new generated.ApplicationsApi(transport);

try {
  const application = await applications.get(applicationId);
  console.log(application.name);
} catch (failed) {
  if (failed instanceof generated.ProblemError && failed.kind === 'NotFound') {
    // the API named a problem, and this is which one
  }
}
```

Every failure the API can report is one `generated.ProblemError` carrying a `kind`, not a class of its own. `kind` is a `generated.ProblemId`, the closed union of the identifiers the API document declares, so comparing against a string it does not declare is a compile error rather than a branch that never runs. Beside it sit `status` and the whole RFC 9457 `problem` the API answered, when it answered one the client could read.

That union is checked where you write it and nowhere else: the answer is parsed and cast, not validated, so an identifier the API grows before you upgrade arrives as a `kind` no arm of yours matches. Eight of the eleven clients refuse such an answer outright; this one, Go and C# do not.

## Errors

Everything the client itself refuses is a `Hook0ClientError`, built through one of its static constructors, and carries its detail in the message:

| Constructor | Built when |
|-------------|------------|
| `EventSending` | A send failed. Carries the event ID it went out under |
| `RetriesExhausted` | Every attempt failed. Carries the attempts made, the milliseconds waited and the last failure |
| `PayloadTooLarge` | The payload crossed `maxPayloadBytes`, before any request went out |
| `InvalidEventType` | An event type is not `service.resource_type.verb` |
| `GetAvailableEventTypes` | Listing the application's event types failed |
| `InvalidSignature`, `ExpiredWebhook`, `MissingHeader` | A delivery was refused |
| `SignatureParsing`, `TimestampParsingInSignature` | The signature header, or the timestamp in it, could not be read |

```typescript example=usingEvent
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

A `ProblemError` from a generated group is not a `Hook0ClientError`: the two halves of the package report failures separately, so a `catch` around a generated call names the one it means.

## Links

- **Package**: [hook0-client on npm](https://www.npmjs.com/package/hook0-client)
- **Source**: [clients/typescript](https://gitlab.com/hook0/hook0/-/tree/master/clients/typescript)
- **Public surface**: [api-surface.md](https://gitlab.com/hook0/hook0/-/blob/master/clients/typescript/api-surface.md), every export with its signature, regenerated and held against the code by the test suite
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
