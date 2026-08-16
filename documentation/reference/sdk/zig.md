---
title: "Zig webhook SDK — hook0_client"
description: "Send Hook0 events and verify webhook signatures from Zig 0.16. Blocking, no dependencies, clock and sockets injected through std.Io. Fetched from source, not a registry."
keywords: [Zig webhook SDK, Hook0 Zig client, verify webhook signature Zig, zig fetch dependency, std.Io HTTP client, send webhook event Zig]
sdkTarget: zig
---

# Zig SDK

The Hook0 SDK for Zig sends events and verifies webhook signatures. It is blocking, and every call takes the caller's `std.Io`: the clock, the source of randomness and the sockets all come through it, so nothing here reaches a global.

The package declares no dependencies. HMAC-SHA256 comes from `std.crypto`, documents from `std.json`, and the HTTP/1.1 the transport speaks is written out in the package rather than taken from `std.http`.

## Zig version

**0.16.0**, and only that one. The package is built on `std.Io`, which is new in that release and is not the API 0.15 or earlier offered.

## Installation

:::info Zig has no package registry
There is nowhere to publish a Zig package to. `zig fetch` resolves a path, a tarball or a git ref and records the hash it got.

The package manifest lives in `clients/zig` rather than at the root of the Hook0 repository, so fetching the repository URL does not find it. Depend on a checkout.
:::

```bash
git clone https://gitlab.com/hook0/hook0.git
cd your-project
zig fetch --save=hook0 ../hook0/clients/zig
```

That writes the dependency and the hash of what it fetched into your `build.zig.zon`, and every later build is held to it.

Use `--save=hook0` rather than a bare `--save`. The manifest names the package `hook0_client`, and a bare `--save` records it under that name, which is then the name `b.dependency` needs.

In your `build.zig`:

```zig example=program
const hook0 = b.dependency("hook0", .{ .target = target, .optimize = optimize });
exe.root_module.addImport("hook0", hook0.module("hook0"));
```

## Send an event

```zig example=send
const hook0 = @import("hook0");

var client: hook0.Client = .init(io, "https://app.hook0.com/api/v1", application_id, token, .{});

const sent = try client.sendEvent(allocator, .{
    .event_type = "billing.invoice.paid",
    .payload = "{\"invoice\":\"in_123\"}",
    .payload_content_type = "application/json",
    .labels = &.{.{ .key = "environment", .value = "production" }},
});
defer sent.deinit();

std.log.info("ingested as {s}", .{sent.value});
```

`Client` holds only slices the caller owns, so there is no `deinit` on it. What a call returns is an `Owned(T)`, which owns the arena everything it points into came from: one `deinit` frees the identifier, the body that was sent, and everything read back.

`Event` has three required fields and four optional ones:

```zig example=event
hook0.Event{
    .event_type = "billing.invoice.paid",
    .payload = "{\"invoice\":\"in_123\"}",
    .payload_content_type = "application/json",
    .labels = &.{.{ .key = "environment", .value = "production" }},
    .metadata = &.{.{ .key = "emitter", .value = "billing-worker" }},
    .occurred_at = "2026-08-15T09:30:00Z", // RFC 3339; the current moment when null
    .event_id = null,
}
```

Labels and metadata are slices of `Label`, which is `struct { key: []const u8, value: []const u8 }`.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set on `event_id`, or a UUIDv7 it mints when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `sendEvent` answers.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and fails.

`hook0.generateEventId(io, &buffer)` writes a UUIDv7 into a `[36]u8` if you want the ID before the send.

## Bounds, and how to change them

```zig example=configure
var client: hook0.Client = .init(io, api_url, application_id, token, .{
    .retry_policy = .{
        .max_attempts = 4,
        .initial_backoff_ms = 100,
        .max_backoff_ms = 2_000,
        .max_total_delay_ms = 5_000,
    },
    .request_timeout_ms = 10_000,
    .max_payload_bytes = 1024 * 1024,
    .max_response_bytes = 8 * 1024 * 1024,
});
```

