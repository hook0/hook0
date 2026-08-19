# Hook0 Python Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE.txt)
[![Latest Version](https://img.shields.io/pypi/v/hook0-client)](https://pypi.org/project/hook0-client/)
[![Supported Python versions](https://img.shields.io/pypi/pyversions/hook0-client)](https://pypi.org/project/hook0-client/)

This is the Python SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded, from a blocking or an awaiting application.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one class per schema Hook0 declares, one exception per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## No dependencies

The SDK reaches the network, verifies signatures and decodes what the API answers with the standard library alone: `urllib.request` for the blocking client, `asyncio` for the awaiting one, `hmac` and `hashlib` for signatures. Installing it never drags a transitive dependency into an application that only wanted to send an event.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. **Passing no ID does not mean the ID comes from Hook0** — the value comes from the client, is sent with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice; without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` naming a spent quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` the answer carries is honoured and clamped to what is left of the delay budget. A retried request Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is reported as an error.

Every send is bounded. The bounds below are yours to set. Three more are not. The head of an answer is held to `MAX_HEADERS`, `MAX_LINE_BYTES` and `MAX_HEAD_BYTES` in `hook0.transport`, constants rather than options, since nothing a caller sets makes an oversized head safe to read.

```python
from hook0 import Event, Hook0Client, Hook0ClientOptions, RetryPolicy

client = Hook0Client(
    "https://app.hook0.com/api/v1",
    application_id,
    token,
    Hook0ClientOptions(
        retry_policy=RetryPolicy(
            max_attempts=4,
            initial_backoff=0.1,
            max_backoff=2.0,
            max_total_delay=5.0,
        ),
        request_timeout=10.0,
        max_payload_bytes=1024 * 1024,
        max_response_bytes=8 * 1024 * 1024,
    ),
)

event_id = client.send_event(
    Event(
        event_type="billing.invoice.paid",
        payload='{"invoice": "in_123"}',
        payload_content_type="application/json",
        labels={"environment": "production"},
    )
)
```

Those are the defaults. `RetryPolicy.disabled()` sends each event exactly once. A payload above the maximum is refused before any request is issued.

## Awaiting instead of waiting

`Hook0AsyncClient` applies the same bounds, reads the same answers and retries for the same reasons; it awaits where the other waits.

```python
from hook0 import Hook0AsyncClient

client = Hook0AsyncClient("https://app.hook0.com/api/v1", application_id, token)
event_id = await client.send_event(event)
```

## Verifying webhook signatures

```python
from hook0 import Hook0ClientError, verify_webhook_signature

try:
    verify_webhook_signature(
        request.headers["X-Hook0-Signature"],
        request.body,
        request.headers,
        subscription_secret,
        tolerance=300,
    )
except Hook0ClientError as refused:
    ...  # answer 400, and do not act on the delivery
```

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

## Calling the rest of the API

Every operation Hook0 declares is a method of a generated group, over either transport:

```python
from hook0 import HttpTransport
from hook0.generated import ApplicationsApi
from hook0.generated.errors import NotFoundError

applications = ApplicationsApi(HttpTransport("https://app.hook0.com", token))

try:
    application = applications.get(application_id)
except NotFoundError as reported:
    ...  # every problem the API names is its own exception, all of them `ProblemError`
```

`hook0.generated.aio` carries the same groups for an application that awaits.

## Getting Started

Run `pip install hook0-client` in your project's environment. Python 3.11 or later is required.

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Contributing

`src/hook0/generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and is rewritten wholesale on every regeneration — a hand edit there is reverted the next time anyone regenerates. Change the generator instead, then run:

```
UPDATE_SDK=python cargo test -p hook0-sdkgen sdk_targets
```

Everything else under `src/hook0/` is hand-written and never regenerated, and so is everything under `tests/`.

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)

### LICENSE

Hook0 Python SDK is free and open-source. It is released under the [MIT License](../../LICENSE.txt).

This license grants you the freedom to use, modify, distribute, and sublicense the SDK with minimal restrictions. You may use it in both open-source and commercial projects, as long as you include the original copyright notice.
