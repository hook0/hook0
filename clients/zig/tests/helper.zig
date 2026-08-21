//! A Hook0 API on a loopback port, and what every case is written against.
//!
//! Every case goes over a real socket: the request the client builds, the headers it sets, the way
//! it reads an answer and the way it gives up on one are all the real ones. Nothing here stands in
//! for a part of the client, so a case that passes says the client works rather than that it was
//! called.
//!
//! The API runs on a thread of its own, answering one connection at a time in the order a case
//! scripted, and it speaks as much HTTP/1.1 as one exchange needs.

const std = @import("std");
const hook0 = @import("hook0");

const net = std.Io.net;

/// Where the shared contract every SDK is held to sits, which is beside the clients that read it.
///
/// A path relative to the package rather than an absolute one: `zig build test` runs from here.
pub const corpus = "../conformance";

/// Largest document of the corpus read back. The corpus is committed, so one above this is one that
/// grew out of shape rather than one somebody meant.
pub const max_corpus_bytes: usize = 512 * 1024;

/// Most connections one case opens, which bounds what the API in the other thread holds at once.
pub const max_connections: usize = 64;

/// Largest request one case sends, in bytes.
pub const max_request_bytes: usize = 64 * 1024;

/// An allocator that writes over what it frees, and keeps it readable afterwards.
///
/// Memory a case reads after the client freed it holds whatever it held before, until something
/// else happens to be given the same address — so a case reading a value the client no longer owns
/// passes or fails on where the pages of one run landed rather than on what the client did. Under
/// this one it reads `written_over` instead, whatever the addresses did: what is freed is painted
/// first, and the allocator underneath hands nothing back to the operating system before the case
/// is over, so the paint stays where a dangling slice points.
///
/// `deinit` answers what it does because the other half of the same question is whether the client
/// freed it at all: memory kept alive past the call it was allocated in and never freed is a leak
/// rather than a fix, and this is what says which of the two happened.
pub const Poisoned = struct {
    holding: std.heap.DebugAllocator(.{
        .safety = true,
        .never_unmap = true,
        .retain_metadata = true,
    }) = .init,

    /// What freed memory reads as. Not zero and not a byte any body of this suite carries, so a
    /// slice pointing into freed memory is one no case can mistake for what it used to say.
    pub const written_over: u8 = 0xA5;

    pub fn allocator(self: *Poisoned) std.mem.Allocator {
        return .{ .ptr = self, .vtable = &.{
            .alloc = alloc,
            .resize = resize,
            .remap = remap,
            .free = free,
        } };
    }

    /// Frees everything still held, and answers whether anything was still held.
    pub fn deinit(self: *Poisoned) std.heap.Check {
        return self.holding.deinit();
    }

    fn alloc(context: *anyopaque, len: usize, alignment: std.mem.Alignment, at: usize) ?[*]u8 {
        const self: *Poisoned = @ptrCast(@alignCast(context));
        return self.holding.allocator().rawAlloc(len, alignment, at);
    }

    fn resize(
        context: *anyopaque,
        memory: []u8,
        alignment: std.mem.Alignment,
        new_len: usize,
        at: usize,
    ) bool {
        const self: *Poisoned = @ptrCast(@alignCast(context));
        if (!self.holding.allocator().rawResize(memory, alignment, new_len, at)) return false;
        if (new_len < memory.len) @memset(memory[new_len..], written_over);
        return true;
    }

    /// Nothing is ever moved in place: a caller told a move is not on answers it by allocating,
    /// copying and freeing, and that last step is the one this exists to paint over.
    fn remap(
        context: *anyopaque,
        memory: []u8,
        alignment: std.mem.Alignment,
        new_len: usize,
        at: usize,
    ) ?[*]u8 {
        _ = .{ context, memory, alignment, new_len, at };
        return null;
    }

    fn free(context: *anyopaque, memory: []u8, alignment: std.mem.Alignment, at: usize) void {
        const self: *Poisoned = @ptrCast(@alignCast(context));
        @memset(memory, written_over);
        self.holding.allocator().rawFree(memory, alignment, at);
    }
};

/// One document of the shared contract, read and parsed, owned by the caller.
pub fn contract(
    io: std.Io,
    allocator: std.mem.Allocator,
    name: []const u8,
) !std.json.Parsed(std.json.Value) {
    const path = try std.fmt.allocPrint(allocator, "{s}/{s}", .{ corpus, name });
    defer allocator.free(path);

    const written = try std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .limited(max_corpus_bytes));
    defer allocator.free(written);

    return std.json.parseFromSlice(std.json.Value, allocator, written, .{});
}

