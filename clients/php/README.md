# Hook0 PHP Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![Latest Version](https://img.shields.io/packagist/v/hook0/client)](https://packagist.org/packages/hook0/client)

This is the PHP SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one class per schema Hook0 declares, one backed enum per closed list of values, one exception per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## No dependencies

The SDK reaches the network, verifies signatures and decodes what the API answers with the language and the extensions every distribution of it ships: `ext-curl` for requests, `ext-hash` for signatures, `ext-json` for documents, `random_bytes` for identifiers. Installing it never drags a package into an application that only wanted to send an event.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. **Passing no ID does not mean the ID comes from Hook0** — the value comes from the client, is sent with the request, and is what `sendEvent` answers.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice; without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

Only what could end differently is retried: a request that got no answer, a server error, and an instance saying it is being reached faster than it accepts. A quota that is spent and a payload the API will not read are reported as is. A retried request Hook0 answers with `EventAlreadyIngested` reports success — an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is reported as an error.

Every send is bounded, and every bound is configurable:

```php
use Hook0\Client;
use Hook0\Event;
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
    ),
);

$eventId = $client->sendEvent(new Event(
    eventType: 'billing.invoice.paid',
    payload: '{"invoice": "in_123"}',
    payloadContentType: 'application/json',
    labels: ['environment' => 'production'],
));
```

Those are the defaults. `RetryPolicy::disabled()` sends each event exactly once. A payload above the maximum is refused before any request is issued.

What an answer may cost is bounded too, and by this client rather than by whatever its HTTP layer happens to do: the body it reads, the number of header lines, the length of one of them, and the head as a whole.

## Verifying webhook signatures

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

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed. `Signature::verifyWithCurrentTime()` takes the moment to hold the signature against, for a caller that has one.

## Calling the rest of the API

Every operation Hook0 declares is a method of a generated group, over the transport this package ships:

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

A value read out of an answer is one of the classes under `Hook0\Generated`; `toArray()` writes one back the way the API reads it, and `equals()` holds two of them against each other. A closed list of values is a backed enum, so a value the API does not declare cannot be built at all. Names travel as the API spells them, and PHP reads what follows `->`, `::` and `$` as a name rather than as a keyword, so a member called `method`, `until` or `type` is called exactly that on both sides.

## Getting Started

Run `composer require hook0/client`. PHP 8.2 or later is required.

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Contributing

`src/Generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and is rewritten wholesale on every regeneration — a hand edit there is reverted the next time anyone regenerates. Change the generator instead, then run:

```
UPDATE_SDK=php cargo test -p hook0-sdkgen sdk_targets
```

Everything else under `src/` is hand-written and never regenerated, and so is everything under `tests/`. What a send retries, the bounds it is held to and how a signature is verified are dictated by the corpus at `clients/conformance`, which the suite of every SDK reads; a verdict changed there fails this client until it agrees again.

The suites run against a real HTTP server on a loopback port and install nothing beyond the linter and the test runner, neither of which the package itself requires:

```
phpcs
phpunit --no-coverage
```

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)

### LICENSE

Hook0 PHP SDK is free and open-source. It is released under the [MIT License](./LICENSE).

This license grants you the freedom to use, modify, distribute, and sublicense the SDK with minimal restrictions. You may use it in both open-source and commercial projects, as long as you include the original copyright notice.
