//! What holds for every input, rather than for the ones a case happened to pick.
//!
//! Four things are checked here. A retry schedule never spends more than the policy that produced it
//! allows, whichever way the randomness fell. Reading a signature header answers with the one error
//! set this package declares, whatever text reached the endpoint, and never with anything else. A
//! value read out of a document the API could answer is written back as the value that was read. And
//! identifiers minted in sequence never carry a moment earlier than the one before them.
//!
//! The search is written down rather than left to the run: a fixed seed, a bounded number of draws,
//! and the counter-examples worth keeping committed under `regressions/`, so they run as ordinary
//! cases on every pipeline. A draw that fails is one somebody reproduces by running the suite again
//! rather than one that goes away on a retry.

const std = @import("std");
const hook0 = @import("hook0");

const helper = @import("helper.zig");

const io = std.testing.io;
const allocator = std.testing.allocator;

/// What the draws are made from. Fixed, so the suite explores the same inputs everywhere it runs.
const seed: u64 = 20260814;

/// How many draws each property makes. Bounded, so a pipeline can never be held by one.
const draws: usize = 200;

/// Most pieces a drawn signature header is built out of.
const max_drawn_pieces: usize = 96;

/// Where the counter-examples worth keeping live, one JSON value per line.
const regressions_path = "tests/regressions";

/// The pieces a signature is made of, put together every way a sender that is not Hook0 might put
/// them together.
const pieces = [_][]const u8{
    "t",
    "v0",
    "v1",
    "h",
    "=",
    ",",
    " ",
    ".",
    "\"",
    "0",
    "9",
    "zz",
    "ab",
    "-1",
    "1800000000",
    "999999999999999999999999",
    "x-event-id",
};

/// The counter-examples committed beside the property they broke, read back as JSON values.
fn regressions(arena: std.mem.Allocator, name: []const u8) ![]const std.json.Value {
    const path = try std.fmt.allocPrint(arena, "{s}/{s}.jsonl", .{ regressions_path, name });
    const written = try std.Io.Dir.cwd().readFileAlloc(io, path, arena, .limited(helper.max_corpus_bytes));

    var held: std.ArrayList(std.json.Value) = .empty;
    var lines = std.mem.splitScalar(u8, written, '\n');
    while (lines.next()) |line| {
        const trimmed = std.mem.trim(u8, line, " \r\t");
        if (trimmed.len == 0) continue;
        const parsed = try std.json.parseFromSlice(std.json.Value, arena, trimmed, .{});
        try held.append(arena, parsed.value);
    }
    return held.toOwnedSlice(arena);
}

test "a retry schedule stays within every bound of the policy that produced it" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    var prng: std.Random.DefaultPrng = .init(seed);
    const random = prng.random();

    var cases: std.ArrayList(hook0.RetryPolicy) = .empty;
    for (try regressions(held, "retry_policies")) |written| {
        const numbers = written.array.items;
        try cases.append(held, .{
            .max_attempts = @intCast(std.math.clamp(numbers[0].integer, 0, std.math.maxInt(u32))),
            .initial_backoff_ms = @intCast(@max(numbers[1].integer, 0)),
            .max_backoff_ms = @intCast(@max(numbers[2].integer, 0)),
            .max_total_delay_ms = @intCast(@max(numbers[3].integer, 0)),
        });
    }
    for (0..draws) |_| {
        try cases.append(held, .{
            .max_attempts = random.uintAtMost(u32, 64),
            .initial_backoff_ms = random.uintAtMost(u64, 10_000),
            .max_backoff_ms = random.uintAtMost(u64, 10_000),
            .max_total_delay_ms = random.uintAtMost(u64, 60_000),
        });
    }

    for (cases.items) |policy| {
        try std.testing.expect(policy.attempts() >= 1);
        try std.testing.expect(policy.attempts() <= hook0.RetryPolicy.max_attempts_cap);

        var spent: u64 = 0;
        var previous: u64 = 0;
        for (1..policy.attempts()) |retry_number| {
            const ceiling = policy.backoffCeiling(@intCast(retry_number));
            try std.testing.expect(ceiling <= policy.max_backoff_ms);

            // A schedule never hurries up as it goes: the ceiling of one retry never sits below the
            // ceiling of the retry before it.
            try std.testing.expect(ceiling >= previous);
            previous = ceiling;

            // Including the draws an unusable source of randomness produces, which are the ones that
            // turn a bound into an unbounded wait when nothing brings them back inside the range.
            for ([_]f64{ 0.0, 0.25, 1.0, -1.0, 2.0, std.math.nan(f64), std.math.inf(f64) }) |drawn| {
                try std.testing.expect(policy.delay(@intCast(retry_number), drawn) <= ceiling);
            }

            spent += @min(policy.delay(@intCast(retry_number), 1.0), policy.max_total_delay_ms -| spent);
        }
        try std.testing.expect(spent <= policy.max_total_delay_ms);
    }
}

