# Hook0 TypeScript/JavaScript Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE.md)
[![Latest Version](https://img.shields.io/npm/v/hook0-client)](https://www.npmjs.com/package/hook0-client)
[![Documentation](https://img.shields.io/badge/docs-documentation-blue)](https://documentation.hook0.com/docs/sdk-javascript-typescript)

This is the TypeScript/JavaScript SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it
generates when the event carries none. **Passing no ID no longer means the ID comes from Hook0** —
the interface is unchanged, but the value now comes from the client, is sent with the request, and
is what `sendEvent` resolves to.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a
network failure or a server error ingests the event once rather than twice; without a
client-chosen ID, a repeated request would create a second event and deliver it to every
subscriber.

Only a network failure or a server error is retried. A retried request Hook0 answers with
`EventAlreadyIngested` resolves — an earlier attempt of that same send reached the API. The same
answer to a *first* attempt is a genuine conflict and rejects.

Every send is bounded, and every bound is configurable:

```typescript
import { Hook0Client, Hook0ClientOptions, RetryPolicy } from 'hook0-client';

const client = new Hook0Client(apiUrl, applicationId, token, false, new Hook0ClientOptions(
  new RetryPolicy(
    4,    // attempts, the first one included
    100,  // ceiling of the delay before the first retry, in milliseconds
    2000, // ceiling no single delay ever exceeds, in milliseconds
    5000  // budget all the delays of one send share, in milliseconds
  ),
  10000,          // longest one attempt is given, in milliseconds
  1024 * 1024     // largest event payload the client sends, in bytes
));
```

Those are the defaults. `RetryPolicy.disabled()` sends each event exactly once. A payload above the
maximum is refused before any request is issued.

## Getting Started

To add the Hook0 client in your TS/JS project, install it via npm:

```bash
npm install hook0-client
```

### Both module systems, and one copy of the code

`require` and `import` both reach the package, and the `exports` map says so rather than leaving it
to be inferred:

```javascript
const { Hook0Client } = require('hook0-client'); // CommonJS
```

```javascript
import { Hook0Client } from 'hook0-client'; // ES modules, named
import hook0Client from 'hook0-client'; // ES modules, the whole package
```

TypeScript resolves the declarations for whichever half it picks, under `bundler`, `node16` and
`nodenext` alike.

The ES module half re-exports the compiled CommonJS instead of being a second compilation of it, so
a dependency tree that reaches the package both ways still gets **one** `Hook0ClientError` class
rather than two. That matters here: `verifyWebhookSignature` and `EventType.fromString` answer with
either a value or a `Hook0ClientError`, so telling the two apart means writing
`instanceof Hook0ClientError` — and against an error built by a second copy of the class, that test
answers `false`, which would read a refused signature as a verified one.

The compiled layout is not part of the contract: reaching into `hook0-client/dist/…` is refused at
resolution. Import the package.

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)

### LICENSE  
Hook0 TypeScript SDK is free and open-source. It is released under the [MIT License](./LICENSE.md).  

This license grants you the freedom to use, modify, distribute, and sublicense the SDK with minimal restrictions. You may use it in both open-source and commercial projects, as long as you include the original copyright notice.  

For more details, refer to the full [LICENSE.md](./LICENSE.md) file.
