//! What the generated half of this package reads and writes values through.
//!
//! Everything here is hand-written and never regenerated. It is the one seam between what the API
//! declares — the structs, the problems and the methods the generator writes under `generated/` —
//! and what it does not: how a JSON document is turned into a value, and what happens to a document
//! that does not say what it was declared to say.
//!
//! Reading is deliberately strict. A member the document declares as a string and the API answered
//! as a number stops the read rather than yielding a value whose documentation lies about what it
//! holds. Every failure of that kind is one of `DecodeError`, and the error is the message: Zig
//! errors carry no payload, so what a Ruby client would have said in a sentence is said by the name
//! of the error instead.
//!
//! Allocation is explicit and, more to the point, *owned*. Everything a decoded value points into —
//! the body that arrived, the document it was parsed into, every slice read out of it — is allocated
//! from one arena, and that arena travels with the value as an `Owned(T)`. One `deinit` frees the
//! lot; nothing is reachable afterwards, and nothing outlives it by accident. That is the shape the
//! ceilings of the shared corpus take here: a bound the other targets keep by discipline is a bound
//! the allocator applies, since nothing is read into memory the caller did not hand over.

const std = @import("std");

/// What the API answered is not what it declares it answers.
///
/// One error per way a document can fail to be what it says it is, rather than one error and a
/// string: the error is what a caller matches on, so it is what has to carry the meaning.
pub const DecodeError = error{
    /// The value read is not a JSON object, where a declared type was expected.
    NotAnObject,
    /// The value read is not a JSON array, where a list was expected.
    NotAnArray,
    /// The value read is not a string.
    NotAString,
    /// The value read is not a whole number, or does not fit the width it is declared at.
    NotAWholeNumber,
    /// The value read is not a number.
    NotANumber,
    /// The value read is not a boolean.
    NotABoolean,
    /// A member the document requires was not answered.
    MemberMissing,
    /// A string the document draws from a closed list is not one of the values it declares.
    ValueNotDeclared,
    /// The body is larger than this client reads.
    PayloadTooLarge,
    /// The body is not JSON.
    PayloadNotJson,
    /// There was no room to hold what was read.
    OutOfMemory,
};

/// What can go wrong writing a value back out, which is running out of room and nothing else.
pub const WriteError = std.mem.Allocator.Error;

/// Largest JSON document read out of a response body, in bytes. The transport caps what it reads off
/// a socket; this caps what is handed to the parser whichever way the bytes arrived.
pub const max_payload_bytes: usize = 8 * 1024 * 1024;

/// Deepest a JSON document may nest before the parser gives up, which is what keeps a document that
/// is nothing but brackets from growing the stack.
pub const max_payload_nesting: usize = 64;

/// Longest fragment of a response body a message carries. Bodies are answered by a server this
/// package does not control, so they are cut at a fixed budget rather than copied whole into
/// whatever the caller logs.
pub const max_preview_bytes: usize = 256;

/// An arena something is held in, which travels with what it holds rather than with the call.
///
/// The arena sits behind a pointer for the same reason [Owned]'s does: an allocator answered by an
/// arena holds the address of that arena, so an arena that moved would hand out an allocator
/// pointing at where it used to be.
///
/// Whoever asked for one frees it, once, with `deinit`. Nothing else refers to it.
pub const Kept = struct {
    arena: *std.heap.ArenaAllocator,

    pub fn init(allocator: std.mem.Allocator) std.mem.Allocator.Error!Kept {
        const arena = try allocator.create(std.heap.ArenaAllocator);
        arena.* = .init(allocator);
        return .{ .arena = arena };
    }

    /// Frees everything held in here, in one step.
    pub fn deinit(self: Kept) void {
        const child = self.arena.child_allocator;
        self.arena.deinit();
        child.destroy(self.arena);
    }
};

/// A value together with the arena everything it points into was allocated from.
///
/// The arena sits behind a pointer rather than inside this struct on purpose: an allocator answered
/// by an arena holds the address of that arena, so an arena that moved — which is exactly what
/// returning one by value does — would hand out an allocator pointing at where it used to be.
pub fn Owned(comptime T: type) type {
    return struct {
        arena: *std.heap.ArenaAllocator,
        value: T,

        const Self = @This();

        pub fn init(allocator: std.mem.Allocator) std.mem.Allocator.Error!Self {
            const arena = try allocator.create(std.heap.ArenaAllocator);
            arena.* = .init(allocator);
            return .{ .arena = arena, .value = undefined };
        }

        /// Frees the value and everything it points into, in one step.
        pub fn deinit(self: Self) void {
            const child = self.arena.child_allocator;
            self.arena.deinit();
            child.destroy(self.arena);
        }
    };
}

