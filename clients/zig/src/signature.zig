//! Verifying that a webhook came from Hook0, and that nothing in it changed on the way.
//!
//! A signature names the moment it was signed and one or two message authentication codes over the
//! body. The `v1` scheme also covers a list of request headers, so a receiver can tell apart two
//! deliveries that carry the same body but not the same context; `v0` covers the body alone and is
//! what an older sender still produces. When both are offered, `v1` is the one verified: accepting
//! the weaker of two schemes on the strength of the sender offering it is how a downgrade works.
//!
//! Two things are refused before any code is computed. A header the signature says it covers but the
//! request did not carry is refused outright, because signing over an absent value would let a sender
//! drop a header and keep the signature valid. And a signature whose codes are not whole hexadecimal
//! is refused rather than decoded as far as it goes: a decoder that stops at the first bad character
//! compares a prefix, and a prefix of the right code is not the right code.
//!
//! Nothing here allocates. A signature is read into a fixed-size value on the stack, the codes are
//! decoded into fixed arrays, and the message a code is computed over is fed to the keyed hash in
//! pieces rather than joined first — so what a hostile header can cost is settled by this file's own
//! ceilings rather than by an allocator.

const std = @import("std");

const Hmac = std.crypto.auth.hmac.sha2.HmacSha256;

/// Every way a delivery is refused.
///
/// One error per way rather than one error and a string: a caller that has to tell a replayed
/// delivery from a forged one matches on the error, and the four the shared conformance corpus names
/// are the four here.
pub const SignatureError = error{
    /// The header carries a code that is not whole hexadecimal.
    CodeNotHexadecimal,
    /// The signature covers a header the request did not carry.
    HeaderNotDelivered,
    /// The code is not the one the subscription secret produces.
    CodeMismatch,
    /// The moment the signature names sits outside the window the caller allowed.
    OutsideTolerance,
    /// The header is not one this client can read at all.
    Unreadable,
};

/// Longest signature header read. The header is written by whoever reached the endpoint, so its size
/// is bounded before any of it is split, decoded or compared.
pub const max_signature_bytes: usize = 8 * 1024;

/// Most `key=value` parts one signature header is split into.
pub const max_signature_parts: usize = 32;

/// Most header names one signature covers.
pub const max_covered_headers: usize = 64;

/// Furthest from the epoch, in either direction, a signature's moment may sit.
pub const max_timestamp: i64 = 1_000_000_000_000;

/// What separates one part of the signature header from the next.
pub const part_separator: u8 = ',';

/// What separates the name of a part from its value. Only the first one counts: a value may hold
/// further ones, and splitting on all of them would silently drop everything past the second.
pub const part_assignator: u8 = '=';

/// What separates two header names inside the `h` part, and what they are joined back with.
pub const header_name_separator: u8 = ' ';

/// What separates the pieces of the message a code is computed over.
pub const message_separator: u8 = '.';

/// Part naming the moment the delivery was signed, in whole seconds since the Unix epoch.
pub const timestamp_part = "t";

/// Part carrying the code covering the body alone.
pub const body_scheme_part = "v0";

/// Part carrying the code covering the covered headers and the body.
pub const headers_scheme_part = "v1";

/// Part listing the headers the `v1` code covers, in the order it covers them.
pub const covered_headers_part = "h";

/// One header of a delivery, as it arrived.
pub const Header = struct {
    name: []const u8,
    value: []const u8,
};

/// A signature header, read into the pieces a verification needs.
pub const Signature = struct {
    /// The moment the delivery was signed, in whole seconds since the epoch.
    timestamp: i64,
    /// The headers the stronger scheme covers, lowercased and in the order it covers them.
    covered_headers: [max_covered_headers][]const u8 = undefined,
    covered_count: usize = 0,
    /// The `v0` code, decoded, when the header offered one.
    body_code: ?[Hmac.mac_length]u8 = null,
    /// The `v1` code, decoded, when the header offered one.
    headers_code: ?[Hmac.mac_length]u8 = null,

    /// The headers this signature covers, as the slice it filled in.
    pub fn covered(self: *const Signature) []const []const u8 {
        return self.covered_headers[0..self.covered_count];
    }
};

