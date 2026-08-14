//! The cases the shared conformance corpus dictates, run against this client.
//!
//! The corpus sits at `clients/conformance`, is hand-authored, and is read by the suite of every SDK.
//! Nothing below writes down a verdict, a bound or a signature of its own: they are read out of the
//! committed documents — `retry.json`, `bounds.json`, `signature.json` and `request.json` — and this
//! client is driven against them over a real socket. A case added to the corpus is therefore
//! exercised here without this file being touched, and a verdict changed there fails here until this
//! client agrees with it again.

const std = @import("std");
const hook0 = @import("hook0");

const helper = @import("helper.zig");

const io = std.testing.io;
const allocator = std.testing.allocator;

/// The budget the delay cases share. A delay the API names above it is expected to be cut down to
/// it, so this also bounds what those cases cost.
const delay_budget_ms: u64 = 1_100;

/// What a wait may overshoot by before it is read as more than what was asked for: a loopback round
/// trip, a timer and a scheduler all sit inside it.
const delay_slack_ms: u64 = 900;

/// How a refusal the corpus names reads in this client's own words.
///
/// Every name the corpus declares is looked up here, so one added there stops this suite until it is
/// mapped rather than passing under whatever the client happened to answer.
fn refusalOf(name: []const u8) ?hook0.SignatureError {
    if (std.mem.eql(u8, name, "code_not_hexadecimal")) return error.CodeNotHexadecimal;
    if (std.mem.eql(u8, name, "header_not_delivered")) return error.HeaderNotDelivered;
    if (std.mem.eql(u8, name, "code_mismatch")) return error.CodeMismatch;
    if (std.mem.eql(u8, name, "outside_tolerance")) return error.OutsideTolerance;
    return null;
}

/// How many requests a send made when the API answered that way, and whether it ended up ingesting
/// the event.
fn issuedBy(scripted: []const helper.Scripted, chosen: hook0.Options) !struct { usize, bool } {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();

    const api = try helper.FakeApi.init(io, arena.allocator(), scripted);
    defer api.deinit();

    var built = try helper.client(io, arena.allocator(), api, chosen);
    const sent = built.sendEvent(arena.allocator(), helper.anEvent());

    if (sent) |owned| {
        defer owned.deinit();
        return .{ api.count(), true };
    } else |_| {
        return .{ api.count(), false };
    }
}

test "the corpus says what every problem does to a send" {
    // The status is not what decides: the corpus carries problems answering the same status with
    // opposite verdicts, and a client reading the status alone fails half of them.
    const read = try helper.contract(io, allocator, "retry.json");
    defer read.deinit();

    for (read.value.object.get("problems").?.array.items) |rule| {
        const problem = rule.object.get("problem").?.string;
        const status: u16 = @intCast(rule.object.get("status").?.integer);
        const retryable = rule.object.get("retryable").?.bool;

        var arena: std.heap.ArenaAllocator = .init(allocator);
        defer arena.deinit();

        const scripted = [_]helper.Scripted{
            try helper.refusal(arena.allocator(), status, problem, &.{}),
            helper.ingested(helper.ingested_id),
        };
        const issued, const survived = try issuedBy(&scripted, helper.options(4));
        const expected: usize = if (retryable) 2 else 1;

        std.testing.expectEqual(expected, issued) catch |failed| {
            std.debug.print("`{s}` under {d}: {s}\n", .{ problem, status, rule.object.get("reason").?.string });
            return failed;
        };
        try std.testing.expectEqual(retryable, survived);
    }
}

test "the corpus says what every status does to a send" {
    // A body naming no problem this client could read is also what an older client meets when the
    // API names a problem it has never heard of.
    const read = try helper.contract(io, allocator, "retry.json");
    defer read.deinit();

    for (read.value.object.get("statuses").?.array.items) |rule| {
        const status: u16 = @intCast(rule.object.get("status").?.integer);
        const retryable = rule.object.get("retryable").?.bool;

        var arena: std.heap.ArenaAllocator = .init(allocator);
        defer arena.deinit();

        const scripted = [_]helper.Scripted{
            try helper.refusal(arena.allocator(), status, "AProblemThisClientHasNeverHeardOf", &.{}),
            helper.ingested(helper.ingested_id),
        };
        const issued, _ = try issuedBy(&scripted, helper.options(4));
        const expected: usize = if (retryable) 2 else 1;

        std.testing.expectEqual(expected, issued) catch |failed| {
            std.debug.print("a status of {d}: {s}\n", .{ status, rule.object.get("reason").?.string });
            return failed;
        };
    }
}