/// What one request the generated half issues is made of.
pub const Request = struct {
    method: []const u8,
    path: []const u8,
    query: []const QueryPair = &.{},
    body: ?std.json.Value = null,
};

/// What the API answered, which is a status and the bytes.
pub const Answer = struct {
    status: u16,
    payload: []const u8,
};

/// What one request is issued through.
///
/// A pointer and a function rather than an interface the generated half declares: nothing under
/// `generated/` knows what a socket is, and the transport that ships with this package is one
/// implementation of this among however many a caller writes.
pub const Transport = struct {
    context: *anyopaque,
    requestFn: *const fn (
        context: *anyopaque,
        allocator: std.mem.Allocator,
        request: Request,
    ) anyerror!Answer,

    pub fn request(
        self: Transport,
        allocator: std.mem.Allocator,
        asked: Request,
    ) anyerror!Answer {
        return self.requestFn(self.context, allocator, asked);
    }
};

/// One name and value travelling in a path or a query string.
pub const QueryPair = struct {
    name: []const u8,
    value: QueryValue,
};

/// A value as it travels in a request line, before anything is written.
///
/// `absent` is what an optional the caller passed nothing for reads as, which is what lets the
/// emitted method list every parameter the operation declares and leave the deciding to here.
pub const QueryValue = union(enum) {
    absent,
    text: []const u8,
    integer: i64,
    number: f64,
    boolean: bool,

    /// How this travels, written into `buffer` when it has to be written at all.
    pub fn written(self: QueryValue, buffer: []u8) ?[]const u8 {
        return switch (self) {
            .absent => null,
            .text => |carried| carried,
            .integer => |carried| std.fmt.bufPrint(buffer, "{d}", .{carried}) catch null,
            .number => |carried| std.fmt.bufPrint(buffer, "{d}", .{carried}) catch null,
            .boolean => |carried| if (carried) "true" else "false",
        };
    }
};

/// Longest a query value written out of a number may be.
pub const max_written_value_bytes: usize = 64;

/// That value, as one that travels in a request line.
///
/// Which of the shapes it takes is worked out from its type rather than said at the call site, so an
/// operation whose parameter is a number and one whose parameter is text are emitted the same way.
pub fn value(carried: anytype) QueryValue {
    const T = @TypeOf(carried);
    const info = @typeInfo(T);

    if (info == .optional) {
        return if (carried) |held| value(held) else .absent;
    }
    if (info == .null) {
        return .absent;
    }
    if (T == bool) {
        return .{ .boolean = carried };
    }
    if (info == .int or info == .comptime_int) {
        return .{ .integer = @intCast(carried) };
    }
    if (info == .float or info == .comptime_float) {
        return .{ .number = @floatCast(carried) };
    }
    return .{ .text = carried };
}

/// The characters a path segment carries as themselves; everything else travels percent-encoded.
fn unreserved(character: u8) bool {
    return switch (character) {
        'A'...'Z', 'a'...'z', '0'...'9', '-', '.', '_', '~' => true,
        else => false,
    };
}

/// That text, with nothing left in it that could name another segment or another parameter.
pub fn encoded(allocator: std.mem.Allocator, text_value: []const u8) WriteError![]const u8 {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    for (text_value) |character| {
        if (unreserved(character)) {
            try out.append(allocator, character);
        } else {
            try out.print(allocator, "%{X:0>2}", .{character});
        }
    }
    return out.toOwnedSlice(allocator);
}

/// Where a request lands, with each placeholder of the template filled in.
pub fn path(
    allocator: std.mem.Allocator,
    template: []const u8,
    filled: []const QueryPair,
) WriteError![]const u8 {
    var filled_in: []const u8 = try allocator.dupe(u8, template);

    for (filled) |pair| {
        var buffer: [max_written_value_bytes]u8 = undefined;
        const carried = pair.value.written(&buffer) orelse "";
        const placeholder = try std.fmt.allocPrint(allocator, "{{{s}}}", .{pair.name});
        const segment = try encoded(allocator, carried);
        filled_in = try std.mem.replaceOwned(u8, allocator, filled_in, placeholder, segment);
    }

    return filled_in;
}

/// Whether that list of values carries the one asked about.
pub fn declares(values: []const []const u8, carried: []const u8) bool {
    for (values) |declared| {
        if (std.mem.eql(u8, declared, carried)) return true;
    }
    return false;
}

/// The type a reader answers, worked out from the reader itself.
///
/// What keeps every emitted decoder to one line: the type a member carries is already said by the
/// reader that reads it, so the call site does not repeat it.
pub fn ReaderValue(comptime reader: anytype) type {
    const info = @typeInfo(@TypeOf(reader)).@"fn";
    return @typeInfo(info.return_type.?).error_union.payload;
}

