# Hook0 Lua Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE.txt)

This is the Lua SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one table per schema Hook0 declares, one error kind per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## Two dependencies, and why there are any

Every other Hook0 SDK installs nothing at all. This one declares `luasocket` and `luasec`, and only those, because Lua's standard library has no way to open a socket — not a slow way or an awkward way, none at all. The only escape without a C library is `io.popen`, which is a shell rather than a socket, and no SDK should be shelling out to send a webhook.

Everything else Lua's standard library also lacks is written out here rather than depended on:

- **JSON** — [`src/json.lua`](src/json.lua). Bounded in depth, in size and in how many members one container may hold. It marks an array and an object apart through a metatable, so an empty one still says which it is, and `Json.null` is the value a null actually carried, since `nil` already means absent and a table cannot hold it.
- **SHA-256 and HMAC** — [`src/sha256.lua`](src/sha256.lua), written against FIPS 180-4 and RFC 2104. It is held to the published vectors of both, and to signature codes the shared conformance corpus computed with a general-purpose HMAC tool outside this repository: the suite is not one where the implementation grades its own homework.

`spec/rockspec_spec.lua` fails if a third runtime dependency ever appears.

## Install

```sh
luarocks install hook0-client
```

## Send an event

```lua
local Hook0 = require("hook0")

local client = Hook0.Client.new("https://app.hook0.com/api/v1", application_id, token)

local event_id = client:send_event({
  event_type = "billing.invoice.created",
  payload = '{"invoice":"in_1"}',
  payload_content_type = "application/json",
  labels = { environment = "production" },
})
```

`send_event` answers the identifier the event was sent under. When the event carries no `event_id`, the client mints a UUIDv7 and sends that — which is what makes a retry safe: Hook0 keys events on the identifier, so a repeated request ingests the event once rather than twice.

## Verify a webhook

```lua
local ok, refused = pcall(Hook0.verify_webhook_signature,
  request.headers["x-hook0-signature"],
  request.body,
  request.headers,
  subscription_secret,
  300)

if not ok then
  return 400, Hook0.message(refused)
end
```

The tolerance is bilateral: a delivery dated too far ahead is refused exactly like one dated too far behind, because a window that only looked backwards is one a sender widens by dating its own delivery in the future.

## Match a failure, don't match a message

Every failure this client raises is a table carrying the kind it is, not a string. Kinds chain, so one test covers a whole family:

```lua
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

A failure the API reported also carries `raised.status` and `raised.problem`, the problem document as the API answered it.

## The whole API

```lua
local api = Hook0.api(client.transport)
local secrets = api.ApplicationSecretsApi:list(application_id)
```

One group per entity the API declares, one method per operation. Both are generated: `src/generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and nothing under it is edited by hand.

## Two halves

| | Where | Written by |
|---|---|---|
| Types, problems, one method per operation | `src/generated/` | `UPDATE_SDK=lua cargo test -p hook0-sdkgen sdk_targets` |
| Transport, retries, bounds, signatures, JSON, SHA-256 | `src/*.lua` | by hand |
| The suite | `spec/` | by hand, never regenerated |

The two meet at two seams and nowhere else: the generated code reads its decoders from `hook0.runtime`, and it calls whatever object it was handed as a transport.

## Run the suite

```sh
luarocks install --deps-only hook0-client-*.rockspec
luarocks install busted
luarocks install luacheck

luacheck .
busted
```

Every case goes over a real loopback socket, against a Hook0 API running in a second process: what the client writes on the wire is what the suite reads back. Nothing here stands in for a part of the client.
