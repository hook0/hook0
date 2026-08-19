//! Everything the generator wrote, found by looking at what it wrote, and one value of each.
//!
//! Zig holds the shape of every value the API declares in the type system, so what a member is is
//! read off the type rather than out of a list somebody kept: a schema the API grows is built here
//! the moment the generated files carry it, and one it drops takes its case with it.
//!
//! The one thing the type system does not carry is which of the members spelled `[]const u8` are
//! closed lists of strings — the generator writes those as text on purpose, so that a value the API
//! grows is not a value this package has to be regenerated to hold. What says which is what the
//! generator wrote above them, so that is read out of the emitted source.

const std = @import("std");
const hook0 = @import("hook0");

/// Where the generator lands, from the directory `zig build test` runs in.
pub const models_path = "src/generated/models.zig";

/// Where the API document the generator was run against sits, from the same directory.
pub const document_path = "../../api/openapi.snapshot.json";

/// Largest emitted file read back, far above what the generator ever writes.
pub const max_source_bytes: usize = 8 * 1024 * 1024;

/// What every string-shaped member of a value is given.
pub const model_text = "3f2504e0-4f89-41d3-9a0c-0305e82c3301";

/// What every string-shaped argument of an operation is given. It carries the two characters a path
/// segment may not leave as they are, so a value reaching a path proves it was escaped.
pub const argument_text = "a value/with a space";

/// The same, as it reaches the wire once the transport has escaped it.
pub const escaped_argument_text = "a%20value%2Fwith%20a%20space";

/// One closed list of strings the generator wrote, under the name it declared it.
pub const ClosedList = struct { name: []const u8, first: []const u8 };

/// Every closed list of strings the generator wrote.
pub const closed_lists: []const ClosedList = found: {
    var held: []const ClosedList = &.{};
    for (@typeInfo(hook0.models).@"struct".decls) |decl| {
        const declared = @field(hook0.models, decl.name);
        if (@TypeOf(declared) != type) continue;
        if (@typeInfo(declared) != .@"struct") continue;
        if (!@hasDecl(declared, "values") or !@hasDecl(declared, "member")) continue;
        held = held ++ [_]ClosedList{.{ .name = decl.name, .first = declared.values[0] }};
    }
    break :found held;
};

/// Every value the generator wrote a decoder for, by the name it declared it under.
pub const model_names: []const []const u8 = found: {
    var held: []const []const u8 = &.{};
    for (@typeInfo(hook0.models).@"struct".decls) |decl| {
        const declared = @field(hook0.models, decl.name);
        if (@TypeOf(declared) != type) continue;
        if (@typeInfo(declared) != .@"struct") continue;
        if (!@hasDecl(declared, "fromJson")) continue;
        held = held ++ [_][]const u8{decl.name};
    }
    break :found held;
};

/// Which closed list each member of each value is one of, read out of what the generator wrote.
///
/// Keyed by the value and the member together, since the same member name is declared by more than
/// one value and each names a list of its own.
pub const Lists = struct {
    entries: []const Entry,

    pub const Entry = struct { owner: []const u8, member: []const u8, list: []const u8 };

    /// The first value of the list a member is one of, when the generator said it is one.
    pub fn firstOf(self: Lists, owner: []const u8, member: []const u8) ?[]const u8 {
        for (self.entries) |entry| {
            if (!std.mem.eql(u8, entry.owner, owner)) continue;
            if (!std.mem.eql(u8, entry.member, member)) continue;
            for (closed_lists) |list| {
                if (std.mem.eql(u8, list.name, entry.list)) return list.first;
            }
            return null;
        }
        return null;
    }
};

/// What the generator wrote above every member of every value, read out of the emitted source.
///
/// The emitted source is regular: a value opens with `pub const <name> = struct {`, a member is one
/// indented `<name>: <type>,` line, and a member that is one of a closed list carries `one of
/// `models.<list>.values`` in the comment right above it.
pub fn listsOf(io: std.Io, allocator: std.mem.Allocator) !Lists {
    const written = try std.Io.Dir.cwd().readFileAlloc(io, models_path, allocator, .limited(max_source_bytes));

    var entries: std.ArrayList(Lists.Entry) = .empty;
    var owner: []const u8 = "";
    var pending: ?[]const u8 = null;
    var commenting = false;

    var lines = std.mem.splitScalar(u8, written, '\n');
    while (lines.next()) |line| {
        if (std.mem.startsWith(u8, line, "pub const ")) {
            const rest = line["pub const ".len..];
            const at = std.mem.indexOf(u8, rest, " = struct {") orelse continue;
            owner = rest[0..at];
            pending = null;
            commenting = false;
            continue;
        }

        const trimmed = std.mem.trim(u8, line, " ");
        if (std.mem.startsWith(u8, trimmed, "///")) {
            // One comment may run over several lines, and what it says about a member may be on any
            // of them, so the block is read as a whole rather than line by line.
            if (!commenting) pending = null;
            commenting = true;
            if (namedList(trimmed)) |found| pending = found;
            continue;
        }
        commenting = false;

        if (pending) |list| {
            if (std.mem.indexOf(u8, trimmed, ": ")) |colon| {
                try entries.append(allocator, .{ .owner = owner, .member = trimmed[0..colon], .list = list });
            }
        }
        pending = null;
    }

    return .{ .entries = try entries.toOwnedSlice(allocator) };
}

