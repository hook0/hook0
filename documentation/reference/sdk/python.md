---
title: "Python webhook SDK — hook0-client"
description: "Send Hook0 events and verify webhook signatures from Python, blocking or with asyncio. Install from PyPI, no third-party dependencies, Python 3.11 or later."
keywords: [Python webhook SDK, Hook0 Python client, hook0-client PyPI, verify webhook signature Python, asyncio webhook client, send webhook event Python]
---

# Python SDK

The Hook0 SDK for Python sends events and verifies webhook signatures. It ships a blocking client and an `asyncio` one that share the same behaviour.

The SDK has no third-party dependencies. It reaches the network with `urllib.request` and `asyncio`, and computes signatures with `hmac` and `hashlib`, so installing it never pulls anything else into your environment.

## Installation

```bash
pip install hook0-client
```

Python 3.11 or later is required. The distribution is named `hook0-client`; the import name is `hook0`.

## Send an event

```python
from hook0 import Event, Hook0Client

client = Hook0Client(
    "https://app.hook0.com/api/v1",
    application_id,
    token,
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

`Event` carries three required fields and four optional ones:

```python
Event(
    event_type="billing.invoice.paid",
    payload='{"invoice": "in_123"}',
    payload_content_type="application/json",
    labels={"environment": "production"},
    metadata={"emitter": "billing-worker"},
    occurred_at=datetime.now(tz=timezone.utc),
    event_id=None,
)
```

The token goes in without a `Bearer` prefix; the client adds it.

## Awaiting instead of waiting

`Hook0AsyncClient` applies the same bounds, reads the same answers and retries for the same reasons. It awaits where the other waits.

```python
from hook0 import Event, Hook0AsyncClient

client = Hook0AsyncClient("https://app.hook0.com/api/v1", application_id, token)

event_id = await client.send_event(
    Event(
        event_type="billing.invoice.paid",
        payload='{"invoice": "in_123"}',
        payload_content_type="application/json",
    )
)
```

Both classes take the same four arguments and expose the same two methods, `send_event` and `upsert_event_types`. Nothing else differs.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and raises.

## Bounds, and how to change them

Every send is bounded, and every bound is configurable:

```python
from hook0 import Hook0Client, Hook0ClientOptions, RetryPolicy

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
```

Those are the defaults, exported as `DEFAULT_REQUEST_TIMEOUT`, `DEFAULT_MAX_PAYLOAD_BYTES` and `DEFAULT_MAX_RESPONSE_BYTES`. Durations are seconds, as floats.

| Bound | Default |
|-------|---------|
| `max_attempts` (the first attempt included) | `4`, capped at `MAX_ATTEMPTS_CAP` = 16 |
| `initial_backoff` | 0.1 s |
| `max_backoff` | 2.0 s |
| `max_total_delay`, the budget all delays of one send share | 5.0 s |
| `request_timeout`, per attempt | 10.0 s |
| `max_payload_bytes` | 1 MiB |
| `max_response_bytes` | 8 MiB |

`RetryPolicy.disabled()` sends each event exactly once. A payload above the maximum raises before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```python
from hook0 import Hook0ClientError, verify_webhook_signature

try:
    verify_webhook_signature(
        request.headers["X-Hook0-Signature"],
        request.body,
        request.headers,
        subscription_secret,
        300,
    )
except Hook0ClientError:
    ...  # answer 400, and do not act on the delivery
```

The function returns `True` and raises `Hook0ClientError` for every reason a webhook may be refused. It never returns `False`.

Pass the raw request body as `bytes`. A body that has been parsed and re-serialised no longer hashes to what was signed. `headers` accepts a mapping or an iterable of pairs, and `tolerance` is seconds.

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

To hold the signature against a moment you choose, in a test for instance, use `verify_webhook_signature_with_current_time`, which takes the same arguments followed by a `datetime`.

### Flask

```python
from flask import Flask, request
from hook0 import Hook0ClientError, verify_webhook_signature

app = Flask(__name__)

@app.post("/webhook")
def webhook():
    try:
        verify_webhook_signature(
            request.headers["X-Hook0-Signature"],
            request.get_data(),
            dict(request.headers),
            subscription_secret,
            300,
        )
    except (KeyError, Hook0ClientError):
        return {"error": "invalid signature"}, 400

    payload = request.get_json()
    return {"status": "processed"}, 200
```

`request.get_data()` is the raw body, before Flask parses it.

## Upsert event types

An event whose type the application does not declare is refused. `upsert_event_types` creates the ones that are missing and returns only those it created:

```python
created = client.upsert_event_types(
    [
        "billing.invoice.paid",
        "billing.invoice.voided",
    ]
)
```

An event type is written `service.resource_type.verb`. `EventType.parse` reads one and raises `Hook0ClientError` on anything else:

```python
from hook0 import EventType

parsed = EventType.parse("billing.invoice.paid")
print(parsed.service, parsed.resource_type, parsed.verb)
```

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group, and every problem it reports is its own exception:

```python
from hook0 import HttpTransport
from hook0.generated import ApplicationsApi
from hook0.generated.errors import NotFoundError

applications = ApplicationsApi(HttpTransport("https://app.hook0.com", token))

try:
    application = applications.get(application_id)
except NotFoundError:
    ...
```

`hook0.generated.aio` carries the same groups for an application that awaits, each named with an `AsyncApi` suffix, and `AsyncHttpTransport` is the transport to give them.

Every generated exception derives from `ProblemError`, which carries the HTTP `status` and the parsed `problem`, so you can catch one problem or all of them.

## Errors

| Exception | Raised when |
|-----------|-------------|
| `Hook0ClientError` | A send failed, retries ran out, a payload was too large, an event type was invalid or could not be created, or a signature was refused |
| `TransportError` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `transient` |
| `DecodeError` | A response body could not be read as the shape it declared |
| `ProblemError` and its subclasses | The API reported a problem, from the generated groups |

`Hook0ClientError` carries its detail in the message:

```python
from hook0 import Hook0ClientError

try:
    client.send_event(event)
except Hook0ClientError as refused:
    logger.error("event not sent: %s", refused)
```

## Links

- **Package**: [hook0-client on PyPI](https://pypi.org/project/hook0-client/)
- **Source**: [clients/python](https://gitlab.com/hook0/hook0/-/tree/master/clients/python)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
