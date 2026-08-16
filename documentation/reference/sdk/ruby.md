---
title: "Ruby webhook SDK — hook0-client gem"
description: "Send Hook0 events and verify webhook signatures from Ruby. No runtime dependencies, retries and payload bounds built in, Ruby 3.1 or later."
keywords: [Ruby webhook SDK, Hook0 Ruby client, hook0-client gem, verify webhook signature Ruby, Rails webhook endpoint, send webhook event Ruby]
sdkTarget: ruby
---

# Ruby SDK

The Hook0 SDK for Ruby sends events and verifies webhook signatures. Every call blocks; concurrency is yours to arrange.

The gem declares no runtime dependencies. Sockets, HMAC and JSON all come from the standard library, so installing it adds one gem to your bundle and nothing else.

## Installation

```bash
gem install hook0-client
```

Or in a `Gemfile`:

```ruby example=gemfile
gem "hook0-client"
```

Ruby 3.1 or later is required.

## Send an event

```ruby example=send
require "hook0"

client = Hook0::Client.new(
  "https://app.hook0.com/api/v1",
  application_id,
  token
)

event_id = client.send_event(
  Hook0::Event.new(
    event_type: "billing.invoice.paid",
    payload: '{"invoice": "in_123"}',
    payload_content_type: "application/json",
    labels: { "environment" => "production" }
  )
)
```

`Hook0::Event` takes three required keywords and four optional ones:

```ruby example=event
Hook0::Event.new(
  event_type: "billing.invoice.paid",
  payload: '{"invoice": "in_123"}',
  payload_content_type: "application/json",
  labels: { "environment" => "production" },
  metadata: { "emitter" => "billing-worker" },
  occurred_at: Time.now.utc,
  event_id: nil
)
```

The token goes in without a `Bearer` prefix; the client adds it.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and raises.

## Bounds, and how to change them

```ruby example=options
client = Hook0::Client.new(
  "https://app.hook0.com/api/v1",
  application_id,
  token,
  Hook0::Options.new(
    retry_policy: Hook0::RetryPolicy.new(
      max_attempts: 4,
      initial_backoff: 0.1,
      max_backoff: 2.0,
      max_total_delay: 5.0
    ),
    request_timeout: 10.0,
    max_payload_bytes: 1024 * 1024,
    max_response_bytes: 8 * 1024 * 1024
  )
)
```

Those are the defaults. Durations are seconds, as floats.

| Bound | Default |
|-------|---------|
| `max_attempts` (the first attempt included) | `4`, capped at `Hook0::RetryPolicy::MAX_ATTEMPTS_CAP` = 16 |
| `initial_backoff` | 0.1 s |
| `max_backoff` | 2.0 s |
| `max_total_delay`, the budget all delays of one send share | 5.0 s |
| `request_timeout`, per attempt | 10.0 s |
| `max_payload_bytes` | 1 MiB |
| `max_response_bytes` | 8 MiB |

`Hook0::RetryPolicy.disabled` sends each event exactly once. A payload above the maximum raises before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```ruby example=verify
begin
  Hook0.verify_webhook_signature(
    request.headers["X-Hook0-Signature"],
    request.body,
    request.headers,
    subscription_secret,
    300
  )
rescue Hook0::ClientError
  # answer 400, and do not act on the delivery
end
```

The method returns nothing and raises `Hook0::ClientError` for every reason a webhook may be refused.

Pass the raw request body. A body that has been parsed and re-serialised no longer hashes to what was signed. `headers` accepts a hash or an array of pairs, and `tolerance` is seconds.

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

`Hook0.verify_webhook_signature_with_current_time` takes the same arguments followed by a `Time`, for holding a signature against a moment you choose.

### Rails

```ruby example=rails
class WebhooksController < ApplicationController
  skip_before_action :verify_authenticity_token

  def create
    request.body.rewind
    body = request.body.read

    Hook0.verify_webhook_signature(
      request.headers["X-Hook0-Signature"],
      body,
      request.headers.to_h.transform_keys(&:to_s),
      ENV.fetch("HOOK0_SUBSCRIPTION_SECRET"),
      300
    )

    process_delivery(JSON.parse(body))
    head :ok
  rescue Hook0::ClientError
    head :bad_request
  end
end
```

Read `request.body` yourself rather than `params`. Rails has already reshaped `params` by the time the action runs, and the signature covers the bytes that arrived.

## Upsert event types

An event whose type the application does not declare is refused. `upsert_event_types` creates the ones that are missing and returns only those it created:

```ruby example=upsert
created = client.upsert_event_types(
  %w[billing.invoice.paid billing.invoice.voided]
)
```

An event type is written `service.resource_type.verb`. `Hook0::EventType.parse` reads one and raises `Hook0::ClientError` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group:

```ruby example=rest
applications = Hook0::Generated::ApplicationsApi.new(
  Hook0::Transport.new("https://app.hook0.com", token)
)

begin
  application = applications.get(application_id)
rescue Hook0::Generated::NotFoundError
  # every problem the API names is its own exception, all of them `ProblemError`
end
```

## Errors

| Class | Raised when |
|-------|-------------|
| `Hook0::ClientError` | A send failed, retries ran out, a payload was too large, an event type was invalid or could not be created, or a signature was refused |
| `Hook0::TransportError` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `cause_name` and answers `retryable?` |
| `Hook0::Runtime::DecodeError` | A response body could not be read as the shape it declared |
| `Hook0::Generated::ProblemError` and its subclasses | The API reported a problem |

```ruby example=errors
begin
  client.send_event(event)
rescue Hook0::TransportError => refused
  logger.warn("no answer from Hook0 (#{refused.cause_name}), retryable: #{refused.retryable?}")
rescue Hook0::ClientError => refused
  logger.error("event not sent: #{refused.message}")
end
```

## Links

- **Gem**: [hook0-client on RubyGems](https://rubygems.org/gems/hook0-client)
- **Source**: [clients/ruby](https://gitlab.com/hook0/hook0/-/tree/master/clients/ruby)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