Those are the defaults, which is what `.{}` gives.

| Bound | Default |
|-------|---------|
| `max_attempts` (the first attempt included) | `4`, capped at `RetryPolicy.max_attempts_cap` = 16 |
| `initial_backoff_ms` | 100 |
| `max_backoff_ms` | 2 000 |
| `max_total_delay_ms`, the budget all delays of one send share | 5 000 |
| `request_timeout_ms`, per attempt | 10 000 |
| `max_payload_bytes` | 1 MiB |
| `max_response_bytes` | 8 MiB |
| `max_response_headers` | 64 |
| `max_header_bytes` | 64 KiB |
| `max_head_bytes` | 16 KiB |

`hook0.RetryPolicy.disabled` is a constant, not a call, and sends each event exactly once. A payload above the maximum is refused before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

:::caution Opening the connection is not bounded
`request_timeout_ms` bounds everything after the connection is open. Zig 0.16 declares a connect timeout in `std.Io.net.ConnectOptions` but panics when it is set, so this client leaves the connect to the operating system.
:::

## Verify a webhook signature

```zig example=verify
hook0.verifyWebhookSignature(
    io,
    delivered.header("x-hook0-signature").?,
    delivered.body,
    &headers,
    subscription_secret,
    300,
) catch |refused| switch (refused) {
    error.OutsideTolerance => return .{ .status = 400 },
    else => return .{ .status = 400 },
};
```

The tolerance is seconds. Nothing here allocates, and the moment comes from the `io` the caller hands in rather than from a clock the package reaches on its own, which is what lets a test move the window rather than wait for it.

Pass the raw body. A body that has been parsed and re-serialised no longer hashes to what was signed. `headers` is a slice of `hook0.signature.Header`, which is `struct { name: []const u8, value: []const u8 }`.

The clock window is bilateral: a delivery dated too far ahead is refused exactly like one dated too far behind. A header the signature covers but the request did not carry is refused before any code is computed.

`hook0.verifyWebhookSignatureWithCurrentTime` takes the same arguments without the `io`, followed by the moment to hold the signature against, as seconds since the epoch.

The error set is closed:

```zig example=errors
pub const SignatureError = error{
    CodeNotHexadecimal,
    HeaderNotDelivered,
    CodeMismatch,
    OutsideTolerance,
    Unreadable,
};
```

## Upsert event types

An event whose type the application does not declare is refused. `upsertEventTypes` creates the ones that are missing and answers only those it created:

```zig example=upsert
const created = try client.upsertEventTypes(allocator, &.{
    "billing.invoice.paid",
    "billing.invoice.voided",
});
defer created.deinit();
```

An event type is written `service.resource_type.verb`. `hook0.EventType.parse` reads one and answers `error.NotAnEventType` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group, built on the transport the client already holds:

```zig example=api_group
var group: hook0.api.ApplicationSecretsApi = .{ .transport = client.transportOf() };
const secrets = try group.list(allocator, application_id);
defer secrets.deinit();
```

One group per entity, one method per operation, and one member of `hook0.errors.Failure` per problem the API can report. A group carries `reported`, which holds the status and the problem document of its last failure, since an error value alone cannot.

## Errors

`sendEvent` has an inferred error set, so it can answer an allocator failure or a transport failure as well as the four it names itself:

| Error | Answered when |
|-------|---------------|
| `error.PayloadTooLarge` | The payload crossed `max_payload_bytes`, before any request went out |
| `error.Refused` | The API refused the event and repeating the request would not help |
| `error.RetriesExhausted` | Every attempt failed |
| `error.Unreadable` | The API accepted the event but answered something the client could not read |
| `error.NoAnswer`, `error.AnswerAboveABound`, `error.UnusableApiUrl` | From `hook0.TransportError` |

`client.detail` carries what the last failure said, which an error value cannot. `hook0.Cause` maps a `TransportError` to a name and to whether repeating the request could end differently.

## Links

- **Source**: [clients/zig](https://gitlab.com/hook0/hook0/-/tree/master/clients/zig)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
