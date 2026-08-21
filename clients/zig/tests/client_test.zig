//! What a send does, driven against a real socket.
//!
//! Everything the shared corpus dictates is exercised in `conformance_test.zig`; what is here is what
//! this client decides for itself: the identifier it mints, the bounds it applies to what it sends,
//! and what a caller is left holding when a send does not land.

const std = @import("std");
const hook0 = @import("hook0");

const helper = @import("helper.zig");

const io = std.testing.io;
const allocator = std.testing.allocator;

/// The shape a UUID has, whichever version it carries.
fn isUuid(written: []const u8) bool {
    if (written.len != 36) return false;
    for (written, 0..) |character, index| {
        const dashed = index == 8 or index == 13 or index == 18 or index == 23;
        if (dashed) {
            if (character != '-') return false;
        } else if (std.fmt.charToDigit(character, 16) catch null == null) {
            return false;
        }
    }
    return true;
}

/// The event identifier one request carried, as the API read it.
fn eventIdOf(arena: std.mem.Allocator, body: []const u8) ![]const u8 {
    const parsed = try std.json.parseFromSlice(std.json.Value, arena, body, .{});
    return parsed.value.object.get("event_id").?.string;
}

test "a send answers the identifier it minted, and mints one shaped like a UUIDv7" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{helper.ingested(helper.ingested_id)};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(1));
    const owned = try built.sendEvent(held, helper.anEvent());
    defer owned.deinit();

    try std.testing.expectEqualStrings(helper.ingested_id, owned.value);
    try std.testing.expectEqual(@as(usize, 1), api.count());

    const minted = try eventIdOf(held, api.at(0).?.body);
    std.testing.expect(isUuid(minted)) catch |failed| {
        std.debug.print("`{s}` is not shaped like a UUID\n", .{minted});
        return failed;
    };
    try std.testing.expectEqual(@as(u8, '7'), minted[14]);
}

test "a send sends the identifier the caller set, when the caller set one" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{helper.ingested(helper.ingested_id)};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var event = helper.anEvent();
    event.event_id = helper.ingested_id;

    var built = try helper.client(io, held, api, helper.options(1));
    const owned = try built.sendEvent(held, event);
    defer owned.deinit();

    try std.testing.expectEqualStrings(helper.ingested_id, try eventIdOf(held, api.at(0).?.body));
}

test "a send repeats one identifier across every attempt it makes" {
    // The whole reason the identifier is minted here rather than by the API: a request repeated
    // after a failure has to be the same request, or Hook0 ingests the event twice and delivers it
    // to every subscriber twice.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const unavailable: helper.Scripted = .{ .status = 500, .body = "{\"id\":\"InternalServerError\",\"status\":500}" };
    const scripted = [_]helper.Scripted{ unavailable, unavailable, helper.ingested(helper.ingested_id) };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(4));
    const owned = try built.sendEvent(held, helper.anEvent());
    defer owned.deinit();

    try std.testing.expectEqual(@as(usize, 3), api.count());

    // That every attempt carried the *same* identifier says nothing on its own: a client that sent
    // none at all would satisfy it too, and that is the client whose retries ingest the event twice.
    const first = try eventIdOf(held, api.at(0).?.body);
    std.testing.expect(isUuid(first)) catch |failed| {
        std.debug.print("`{s}` is not an identifier this client minted\n", .{first});
        return failed;
    };

    for (1..api.count()) |index| {
        try std.testing.expectEqualStrings(first, try eventIdOf(held, api.at(index).?.body));
    }
}

test "a conflict on a repeated attempt is that attempt having landed" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{
        .{ .status = 500, .body = "{\"id\":\"InternalServerError\",\"status\":500}" },
        .{ .status = 409, .body = "{\"id\":\"EventAlreadyIngested\",\"status\":409}" },
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(4));
    const owned = try built.sendEvent(held, helper.anEvent());
    defer owned.deinit();

    try std.testing.expectEqual(@as(usize, 2), api.count());
    try std.testing.expectEqualStrings(try eventIdOf(held, api.at(0).?.body), owned.value);
}

