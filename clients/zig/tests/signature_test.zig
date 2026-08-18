//! Verifying a webhook, beyond the vectors the shared corpus pins.
//!
//! Every accepted and refused delivery the corpus declares is run in `conformance_test.zig`. What is
//! here is what a vector cannot say: that the pieces this is built out of are the right ones, that
//! the window looks both ways, and that a header nothing can be read out of is refused rather than
//! guessed at.

const std = @import("std");
const hook0 = @import("hook0");

const Hmac = std.crypto.auth.hmac.sha2.HmacSha256;

const secret = "a-subscription-secret";
const payload = "{\"event\":\"user.created\"}";
const moment: i64 = 1800000000;
const tolerance: i64 = 300;

const headers = [_]hook0.signature.Header{
    .{ .name = "x-event-id", .value = "evt-1" },
    .{ .name = "x-delivery-id", .value = "dlv-1" },
    .{ .name = "content-type", .value = "application/json" },
};

/// A signature over the body alone, at that moment.
fn bodyScheme(buffer: []u8, at: i64) ![]const u8 {
    var message: [64]u8 = undefined;
    const written = try std.fmt.bufPrint(&message, "{d}.", .{at});

    var code: Hmac = .init(secret);
    code.update(written);
    code.update(payload);

    var out: [Hmac.mac_length]u8 = undefined;
    code.final(&out);
    return std.fmt.bufPrint(buffer, "t={d},v0={x}", .{ at, &out });
}

fn verified(header: []const u8, at: i64) hook0.SignatureError!void {
    return hook0.verifyWebhookSignatureWithCurrentTime(header, payload, &headers, secret, tolerance, at);
}

test "the keyed hash answers what RFC 4231 publishes" {
    // Held against values computed outside this repository: a suite that hashed with the module it
    // is testing and compared against the same module would pass whatever the two agreed on.
    var out: [Hmac.mac_length]u8 = undefined;
    var written: [64]u8 = undefined;

    Hmac.create(&out, "what do ya want for nothing?", "Jefe");
    try std.testing.expectEqualStrings(
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
        try std.fmt.bufPrint(&written, "{x}", .{&out}),
    );

    Hmac.create(&out, "Hi There", &([_]u8{0x0b} ** 20));
    try std.testing.expectEqualStrings(
        "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
        try std.fmt.bufPrint(&written, "{x}", .{&out}),
    );
}

test "a signature verifies at the edge of the window, on either side of it" {
    // The window is the width a delivery is accepted within, so its own edges are inside it.
    var buffer: [256]u8 = undefined;
    try verified(try bodyScheme(&buffer, moment - tolerance), moment);
    try verified(try bodyScheme(&buffer, moment + tolerance), moment);
}

test "a signature is refused as far ahead of the window as behind it" {
    // A window that only looked backwards is one a sender widens by dating its own delivery in the
    // future, which is the same replay the window exists to bound.
    var buffer: [256]u8 = undefined;
    try std.testing.expectError(
        error.OutsideTolerance,
        verified(try bodyScheme(&buffer, moment - tolerance - 1), moment),
    );
    try std.testing.expectError(
        error.OutsideTolerance,
        verified(try bodyScheme(&buffer, moment + tolerance + 1), moment),
    );
}

test "a signature is read on the first assignator of each part, and never on the rest" {
    // A value may hold further assignators, and splitting on all of them would drop everything past
    // the second.
    const read = try hook0.signature.parse("t=1800000000,note=a=b=c,v0=" ++ ("ab" ** 32));
    try std.testing.expectEqual(moment, read.timestamp);
}

test "a code that is not whole hexadecimal is refused rather than read as far as it goes" {
    for ([_][]const u8{ "abc", "zz", "", "abcg", "ab" ** 31, "ab" ** 33 }) |code| {
        const header = try std.fmt.allocPrint(std.testing.allocator, "t=1800000000,v0={s}", .{code});
        defer std.testing.allocator.free(header);

        const read = hook0.signature.parse(header);
        std.testing.expect(std.meta.isError(read)) catch |failed| {
            std.debug.print("`{s}` was read as a code\n", .{code});
            return failed;
        };
    }
}

test "a header carrying no moment, no code, or nothing at all is refused" {
    for ([_][]const u8{
        "",
        "t=1800000000",
        "v0=" ++ ("ab" ** 32),
        "nonsense",
        "t=later,v0=" ++ ("ab" ** 32),
        "t=1_0,v0=" ++ ("ab" ** 32),
        "t=+1,v0=" ++ ("ab" ** 32),
        "t=999999999999999999999999,v0=" ++ ("ab" ** 32),
    }) |header| {
        const read = hook0.signature.parse(header);
        std.testing.expect(std.meta.isError(read)) catch |failed| {
            std.debug.print("`{s}` was read as a signature\n", .{header});
            return failed;
        };
    }
}

test "a covered header that was not delivered is settled before any code is computed" {
    // Signing over an absent value would let a sender drop a header and keep the signature valid, so
    // the refusal comes first — and it is that refusal rather than a mismatch.
    try std.testing.expectError(
        error.HeaderNotDelivered,
        verified("t=1800000000,h=x-event-id x-missing,v1=" ++ ("ab" ** 32), moment),
    );
}

test "the covered headers are read in the order the signature names them" {
    // The same two headers named the other way round cover their values the other way round, which
    // is a different message.
    const one = "t=1800000000,h=x-event-id x-delivery-id,v1=" ++
        "19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347";
    const other = "t=1800000000,h=x-delivery-id x-event-id,v1=" ++
        "19a6fb8f6581715b241a93af02a58611c3b0ac7b747a8d2a5b120ee418d0c347";

    try verified(one, moment);
    try std.testing.expectError(error.CodeMismatch, verified(other, moment));
}

test "a header longer than this client reads is refused before any of it is split" {
    const held = try std.testing.allocator.alloc(u8, hook0.signature.max_signature_bytes + 1);
    defer std.testing.allocator.free(held);
    @memset(held, 't');

    try std.testing.expectError(error.Unreadable, hook0.signature.parse(held));
}

test "a delivery signed at the moment the clock reads verifies against that clock" {
    // The other cases hand the moment in so that a window can be looked at from both sides. This one
    // is what a caller actually writes: the client asks the clock itself.
    var buffer: [128]u8 = undefined;
    const now = std.Io.Timestamp.now(std.testing.io, .real).toSeconds();

    try hook0.verifyWebhookSignature(
        std.testing.io,
        try bodyScheme(&buffer, now),
        payload,
        &headers,
        secret,
        tolerance,
    );
}
