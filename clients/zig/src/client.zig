//! Sending events to Hook0, idempotently and under bounds the caller sets.
//!
//! Every event is sent under an identifier this client knows: the one set on the event, or a UUIDv7
//! it generates when the event carries none. Passing none does not mean the identifier comes from
//! Hook0 — the value comes from here, travels with the request, and is what `sendEvent` answers.
//!
//! That is what makes retrying safe. Hook0 keys events on that identifier, so a request repeated
//! after a network failure or a server error ingests the event once rather than twice; without a
//! client-chosen identifier, a repeated request would create a second event and deliver it to every
//! subscriber. It also gives the answer to a retry its meaning: `EventAlreadyIngested` in reply to a
//! *repeated* request says an earlier attempt of that same send reached the API, so the send
//! succeeded. The same answer to a *first* attempt is a genuine conflict and is reported as one.
//!
//! Only what could end differently is retried: a request that got no answer, a server error, and an
//! instance saying it is being reached faster than it accepts. What the API refuses outright — a
//! quota that is spent, a payload it will not read — is reported as is, since repeating it would only
//! spend the same round trip again. The verdict for every problem the API can report is written down
//! in the conformance corpus committed beside this package, which the suite here reads.

const std = @import("std");

const runtime = @import("runtime.zig");
const transport = @import("transport.zig");

/// How a client spaces out the attempts of a single send.
///
/// The delay before a retry doubles from `initial_backoff_ms` and is capped by `max_backoff_ms`; the
/// delay actually waited is then drawn anywhere between zero and that ceiling, so that emitters which
/// failed at the same moment do not come back at the same moment. Retrying stops as soon as the
/// delays of the send would add up to more than `max_total_delay_ms`.
///
/// The defaults are four attempts spread over at most five seconds: three retries absorb the blips a
/// webhook emitter meets in production — a connection reset, a rolling deployment answering 503 —
/// without holding the caller for long, and the five-second budget bounds what the worst send costs
/// whatever the individual delays turn out to be.
pub const RetryPolicy = struct {
    max_attempts: u32 = 4,
    initial_backoff_ms: u64 = 100,
    max_backoff_ms: u64 = 2_000,
    max_total_delay_ms: u64 = 5_000,

    /// Most attempts a policy can ever make, whatever `max_attempts` says.
    ///
    /// A policy is configuration, and configuration can be wrong; this cap keeps a mistyped
    /// `max_attempts` from turning one send into an unbounded series of requests.
    pub const max_attempts_cap: u32 = 16;

    /// Beyond this many doublings any backoff has long since reached its ceiling.
    pub const max_backoff_doublings: u6 = 30;

    /// A policy that never retries: one attempt, and the caller hears what it answered.
    pub const disabled: RetryPolicy = .{
        .max_attempts = 1,
        .initial_backoff_ms = 0,
        .max_backoff_ms = 0,
        .max_total_delay_ms = 0,
    };

    /// Attempts this policy actually makes: `max_attempts`, brought inside `1..max_attempts_cap`.
    pub fn attempts(self: RetryPolicy) u32 {
        return std.math.clamp(self.max_attempts, 1, max_attempts_cap);
    }

    /// Ceiling of the delay before retry number `retry_number`, where `1` is the first retry.
    ///
    /// It doubles from `initial_backoff_ms` and never exceeds `max_backoff_ms`, so the ceilings of
    /// successive retries never decrease.
    pub fn backoffCeiling(self: RetryPolicy, retry_number: u32) u64 {
        const doublings: u6 = @intCast(@min(retry_number -| 1, max_backoff_doublings));
        const grown = std.math.shlExact(u64, self.initial_backoff_ms, doublings) catch
            return self.max_backoff_ms;
        return @min(grown, self.max_backoff_ms);
    }

    /// How long to wait before retry number `retry_number`, given a draw in `[0, 1]`.
    ///
    /// A draw outside that range is brought back inside it, which is what makes an unusable source of
    /// randomness one that waits longer rather than one that waits less. The answer is then held to
    /// the ceiling a second time: past 2^53 milliseconds a `f64` no longer counts in whole ones, and
    /// a ceiling that rounded up would be a bound the delay it bounds sits above.
    pub fn delay(self: RetryPolicy, retry_number: u32, drawn: f64) u64 {
        const ceiling = self.backoffCeiling(retry_number);
        const usable = if (std.math.isNan(drawn)) 1.0 else std.math.clamp(drawn, 0.0, 1.0);
        const scaled: u64 = @intFromFloat(@as(f64, @floatFromInt(ceiling)) * usable);
        return @min(scaled, ceiling);
    }
};