test "a conflict on a first attempt is the conflict it is" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{
        .{ .status = 409, .body = "{\"id\":\"EventAlreadyIngested\",\"status\":409}" },
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(4));
    try std.testing.expectError(error.Refused, built.sendEvent(held, helper.anEvent()));
    try std.testing.expectEqual(@as(usize, 1), api.count());
}

test "a send gives up on an attempt that runs out of the time it was given" {
    // One attempt, so what is measured is the timeout and nothing about how attempts are spaced.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{.{ .status = 201, .held_for_ms = 3_000, .body = "{}" }};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var chosen = helper.options(1);
    chosen.request_timeout_ms = 200;
    var built = try helper.client(io, held, api, chosen);

    const started = std.Io.Timestamp.now(io, .awake);
    try std.testing.expect(std.meta.isError(built.sendEvent(held, helper.anEvent())));
    const spent = @divTrunc(started.durationTo(std.Io.Timestamp.now(io, .awake)).nanoseconds, std.time.ns_per_ms);

    try std.testing.expect(spent < 2_000);
}

test "a payload above the bound this client sends is refused before a socket is opened" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{helper.ingested(helper.ingested_id)};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var chosen = helper.options(1);
    chosen.max_payload_bytes = 16;
    var built = try helper.client(io, held, api, chosen);

    var event = helper.anEvent();
    event.payload = "x" ** 17;

    try std.testing.expectError(error.PayloadTooLarge, built.sendEvent(held, event));
    try std.testing.expectEqual(@as(usize, 0), api.count());
}

test "a send sends the whole event the API reads" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{helper.ingested(helper.ingested_id)};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var event = helper.anEvent();
    event.metadata = &.{.{ .key = "region", .value = "eu-west-1" }};

    var built = try helper.client(io, held, api, helper.options(1));
    const owned = try built.sendEvent(held, event);
    defer owned.deinit();

    const parsed = try std.json.parseFromSlice(std.json.Value, held, api.at(0).?.body, .{});
    const sent = parsed.value.object;

    try std.testing.expectEqualStrings("app-123", sent.get("application_id").?.string);
    try std.testing.expectEqualStrings("auth.user.create", sent.get("event_type").?.string);
    try std.testing.expectEqualStrings("production", sent.get("labels").?.object.get("environment").?.string);
    try std.testing.expectEqualStrings("eu-west-1", sent.get("metadata").?.object.get("region").?.string);
    try std.testing.expectEqual(@as(usize, 20), sent.get("occurred_at").?.string.len);
    try std.testing.expectEqualStrings("POST", api.at(0).?.method);
}

test "a schedule never spends more than the budget the policy gave it" {
    const policy: hook0.RetryPolicy = .{
        .max_attempts = 8,
        .initial_backoff_ms = 1_000,
        .max_backoff_ms = 4_000,
        .max_total_delay_ms = 2_500,
    };

    var spent: u64 = 0;
    for (1..policy.attempts()) |retry_number| {
        const asked = policy.delay(@intCast(retry_number), 1.0);
        spent += @min(asked, policy.max_total_delay_ms -| spent);
        try std.testing.expect(asked <= policy.max_backoff_ms);
    }
    try std.testing.expect(spent <= policy.max_total_delay_ms);
}

test "a policy makes at most the attempts nothing may cross" {
    const many: hook0.RetryPolicy = .{ .max_attempts = 10_000 };
    try std.testing.expectEqual(hook0.RetryPolicy.max_attempts_cap, many.attempts());

    const none: hook0.RetryPolicy = .{ .max_attempts = 0 };
    try std.testing.expectEqual(@as(u32, 1), none.attempts());
    try std.testing.expectEqual(@as(u32, 1), hook0.RetryPolicy.disabled.attempts());
}