test "the corpus says what a request the API never answered does" {
    // Every cause the corpus names is provoked for real rather than reported: a socket that hangs
    // up, an answer above a ceiling this client set for itself, and a URL nothing can be sent to.
    const read = try helper.contract(io, allocator, "retry.json");
    defer read.deinit();

    for (read.value.object.get("transport").?.object.get("causes").?.array.items) |rule| {
        const cause = rule.object.get("cause").?.string;
        const retryable = rule.object.get("retryable").?.bool;
        const expected: usize = if (retryable) 2 else 1;

        if (std.mem.eql(u8, cause, "no_answer")) {
            const scripted = [_]helper.Scripted{
                .{ .close = true },
                helper.ingested(helper.ingested_id),
            };
            const issued, const survived = try issuedBy(&scripted, helper.options(4));
            try std.testing.expectEqual(expected, issued);
            try std.testing.expectEqual(retryable, survived);
        } else if (std.mem.eql(u8, cause, "answer_above_a_bound")) {
            var arena: std.heap.ArenaAllocator = .init(allocator);
            defer arena.deinit();

            const padding = try arena.allocator().alloc(u8, 2048);
            @memset(padding, 'x');
            const scripted = [_]helper.Scripted{
                .{
                    .status = 201,
                    .body = try std.fmt.allocPrint(
                        arena.allocator(),
                        "{{\"event_id\":\"{s}\",\"padding\":\"{s}\"}}",
                        .{ helper.ingested_id, padding },
                    ),
                },
                helper.ingested(helper.ingested_id),
            };
            var chosen = helper.options(4);
            chosen.max_response_bytes = 256;

            const issued, const survived = try issuedBy(&scripted, chosen);
            try std.testing.expectEqual(expected, issued);
            try std.testing.expectEqual(retryable, survived);
        } else if (std.mem.eql(u8, cause, "unusable_api_url")) {
            // Nothing is ever sent, so nothing reaches an API at all.
            var arena: std.heap.ArenaAllocator = .init(allocator);
            defer arena.deinit();

            var built: hook0.Client = .init(
                io,
                "gopher://nowhere.invalid",
                "app-123",
                "token-xyz",
                helper.options(4),
            );
            const sent = built.sendEvent(arena.allocator(), helper.anEvent());
            try std.testing.expect(std.meta.isError(sent));
        } else {
            std.debug.print("the corpus names a cause `{s}` this suite cannot provoke\n", .{cause});
            return error.UnknownCause;
        }
    }
}

test "the delay the API names is honoured and bounded" {
    // The header is written by the other end, so honouring it whole would hand a stranger the length
    // of this client's send. What the corpus asks for is that a delay be waited out when the budget
    // can afford it and cut down to what is left of the budget when it cannot.
    const read = try helper.contract(io, allocator, "retry.json");
    defer read.deinit();

    const retry_after = read.value.object.get("retry_after").?.object;
    const header = retry_after.get("header").?.string;

    // The one problem the corpus classifies as pacing rather than as a quota, found rather than
    // named: it is the answer the API names a delay beside.
    var paced: ?std.json.Value = null;
    for (read.value.object.get("problems").?.array.items) |rule| {
        if (rule.object.get("retryable").?.bool and rule.object.get("status").?.integer == 429) {
            paced = rule;
        }
    }
    try std.testing.expect(paced != null);

    for (retry_after.get("cases").?.array.items) |named| {
        var arena: std.heap.ArenaAllocator = .init(allocator);
        defer arena.deinit();

        const honoured = named.object.get("honoured").?.bool;
        const asked: u64 = if (honoured) @intCast(named.object.get("seconds").?.integer) else 0;
        const expected = @min(asked * std.time.ms_per_s, delay_budget_ms);

        const scripted = [_]helper.Scripted{
            try helper.refusal(
                arena.allocator(),
                @intCast(paced.?.object.get("status").?.integer),
                paced.?.object.get("problem").?.string,
                &.{.{ header, named.object.get("header").?.string }},
            ),
            helper.ingested(helper.ingested_id),
        };

        const api = try helper.FakeApi.init(io, arena.allocator(), &scripted);
        defer api.deinit();

        var chosen = helper.options(4);
        chosen.retry_policy.max_total_delay_ms = delay_budget_ms;
        var built = try helper.client(io, arena.allocator(), api, chosen);

        const started = std.Io.Timestamp.now(io, .awake);
        const owned = try built.sendEvent(arena.allocator(), helper.anEvent());
        defer owned.deinit();
        const waited: u64 = @intCast(@max(
            @divTrunc(started.durationTo(std.Io.Timestamp.now(io, .awake)).nanoseconds, std.time.ns_per_ms),
            0,
        ));

        try std.testing.expectEqual(@as(usize, 2), api.count());
        std.testing.expect(waited + 20 >= expected and waited <= expected + delay_slack_ms) catch |failed| {
            std.debug.print(
                "`{s}: {s}` was retried after {d}ms, where the corpus expects {d}ms\n",
                .{ header, named.object.get("header").?.string, waited, expected },
            );
            return failed;
        };
    }
}