/// The closed list a comment names, when it names one.
fn namedList(comment: []const u8) ?[]const u8 {
    const opened = "one of `models.";
    const closed = ".values`";
    const at = std.mem.indexOf(u8, comment, opened) orelse return null;
    const rest = comment[at + opened.len ..];
    const end = std.mem.indexOf(u8, rest, closed) orelse return null;
    return rest[0..end];
}

/// One value of a schema the API declares, with every member it may leave out set or not.
pub fn build(
    comptime T: type,
    allocator: std.mem.Allocator,
    lists: Lists,
    optionals: bool,
) !T {
    var held: T = undefined;
    inline for (@typeInfo(T).@"struct".fields) |field| {
        @field(held, field.name) = try valueOf(
            field.type,
            allocator,
            lists,
            optionals,
            shortNameOf(T),
            field.name,
            model_text,
        );
    }
    return held;
}

/// One value of the type a member or an argument declares.
pub fn valueOf(
    comptime F: type,
    allocator: std.mem.Allocator,
    lists: Lists,
    optionals: bool,
    owner: []const u8,
    member: []const u8,
    text: []const u8,
) !F {
    if (F == std.json.Value) return .{ .string = "the document describes none of this" };
    if (F == bool) return true;
    if (F == std.mem.Allocator) return allocator;

    const info = @typeInfo(F);
    switch (info) {
        .optional => |held| {
            if (!optionals) return null;
            return try valueOf(held.child, allocator, lists, optionals, owner, member, text);
        },
        .int => return 12,
        .float => return 1.5,
        .pointer => |held| {
            if (held.size != .slice) @compileError("nothing here builds a " ++ @typeName(F));
            if (held.child == u8) {
                return lists.firstOf(owner, member) orelse text;
            }
            const item = try valueOf(held.child, allocator, lists, optionals, owner, member, text);
            return try allocator.dupe(held.child, &.{item});
        },
        .@"struct" => {
            // A map the document leaves the keys of open, which the runtime holds as its entries.
            if (@hasField(F, "entries") and @hasDecl(F, "Entry")) {
                const Entry = F.Entry;
                const carried = try valueOf(
                    @FieldType(Entry, "value"),
                    allocator,
                    lists,
                    optionals,
                    owner,
                    member,
                    text,
                );
                return .{ .entries = try allocator.dupe(Entry, &.{.{ .key = "a key", .value = carried }}) };
            }
            return try build(F, allocator, lists, optionals);
        },
        else => @compileError("nothing here builds a " ++ @typeName(F)),
    }
}

/// The name a type was declared under, without the modules it was reached through.
pub fn shortNameOf(comptime T: type) []const u8 {
    const whole = @typeName(T);
    const at = comptime std.mem.lastIndexOfScalar(u8, whole, '.') orelse whole.len;
    return if (at == whole.len) whole else whole[at + 1 ..];
}

/// One operation the API document declares, as a request has to look to be it.
pub const Declared = struct {
    verb: []const u8,
    /// The path with its parameters still written `{like_this}`.
    template: []const u8,
    required_query: []const []const u8,
    optional_query: []const []const u8,

    /// Whether a request landed on this operation.
    pub fn matches(self: Declared, verb: []const u8, target: []const u8) bool {
        if (!std.mem.eql(u8, verb, self.verb)) return false;

        var wanted = std.mem.splitScalar(u8, self.template, '/');
        var sent = std.mem.splitScalar(u8, pathOf(target), '/');
        while (wanted.next()) |declared| {
            const segment = sent.next() orelse return false;
            if (std.mem.startsWith(u8, declared, "{") and std.mem.endsWith(u8, declared, "}")) {
                // A parameter stands for a segment that is there; an empty one is the trailing
                // slash of another path rather than a value.
                if (segment.len == 0) return false;
                continue;
            }
            if (!std.mem.eql(u8, declared, segment)) return false;
        }
        return sent.next() == null;
    }

    /// Every name the query may carry, whether the operation requires it or not.
    pub fn queryNames(
        self: Declared,
        allocator: std.mem.Allocator,
        with_optional: bool,
    ) ![]const []const u8 {
        var held: std.ArrayList([]const u8) = .empty;
        try held.appendSlice(allocator, self.required_query);
        if (with_optional) try held.appendSlice(allocator, self.optional_query);
        const names = try held.toOwnedSlice(allocator);
        std.mem.sort([]const u8, names, {}, lessThan);
        return names;
    }
};

fn lessThan(_: void, one: []const u8, other: []const u8) bool {
    return std.mem.order(u8, one, other) == .lt;
}