/// The text either side of the first assignator, and nothing when there is not one.
fn partitioned(part: []const u8) ?struct { name: []const u8, value: []const u8 } {
    const at = std.mem.indexOfScalar(u8, part, part_assignator) orelse return null;
    return .{ .name = part[0..at], .value = part[at + 1 ..] };
}

fn trimmed(text: []const u8) []const u8 {
    return std.mem.trim(u8, text, " \t");
}

/// What a header name is written with, as RFC 9110 spells a token.
fn isHeaderName(name: []const u8) bool {
    if (name.len == 0) return false;
    for (name) |character| {
        const usable = switch (character) {
            'A'...'Z', 'a'...'z', '0'...'9' => true,
            '!', '#', '$', '%', '&', '\'', '*', '+', '-', '.', '^', '_', '`', '|', '~' => true,
            else => false,
        };
        if (!usable) return false;
    }
    return true;
}

/// One of the codes a signature offers, decoded whole or not at all.
///
/// A code of the wrong length, an odd number of digits or a character that is not hexadecimal are
/// all one refusal: a decoder that stopped at the first of them would compare a prefix.
fn codeOf(written: []const u8) SignatureError![Hmac.mac_length]u8 {
    if (written.len == 0 or written.len % 2 != 0) return error.CodeNotHexadecimal;
    for (written) |character| {
        if (std.fmt.charToDigit(character, 16) catch null == null) return error.CodeNotHexadecimal;
    }
    if (written.len != Hmac.mac_length * 2) return error.CodeNotHexadecimal;

    var decoded: [Hmac.mac_length]u8 = undefined;
    _ = std.fmt.hexToBytes(&decoded, written) catch return error.CodeNotHexadecimal;
    return decoded;
}

/// The moment the signature names, which it is not a signature without.
fn timestampOf(written: []const u8) SignatureError!i64 {
    if (written.len == 0) return error.Unreadable;

    const digits = if (written[0] == '-') written[1..] else written;
    if (digits.len == 0) return error.Unreadable;
    for (digits) |character| {
        if (character < '0' or character > '9') return error.Unreadable;
    }

    const seconds = std.fmt.parseInt(i64, written, 10) catch return error.Unreadable;
    if (seconds > max_timestamp or seconds < -max_timestamp) return error.Unreadable;
    return seconds;
}

/// Reads a signature header, refusing anything it cannot read whole.
pub fn parse(header: []const u8) SignatureError!Signature {
    if (header.len > max_signature_bytes) return error.Unreadable;

    var read: Signature = .{ .timestamp = 0 };
    var moment: ?[]const u8 = null;
    var covered: ?[]const u8 = null;
    var body: ?[]const u8 = null;
    var headers: ?[]const u8 = null;
    var named: usize = 0;

    var parts = std.mem.splitScalar(u8, header, part_separator);
    var seen: usize = 0;
    while (parts.next()) |part| {
        seen += 1;
        if (seen > max_signature_parts) return error.Unreadable;

        const split = partitioned(part) orelse continue;
        const name = trimmed(split.name);
        const carried = trimmed(split.value);

        // What is counted is the parts a signature names rather than the parts it carries: a header
        // naming one of them twice says one thing, not two, and the second value is the one kept.
        if (std.mem.eql(u8, name, timestamp_part)) {
            if (moment == null) named += 1;
            moment = carried;
        } else if (std.mem.eql(u8, name, body_scheme_part)) {
            if (body == null) named += 1;
            body = carried;
        } else if (std.mem.eql(u8, name, headers_scheme_part)) {
            if (headers == null) named += 1;
            headers = carried;
        } else if (std.mem.eql(u8, name, covered_headers_part)) {
            if (covered == null) named += 1;
            covered = carried;
        } else {
            named += 1;
        }
    }

    if (named < 2) return error.Unreadable;

    if (body) |written| read.body_code = try codeOf(written);
    if (headers) |written| read.headers_code = try codeOf(written);
    if (read.body_code == null and read.headers_code == null) return error.Unreadable;

    read.timestamp = try timestampOf(moment orelse return error.Unreadable);

    if (covered) |written| {
        if (written.len > 0) {
            var names = std.mem.splitScalar(u8, written, header_name_separator);
            while (names.next()) |name| {
                if (read.covered_count == max_covered_headers) return error.Unreadable;
                if (!isHeaderName(name)) return error.Unreadable;
                read.covered_headers[read.covered_count] = name;
                read.covered_count += 1;
            }
        }
    }

    return read;
}

