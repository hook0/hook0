//! How a request reaches the API, and what a server on the other end is not allowed to cost.
//!
//! The transport answers the status and the bytes and knows nothing of what the API declares:
//! reading those bytes is the generated half's job, and deciding whether to send them again is the
//! client's. That is what lets one HTTP implementation serve both the hand-written event path and
//! every generated method — a generated group calls whatever `runtime.Transport` it was built on,
//! and this is the one this package ships.
//!
//! The exchange is written here rather than taken from `std.http.Client`, and that is the point: a
//! client that reads a head into a buffer it sized for itself decides how much a broken or hostile
//! server may spend on a caller's behalf, and the shared conformance corpus names four separate
//! ceilings that decision has to honour. Every one of them is applied on the line that crosses it —
//! how many header lines an answer may carry, how long one of them may be, how large the whole head
//! may come to, and how many bytes of body are read off the socket — and each is a real allocation
//! bound rather than a rule somebody remembered: the buffers are sized from the bounds, so a head
//! this client will not accept is one it cannot hold.
//!
//! Every read is given what is left of the attempt's budget, worked out again before each one, so a
//! server answering one byte just under the timeout cannot hold a caller for as long as it likes.
//! Connecting is the one step that is not: `std.Io.Threaded` in Zig 0.16.0 refuses a
//! `ConnectOptions.timeout` outright — `@panic("TODO implement netConnectIpPosix with timeout")` —
//! so that step is bounded by the kernel's own connect timeout until the standard library implements
//! it. It is stated here rather than left for somebody to find.

const std = @import("std");

const runtime = @import("runtime.zig");

const net = std.Io.net;

/// Which of the corpus's causes a failure that produced no answer is.
///
/// The three are told apart because only one of them could end differently. A request that got no
/// answer — a connection refused or reset, an attempt out of time, a body that stopped mid-way —
/// says nothing about whether the API acted on it, which is exactly why a send carries an identifier
/// the client chose itself, and why repeating it is safe and worth doing. An answer that crossed a
/// ceiling this client set for itself draws the same answer the second time, and reading it again
/// four times over costs the caller four times as much for the same failure. A URL nothing can be
/// sent to was never sent at all, and a repetition builds the same unusable request, turning a
/// misconfiguration into a message that accuses the network.
///
/// The names are the ones the shared conformance corpus gives them, so the verdict a client applies
/// and the verdict that corpus writes down are the same words.
pub const Cause = enum {
    no_answer,
    answer_above_a_bound,
    unusable_api_url,

    /// Whether repeating a request that met this could end differently.
    pub fn retryable(self: Cause) bool {
        return self == .no_answer;
    }

    /// What the corpus calls this cause.
    pub fn name(self: Cause) []const u8 {
        return @tagName(self);
    }
};

/// A request the API never answered.
pub const TransportError = error{
    /// The API was reached for and answered nothing this client could read to its end.
    NoAnswer,
    /// The API answered, and what it answered crossed a ceiling this client set for itself.
    AnswerAboveABound,
    /// There is nowhere to send the request, so nothing was sent.
    UnusableApiUrl,
};

/// Which cause an error of this transport is.
pub fn causeOf(failed: TransportError) Cause {
    return switch (failed) {
        error.NoAnswer => .no_answer,
        error.AnswerAboveABound => .answer_above_a_bound,
        error.UnusableApiUrl => .unusable_api_url,
    };
}

/// Longest one attempt at reaching the API is given before it is abandoned, in milliseconds.
///
/// Ten seconds is far above what ingesting an event takes when the API is healthy, and short enough
/// that a stuck connection does not hold a caller for a noticeable time.
pub const default_request_timeout_ms: u64 = 10_000;

/// Largest response body read off a socket, in bytes.
pub const default_max_response_bytes: usize = 8 * 1024 * 1024;

/// How many header lines an answer may carry before it is refused. Sixty-four is well above what the
/// API sends.
pub const default_max_response_headers: usize = 64;

/// Longest one header line may be, name and value together, in bytes.
pub const default_max_header_bytes: usize = 64 * 1024;