/// What the API answers to one request, in the order a case scripted it.
pub const Scripted = struct {
    status: u16 = 200,
    /// The body, already written out.
    body: []const u8 = "{}",
    /// Headers the answer carries beside the two every answer carries.
    headers: []const [2][]const u8 = &.{},
    /// How long the API sits on the answer before writing it, in milliseconds.
    held_for_ms: u64 = 0,
    /// Whether the API hangs up without answering at all, which is one of the ways the corpus names
    /// for a request that got no answer.
    close: bool = false,
};

/// A request the API received, in the order it received it.
pub const Received = struct {
    method: []const u8,
    target: []const u8,
    headers: []const [2][]const u8,
    body: []const u8,

    /// What that header carried, matched without regard to case as HTTP compares them.
    pub fn get(self: Received, name: []const u8) ?[]const u8 {
        for (self.headers) |header| {
            if (std.ascii.eqlIgnoreCase(header[0], name)) return header[1];
        }
        return null;
    }
};

/// A Hook0 API listening on a loopback port for the lifetime of one case.
pub const FakeApi = struct {
    io: std.Io,
    allocator: std.mem.Allocator,
    server: net.Server,
    port: u16,
    scripted: []const Scripted,
    answered: usize = 0,
    received: std.ArrayList(Received) = .empty,
    thread: ?std.Thread = null,
    stopping: std.atomic.Value(bool) = .init(false),

    pub fn init(
        io: std.Io,
        allocator: std.mem.Allocator,
        scripted: []const Scripted,
    ) !*FakeApi {
        var address: net.IpAddress = .{ .ip4 = .loopback(0) };
        const server = try address.listen(io, .{});

        const self = try allocator.create(FakeApi);
        self.* = .{
            .io = io,
            .allocator = allocator,
            .server = server,
            .port = server.socket.address.getPort(),
            .scripted = scripted,
        };
        self.thread = try std.Thread.spawn(.{}, serve, .{self});
        return self;
    }

    /// Where the client reaches this API.
    pub fn baseUrl(self: *FakeApi, allocator: std.mem.Allocator) ![]const u8 {
        return std.fmt.allocPrint(allocator, "http://127.0.0.1:{d}", .{self.port});
    }

    /// Stops the API and frees everything it held.
    pub fn deinit(self: *FakeApi) void {
        self.stopping.store(true, .release);

        // One connection of our own, so the accept the thread is blocked in answers and it can see
        // that it is being stopped.
        var target: net.IpAddress = .{ .ip4 = .loopback(self.port) };
        if (target.connect(self.io, .{ .mode = .stream })) |stream| {
            stream.close(self.io);
        } else |_| {}

        if (self.thread) |thread| thread.join();
        self.server.deinit(self.io);
        self.received.deinit(self.allocator);
        self.allocator.destroy(self);
    }

    /// How many requests this API received.
    pub fn count(self: *FakeApi) usize {
        return self.received.items.len;
    }

    /// The request it received at that index.
    pub fn at(self: *FakeApi, index: usize) ?Received {
        if (index >= self.received.items.len) return null;
        return self.received.items[index];
    }

    fn serve(self: *FakeApi) void {
        for (0..max_connections) |_| {
            const stream = self.server.accept(self.io) catch return;
            defer stream.close(self.io);

            if (self.stopping.load(.acquire)) return;
            self.exchange(stream) catch {};
        }
    }

    fn exchange(self: *FakeApi, stream: net.Stream) !void {
        var read_buffer: [max_request_bytes]u8 = undefined;
        var reader = stream.reader(self.io, &read_buffer);

        // Inclusive rather than exclusive: the exclusive form leaves the delimiter where it was,
        // so a second call would answer nothing at all and every header would go unread.
        const request_line = try reader.interface.takeDelimiterInclusive('\n');
        const line = std.mem.trimEnd(u8, request_line, "\r\n");
        const first = std.mem.indexOfScalar(u8, line, ' ') orelse return error.NotARequest;
        const second = std.mem.indexOfScalarPos(u8, line, first + 1, ' ') orelse line.len;

        var headers: std.ArrayList([2][]const u8) = .empty;
        var length: usize = 0;
        while (true) {
            const header = try reader.interface.takeDelimiterInclusive('\n');
            const trimmed = std.mem.trimEnd(u8, header, "\r\n");
            if (trimmed.len == 0) break;

            const colon = std.mem.indexOfScalar(u8, trimmed, ':') orelse continue;
            const name = try self.allocator.dupe(u8, trimmed[0..colon]);
            const value = try self.allocator.dupe(u8, std.mem.trim(u8, trimmed[colon + 1 ..], " \t"));
            try headers.append(self.allocator, .{ name, value });
            if (std.ascii.eqlIgnoreCase(name, "content-length")) {
                length = std.fmt.parseInt(usize, value, 10) catch 0;
            }
        }

        var body: []const u8 = "";
        if (length > 0) {
            if (length > max_request_bytes) return error.RequestTooLarge;
            const held = try self.allocator.alloc(u8, length);
            try reader.interface.readSliceAll(held);
            body = held;
        }

        try self.received.append(self.allocator, .{
            .method = try self.allocator.dupe(u8, line[0..first]),
            .target = try self.allocator.dupe(u8, line[first + 1 .. second]),
            .headers = try headers.toOwnedSlice(self.allocator),
            .body = body,
        });

        const scripted = self.next();
        if (scripted.held_for_ms > 0) {
            std.Io.sleep(
                self.io,
                .fromNanoseconds(@as(i96, @intCast(scripted.held_for_ms)) * std.time.ns_per_ms),
                .awake,
            ) catch {};
        }
        // A socket that hangs up mid-exchange, which is one of the ways the corpus names for a
        // request that got no answer at all.
        if (scripted.close) return;

        var write_buffer: [64 * 1024]u8 = undefined;
        var writer = stream.writer(self.io, &write_buffer);
        const out = &writer.interface;

        try out.print("HTTP/1.1 {d} Answer\r\n", .{scripted.status});
        try out.writeAll("Content-Type: application/json\r\n");
        try out.print("Content-Length: {d}\r\n", .{scripted.body.len});
        for (scripted.headers) |header| {
            try out.print("{s}: {s}\r\n", .{ header[0], header[1] });
        }
        try out.writeAll("Connection: close\r\n\r\n");
        try out.writeAll(scripted.body);
        try out.flush();
    }

    fn next(self: *FakeApi) Scripted {
        const index = self.answered;
        self.answered += 1;
        if (index >= self.scripted.len) {
            return .{ .status = 500, .body = "{\"error\":\"the case scripted no answer for this request\"}" };
        }
        return self.scripted[index];
    }
};