/// Every bound a client applies to one send.
pub const Options = struct {
    retry_policy: RetryPolicy = .{},
    request_timeout_ms: u64 = transport.default_request_timeout_ms,
    /// Largest event payload the client agrees to send, in bytes.
    ///
    /// Hook0's API refuses request bodies above 2 MiB, so a payload above 1 MiB is already at risk of
    /// being refused once the JSON envelope around it — metadata, labels, identifiers — is counted.
    /// The client rules such an event out rather than spending a round trip, and every retry after
    /// it, on a request that cannot be accepted.
    max_payload_bytes: usize = 1024 * 1024,
    max_response_bytes: usize = transport.default_max_response_bytes,
    max_response_headers: usize = transport.default_max_response_headers,
    max_header_bytes: usize = transport.default_max_header_bytes,
    max_head_bytes: usize = transport.default_max_head_bytes,
};

/// An event to send to Hook0.
///
/// `event_id` is the caller's to set when it already has one to key the event on. Left unset, the
/// client generates a UUIDv7, sends it and answers it.
pub const Event = struct {
    event_type: []const u8,
    payload: []const u8,
    payload_content_type: []const u8,
    labels: []const Label = &.{},
    metadata: ?[]const Label = null,
    /// When the event happened, as RFC 3339 spells a moment; the current one when unset.
    occurred_at: ?[]const u8 = null,
    event_id: ?[]const u8 = null,

    pub const Label = struct { key: []const u8, value: []const u8 };
};

/// An event type, read out of the `service.resource_type.verb` it is written as.
pub const EventType = struct {
    service: []const u8,
    resource_type: []const u8,
    verb: []const u8,

    pub const ParseError = error{NotAnEventType};

    /// Reads an event type, refusing one that does not name all three of its parts.
    pub fn parse(spelled: []const u8) ParseError!EventType {
        var parts = std.mem.splitScalar(u8, spelled, '.');
        const service = parts.next() orelse return error.NotAnEventType;
        const resource_type = parts.next() orelse return error.NotAnEventType;
        const verb = parts.next() orelse return error.NotAnEventType;
        if (parts.next() != null) return error.NotAnEventType;

        for ([_][]const u8{ service, resource_type, verb }) |part| {
            if (part.len == 0) return error.NotAnEventType;
            for (part) |character| {
                const usable = switch (character) {
                    'A'...'Z', 'a'...'z', '0'...'9', '_' => true,
                    else => false,
                };
                if (!usable) return error.NotAnEventType;
            }
        }
        return .{ .service = service, .resource_type = resource_type, .verb = verb };
    }

    /// The event type as the API reads one.
    pub fn written(self: EventType, allocator: std.mem.Allocator) ![]const u8 {
        return std.fmt.allocPrint(allocator, "{s}.{s}.{s}", .{ self.service, self.resource_type, self.verb });
    }
};

/// The identifier Hook0 gives the problem it answers when an event identifier is already taken.
pub const already_ingested = "EventAlreadyIngested";