/// Largest whole head an answer may carry, every line counted together, in bytes.
///
/// This is the one that bounds what a head costs. A line count and a size per line multiply:
/// sixty-four lines of sixty-four kilobytes each is four megabytes of head, and both of the bounds
/// above admit it. They earn their place by refusing early, on the line that crosses them rather
/// than at the end of the head; this one sets the ceiling.
///
/// Sixteen kilobytes is what Node enforces by default, and matching it is the point: a lower ceiling
/// would refuse heads another target accepts, and a higher one would not bind there at all, leaving
/// each language a different effective limit.
pub const default_max_head_bytes: usize = 16 * 1024;

/// What a request body says it carries, and what an answer is asked for in.
pub const json_media_type = "application/json";

/// The ceilings one exchange is held to.
pub const Bounds = struct {
    request_timeout_ms: u64 = default_request_timeout_ms,
    max_response_bytes: usize = default_max_response_bytes,
    max_response_headers: usize = default_max_response_headers,
    max_header_bytes: usize = default_max_header_bytes,
    max_head_bytes: usize = default_max_head_bytes,
};

/// Where this transport reaches, read out of a URL.
pub const Reached = struct {
    secure: bool,
    host: []const u8,
    port: u16,
    target: []const u8,

    /// What this transport reaches, refusing the text it cannot send anything to.
    ///
    /// Written here rather than taken from `std.Uri`, which reads a table of pieces out of almost
    /// any text: what a transport needs is a refusal for a URL nothing can be sent to, and that is
    /// one of the three causes the corpus classifies.
    pub fn parse(url: []const u8) TransportError!Reached {
        const at = std.mem.indexOf(u8, url, "://") orelse return error.UnusableApiUrl;
        const scheme = url[0..at];
        const rest = url[at + 3 ..];

        const secure = if (std.ascii.eqlIgnoreCase(scheme, "https"))
            true
        else if (std.ascii.eqlIgnoreCase(scheme, "http"))
            false
        else
            return error.UnusableApiUrl;

        const ends = std.mem.indexOfAny(u8, rest, "/?#") orelse rest.len;
        var authority = rest[0..ends];
        const target = if (ends == rest.len) "/" else rest[ends..];

        if (std.mem.lastIndexOfScalar(u8, authority, '@')) |credential| {
            authority = authority[credential + 1 ..];
        }

        var host = authority;
        var port: u16 = if (secure) 443 else 80;
        if (std.mem.lastIndexOfScalar(u8, authority, ':')) |colon| {
            if (std.mem.indexOfScalar(u8, authority[colon..], ']') == null) {
                host = authority[0..colon];
                port = std.fmt.parseInt(u16, authority[colon + 1 ..], 10) catch
                    return error.UnusableApiUrl;
            }
        }
        host = std.mem.trim(u8, host, "[]");
        if (host.len == 0) return error.UnusableApiUrl;

        return .{ .secure = secure, .host = host, .port = port, .target = target };
    }
};