/// A schedule short enough that a case spends its time on requests rather than on waiting.
pub fn retries(max_attempts: u32) hook0.RetryPolicy {
    return .{
        .max_attempts = max_attempts,
        .initial_backoff_ms = 5,
        .max_backoff_ms = 5,
        .max_total_delay_ms = 1_000,
    };
}

/// The bounds a case holds one send to.
pub fn options(max_attempts: u32) hook0.Options {
    return .{ .retry_policy = retries(max_attempts), .request_timeout_ms = 5_000 };
}

/// A client pointed at that API.
pub fn client(
    io: std.Io,
    allocator: std.mem.Allocator,
    api: *FakeApi,
    chosen: hook0.Options,
) !hook0.Client {
    return .init(io, try api.baseUrl(allocator), "app-123", "token-xyz", chosen);
}

/// An event a case sends.
pub fn anEvent() hook0.Event {
    return .{
        .event_type = "auth.user.create",
        .payload = "{\"email\": \"test@example.com\"}",
        .payload_content_type = "application/json",
        .labels = &.{.{ .key = "environment", .value = "production" }},
    };
}

/// What the API answers when it takes the event.
pub fn ingested(event_id: []const u8) Scripted {
    _ = event_id;
    return .{
        .status = 201,
        .body =
        \\{"application_id":"app-123","event_id":"01961234-5678-7abc-8def-0123456789ac","received_at":"2026-01-01"}
        ,
    };
}

/// The identifier the API in these cases says it ingested every event under.
pub const ingested_id = "01961234-5678-7abc-8def-0123456789ac";

/// What the API says when it refuses a request, in the shape every Hook0 failure takes.
pub fn refusal(
    allocator: std.mem.Allocator,
    status: u16,
    problem: []const u8,
    headers: []const [2][]const u8,
) !Scripted {
    return .{
        .status = status,
        .headers = headers,
        .body = try std.fmt.allocPrint(allocator,
            \\{{"id":"{s}","status":{d},"title":"refused","detail":"what the corpus scripted","type":"https://hook0.com/documentation/errors/{s}"}}
        , .{ problem, status, problem }),
    };
}