/// The identifier Hook0 gives the problem it answers when requests are reaching the instance faster
/// than it accepts them.
///
/// It shares its status with the quota problems, and is the only one of them worth repeating: a quota
/// clears when a plan changes or a day turns, neither of which happens inside the seconds a send is
/// given, while pacing clears on its own and the answer says when.
pub const rate_limited = "RateLimited";

/// What Hook0 answers when the event identifier a request carries is already taken.
pub const conflict: u16 = 409;

/// What Hook0 answers both when a quota is spent and when requests are coming in faster than the
/// instance accepts them.
pub const paced: u16 = 429;

/// First status saying the failure is on Hook0's side, and so could clear on its own.
pub const lowest_server_error: u16 = 500;

/// What the API names the delay before the request becomes servable in, in whole seconds.
pub const delay_header = "retry-after";

/// Longest value of that header read, and the largest delay it may name.
pub const max_delay_header_bytes: usize = 32;
pub const max_named_delay_seconds: u64 = (1 << 31) - 1;

/// Where an event is ingested, under the API URL.
pub const event_path = "event";

/// Where event types are read and created, under the API URL.
pub const event_types_path = "event_types";

/// Everything that stops a send from landing.
pub const SendError = error{
    /// The payload is larger than this client sends.
    PayloadTooLarge,
    /// The API refused the event, and repeating the request would meet the same refusal.
    Refused,
    /// Every attempt the policy allowed was made, and none of them landed.
    RetriesExhausted,
    /// The API answered a success this client cannot read.
    Unreadable,
};

/// A UUIDv7, the shape of identifier Hook0 mints when it is the one choosing.
///
/// Its leading 48 bits are the moment it was minted, in milliseconds, so identifiers generated in
/// sequence are ordered — which is what keeps the index they end up in from being written all over.
/// The moment comes from the `Io` the caller handed in rather than from a clock this package reaches
/// on its own, which is what makes it something a test can move.
pub fn generateEventId(io: std.Io, out: *[36]u8) void {
    var bytes: [16]u8 = undefined;
    io.random(&bytes);

    const milliseconds: u64 = @intCast(@max(std.Io.Timestamp.now(io, .real).toMilliseconds(), 0));
    for (0..6) |index| {
        bytes[index] = @truncate(milliseconds >> @intCast(8 * (5 - index)));
    }
    bytes[6] = (bytes[6] & 0x0F) | 0x70;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    var written: [32]u8 = undefined;
    _ = std.fmt.bufPrint(&written, "{x}", .{&bytes}) catch unreachable;
    _ = std.fmt.bufPrint(out, "{s}-{s}-{s}-{s}-{s}", .{
        written[0..8],
        written[8..12],
        written[12..16],
        written[16..20],
        written[20..32],
    }) catch unreachable;
}

/// What one attempt at sending an event ended with.
const Attempt = struct {
    ingested: ?[]const u8 = null,
    already_ingested: bool = false,
    retryable: bool = false,
    retry_after_ms: ?u64 = null,
    detail: []const u8 = "",
};