/// The path a request landed on, without the query it carried.
pub fn pathOf(target: []const u8) []const u8 {
    const at = std.mem.indexOfScalar(u8, target, '?') orelse return target;
    return target[0..at];
}

/// The tag that marks an operation as part of the surface an SDK exposes, which is the rule the
/// generator applies — see `PUBLIC_TAG` in `clients/sdkgen/src/snapshot.rs`.
pub const sdk_tag = "sdk";

/// The methods a request line can carry, which is what tells an operation from the rest.
const verbs = [_][]const u8{ "get", "put", "post", "delete", "options", "head", "patch", "trace" };

/// Every operation an SDK is built out of, read out of the document the generator was run against.
///
/// A document that marks nothing public exposes all of itself, and one that marks anything exposes
/// what it marked. Both are what the generator does with the tag.
pub fn declaredOperations(io: std.Io, allocator: std.mem.Allocator) ![]const Declared {
    const written = try std.Io.Dir.cwd().readFileAlloc(io, document_path, allocator, .limited(max_source_bytes));
    const document = try std.json.parseFromSliceLeaky(std.json.Value, allocator, written, .{});

    var found: std.ArrayList(Declared) = .empty;
    var public: std.ArrayList(Declared) = .empty;

    var paths = document.object.get("paths").?.object.iterator();
    while (paths.next()) |path_entry| {
        var item = path_entry.value_ptr.object.iterator();
        while (item.next()) |operation_entry| {
            if (!isVerb(operation_entry.key_ptr.*)) continue;

            const declared = try operationOf(
                allocator,
                path_entry.key_ptr.*,
                operation_entry.key_ptr.*,
                operation_entry.value_ptr.*,
            );
            try found.append(allocator, declared);
            if (carriesTag(operation_entry.value_ptr.*, sdk_tag)) try public.append(allocator, declared);
        }
    }
    if (found.items.len == 0) return error.TheDocumentDeclaresNoOperation;

    return if (public.items.len > 0) public.items else found.items;
}

fn isVerb(named: []const u8) bool {
    for (verbs) |verb| {
        if (std.mem.eql(u8, verb, named)) return true;
    }
    return false;
}

fn carriesTag(operation: std.json.Value, named: []const u8) bool {
    const tags = operation.object.get("tags") orelse return false;
    for (tags.array.items) |tag| {
        if (tag == .string and std.mem.eql(u8, tag.string, named)) return true;
    }
    return false;
}

fn operationOf(
    allocator: std.mem.Allocator,
    template: []const u8,
    verb: []const u8,
    operation: std.json.Value,
) !Declared {
    var required: std.ArrayList([]const u8) = .empty;
    var optional: std.ArrayList([]const u8) = .empty;

    if (operation.object.get("parameters")) |parameters| {
        for (parameters.array.items) |parameter| {
            const where = parameter.object.get("in") orelse continue;
            if (where != .string or !std.mem.eql(u8, where.string, "query")) continue;

            const name = parameter.object.get("name").?.string;
            const asked = parameter.object.get("required");
            const into = if (asked != null and asked.? == .bool and asked.?.bool) &required else &optional;
            try into.append(allocator, name);
        }
    }

    const upper = try allocator.alloc(u8, verb.len);
    _ = std.ascii.upperString(upper, verb);

    return .{
        .verb = upper,
        .template = template,
        .required_query = try required.toOwnedSlice(allocator),
        .optional_query = try optional.toOwnedSlice(allocator),
    };
}

/// What the query of a request carried, as name and value pairs, unescaped.
pub fn queryOf(allocator: std.mem.Allocator, target: []const u8) ![]const [2][]const u8 {
    const at = std.mem.indexOfScalar(u8, target, '?') orelse return &.{};

    var carried: std.ArrayList([2][]const u8) = .empty;
    var pairs = std.mem.splitScalar(u8, target[at + 1 ..], '&');
    while (pairs.next()) |pair| {
        if (pair.len == 0) continue;
        const equals = std.mem.indexOfScalar(u8, pair, '=') orelse pair.len;
        try carried.append(allocator, .{
            try unescaped(allocator, pair[0..equals]),
            try unescaped(allocator, if (equals == pair.len) "" else pair[equals + 1 ..]),
        });
    }
    return carried.toOwnedSlice(allocator);
}

/// The inverse of what the transport writes a request line with.
pub fn unescaped(allocator: std.mem.Allocator, written: []const u8) ![]const u8 {
    var out: std.ArrayList(u8) = .empty;
    var index: usize = 0;
    while (index < written.len) {
        if (written[index] == '%' and index + 2 < written.len) {
            const byte = std.fmt.parseInt(u8, written[index + 1 .. index + 3], 16) catch {
                try out.append(allocator, written[index]);
                index += 1;
                continue;
            };
            try out.append(allocator, byte);
            index += 3;
            continue;
        }
        try out.append(allocator, written[index]);
        index += 1;
    }
    return out.toOwnedSlice(allocator);
}
