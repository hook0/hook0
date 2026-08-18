# Hook0 Zig Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE.txt)

This is the Zig SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one struct per schema Hook0 declares, one error per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## Zig version

**0.16.0**, and only that one. The package is built on `std.Io` — the interface that carries the clock, the randomness and the sockets — which is new in this release and is not the API 0.15 or earlier offered. The same version is pinned in `.gitlab-ci.yml`, by name and by the checksum of the tarball it names, so an upstream release is a change somebody made rather than a pipeline that turned red on its own.

## No dependencies

`build.zig.zon` declares `.dependencies = .{}`, and the suite fails if that ever stops being true. Everything this client needs, Zig's standard library already has: `std.crypto.auth.hmac.sha2.HmacSha256` for signatures, `std.json` for documents, `std.Io.net` for sockets. The HTTP/1.1 the transport speaks is written out in [`src/transport.zig`](src/transport.zig) rather than taken from `std.http`, because what this client needs from a request is the four ceilings the shared conformance corpus names — how much head, how many headers, how long one header, how much body — applied on the line that crosses each, and a client of `std.http.Client` applies them after the fact.

## Install

There is no central registry for Zig packages, so "publishing" is a tag and a URL: `zig fetch` resolves a tarball or a git ref and writes the hash it got into your `build.zig.zon`. The URL is the tag archive of `github.com/hook0/hook0-zig`, the read-only mirror this directory is pushed to on every SDK release — a mirror rather than the monorepo because `zig fetch` needs an archive whose root is the package:

```sh
zig fetch --save=hook0 https://github.com/hook0/hook0-zig/archive/refs/tags/v1.1.0.tar.gz
```

That writes a `.hash` beside the URL, and every later build is held to it: the bytes you built against are the bytes anyone else building your project gets. Then, in your `build.zig`:

```zig
const hook0 = b.dependency("hook0", .{ .target = target, .optimize = optimize });
exe.root_module.addImport("hook0", hook0.module("hook0"));
```

## Send an event

```zig
const hook0 = @import("hook0");

var client: hook0.Client = .init(io, "https://app.hook0.com/api/v1", application_id, token, .{});

const sent = try client.sendEvent(allocator, .{
    .event_type = "billing.invoice.created",
    .payload = "{\"invoice\":\"in_1\"}",
    .payload_content_type = "application/json",
    .labels = &.{.{ .key = "environment", .value = "production" }},
});
defer sent.deinit();

std.log.info("ingested as {s}", .{sent.value});
```

`sendEvent` answers the identifier the event was sent under. When the event carries no `event_id`, the client mints a UUIDv7 and sends that — which is what makes a retry safe: Hook0 keys events on the identifier, so a repeated request ingests the event once rather than twice.

The `io` is the caller's. Nothing here reaches a clock, a source of randomness or a socket on its own, which is what lets the suite move the clock rather than wait for it.

## Everything read is owned

Every call that reads something back answers an `Owned(T)`: the value, and the arena everything it points into was allocated from.

```zig
const secrets = try api.ApplicationSecretsApi.list(allocator, application_id);
defer secrets.deinit();

for (secrets.value) |secret| std.log.info("{s}", .{secret.token});
```

One `deinit` frees the body that arrived, the document it was parsed into and every slice read out of it. Nothing outlives it by accident, and nothing is reachable afterwards — which is also how the corpus's ceilings take effect here: a bound the other SDKs keep by discipline is one the allocator applies, since nothing is read into memory the caller did not hand over.

## Match an error, don't match a message

Zig errors carry no payload, so what another SDK would have said in a sentence is said by the name of the error. Every failure is a member of a declared set:

```zig
const sent = client.sendEvent(allocator, event) catch |failed| switch (failed) {
    error.TooManyEventsToday => {
        // a daily quota, which clears at the turn of the day
    },
    error.PayloadTooLarge => {
        // refused before a socket was opened
    },
    error.RetriesExhausted => {
        // every attempt the policy allowed was made; `client.detail` says what the last one met
    },
    else => return failed,
};
```

`client.detail` carries what an error cannot: the problem document the API answered, or how many attempts were spent over how long.

## Verify a webhook

```zig
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

The tolerance is bilateral: a delivery dated too far ahead is refused exactly like one dated too far behind, because a window that only looked backwards is one a sender widens by dating its own delivery in the future.

## The whole API

```zig
var group: hook0.api.ApplicationSecretsApi = .init(allocator, client.transportOf());
defer group.deinit();

const secrets = try group.list(allocator, application_id);
defer secrets.deinit();
```

One group per entity the API declares, one method per operation. The allocator a group is built with
is not the one its calls are handed: a call frees everything it allocated on its way out, while
`group.reported` — the status, the problem document and the message of the last call that failed — is
read after the call has returned, so it is held apart from the call and let go of by `group.deinit()`.

Both are generated: `src/generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and nothing under it is edited by hand.

## Two halves

| | Where | Written by |
|---|---|---|
| Types, problems, one method per operation | `src/generated/` | `UPDATE_SDK=zig cargo test -p hook0-sdkgen sdk_targets` |
| Transport, retries, bounds, signatures | `src/*.zig` | by hand |
| The suite | `tests/` | by hand, never regenerated |

The two meet at two seams and nowhere else: the generated code reads its decoders from `runtime.zig`, and it issues requests through whatever value it was handed as a `runtime.Transport`.

## Run the suite

```sh
zig fmt --check .
zig build test
```

Every case goes over a real loopback socket, against a Hook0 API on a thread of its own: what the client writes on the wire is what the suite reads back. Nothing here stands in for a part of the client.

[`tests/property_test.zig`](tests/property_test.zig) draws its inputs from a fixed seed and keeps the counter-examples worth keeping in [`tests/regressions/`](tests/regressions), so a failure is one somebody reproduces by running the suite again rather than one that goes away on a retry.

## One thing this client does not do yet

There is no timeout on opening the connection. `std.Io.net.ConnectOptions` declares one in 0.16 and the POSIX backend panics with `TODO implement netConnectIpPosix with timeout` when it is set, so setting it would trade a hang for a crash. Everything after the connection — the request, every read of the answer, the whole exchange — is held to `request_timeout_ms`, and the connect is left to the operating system's own. [`src/transport.zig`](src/transport.zig) says so where it happens.