test "the bounds are the ones the corpus names" {
    // This client's defaults, held against the one place the numbers are written down. What is
    // asserted is read from the corpus rather than listed here, so a bound added there and left
    // unapplied fails instead of passing unnoticed.
    const read = try helper.contract(io, allocator, "bounds.json");
    defer read.deinit();

    const built: hook0.Options = .{};
    const policy = built.retry_policy;

    var walked = read.value.object.get("bounds").?.object.iterator();
    while (walked.next()) |entry| {
        const name = entry.key_ptr.*;
        const wanted: u64 = @intCast(entry.value_ptr.integer);

        const applied: ?u64 = if (std.mem.eql(u8, name, "max_attempts"))
            policy.max_attempts
        else if (std.mem.eql(u8, name, "max_attempts_cap"))
            hook0.RetryPolicy.max_attempts_cap
        else if (std.mem.eql(u8, name, "initial_backoff_ms"))
            policy.initial_backoff_ms
        else if (std.mem.eql(u8, name, "max_backoff_ms"))
            policy.max_backoff_ms
        else if (std.mem.eql(u8, name, "max_total_delay_ms"))
            policy.max_total_delay_ms
        else if (std.mem.eql(u8, name, "request_timeout_ms"))
            built.request_timeout_ms
        else if (std.mem.eql(u8, name, "max_payload_bytes"))
            built.max_payload_bytes
        else if (std.mem.eql(u8, name, "max_response_bytes"))
            built.max_response_bytes
        else if (std.mem.eql(u8, name, "max_response_headers"))
            built.max_response_headers
        else if (std.mem.eql(u8, name, "max_header_bytes"))
            built.max_header_bytes
        else if (std.mem.eql(u8, name, "max_head_bytes"))
            built.max_head_bytes
        else
            null;

        if (applied == null) {
            std.debug.print("the corpus names the bound `{s}`, which this client does not apply\n", .{name});
            return error.BoundNotApplied;
        }
        std.testing.expectEqual(wanted, applied.?) catch |failed| {
            std.debug.print("`{s}`: this client applies {d} where the corpus names {d}\n", .{
                name,
                applied.?,
                wanted,
            });
            return failed;
        };
    }
}

test "an answer above every ceiling on what the other end may send is refused" {
    // A bound is a safety property, and conformance to it is shown by the refusal above it: what is
    // exercised here is a head and a body well over each ceiling, never one just under it.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const line = try held.alloc(u8, 2048);
    @memset(line, 'x');

    var many: std.ArrayList([2][]const u8) = .empty;
    for (0..65) |index| {
        try many.append(held, .{ try std.fmt.allocPrint(held, "x-padding-{d}", .{index}), "x" });
    }

    var wide: std.ArrayList([2][]const u8) = .empty;
    for (0..17) |index| {
        try wide.append(held, .{ try std.fmt.allocPrint(held, "x-padding-{d}", .{index}), line });
    }

    const cases = [_]struct { name: []const u8, scripted: helper.Scripted, bounds: hook0.Options }{
        .{
            .name = "max_response_headers",
            .scripted = .{ .status = 200, .headers = many.items },
            .bounds = helper.options(1),
        },
        .{
            .name = "max_head_bytes",
            .scripted = .{ .status = 200, .headers = wide.items },
            .bounds = helper.options(1),
        },
        .{
            .name = "max_header_bytes",
            .scripted = .{ .status = 200, .headers = &.{.{ "x-padding", line }} },
            .bounds = blk: {
                var chosen = helper.options(1);
                chosen.max_header_bytes = 1024;
                break :blk chosen;
            },
        },
        .{
            .name = "max_response_bytes",
            .scripted = .{ .status = 200, .body = line },
            .bounds = blk: {
                var chosen = helper.options(1);
                chosen.max_response_bytes = 256;
                break :blk chosen;
            },
        },
    };

    for (cases) |asked| {
        const scripted = [_]helper.Scripted{asked.scripted};
        const api = try helper.FakeApi.init(io, held, &scripted);
        defer api.deinit();

        var built = try helper.client(io, held, api, asked.bounds);
        const sent = built.sendEvent(held, helper.anEvent());
        std.testing.expect(std.meta.isError(sent)) catch |failed| {
            std.debug.print("an answer above `{s}` was read anyway\n", .{asked.name});
            return failed;
        };
    }
}