/// One exchange with the API, bounded on every axis a server controls.
pub const Transport = struct {
    io: std.Io,
    base_url: []const u8,
    token: []const u8,
    bounds: Bounds = .{},

    /// What the generated half issues its requests through.
    pub fn any(self: *Transport) runtime.Transport {
        return .{ .context = self, .requestFn = issue };
    }

    fn issue(
        context: *anyopaque,
        allocator: std.mem.Allocator,
        asked: runtime.Request,
    ) anyerror!runtime.Answer {
        const self: *Transport = @ptrCast(@alignCast(context));
        return self.request(allocator, asked);
    }

    /// What the API answered, headers included, whether or not it answered a success.
    ///
    /// Header names are lowercased and a later value wins over an earlier one under the same name,
    /// so a caller reads a header without knowing which case the server wrote it in.
    pub fn deliver(
        self: *Transport,
        allocator: std.mem.Allocator,
        asked: runtime.Request,
    ) !Delivered {
        const reached = try self.resolved(allocator, asked);
        const deadline = self.deadlineFromNow();

        var written: ?[]const u8 = null;
        if (asked.body) |body| {
            var out: std.Io.Writer.Allocating = .init(allocator);
            var stringify: std.json.Stringify = .{ .writer = &out.writer };
            try stringify.write(body);
            written = out.written();
        }

        // The address is parsed rather than resolved: what this client is pointed at is a host it
        // was configured with, and a name that has to be looked up is resolved through `Io`.
        const address = net.IpAddress.parse(reached.host, reached.port) catch
            net.IpAddress.resolve(self.io, reached.host, reached.port) catch
            return error.UnusableApiUrl;

        const stream = address.connect(self.io, .{ .mode = .stream }) catch return error.NoAnswer;
        defer stream.close(self.io);

        try self.send(stream, reached, asked.method, written);
        return self.readAnswer(allocator, stream, deadline);
    }

    /// What the API answered, which is the status and the bytes.
    pub fn request(
        self: *Transport,
        allocator: std.mem.Allocator,
        asked: runtime.Request,
    ) !runtime.Answer {
        const answered = try self.deliver(allocator, asked);
        return .{ .status = answered.status, .payload = answered.payload };
    }

    /// What one exchange answered, beside the body.
    pub const Delivered = struct {
        status: u16,
        payload: []const u8,
        /// The headers the answer carried, lowercased.
        headers: []const Header,

        pub const Header = struct { name: []const u8, value: []const u8 };

        /// What that header carried, when the answer carried it.
        pub fn get(self: Delivered, name: []const u8) ?[]const u8 {
            for (self.headers) |header| {
                if (std.mem.eql(u8, header.name, name)) return header.value;
            }
            return null;
        }
    };

    fn deadlineFromNow(self: *Transport) std.Io.Timestamp {
        const now = std.Io.Timestamp.now(self.io, .awake);
        return now.addDuration(.fromNanoseconds(
            @as(i96, @intCast(self.bounds.request_timeout_ms)) * std.time.ns_per_ms,
        ));
    }

    /// What is left of the budget one attempt was given, worked out again before every read.
    fn within(self: *Transport, deadline: std.Io.Timestamp) TransportError!std.Io.Timeout {
        const now = std.Io.Timestamp.now(self.io, .awake);
        const left = now.durationTo(deadline);
        if (left.nanoseconds <= 0) return error.NoAnswer;
        return .{ .duration = .{ .raw = left, .clock = .awake } };
    }

    /// Where the request lands: the path of the base URL, extended by the operation's own, and then
    /// the query the operation asked for.
    fn resolved(
        self: *Transport,
        allocator: std.mem.Allocator,
        asked: runtime.Request,
    ) !Reached {
        var reached = try Reached.parse(self.base_url);

        var target: std.ArrayList(u8) = .empty;
        if (asked.path.len > 0 and asked.path[0] == '/') {
            try target.appendSlice(allocator, asked.path);
        } else {
            try target.appendSlice(allocator, std.mem.trimEnd(u8, reached.target, "/"));
            try target.append(allocator, '/');
            try target.appendSlice(allocator, asked.path);
        }

        var asking = false;
        for (asked.query) |pair| {
            var buffer: [runtime.max_written_value_bytes]u8 = undefined;
            const carried = pair.value.written(&buffer) orelse continue;
            try target.append(allocator, if (asking) '&' else '?');
            asking = true;
            try target.appendSlice(allocator, try runtime.encoded(allocator, pair.name));
            try target.append(allocator, '=');
            try target.appendSlice(allocator, try runtime.encoded(allocator, carried));
        }

        reached.target = try target.toOwnedSlice(allocator);
        return reached;
    }

    fn send(
        self: *Transport,
        stream: net.Stream,
        reached: Reached,
        method: []const u8,
        written: ?[]const u8,
    ) TransportError!void {
        var buffer: [8 * 1024]u8 = undefined;
        var out = stream.writer(self.io, &buffer);
        const writer = &out.interface;

        writer.print("{s} {s} HTTP/1.1\r\n", .{ method, reached.target }) catch return error.NoAnswer;
        writer.print("Host: {s}:{d}\r\n", .{ reached.host, reached.port }) catch return error.NoAnswer;
        writer.print("Authorization: Bearer {s}\r\n", .{self.token}) catch return error.NoAnswer;
        writer.print("Accept: {s}\r\n", .{json_media_type}) catch return error.NoAnswer;
        writer.writeAll("Connection: close\r\n") catch return error.NoAnswer;
        if (written) |body| {
            writer.print("Content-Type: {s}\r\n", .{json_media_type}) catch return error.NoAnswer;
            writer.print("Content-Length: {d}\r\n", .{body.len}) catch return error.NoAnswer;
        }
        writer.writeAll("\r\n") catch return error.NoAnswer;
        if (written) |body| {
            writer.writeAll(body) catch return error.NoAnswer;
        }
        writer.flush() catch return error.NoAnswer;
    }

    /// Everything the answer is, read under every ceiling this client set for itself.
    ///
    /// The bytes arrive through `receiveTimeout`, which is the one read in the standard library that
    /// takes a deadline, and they are held in one buffer sized from the head and body bounds: the
    /// ceilings are what the allocation is, rather than something checked after the fact.
    fn readAnswer(
        self: *Transport,
        allocator: std.mem.Allocator,
        stream: net.Stream,
        deadline: std.Io.Timestamp,
    ) !Delivered {
        var held: std.ArrayList(u8) = .empty;
        var chunk: [16 * 1024]u8 = undefined;

        var status: ?u16 = null;
        var headers: std.ArrayList(Delivered.Header) = .empty;
        var head_bytes: usize = 0;
        var read_from: usize = 0;
        var body_from: ?usize = null;

        while (true) {
            // Every line of the head is settled out of what has already arrived before another read
            // is asked for, so a head above a ceiling is refused on the line that crosses it.
            while (body_from == null) {
                const line_end = std.mem.indexOfScalarPos(u8, held.items, read_from, '\n') orelse break;
                const line = std.mem.trimEnd(u8, held.items[read_from..line_end], "\r");
                read_from = line_end + 1;

                if (status == null) {
                    if (line.len > self.bounds.max_header_bytes) return error.AnswerAboveABound;
                    const answered = std.mem.indexOfScalar(u8, line, ' ') orelse return error.NoAnswer;
                    status = std.fmt.parseInt(u16, line[answered + 1 .. answered + 4], 10) catch
                        return error.NoAnswer;
                    continue;
                }
                if (line.len == 0) {
                    body_from = read_from;
                    break;
                }

                if (line.len > self.bounds.max_header_bytes) return error.AnswerAboveABound;
                if (headers.items.len == self.bounds.max_response_headers) return error.AnswerAboveABound;
                head_bytes += line.len;
                if (head_bytes > self.bounds.max_head_bytes) return error.AnswerAboveABound;

                const colon = std.mem.indexOfScalar(u8, line, ':') orelse continue;
                try headers.append(allocator, .{
                    .name = try std.ascii.allocLowerString(allocator, line[0..colon]),
                    .value = std.mem.trim(u8, line[colon + 1 ..], " \t"),
                });
            }

            if (body_from) |from| {
                if (held.items.len - from > self.bounds.max_response_bytes) return error.AnswerAboveABound;
            } else if (held.items.len > self.bounds.max_head_bytes + self.bounds.max_header_bytes) {
                return error.AnswerAboveABound;
            }

            const timeout = try self.within(deadline);
            const message = stream.socket.receiveTimeout(self.io, &chunk, timeout) catch |failed| switch (failed) {
                error.Timeout => return error.NoAnswer,
                else => return error.NoAnswer,
            };
            if (message.data.len == 0) break;
            try held.appendSlice(allocator, message.data);
        }

        const from = body_from orelse return error.NoAnswer;
        const payload = held.items[from..];
        if (payload.len > self.bounds.max_response_bytes) return error.AnswerAboveABound;

        return .{
            .status = status orelse return error.NoAnswer,
            .payload = payload,
            .headers = try headers.toOwnedSlice(allocator),
        };
    }
};