/// The members of an object the document declares, under the name it declares it with.
///
/// The name is not read at run time — a Zig error carries nothing for it to travel in — and it is
/// asked for anyway, so that a reader of an emitted decoder can see which type it decodes without
/// looking anywhere else.
pub fn asFields(carried: std.json.Value, owner: []const u8) DecodeError!std.json.ObjectMap {
    _ = owner;
    return switch (carried) {
        .object => |fields| fields,
        else => error.NotAnObject,
    };
}

/// A member the document requires, which is therefore missing when it is absent.
pub fn read(
    allocator: std.mem.Allocator,
    fields: std.json.ObjectMap,
    key: []const u8,
    comptime reader: anytype,
) DecodeError!ReaderValue(reader) {
    const carried = fields.get(key) orelse return error.MemberMissing;
    return reader(allocator, carried);
}

/// A member the document does not require, absent as readily as answered as null.
pub fn maybe(
    allocator: std.mem.Allocator,
    fields: std.json.ObjectMap,
    key: []const u8,
    comptime reader: anytype,
) DecodeError!?ReaderValue(reader) {
    const carried = fields.get(key) orelse return null;
    if (carried == .null) return null;
    return try reader(allocator, carried);
}

/// A string, refusing what merely spells like one.
pub fn text(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError![]const u8 {
    _ = allocator;
    return switch (carried) {
        .string, .number_string => |held| held,
        else => error.NotAString,
    };
}

/// A whole number that fits in thirty-two bits.
pub fn integer32(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError!i32 {
    const held = try integer64(allocator, carried);
    return std.math.cast(i32, held) orelse error.NotAWholeNumber;
}

/// A whole number. A boolean is not one, here or on the wire, and neither is a number the document
/// wrote with a fractional part.
pub fn integer64(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError!i64 {
    _ = allocator;
    return switch (carried) {
        .integer => |held| held,
        else => error.NotAWholeNumber,
    };
}

/// A number, whether the document wrote it with a fractional part or not.
pub fn number(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError!f64 {
    _ = allocator;
    return switch (carried) {
        .float => |held| held,
        .integer => |held| @floatFromInt(held),
        else => error.NotANumber,
    };
}

/// A boolean, refusing the numbers that stand in for one elsewhere.
pub fn boolean(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError!bool {
    _ = allocator;
    return switch (carried) {
        .bool => |held| held,
        else => error.NotABoolean,
    };
}

/// A value the document does not describe, which is therefore kept as it arrived.
pub fn jsonValue(allocator: std.mem.Allocator, carried: std.json.Value) DecodeError!std.json.Value {
    _ = allocator;
    return carried;
}

/// Every item of an array, each one read the same way.
pub fn list(comptime reader: anytype) type {
    return struct {
        pub fn read(
            allocator: std.mem.Allocator,
            carried: std.json.Value,
        ) DecodeError![]const ReaderValue(reader) {
            const items = switch (carried) {
                .array => |held| held,
                else => return error.NotAnArray,
            };

            const out = try allocator.alloc(ReaderValue(reader), items.items.len);
            for (items.items, 0..) |item, index| {
                out[index] = try reader(allocator, item);
            }
            return out;
        }
    };
}

/// An object whose keys the document leaves open, as the pairs it carries.
///
/// A slice of pairs rather than a hash map: what arrives is read once and looked at, the order it
/// arrived in is the order it goes back out in, and nothing has to be hashed to write it.
pub fn Map(comptime T: type) type {
    return struct {
        entries: []const Entry = &.{},

        pub const Entry = struct { key: []const u8, value: T };
        const Self = @This();

        /// What that key carries, when the document carried it.
        pub fn get(self: Self, key: []const u8) ?T {
            for (self.entries) |entry| {
                if (std.mem.eql(u8, entry.key, key)) return entry.value;
            }
            return null;
        }

        /// Write one back the way the API reads it.
        pub fn toJson(self: Self, allocator: std.mem.Allocator) WriteError!std.json.Value {
            var out: std.json.ObjectMap = .empty;
            for (self.entries) |entry| {
                try out.put(allocator, entry.key, try written(allocator, entry.value));
            }
            return .{ .object = out };
        }
    };
}

/// Every value of an object whose keys the document leaves open.
pub fn map(comptime reader: anytype) type {
    return struct {
        pub fn read(
            allocator: std.mem.Allocator,
            carried: std.json.Value,
        ) DecodeError!Map(ReaderValue(reader)) {
            const fields = switch (carried) {
                .object => |held| held,
                else => return error.NotAnObject,
            };

            const Held = Map(ReaderValue(reader));
            const out = try allocator.alloc(Held.Entry, fields.count());
            var walked = fields.iterator();
            var index: usize = 0;
            while (walked.next()) |entry| : (index += 1) {
                out[index] = .{
                    .key = entry.key_ptr.*,
                    .value = try reader(allocator, entry.value_ptr.*),
                };
            }
            return .{ .entries = out };
        }
    };
}

/// One of the values a closed list declares, refusing anything the list does not carry.
pub fn memberOf(comptime declared: type) type {
    return struct {
        pub fn read(
            allocator: std.mem.Allocator,
            carried: std.json.Value,
        ) DecodeError![]const u8 {
            const held = try text(allocator, carried);
            if (!declared.member(held)) return error.ValueNotDeclared;
            return held;
        }
    };
}

/// That value, as the JSON it goes out as.
///
/// Which shape it takes is worked out from its type rather than said at the call site, which is what
/// keeps an emitted writer to one line per member whatever the member is.
pub fn written(allocator: std.mem.Allocator, carried: anytype) WriteError!std.json.Value {
    const T = @TypeOf(carried);
    const info = @typeInfo(T);

    if (T == std.json.Value) return carried;
    if (info == .optional) {
        return if (carried) |held| written(allocator, held) else .null;
    }
    if (info == .null) return .null;
    if (T == bool) return .{ .bool = carried };
    if (info == .int or info == .comptime_int) return .{ .integer = @intCast(carried) };
    if (info == .float or info == .comptime_float) return .{ .float = @floatCast(carried) };
    if (comptime isText(T)) return .{ .string = carried };

    if (info == .@"struct" and @hasDecl(T, "toJson")) {
        return carried.toJson(allocator);
    }
    if (info == .pointer and info.pointer.size == .slice) {
        var items: std.json.Array = .init(allocator);
        try items.ensureTotalCapacity(carried.len);
        for (carried) |item| {
            items.appendAssumeCapacity(try written(allocator, item));
        }
        return .{ .array = items };
    }

    @compileError("nothing says how to write a " ++ @typeName(T) ++ " back the way the API reads it");
}

/// Whether that type is the one this package carries text in.
fn isText(comptime T: type) bool {
    const info = @typeInfo(T);
    if (info != .pointer or info.pointer.size != .slice) return false;
    return info.pointer.child == u8;
}

/// Writes one member of a document, and writes nothing at all when it carries nothing.
///
/// A member the document does not require is absent rather than null when the caller passed nothing:
/// what goes out is what the API reads, and the two are not the same thing.
pub fn put(
    out: *std.json.ObjectMap,
    allocator: std.mem.Allocator,
    key: []const u8,
    carried: anytype,
) WriteError!void {
    if (@typeInfo(@TypeOf(carried)) == .optional) {
        const held = carried orelse return;
        try out.put(allocator, key, try written(allocator, held));
        return;
    }
    try out.put(allocator, key, try written(allocator, carried));
}

/// The JSON document a response body carries.
pub fn decodePayload(
    allocator: std.mem.Allocator,
    payload: []const u8,
) DecodeError!std.json.Value {
    if (payload.len > max_payload_bytes) return error.PayloadTooLarge;

    return std.json.parseFromSliceLeaky(std.json.Value, allocator, payload, .{
        .max_value_len = max_payload_bytes,
    }) catch |failed| switch (failed) {
        error.OutOfMemory => error.OutOfMemory,
        else => error.PayloadNotJson,
    };
}

/// The problem document a body names, or nothing when this client cannot read one out of it.
///
/// A body that is not the problem shape is a body naming no problem, rather than a second failure on
/// top of the first.
pub fn problemOf(
    comptime T: type,
    allocator: std.mem.Allocator,
    payload: []const u8,
) ?T {
    const document = decodePayload(allocator, payload) catch return null;
    return T.fromJson(allocator, document) catch null;
}

/// As much of a response body as a message may carry.
pub fn preview(payload: []const u8) []const u8 {
    if (payload.len <= max_preview_bytes) return payload;
    return payload[0..max_preview_bytes];
}

/// What to say about an answer the API document does not describe.
pub fn unreadable(
    allocator: std.mem.Allocator,
    status: u16,
    payload: []const u8,
) WriteError![]const u8 {
    return std.fmt.allocPrint(
        allocator,
        "the API answered {d} with a body this client cannot read: {s}",
        .{ status, preview(payload) },
    );
}

/// What to say about a problem the API reported.
pub fn reported(
    allocator: std.mem.Allocator,
    status: u16,
    payload: []const u8,
) WriteError![]const u8 {
    return std.fmt.allocPrint(
        allocator,
        "the API answered {d}: {s}",
        .{ status, preview(payload) },
    );
}
