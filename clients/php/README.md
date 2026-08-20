<div align="center">

# Hook0 PHP SDK

**Send and verify webhooks with the extensions every distribution already ships**

<br/>

<img src="assets/php-flow.svg" alt="How the Hook0 PHP SDK sits between your application and your users" width="850"/>

<br/>
<br/>

[![Packagist](https://img.shields.io/packagist/v/hook0/client)](https://packagist.org/packages/hook0/client)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

</div>

---

## What is this?

The PHP SDK for [Hook0](https://www.hook0.com/), the open source Webhooks-as-a-Service platform
for SaaS applications. It sends events, declares the event types your application uses, verifies the
signature of a webhook you receive, and calls every operation the API declares through generated,
documented types.

It reaches the network with `ext-curl`, verifies signatures with `ext-hash`, decodes documents with
`ext-json` and mints identifiers with `random_bytes`, all of them shipped by every distribution of
the language. Installing it never drags a package into an application that only wanted to send an
event.

## Features

- **Send events** - under an ID the client mints, so a retry cannot duplicate one
- **Declare event types** - upsert the ones your application emits, in one call
- **Verify signatures** - HMAC-SHA256 over a bilateral clock window
- **The whole API, typed** - one class per schema, one backed enum per closed list, one exception per problem
- **Bounded everywhere** - attempts, backoff, timeouts, payload and answer, all yours to set
- **Zero dependencies** - the language and its bundled extensions, nothing more

---

## Quick Start

### 1. Install

```bash
composer require hook0/client
```

PHP 8.2 or later.

### 2. Send an event

```php
<?php

use Hook0\Client;
use Hook0\Event;

$client = new Client(
    'https://app.hook0.com/api/v1',
    $applicationId,
    $token,
);

$eventId = $client->sendEvent(new Event(
    eventType: 'billing.invoice.paid',
    payload: '{"invoice": "in_123"}',
    payloadContentType: 'application/json',
    labels: ['environment' => 'production'],
));
```

### 3. Verify a webhook you receive

```php
use Hook0\ClientError;
use Hook0\Signature;

try {
    Signature::verify(
        $request->header('X-Hook0-Signature'),
        $request->body(),
        $request->headers(),
        $subscriptionSecret,
        300.0,
    );
} catch (ClientError) {
    // answer 400, and do not act on the delivery
}
```

The clock window is bilateral, so a delivery dated too far ahead is refused exactly like one dated
too far behind, because a window that only looked backwards is one a sender widens by dating its own
delivery in the future. A header the signature covers but the request did not carry is refused
before any code is computed.

---

## Configuration

Every bound one send is held to is yours to set, and every one has a default.

| Bound | Default | What it holds back |
|-|-|-|
| `maxAttempts` | 4 | requests one send issues, capped at 16 whatever a policy says |
| `initialBackoff` | 100 ms | the ceiling of the wait before the first retry |
| `maxBackoff` | 2 s | the ceiling no single wait between attempts crosses |
| `maxTotalDelay` | 5 s | the budget every wait of one send shares |
| `requestTimeout` | 10 s | how long one attempt is given |
| `maxPayloadBytes` | 1 MiB | the payload, refused before a socket is opened |
| `maxResponseBytes` | 8 MiB | the body read off a socket |
| `maxHeadBytes` | 16 KiB | the head of an answer, every line taken together |
| `maxResponseHeaders` | 64 | header lines one answer may carry |
| `maxHeaderBytes` | 64 KiB | one header line |

Every default comes from [`clients/conformance/bounds.json`](https://gitlab.com/hook0/hook0/-/blob/master/clients/conformance/bounds.json),
the corpus every Hook0 SDK reads. A number changed there fails every SDK still carrying the old one,
so no two of them can bound different things.

The last three bound what the other end may cost you. A server that is broken or hostile can
otherwise stream a head, a header or a body of any length into your process.

```php
use Hook0\Client;
use Hook0\Options;
use Hook0\RetryPolicy;

$client = new Client(
    'https://app.hook0.com/api/v1',
    $applicationId,
    $token,
    new Options(
        retryPolicy: new RetryPolicy(
            maxAttempts: 4,
            initialBackoff: 0.1,
            maxBackoff: 2.0,
            maxTotalDelay: 5.0,
        ),
        requestTimeout: 10.0,
        maxPayloadBytes: 1024 * 1024,
        maxResponseBytes: 8 * 1024 * 1024,
    ),
);
```

---

## Usage

### Sending is idempotent, and retried

`sendEvent` sends every event under an ID it knows, either the one set on the event or a UUIDv7 it
mints when the event carries none. **Passing no ID does not mean the ID comes from Hook0.** The value comes
from the client, travels with the request, and is what `sendEvent` answers.

That is what makes a retry safe. Hook0 keys events on their ID, so a request repeated after a network
failure or a server error ingests the event once rather than twice. Without a client-chosen ID, the
repeated request would create a second event and deliver it to every subscriber.

Retrying is limited to what could end differently. A request that got no answer, a server error and
an instance saying it is being reached faster than it accepts are all retried. A `429` naming a
spent quota is not, because a quota clears when a plan changes or a day turns, and no send can wait
for that. A
`Retry-After` the answer carries is honoured, clamped to what is left of the delay budget. A retried
request Hook0 answers with `EventAlreadyIngested` reports success, since an earlier attempt of that
same send reached the API. The same answer to a *first* attempt is a genuine conflict, and is
reported as an error.

### Declaring the event types you use

```php
$created = $client->upsertEventTypes([
    'billing.invoice.paid',
    'billing.invoice.voided',
]);
```

Only the ones your application does not declare yet are created, and those are what comes back.

### Calling the rest of the API

Every operation the API declares is a method of a generated group, one group per entity.

```php
use Hook0\Generated\ApplicationsApi;
use Hook0\Generated\NotFoundError;
use Hook0\Transport;

$applications = new ApplicationsApi(new Transport('https://app.hook0.com', $token));

try {
    $application = $applications->get($applicationId);
} catch (NotFoundError $reported) {
    // every problem the API names is its own exception, all of them `ProblemError`
}
```

### Names travel as the API spells them

A closed list of values is a backed enum, so a value the API does not declare cannot be built at
all. PHP reads what follows `->`, `::` and `$` as a name rather than as a keyword, so a member
called `method`, `until` or `type` is called exactly that on both sides.

---

## Development

`clients/php/src/Generated/` is written by [`hook0-sdkgen`](https://gitlab.com/hook0/hook0/-/tree/master/clients/sdkgen)
from the OpenAPI snapshot the API commits, and is rewritten whole on every regeneration. A hand edit
there is reverted the next time anyone regenerates, and the drift guard says so before that. Change
the generator, then run:

```
UPDATE_SDK=php cargo test -p hook0-sdkgen sdk_targets
```

Everything else under `src/` is hand-written and never regenerated, and so is `tests/`.

What a send retries, the bounds it is held to and how a signature is verified are dictated by the
shared corpus at [`clients/conformance`](https://gitlab.com/hook0/hook0/-/tree/master/clients/conformance),
which every SDK's suite reads, so a verdict changed there fails this client until it agrees again.

Every case runs against a real Hook0 over a loopback socket. Nothing here stands in for a part of
the client.

```
phpcs
phpunit --no-coverage
```

---

## License

The Hook0 PHP SDK is free and open source, released under the [MIT License](./LICENSE). Use it,
change it, ship it, in open source and in commercial work alike, as long as the copyright notice
travels with it.

Hook0 itself is open source too. Read [what Hook0 is](https://documentation.hook0.com/docs/what-is-hook0),
visit [hook0.com](https://www.hook0.com/), join the [community](https://www.hook0.com/community), or
write to [support@hook0.com](mailto:support@hook0.com).

Maintained by [David Sferruzza](mailto:david@hook0.com) and [François-Guillaume Ribreau](mailto:fg@hook0.com).
