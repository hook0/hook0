---
title: "PHP webhook SDK: hook0/client"
description: "Send Hook0 events and verify webhook signatures from PHP 8.2 or later. Retries, idempotent event IDs and payload bounds built in. Install with Composer."
keywords: [PHP webhook SDK, Hook0 PHP client, verify webhook signature PHP, Laravel webhook endpoint, Symfony webhook, send webhook event PHP]
sdkTarget: php
---

# PHP SDK

The Hook0 SDK for PHP sends events and verifies webhook signatures. Every call blocks.

The package requires PHP 8.2 or later and the `curl`, `hash` and `json` extensions. It has no Composer dependencies of its own.

## Installation

```bash
composer require hook0/client
```

The classes autoload under the `Hook0\` namespace.

Packagist reads a repository and its tags rather than an upload, so what it is registered against is `github.com/hook0/hook0-php`, a read-only mirror of `clients/php` that the release pipeline pushes to and tags. Every `sdk-vX.Y.Z` release of the Hook0 SDKs puts the matching `vX.Y.Z` on that mirror and tells Packagist to read it. Issues and merge requests belong on [the monorepo](https://gitlab.com/hook0/hook0); nothing merged into the mirror survives the next release.

## Send an event

```php example=send
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

`Event` takes three required arguments and four optional ones:

```php example=event
new Event(
    eventType: 'billing.invoice.paid',
    payload: '{"invoice": "in_123"}',
    payloadContentType: 'application/json',
    labels: ['environment' => 'production'],
    metadata: ['emitter' => 'billing-worker'],
    occurredAt: new DateTimeImmutable('now', new DateTimeZone('UTC')),
    eventId: null,
);
```

The token goes in without a `Bearer` prefix; the client adds it.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows, either the one set on the `Event` or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `sendEvent` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and throws.

## Bounds, and how to change them

```php example=bounds
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

Those are the defaults. Durations are seconds, as floats.

| Bound | Default |
|-------|---------|
| `maxAttempts` (the first attempt included) | `4`, capped at `RetryPolicy::MAX_ATTEMPTS_CAP` = 16 |
| `initialBackoff` | 0.1 s |
| `maxBackoff` | 2.0 s |
| `maxTotalDelay`, the budget all delays of one send share | 5.0 s |
| `requestTimeout`, per attempt | 10.0 s |
| `maxPayloadBytes` | 1 MiB |
| `maxResponseBytes` | 8 MiB |

`RetryPolicy::disabled()` sends each event exactly once. A payload above the maximum throws before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```php example=verify
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

`Signature::verify` returns nothing and throws `Hook0\ClientError` for every reason a webhook may be refused.

Pass the raw request body. A body that has been parsed and re-serialised no longer hashes to what was signed. `$headers` accepts a map or a list of pairs, and `$tolerance` is seconds.

The clock window is bilateral, so a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

`Signature::verifyWithCurrentTime` takes the same arguments followed by a `DateTimeImmutable`, for holding a signature against a moment you choose.

### Laravel

```php example=laravel
use Hook0\ClientError;
use Hook0\Signature;
use Illuminate\Http\Request;

Route::post('/webhook', function (Request $request) {
    try {
        Signature::verify(
            $request->header('X-Hook0-Signature', ''),
            $request->getContent(),
            $request->headers->all(),
            config('services.hook0.subscription_secret'),
            300.0,
        );
    } catch (ClientError) {
        return response()->json(['error' => 'invalid signature'], 400);
    }

    dispatch(new HandleWebhook($request->json()->all()));

    return response()->noContent();
})->withoutMiddleware([VerifyCsrfToken::class]);
```

`$request->getContent()` is the raw body. `$request->headers->all()` gives each header as a list of values, which the parser reads as pairs.

## Upsert event types

An event whose type the application does not declare is refused. `upsertEventTypes` creates the ones that are missing and returns only those it created:

```php example=upsert
$created = $client->upsertEventTypes([
    'billing.invoice.paid',
    'billing.invoice.voided',
]);
```

An event type is written `service.resource_type.verb`. `Hook0\EventType::parse` reads one and throws `ClientError` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group:

```php example=api
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

:::note `Transport` is final, so a test cannot stand in for it
A generated group takes a `Hook0\Transport`, the concrete class, and that class is `final`. There is no interface behind it and nothing to subclass, so a test cannot hand a group a fake, a recording double or a decorator that adds a header. Where the other SDKs declare a one-method seam a test satisfies without a socket, this one asks for the real HTTP client and a server to point it at. The `$baseUrl` its constructor takes is the whole of what a test can change.
:::

## Errors

| Class | Thrown when |
|-------|-------------|
| `Hook0\ClientError` | A send failed, retries ran out, a payload was too large, an event type was invalid or could not be created, or a signature was refused |
| `Hook0\TransportError` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `$causeName` and `$retryable` |
| `Hook0\DecodeError` | A response body could not be read as the shape it declared |
| `Hook0\Generated\ProblemError` and its subclasses | The API reported a problem |

All of them extend `RuntimeException`.

`sendEvent` and `upsertEventTypes` fold a `TransportError` into a `ClientError` themselves, so a caller of either has one thing to catch. `TransportError` reaches a caller only through the generated API groups, which issue their own requests and let it through unwrapped:

```php example=errors
use Hook0\ClientError;

try {
    $client->sendEvent($event);
} catch (ClientError $refused) {
    $logger->error('event not sent: ' . $refused->getMessage());
}
```

## Links

- **Source**: [clients/php](https://gitlab.com/hook0/hook0/-/tree/master/clients/php)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