/// The value that header of the delivery carried, under the name a signature refers to it by.
///
/// A later value wins over an earlier one under the same name, which is what a table built by the
/// caller would have done.
fn delivered(headers: []const Header, name: []const u8) ?[]const u8 {
    var found: ?[]const u8 = null;
    for (headers) |header| {
        if (std.ascii.eqlIgnoreCase(header.name, name)) found = header.value;
    }
    return found;
}

/// Whether the code this signature carries is the one the secret produces.
///
/// The stronger scheme wins when both are offered, and the comparison is made in constant time: one
/// that gave up at the first differing byte would say, by how long it took, how much of a guess was
/// right.
pub fn matches(
    read: *const Signature,
    payload: []const u8,
    headers: []const Header,
    subscription_secret: []const u8,
) SignatureError!bool {
    var code: Hmac = .init(subscription_secret);

    var moment: [32]u8 = undefined;
    const written = std.fmt.bufPrint(&moment, "{d}", .{read.timestamp}) catch return error.Unreadable;
    code.update(written);
    code.update(&.{message_separator});

    var out: [Hmac.mac_length]u8 = undefined;
    if (read.headers_code) |offered| {
        for (read.covered(), 0..) |name, index| {
            if (index > 0) code.update(&.{header_name_separator});
            code.update(name);
        }
        code.update(&.{message_separator});
        for (read.covered(), 0..) |name, index| {
            if (index > 0) code.update(&.{message_separator});
            code.update(delivered(headers, name) orelse return error.HeaderNotDelivered);
        }
        code.update(&.{message_separator});
        code.update(payload);
        code.final(&out);
        return std.crypto.timing_safe.eql([Hmac.mac_length]u8, out, offered);
    }

    // A signature carrying neither code is refused while it is being read, so what is left here is
    // the body-only scheme.
    code.update(payload);
    code.final(&out);
    return std.crypto.timing_safe.eql([Hmac.mac_length]u8, out, read.body_code.?);
}

/// Verifies a webhook against a moment the caller names.
///
/// The clock window is bilateral. A moment too far in the future is refused exactly like one too far
/// in the past, so the window a given delivery is accepted in stays the width the caller asked for,
/// whichever way a clock drifted.
///
/// `tolerance_seconds` is how far, in either direction, the moment the signature names may sit from
/// `current_time`. Five minutes is a reasonable trade-off between tolerating clock drift and bounding
/// how long a captured delivery can be replayed.
pub fn verifyWithCurrentTime(
    header: []const u8,
    payload: []const u8,
    headers: []const Header,
    subscription_secret: []const u8,
    tolerance_seconds: i64,
    current_time: i64,
) SignatureError!void {
    const read = try parse(header);

    // A header the signature covers that the request did not carry is settled before any code is
    // computed: signing over an absent value would let a sender drop a header and keep the
    // signature valid.
    for (read.covered()) |name| {
        if (delivered(headers, name) == null) return error.HeaderNotDelivered;
    }

    if (!try matches(&read, payload, headers, subscription_secret)) return error.CodeMismatch;

    const drift = current_time - read.timestamp;
    if (drift > tolerance_seconds or -drift > tolerance_seconds) return error.OutsideTolerance;
}

/// Verifies a webhook against the current moment.
///
/// The moment comes from the `Io` the caller hands in rather than from a clock this package reaches
/// on its own, which is what makes the window something a test can move.
pub fn verify(
    io: std.Io,
    header: []const u8,
    payload: []const u8,
    headers: []const Header,
    subscription_secret: []const u8,
    tolerance_seconds: i64,
) SignatureError!void {
    const now = std.Io.Timestamp.now(io, .real).toSeconds();
    return verifyWithCurrentTime(header, payload, headers, subscription_secret, tolerance_seconds, now);
}