/// The Hook0 client, built once and shared wherever an application sends events.
pub const Client = struct {
    io: std.Io,
    api_url: []const u8,
    application_id: []const u8,
    options: Options,
    inner: transport.Transport,
    /// What the last failure of this client said, which an error alone cannot carry.
    detail: []const u8 = "",

    pub fn init(
        io: std.Io,
        api_url: []const u8,
        application_id: []const u8,
        token: []const u8,
        options: Options,
    ) Client {
        return .{
            .io = io,
            .api_url = api_url,
            .application_id = application_id,
            .options = options,
            .inner = .{
                .io = io,
                .base_url = api_url,
                .token = token,
                .bounds = .{
                    .request_timeout_ms = options.request_timeout_ms,
                    .max_response_bytes = options.max_response_bytes,
                    .max_response_headers = options.max_response_headers,
                    .max_header_bytes = options.max_header_bytes,
                    .max_head_bytes = options.max_head_bytes,
                },
            },
        };
    }

    /// What one request is issued through, which is also what a generated operation group is built
    /// on.
    pub fn transportOf(self: *Client) runtime.Transport {
        return self.inner.any();
    }

    /// Sends an event, and answers the identifier it was sent under.
    ///
    /// The answer owns the arena everything it points into was allocated from, so one `deinit` frees
    /// the identifier, the body that was sent and everything read back.
    pub fn sendEvent(
        self: *Client,
        allocator: std.mem.Allocator,
        event: Event,
    ) !runtime.Owned([]const u8) {
        var owned: runtime.Owned([]const u8) = try .init(allocator);
        errdefer owned.deinit();
        const arena = owned.arena.allocator();

        var minted: [36]u8 = undefined;
        const event_id = if (event.event_id) |carried| carried else blk: {
            generateEventId(self.io, &minted);
            break :blk try arena.dupe(u8, &minted);
        };

        if (event.payload.len > self.options.max_payload_bytes) {
            self.detail = try std.fmt.allocPrint(
                arena,
                "the payload of event {s} is {d} bytes, above the {d} this client sends",
                .{ event_id, event.payload.len, self.options.max_payload_bytes },
            );
            return error.PayloadTooLarge;
        }

        const body = try self.fullEvent(arena, event, event_id);
        const policy = self.options.retry_policy;

        var issued: u32 = 0;
        var waited: u64 = 0;
        while (true) {
            issued += 1;
            const outcome = try self.attempt(arena, body);

            if (outcome.ingested) |carried| {
                owned.value = carried;
                return owned;
            }
            if (outcome.already_ingested) {
                if (issued > 1) {
                    owned.value = event_id;
                    return owned;
                }
                self.detail = outcome.detail;
                return error.Refused;
            }

            const budget = self.options.retry_policy.max_total_delay_ms;
            if (!outcome.retryable or issued >= policy.attempts()) {
                self.detail = try std.fmt.allocPrint(
                    arena,
                    "sending event {s} gave up after {d} attempts over {d}ms: {s}",
                    .{ event_id, issued, waited, runtime.preview(outcome.detail) },
                );
                return if (issued > 1) error.RetriesExhausted else error.Refused;
            }

            // What the API asked for when it asked for anything, and this client's own schedule
            // otherwise. Either way it is cut down to what is left of the budget every delay of one
            // send shares, so a delay written by the other end cannot stretch a send past what the
            // caller allowed for it.
            // Jitter only has to keep emitters that failed together from coming back together; it
            // does not have to be unpredictable, so whatever the `Io` draws is enough.
            var drawn: std.Random.IoSource = .{ .io = self.io };
            const scheduled = outcome.retry_after_ms orelse
                policy.delay(issued, drawn.interface().float(f64));
            const waiting = @min(scheduled, budget -| waited);
            if (waiting > 0) {
                std.Io.sleep(self.io, .fromNanoseconds(@as(i96, @intCast(waiting)) * std.time.ns_per_ms), .awake) catch {};
            }
            waited += waiting;
        }
    }

    /// Creates the event types the application does not declare yet, and answers those.
    pub fn upsertEventTypes(
        self: *Client,
        allocator: std.mem.Allocator,
        event_types: []const []const u8,
    ) !runtime.Owned([]const []const u8) {
        var owned: runtime.Owned([]const []const u8) = try .init(allocator);
        errdefer owned.deinit();
        const arena = owned.arena.allocator();

        if (event_types.len == 0) {
            owned.value = &.{};
            return owned;
        }

        var wanted: std.ArrayList(EventType) = .empty;
        for (event_types) |written| {
            try wanted.append(arena, try EventType.parse(written));
        }

        const declared = try self.declaredEventTypes(arena);

        var created: std.ArrayList([]const u8) = .empty;
        for (wanted.items) |event_type| {
            const written = try event_type.written(arena);
            var known = false;
            for (declared) |name| {
                known = known or std.mem.eql(u8, name, written);
            }
            if (!known) {
                try self.createEventType(arena, event_type);
                try created.append(arena, written);
            }
        }

        owned.value = try created.toOwnedSlice(arena);
        return owned;
    }

    /// An event as the API reads one.
    fn fullEvent(
        self: *Client,
        arena: std.mem.Allocator,
        event: Event,
        event_id: []const u8,
    ) !std.json.Value {
        var out: std.json.ObjectMap = .empty;
        try out.put(arena, "event_id", .{ .string = event_id });
        try out.put(arena, "application_id", .{ .string = self.application_id });
        try out.put(arena, "event_type", .{ .string = event.event_type });
        try out.put(arena, "payload", .{ .string = event.payload });
        try out.put(arena, "payload_content_type", .{ .string = event.payload_content_type });
        try out.put(arena, "occurred_at", .{ .string = event.occurred_at orelse try self.nowAsMoment(arena) });
        try out.put(arena, "labels", try labelled(arena, event.labels));
        if (event.metadata) |carried| {
            try out.put(arena, "metadata", try labelled(arena, carried));
        }
        return .{ .object = out };
    }

    fn labelled(arena: std.mem.Allocator, labels: []const Event.Label) !std.json.Value {
        var out: std.json.ObjectMap = .empty;
        for (labels) |label| {
            try out.put(arena, label.key, .{ .string = label.value });
        }
        return .{ .object = out };
    }

    /// The current moment, as RFC 3339 spells one.
    fn nowAsMoment(self: *Client, arena: std.mem.Allocator) ![]const u8 {
        const seconds = std.Io.Timestamp.now(self.io, .real).toSeconds();
        const epoch: std.time.epoch.EpochSeconds = .{ .secs = @intCast(@max(seconds, 0)) };
        const day = epoch.getEpochDay();
        const year_day = day.calculateYearDay();
        const month_day = year_day.calculateMonthDay();
        const clock = epoch.getDaySeconds();

        return std.fmt.allocPrint(arena, "{d:0>4}-{d:0>2}-{d:0>2}T{d:0>2}:{d:0>2}:{d:0>2}Z", .{
            year_day.year,
            month_day.month.numeric(),
            month_day.day_index + 1,
            clock.getHoursIntoDay(),
            clock.getMinutesIntoHour(),
            clock.getSecondsIntoMinute(),
        });
    }

    /// One attempt at sending an already-bounded event.
    fn attempt(self: *Client, arena: std.mem.Allocator, body: std.json.Value) !Attempt {
        const answered = self.inner.deliver(arena, .{
            .method = "POST",
            .path = event_path,
            .body = body,
        }) catch |failed| switch (failed) {
            // Only the three the corpus classifies are read as a failure a send decides about; a
            // failure of this process — no room, a body that would not write — is the caller's.
            error.NoAnswer => return .{ .retryable = true, .detail = @errorName(failed) },
            error.AnswerAboveABound, error.UnusableApiUrl => return .{
                .retryable = false,
                .detail = @errorName(failed),
            },
            else => return failed,
        };

        return self.readAttempt(arena, answered);
    }

    /// What the API answered one attempt, and whether repeating it could end differently.
    fn readAttempt(self: *Client, arena: std.mem.Allocator, answered: transport.Transport.Delivered) !Attempt {
        _ = self;
        const named = membersOf(arena, answered.payload);

        if (answered.status >= 200 and answered.status < 300) {
            const ingested = named.text("event_id") orelse return .{
                .detail = try std.fmt.allocPrint(
                    arena,
                    "Hook0 answered {d} without an event id",
                    .{answered.status},
                ),
            };
            return .{ .ingested = ingested };
        }

        const problem = named.text("id");
        if (answered.status == conflict and problem != null and std.mem.eql(u8, problem.?, already_ingested)) {
            return .{ .already_ingested = true, .detail = answered.payload };
        }

        return .{
            .retryable = retryable(answered.status, problem),
            .retry_after_ms = namedDelay(answered.get(delay_header)),
            .detail = answered.payload,
        };
    }

    /// The event types an application already declares, out of what the API answered.
    fn declaredEventTypes(self: *Client, arena: std.mem.Allocator) ![]const []const u8 {
        const answered = try self.inner.request(arena, .{
            .method = "GET",
            .path = event_types_path,
            .query = &.{.{ .name = "application_id", .value = .{ .text = self.application_id } }},
        });
        if (answered.status < 200 or answered.status >= 300) {
            self.detail = answered.payload;
            return error.Refused;
        }

        const document = runtime.decodePayload(arena, answered.payload) catch return error.Unreadable;
        const items = switch (document) {
            .array => |held| held,
            else => return error.Unreadable,
        };

        var declared: std.ArrayList([]const u8) = .empty;
        for (items.items) |item| {
            const fields = switch (item) {
                .object => |held| held,
                else => continue,
            };
            const name = fields.get("event_type_name") orelse continue;
            switch (name) {
                .string => |held| try declared.append(arena, held),
                else => {},
            }
        }
        return declared.toOwnedSlice(arena);
    }

    /// Declares one event type on the application.
    fn createEventType(self: *Client, arena: std.mem.Allocator, event_type: EventType) !void {
        var out: std.json.ObjectMap = .empty;
        try out.put(arena, "application_id", .{ .string = self.application_id });
        try out.put(arena, "service", .{ .string = event_type.service });
        try out.put(arena, "resource_type", .{ .string = event_type.resource_type });
        try out.put(arena, "verb", .{ .string = event_type.verb });

        const answered = try self.inner.request(arena, .{
            .method = "POST",
            .path = event_types_path,
            .body = .{ .object = out },
        });
        if (answered.status < 200 or answered.status >= 300) {
            self.detail = answered.payload;
            return error.Refused;
        }
    }
};