test "upserting event types creates only the ones the application does not declare yet" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{
        .{ .status = 200, .body = "[{\"event_type_name\":\"auth.user.create\"}]" },
        .{ .status = 201, .body = "{\"event_type_name\":\"billing.invoice.paid\"}" },
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(1));
    const owned = try built.upsertEventTypes(held, &.{ "auth.user.create", "billing.invoice.paid" });
    defer owned.deinit();

    try std.testing.expectEqual(@as(usize, 1), owned.value.len);
    try std.testing.expectEqualStrings("billing.invoice.paid", owned.value[0]);
    try std.testing.expectEqual(@as(usize, 2), api.count());
    try std.testing.expectEqualStrings("GET", api.at(0).?.method);
    try std.testing.expectEqualStrings("POST", api.at(1).?.method);
}

test "an event type that does not name all three of its parts is refused" {
    for ([_][]const u8{ "auth.user", "auth", "auth.user.create.now", "auth..create", "auth.user.créé" }) |written| {
        try std.testing.expectError(error.NotAnEventType, hook0.EventType.parse(written));
    }
    const read = try hook0.EventType.parse("auth.user.create");
    try std.testing.expectEqualStrings("auth", read.service);
}

test "declaring no event type at all reaches the API for nothing" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const api = try helper.FakeApi.init(io, held, &.{});
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(1));
    const owned = try built.upsertEventTypes(held, &.{});
    defer owned.deinit();

    try std.testing.expectEqual(@as(usize, 0), owned.value.len);
    try std.testing.expectEqual(@as(usize, 0), api.count());
}

test "an accepted event the API named no identifier for is reported rather than repeated" {
    // The event was ingested, so repeating the request would meet the same answer; what the caller
    // cannot be given is the identifier it was ingested under.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{.{ .status = 201, .body = "{\"received_at\":\"2026-01-01\"}" }};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(4));
    try std.testing.expectError(error.Refused, built.sendEvent(held, helper.anEvent()));

    try std.testing.expectEqual(@as(usize, 1), api.count());
    try std.testing.expect(std.mem.indexOf(u8, built.detail, "without an event id") != null);
}

test "event types the API refused to answer are reported rather than taken as none" {
    // Nothing is created off a list that was never read: taking a refusal for an empty list would
    // have this client declare every event type the application already has.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{try helper.refusal(held, 503, "ServiceUnavailable", &.{})};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(1));
    try std.testing.expectError(error.Refused, built.upsertEventTypes(held, &.{"auth.user.create"}));

    try std.testing.expectEqual(@as(usize, 1), api.count());
    try std.testing.expect(std.mem.indexOf(u8, built.detail, "ServiceUnavailable") != null);
}

test "an event type the API refused to create is reported rather than answered as created" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{
        .{ .status = 200, .body = "[]" },
        try helper.refusal(held, 409, "EventTypeAlreadyExist", &.{}),
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(1));
    try std.testing.expectError(error.Refused, built.upsertEventTypes(held, &.{"auth.user.create"}));

    try std.testing.expectEqual(@as(usize, 2), api.count());
    try std.testing.expect(std.mem.indexOf(u8, built.detail, "EventTypeAlreadyExist") != null);
}

test "a refusal too long to report whole is cut rather than carried into what a caller logs" {
    // What a refusal carries is written by a server this client does not control, so what reaches
    // whatever the caller logs is cut to a budget rather than passed on whole.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const padding = try held.alloc(u8, hook0.runtime.max_preview_bytes * 2);
    @memset(padding, 'e');
    const scripted = [_]helper.Scripted{
        .{ .status = 500, .body = padding },
        .{ .status = 500, .body = padding },
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(2));
    try std.testing.expectError(error.RetriesExhausted, built.sendEvent(held, helper.anEvent()));

    try std.testing.expect(built.detail.len < padding.len);
    try std.testing.expect(std.mem.indexOf(u8, built.detail, "gave up") != null);
}
