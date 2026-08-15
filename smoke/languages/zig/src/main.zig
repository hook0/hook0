//! The Zig client against a Hook0 that is really running.
//!
//! Three things the loopback suite cannot ask: whether an application secret the API minted is
//! accepted, whether a second send under an identifier already ingested is reported as the conflict
//! it is, and whether a signature the output worker computed verifies. Everything else about this
//! client is settled by `clients/zig/tests`.

const std = @import("std");
const hook0 = @import("hook0");

/// The conflict the API answers a duplicated ingestion with.
const already_ingested = "EventAlreadyIngested";

/// The most bytes of one part of the delivery read back. Every one of them is written by the
/// harness a moment earlier and measured in hundreds of bytes.
const max_part_bytes = 1024 * 1024;

/// The most bytes one setting may be spelled with.
const max_setting_bytes = 4096;

/// What the harness passes, taken from the process rather than from a clock or a global: the
/// entry point is handed the environment, the allocator and the `Io` this program runs on.
pub fn main(init: std.process.Init) !void {
    const allocator = init.gpa;
    const io = init.io;

    var said: [4096]u8 = undefined;
    var out = std.Io.File.stdout().writerStreaming(io, &said);
    var refusal: [4096]u8 = undefined;
    var err = std.Io.File.stderr().writerStreaming(io, &refusal);

    smoke(allocator, io, init.environ_map, &out.interface) catch |refused| {
        err.interface.print("the zig smoke was refused: {t}\n", .{refused}) catch {};
        err.interface.flush() catch {};
        std.process.exit(1);
    };
}

fn smoke(
    allocator: std.mem.Allocator,
    io: std.Io,
    environment: *std.process.Environ.Map,
    out: *std.Io.Writer,
) !void {
    const api_url = try setting(environment, "HOOK0_API_URL");
    const application_id = try setting(environment, "HOOK0_APPLICATION_ID");
    const token = try setting(environment, "HOOK0_TOKEN");
    const event_type = try setting(environment, "HOOK0_EVENT_TYPE");
    const delivery = try setting(environment, "HOOK0_DELIVERY");

    var client: hook0.Client = .init(io, api_url, application_id, token, .{});

    const sent = try client.sendEvent(allocator, event(event_type, null));
    defer sent.deinit();
    try out.print("ingested {s}\n", .{sent.value});

    const identifier = try allocator.dupe(u8, sent.value);
    defer allocator.free(identifier);

    if (client.sendEvent(allocator, event(event_type, identifier))) |accepted| {
        accepted.deinit();
        try out.print("sending the same event twice was accepted twice\n", .{});
        return error.DuplicateAccepted;
    } else |_| {
        if (std.mem.indexOf(u8, client.detail, already_ingested) == null) {
            try out.print(
                "the second send failed without naming {s}: {s}\n",
                .{ already_ingested, client.detail },
            );
            return error.ConflictNotReported;
        }
    }
    try out.print("the second send reported {s}\n", .{already_ingested});

    try verify(allocator, io, delivery);
    try out.print("the signature the instance produced verifies\n", .{});
    try out.flush();
}

/// The event both sends carry, under the identifier the caller names.
fn event(event_type: []const u8, event_id: ?[]const u8) hook0.Event {
    return .{
        .event_type = event_type,
        .payload = "{\"from\":\"the zig smoke\"}",
        .payload_content_type = "application/json",
        .labels = &.{.{ .key = "language", .value = "zig" }},
        .event_id = event_id,
    };
}

/// Verifies what the output worker really delivered, with this client's own verification.
fn verify(allocator: std.mem.Allocator, io: std.Io, delivery: []const u8) !void {
    const signature = try part(allocator, io, delivery, "signature");
    defer allocator.free(signature);
    const secret = try part(allocator, io, delivery, "secret");
    defer allocator.free(secret);
    const body = try part(allocator, io, delivery, "body");
    defer allocator.free(body);
    const tolerance = try part(allocator, io, delivery, "tolerance");
    defer allocator.free(tolerance);
    const lines = try part(allocator, io, delivery, "headers");
    defer allocator.free(lines);

    var headers: std.ArrayList(hook0.signature.Header) = .empty;
    defer headers.deinit(allocator);
    var walking = std.mem.splitScalar(u8, lines, '\n');
    while (walking.next()) |line| {
        const at = std.mem.indexOf(u8, line, ": ") orelse continue;
        try headers.append(allocator, .{ .name = line[0..at], .value = line[at + 2 ..] });
    }

    try hook0.verifyWebhookSignature(
        io,
        std.mem.trim(u8, signature, " \t\r\n"),
        body,
        headers.items,
        std.mem.trim(u8, secret, " \t\r\n"),
        try std.fmt.parseInt(i64, std.mem.trim(u8, tolerance, " \t\r\n"), 10),
    );
}

/// One part of the delivery, as the harness wrote it down.
fn part(allocator: std.mem.Allocator, io: std.Io, delivery: []const u8, name: []const u8) ![]u8 {
    const path = try std.fs.path.join(allocator, &.{ delivery, name });
    defer allocator.free(path);
    return std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .limited(max_part_bytes));
}

/// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report
/// a failure of the client for something the harness never handed it.
fn setting(environment: *std.process.Environ.Map, name: []const u8) ![]const u8 {
    const value = environment.get(name) orelse return error.SettingNotSet;
    if (value.len == 0 or value.len > max_setting_bytes) return error.SettingNotSet;
    return value;
}