/// Whether repeating a request the API answered that way could end differently.
///
/// The status decides on its own everywhere but under the one it answers both a spent quota and a
/// paced instance with: a quota clears when a plan changes or a day turns, and neither is something a
/// send spending seconds can wait for. Only the problem the body names tells the two apart, and a
/// body naming a problem this client has never heard of falls back to what the status says.
fn retryable(status: u16, problem: ?[]const u8) bool {
    if (status == paced) {
        return problem != null and std.mem.eql(u8, problem.?, rate_limited);
    }
    return status >= lowest_server_error;
}

/// The delay the API named before the request becomes servable, in milliseconds.
///
/// Only a whole number of seconds is read. The header may also carry a date, which is a clock this
/// client would be comparing against its own, and anything else is a header nobody meant: both leave
/// the client's own schedule in place rather than being guessed at.
fn namedDelay(carried: ?[]const u8) ?u64 {
    const written = std.mem.trim(u8, carried orelse return null, " \t");
    if (written.len == 0 or written.len > max_delay_header_bytes) return null;
    for (written) |character| {
        if (character < '0' or character > '9') return null;
    }

    const seconds = std.fmt.parseInt(u64, written, 10) catch return null;
    if (seconds > max_named_delay_seconds) return null;
    return seconds * std.time.ms_per_s;
}

/// The members a body carries, read once for the two this client looks at.
///
/// A body that is not a document at all is one carrying no members, rather than a second failure on
/// top of the one being read.
const Members = struct {
    document: ?std.json.Value,

    /// What that member carries, when it carries a string.
    fn text(self: Members, name: []const u8) ?[]const u8 {
        const fields = switch (self.document orelse return null) {
            .object => |held| held,
            else => return null,
        };
        return switch (fields.get(name) orelse return null) {
            .string => |carried| carried,
            else => null,
        };
    }
};

fn membersOf(arena: std.mem.Allocator, payload: []const u8) Members {
    return .{ .document = runtime.decodePayload(arena, payload) catch null };
}