test "a request carries the headers its occasion declares, and only those" {
    // Read back off the socket, on both occasions the corpus declares: a send carries a body, and a
    // read does not. What separates them is the point — a client that sets `Content-Type` on a
    // request with nothing in it is describing a body that is not there.
    const read = try helper.contract(io, allocator, "request.json");
    defer read.deinit();

    const occasions = read.value.object.get("occasions").?.array.items;
    const headers = read.value.object.get("headers").?.array.items;

    for (headers) |header| {
        const when = header.object.get("when").?.string;
        var declared = false;
        for (occasions) |occasion| declared = declared or std.mem.eql(u8, occasion.string, when);
        try std.testing.expect(declared);
    }

    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const scripted = [_]helper.Scripted{
        helper.ingested(helper.ingested_id),
        .{ .status = 200, .body = "[]" },
    };
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var built = try helper.client(io, held, api, helper.options(4));
    {
        const owned = try built.sendEvent(held, helper.anEvent());
        owned.deinit();
    }
    _ = try built.inner.request(held, .{ .method = "GET", .path = "/applications" });

    const carrying = [_]struct { index: usize, body: bool }{
        .{ .index = 0, .body = true },
        .{ .index = 1, .body = false },
    };
    for (carrying) |asked| {
        const request = api.at(asked.index) orelse return error.NothingReceived;

        for (headers) |header| {
            const name = header.object.get("name").?.string;
            const when = header.object.get("when").?.string;
            const carried = request.get(name);
            const on_this = std.mem.eql(u8, when, "every request") or asked.body;

            if (!on_this) {
                std.testing.expect(carried == null) catch |failed| {
                    std.debug.print("a request carried `{s}`, which the corpus carries only on `{s}`\n", .{
                        name,
                        when,
                    });
                    return failed;
                };
                continue;
            }

            const wanted = try std.mem.replaceOwned(
                u8,
                held,
                header.object.get("value").?.string,
                "${token}",
                "token-xyz",
            );
            std.testing.expectEqualStrings(wanted, carried orelse "") catch |failed| {
                std.debug.print("`{s}`: {s}\n", .{ name, header.object.get("reason").?.string });
                return failed;
            };
        }
    }
}

test "every delivery of the corpus is verified as it says" {
    // A refused delivery has to be refused for the reason the corpus names: a client that computed a
    // code over a header that never arrived and reported a mismatch would otherwise look right.
    const read = try helper.contract(io, allocator, "signature.json");
    defer read.deinit();

    for (read.value.object.get("refusals").?.array.items) |name| {
        std.testing.expect(refusalOf(name.string) != null) catch |failed| {
            std.debug.print("the corpus declares `{s}`, which this suite maps to nothing\n", .{name.string});
            return failed;
        };
    }

    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();

    for (read.value.object.get("vectors").?.array.items) |vector| {
        const held = vector.object;
        var headers: std.ArrayList(hook0.signature.Header) = .empty;
        for (held.get("headers").?.array.items) |pair| {
            try headers.append(arena.allocator(), .{
                .name = pair.array.items[0].string,
                .value = pair.array.items[1].string,
            });
        }

        const verified = hook0.verifyWebhookSignatureWithCurrentTime(
            held.get("signature").?.string,
            held.get("payload").?.string,
            headers.items,
            held.get("secret").?.string,
            held.get("tolerance_seconds").?.integer,
            held.get("current_time").?.integer,
        );

        if (std.mem.eql(u8, held.get("verdict").?.string, "accepted")) {
            verified catch |failed| {
                std.debug.print("`{s}` was refused as {s}: {s}\n", .{
                    held.get("name").?.string,
                    @errorName(failed),
                    held.get("reason").?.string,
                });
                return failed;
            };
            continue;
        }

        const wanted = refusalOf(held.get("refusal").?.string).?;
        std.testing.expectError(wanted, verified) catch |failed| {
            std.debug.print("`{s}`: {s}\n", .{ held.get("name").?.string, held.get("reason").?.string });
            return failed;
        };
    }
}
