# Hook0 Ruby Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE.txt)
[![Latest Version](https://img.shields.io/gem/v/hook0-client)](https://rubygems.org/gems/hook0-client)

This is the Ruby SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one class per schema Hook0 declares, one exception per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## No dependencies

The SDK reaches the network, verifies signatures and decodes what the API answers with the standard library alone: `net/http` for requests, `openssl` for signatures, `json` for documents, `securerandom` for identifiers. Installing it never drags a transitive dependency into an application that only wanted to send an event.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. **Passing no ID does not mean the ID comes from Hook0** — the value comes from the client, is sent with the request, and is what `send_event` answers.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice; without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

Only what could end differently is retried: a request that got no answer, a server error, and an instance saying it is being reached faster than it accepts. A quota that is spent and a payload the API will not read are reported as is. A retried request Hook0 answers with `EventAlreadyIngested` reports success — an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is reported as an error.

Every send is bounded, and every bound is configurable:

```ruby
require "hook0"

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
    max_response_bytes: 8 * 1024 * 1024,
    max_response_headers: 64,
    max_header_bytes: 64 * 1024,
    max_head_bytes: 16 * 1024
  )
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

Those are the defaults. `Hook0::RetryPolicy.disabled` sends each event exactly once. A payload above the maximum is refused before any request is issued.

## Verifying webhook signatures

```ruby
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

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

## Calling the rest of the API

Every operation Hook0 declares is a method of a generated group, over the transport this gem ships:

```ruby
applications = Hook0::Generated::ApplicationsApi.new(
  Hook0::Transport.new("https://app.hook0.com", token)
)

begin
  application = applications.get(application_id)
rescue Hook0::Generated::NotFoundError
  # every problem the API names is its own exception, all of them `ProblemError`
end
```

A value read out of an answer is one of the classes under `Hook0::Generated`; `to_h` writes one back the way the API reads it. Names travel as the API spells them: a member Ruby keeps for itself — `method`, `until` — is spelled with a trailing underscore in Ruby and under its own name on the wire.

## Getting Started

Run `gem install hook0-client`, or add `gem "hook0-client"` to your Gemfile. Ruby 3.1 or later is required.

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Contributing

`lib/hook0/generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and is rewritten wholesale on every regeneration — a hand edit there is reverted the next time anyone regenerates. Change the generator instead, then run:

```
UPDATE_SDK=ruby cargo test -p hook0-sdkgen sdk_targets
```

Everything else under `lib/` is hand-written and never regenerated, and so is everything under `test/`. What a send retries, the bounds it is held to and how a signature is verified are dictated by the corpus at `clients/conformance`, which the suite of every SDK reads; a verdict changed there fails this client until it agrees again.

The suites run against a real HTTP server on a loopback port and install nothing beyond the linter and the test framework:

```
rubocop
ruby -Ilib -Itest -e 'Dir["test/**/*_test.rb"].sort.each { |f| require File.expand_path(f) }'
```

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)

### LICENSE

Hook0 Ruby SDK is free and open-source. It is released under the [MIT License](../../LICENSE.txt).

This license grants you the freedom to use, modify, distribute, and sublicense the SDK with minimal restrictions. You may use it in both open-source and commercial projects, as long as you include the original copyright notice.
