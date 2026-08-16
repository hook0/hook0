---
title: "Lua webhook SDK — hook0-client rock"
description: "Send Hook0 events and verify webhook signatures from Lua 5.3 or 5.4. Blocking, two dependencies, retries and payload bounds built in."
keywords: [Lua webhook SDK, Hook0 Lua client, verify webhook signature Lua, OpenResty webhook, luarocks hook0-client, send webhook event Lua]
sdkTarget: lua
---

# Lua SDK

The Hook0 SDK for Lua sends events and verifies webhook signatures. Every call blocks.

It declares two dependencies, `luasocket` and `luasec`, and only those, because Lua's standard library has no way to open a socket. Everything else the client needs is written out in the rock: JSON, SHA-256 and HMAC included.

## Installation

Lua 5.3 or 5.4 is required (`>= 5.3, < 5.5`).

:::warning Not published to LuaRocks yet
`luarocks install hook0-client` does not resolve. No release job publishes this rock, so the only version that exists is the one in the repository.

Build it from the rockspec in a checkout instead.
:::

```bash
git clone https://gitlab.com/hook0/hook0.git
cd hook0/clients/lua
luarocks install --deps-only hook0-client-1.1.0-1.rockspec
luarocks make hook0-client-1.1.0-1.rockspec
```

`luarocks make` builds from the directory you are in rather than fetching the source the rockspec names, which is what you want from a checkout.

## Send an event

```lua example=send
local Hook0 = require("hook0")

local client = Hook0.Client.new("https://app.hook0.com/api/v1", application_id, token)

local event_id = client:send_event({
  event_type = "billing.invoice.paid",
  payload = '{"invoice": "in_123"}',
  payload_content_type = "application/json",
  labels = { environment = "production" },
})
```

An event is a plain table. Three fields are required and four are optional:

```lua example=event
{
  event_type = "billing.invoice.paid",
  payload = '{"invoice": "in_123"}',
  payload_content_type = "application/json",
  labels = { environment = "production" },
  metadata = { emitter = "billing-worker" },
  occurred_at = "2026-08-15T09:30:00Z", -- RFC 3339; the current moment when absent
  event_id = nil,
}
```

The token goes in without a `Bearer` prefix; the client adds it.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the event, or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is raised.

:::note Seed the generator on Lua 5.3
The random half of a generated ID comes from `math.random`. Lua 5.4 seeds it at startup; Lua 5.3 does not, so call `math.randomseed(os.time())` once when your process starts, or two processes started in the same millisecond can mint the same ID.
:::

## Bounds, and how to change them

```lua example=bounds
local client = Hook0.Client.new(
  "https://app.hook0.com/api/v1",
  application_id,
  token,
  Hook0.Options.new({
    retry_policy = Hook0.RetryPolicy.new({
      max_attempts = 4,
      initial_backoff = 0.1,
      max_backoff = 2.0,
      max_total_delay = 5.0,
    }),
    request_timeout = 10.0,
    max_payload_bytes = 1024 * 1024,
    max_response_bytes = 8 * 1024 * 1024,
  })
)
```

Those are the defaults. Durations are seconds.

| Bound | Default |
|-------|---------|
| `max_attempts` (the first attempt included) | `4`, capped at `Hook0.RetryPolicy.MAX_ATTEMPTS_CAP` = 16 |
| `initial_backoff` | 0.1 s |
| `max_backoff` | 2.0 s |
| `max_total_delay`, the budget all delays of one send share | 5.0 s |
| `request_timeout`, per attempt | 10.0 s |
| `max_payload_bytes` | 1 MiB |
| `max_response_bytes` | 8 MiB |

`Hook0.RetryPolicy.disabled()` sends each event exactly once. A payload above the maximum is refused before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```lua example=verify
local ok, refused = pcall(
  Hook0.verify_webhook_signature,
  request.headers["x-hook0-signature"],
  request.body,
  request.headers,
  subscription_secret,
  300
)

if not ok then
  return 400, Hook0.message(refused)
end
```

Verification raises rather than returning a flag, so wrap it in `pcall`. Pass the raw request body: a body that has been parsed and re-serialised no longer hashes to what was signed. `headers` accepts a table keyed by name or a list of `{name, value}` pairs, and the tolerance is seconds.

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

`Hook0.verify_webhook_signature_with_current_time` takes the same arguments followed by a number of seconds since the epoch, for holding a signature against a moment you choose.

## Match a failure, do not match a message

Every failure this client raises is a table carrying the kind it is, never a string. Kinds chain, so one test covers a whole family:

```lua example=match
local ok, raised = pcall(function() return client:send_event(event) end)

if not ok then
  if Hook0.is(raised, Hook0.Generated.errors.TooManyEventsTodayError) then
    -- a daily quota, which clears at the turn of the day
  elseif Hook0.is(raised, Hook0.TransportError) then
    -- the API answered nothing at all; `raised.cause` says which of the three it was
  elseif Hook0.is(raised, Hook0.ClientError) then
    -- anything else this client raises
  end
end
```

A failure the API reported also carries `raised.status` and `raised.problem`, the problem document as the API answered it. `Hook0.message(raised)` reads the message off any of them.

The kinds are `Hook0.ClientError` at the root, with `Hook0.TransportError`, `Hook0.DecodeError` and `Hook0.Generated.errors.ProblemError` under it.

## Upsert event types

An event whose type the application does not declare is refused. `upsert_event_types` creates the ones that are missing and returns only those it created:

```lua example=upsert
local created = client:upsert_event_types({
  "billing.invoice.paid",
  "billing.invoice.voided",
})
```

An event type is written `service.resource_type.verb`. `Hook0.EventType.parse` reads one and raises on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. `Hook0.api` builds one group per entity over a transport:

```lua example=api
local api = Hook0.api(client.transport)
local secrets = api.ApplicationSecretsApi:list(application_id)
```

One group per entity the API declares, one method per operation, and one error kind per problem it can report.

## Links

- **Source**: [clients/lua](https://gitlab.com/hook0/hook0/-/tree/master/clients/lua)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