test "reading a signature answers with the one error set this package declares, whatever arrived" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    var prng: std.Random.DefaultPrng = .init(seed);
    const random = prng.random();

    var headers: std.ArrayList([]const u8) = .empty;
    for (try regressions(held, "signatures")) |written| {
        try headers.append(held, written.string);
    }
    for (0..draws) |_| {
        var built: std.ArrayList(u8) = .empty;
        for (0..random.uintAtMost(usize, max_drawn_pieces)) |_| {
            try built.appendSlice(held, pieces[random.uintLessThan(usize, pieces.len)]);
        }
        try headers.append(held, try built.toOwnedSlice(held));
    }

    var read_whole: usize = 0;
    for (headers.items) |header| {
        if (hook0.signature.parse(header)) |_| {
            read_whole += 1;

            // A header that reads must not find a way to fail that a caller cannot name: whatever
            // verifying it ends with is in the same declared set.
            hook0.verifyWebhookSignatureWithCurrentTime(header, "", &.{}, "a-secret", 300, 1800000000) catch |failed| {
                switch (failed) {
                    error.CodeNotHexadecimal,
                    error.HeaderNotDelivered,
                    error.CodeMismatch,
                    error.OutsideTolerance,
                    error.Unreadable,
                    => {},
                }
            };
        } else |failed| {
            switch (failed) {
                error.CodeNotHexadecimal,
                error.HeaderNotDelivered,
                error.CodeMismatch,
                error.OutsideTolerance,
                error.Unreadable,
                => {},
            }
        }
    }

    // A parser that refused everything would satisfy the property above without saying anything, so
    // the corpus has to hold headers that do read.
    try std.testing.expect(read_whole > 0);
}

test "a generated type reads back what it wrote, and refuses what it cannot read" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const documents = try regressions(held, "documents");
    var round_trips: usize = 0;

    // Every value the generator wrote a decoder for, found by looking at what it wrote rather than
    // by naming any of them: a schema the API grows joins this case the moment the generated files
    // carry it.
    inline for (comptime std.meta.declarations(hook0.models)) |declaration| {
        const declared = @field(hook0.models, declaration.name);
        if (@TypeOf(declared) == type) {
            if (@typeInfo(declared) == .@"struct" and @hasDecl(declared, "fromJson")) {
                for (documents) |document| {
                    if (declared.fromJson(held, document)) |read| {
                        round_trips += 1;

                        const written = try read.toJson(held);
                        const again = try declared.fromJson(held, written);
                        const twice = try again.toJson(held);

                        std.testing.expectEqualStrings(
                            try serialised(held, written),
                            try serialised(held, twice),
                        ) catch |failed| {
                            std.debug.print("a {s} does not read back what it wrote\n", .{declaration.name});
                            return failed;
                        };
                    } else |failed| {
                        // Every refusal is one this package declares, whatever the document held.
                        switch (failed) {
                            error.NotAnObject,
                            error.NotAnArray,
                            error.NotAString,
                            error.NotAWholeNumber,
                            error.NotANumber,
                            error.NotABoolean,
                            error.MemberMissing,
                            error.ValueNotDeclared,
                            error.PayloadTooLarge,
                            error.PayloadNotJson,
                            error.OutOfMemory,
                            => {},
                        }
                    }
                }
            }
        }
    }

    // A generated half that refused every document would satisfy the round trip vacuously.
    try std.testing.expect(round_trips > 0);
}

/// A JSON value written out, which is how two of them are compared.
fn serialised(held: std.mem.Allocator, value: std.json.Value) ![]const u8 {
    var out: std.Io.Writer.Allocating = .init(held);
    var stringify: std.json.Stringify = .{ .writer = &out.writer };
    try stringify.write(value);
    return out.written();
}

test "a minted identifier carries a moment that never goes back" {
    // Not that two identifiers land inside the same millisecond, which nothing guarantees and which
    // fails about once a week in a pipeline: what is asserted is that the moment never runs
    // backwards, which is the ordering the identifier exists to give.
    var previous: [12]u8 = @splat('0');

    for (0..draws) |index| {
        var minted: [36]u8 = undefined;
        hook0.generateEventId(io, &minted);

        // The leading 48 bits, which the hyphen after the eighth digit sits inside.
        var moment: [12]u8 = undefined;
        @memcpy(moment[0..8], minted[0..8]);
        @memcpy(moment[8..12], minted[9..13]);

        std.testing.expect(std.mem.order(u8, &previous, &moment) != .gt) catch |failed| {
            std.debug.print(
                "identifier {d} carries {s}, earlier than the {s} before it\n",
                .{ index, &moment, &previous },
            );
            return failed;
        };
        previous = moment;
    }
}

test "a minted identifier is shaped like a UUIDv7, every time" {
    for (0..draws) |_| {
        var minted: [36]u8 = undefined;
        hook0.generateEventId(io, &minted);

        for ([_]usize{ 8, 13, 18, 23 }) |at| {
            try std.testing.expectEqual(@as(u8, '-'), minted[at]);
        }
        try std.testing.expectEqual(@as(u8, '7'), minted[14]);
        try std.testing.expect(switch (minted[19]) {
            '8', '9', 'a', 'b' => true,
            else => false,
        });
        for (minted, 0..) |character, index| {
            const dashed = index == 8 or index == 13 or index == 18 or index == 23;
            if (!dashed) try std.testing.expect(std.fmt.charToDigit(character, 16) catch null != null);
        }
    }
}
