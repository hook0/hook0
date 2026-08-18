//! What the generator wrote, exercised through what it wrote rather than through a list of it.
//!
//! Nothing below names a schema, an operation or a problem. The values are found on the generated
//! namespace and built out of their own types, the operations are found on the groups the generator
//! wrote and called with arguments built out of their own signatures, and what every request is held
//! to is read off the API document the generator was run against.
//!
//! Zig only emits what something references, so a percentage measured over this package says
//! whatever the suite happened to reach and nothing about what it did not. The closing case of each
//! half is therefore not a percentage but a subtraction: what the generator declared, minus what
//! this suite drove, has to be empty — and so does the other way round. An operation the API grows
//! fails that subtraction until something here drives it.

const std = @import("std");
const hook0 = @import("hook0");

const helper = @import("helper.zig");
const surface = @import("surface.zig");

const io = std.testing.io;
const allocator = std.testing.allocator;

/// Whether a set of names carries one.
fn carries(named: []const []const u8, one: []const u8) bool {
    for (named) |held| {
        if (std.mem.eql(u8, held, one)) return true;
    }
    return false;
}

/// What one set of names holds that the other does not.
fn expectSameSet(declared: []const []const u8, driven: []const []const u8) !void {
    for (declared) |one| {
        std.testing.expect(carries(driven, one)) catch |failed| {
            std.debug.print("`{s}` was declared and reached by nothing\n", .{one});
            return failed;
        };
    }
    for (driven) |one| {
        std.testing.expect(carries(declared, one)) catch |failed| {
            std.debug.print("`{s}` was reached although nothing declares it\n", .{one});
            return failed;
        };
    }
}

/// The same, of a set that would say nothing at all if it were empty — which is every set the walks
/// below are held to being exhaustive over.
fn expectSameNames(declared: []const []const u8, driven: []const []const u8) !void {
    try expectSameSet(declared, driven);
    try std.testing.expect(declared.len > 0);
}

/// A value written out and read back in is the value it started as, member for member.
///
/// Run once with every member the schema may leave out set and once with none of them, which is what
/// tells a member that was read apart from one that was defaulted to the same thing.
fn readsBackWhatItWrote(optionals: bool) !void {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);
    var driven: std.ArrayList([]const u8) = .empty;

    inline for (surface.model_names) |name| {
        const T = @field(hook0.models, name);
        const built = try surface.build(T, held, lists, optionals);

        const written = try hook0.runtime.written(held, built);
        try std.testing.expect(written == .object);

        const read = try T.fromJson(held, written);
        const back = try hook0.runtime.written(held, read);

        std.testing.expectEqualStrings(
            try stringified(held, written),
            try stringified(held, back),
        ) catch |failed| {
            std.debug.print("{s} does not read back what it wrote\n", .{name});
            return failed;
        };

        // A member the document does not require is absent rather than written back as nothing.
        inline for (@typeInfo(T).@"struct".fields) |field| {
            if (@typeInfo(field.type) == .optional and @field(built, field.name) == null) {
                std.testing.expect(written.object.get(field.name) == null) catch |failed| {
                    std.debug.print("{s} wrote `{s}` out although it holds nothing\n", .{ name, field.name });
                    return failed;
                };
            }
        }

        try driven.append(held, name);
    }

    try expectSameNames(surface.model_names, driven.items);
}

/// A value as the one document both sides of the wire agree on.
fn stringified(held: std.mem.Allocator, value: std.json.Value) ![]const u8 {
    var out: std.Io.Writer.Allocating = .init(held);
    var stringify: std.json.Stringify = .{ .writer = &out.writer };
    try stringify.write(value);
    return out.written();
}

test "every value the API declares reads back what it wrote, with everything it may carry" {
    try readsBackWhatItWrote(true);
}

test "every value the API declares reads back what it wrote, with only what it requires" {
    try readsBackWhatItWrote(false);
}

test "every value the API declares refuses a document that is not the object it declares" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const answered = [_]std.json.Value{
        .{ .integer = 1 },
        .{ .string = "text" },
        .{ .bool = true },
        .{ .array = .init(held) },
        .null,
    };

    inline for (surface.model_names) |name| {
        const T = @field(hook0.models, name);
        for (answered) |value| {
            try std.testing.expectError(error.NotAnObject, T.fromJson(held, value));
        }
    }
}

test "every value the API declares refuses a member the document does not declare it as" {
    // Everything a schema describes is refused when it arrives as something else. What it leaves
    // undescribed is kept as it arrived and so is refused by nothing, which is why the members typed
    // as whatever the document carried are the ones passed over here.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);
    var walked: usize = 0;

    inline for (surface.model_names) |name| {
        const T = @field(hook0.models, name);
        const written = try hook0.runtime.written(held, try surface.build(T, held, lists, true));

        inline for (@typeInfo(T).@"struct".fields) |field| {
            const carried = comptime peeled(field.type);
            if (carried != std.json.Value) {
                var wrong = try written.object.clone(held);
                // Neither an object nor a scalar any of the readers accept, whichever the member is.
                var nested: std.json.ObjectMap = .empty;
                try nested.put(held, "neither", .{ .string = "a scalar" });
                var items: std.json.Array = .init(held);
                try items.append(.{ .object = nested });
                try wrong.put(held, field.name, .{ .array = items });

                if (T.fromJson(held, .{ .object = wrong })) |_| {
                    std.debug.print("{s} read `{s}` as what it declares\n", .{ name, field.name });
                    return error.AMemberWasReadAsSomethingItIsNot;
                } else |_| {}
                walked += 1;
            }
        }
    }

    try std.testing.expect(walked > 0);
}

/// What a member is when it is there.
fn peeled(comptime F: type) type {
    return switch (@typeInfo(F)) {
        .optional => |held| held.child,
        else => F,
    };
}

test "every closed list of strings the API declares carries the values it lists and no other" {
    inline for (@typeInfo(hook0.models).@"struct".decls) |decl| {
        const declared = @field(hook0.models, decl.name);
        if (@TypeOf(declared) == type and @typeInfo(declared) == .@"struct" and @hasDecl(declared, "member")) {
            try std.testing.expect(declared.values.len > 0);
            for (declared.values) |value| {
                std.testing.expect(declared.member(value)) catch |failed| {
                    std.debug.print("{s} does not carry `{s}`, which it lists\n", .{ decl.name, value });
                    return failed;
                };
            }
            try std.testing.expect(!declared.member("a value the API never declared"));
        }
    }
}

/// Every group of operations the generator wrote, by the name it declared it under.
const group_names: []const []const u8 = found: {
    var held: []const []const u8 = &.{};
    for (@typeInfo(hook0.api).@"struct".decls) |decl| {
        const declared = @field(hook0.api, decl.name);
        if (@TypeOf(declared) != type) continue;
        if (@typeInfo(declared) != .@"struct") continue;
        if (!@hasDecl(declared, "init") or !@hasField(declared, "transport")) continue;
        held = held ++ [_][]const u8{decl.name};
    }
    break :found held;
};

/// Every operation one group carries, under the name it is called by.
fn operationsOf(comptime Group: type) []const []const u8 {
    comptime var held: []const []const u8 = &.{};
    comptime {
        for (@typeInfo(Group).@"struct".decls) |decl| {
            if (std.mem.eql(u8, decl.name, "init")) continue;
            if (@typeInfo(@TypeOf(@field(Group, decl.name))) != .@"fn") continue;
            held = held ++ [_][]const u8{decl.name};
        }
    }
    return held;
}

/// Every operation the generator wrote, spelled the way a failure names it.
const operation_names: []const []const u8 = found: {
    var held: []const []const u8 = &.{};
    for (group_names) |group_name| {
        for (operationsOf(@field(hook0.api, group_name))) |name| {
            held = held ++ [_][]const u8{group_name ++ "." ++ name};
        }
    }
    break :found held;
};

/// What one operation answers when it answers a value: the type of it, or nothing at all.
fn Answered(comptime Fn: type) ?type {
    const returned = @typeInfo(Fn).@"fn".return_type.?;
    const payload = @typeInfo(returned).error_union.payload;
    if (payload == void) return null;
    return @FieldType(payload, "value");
}

/// Every operation the document declares issues the request it is declared as, and reads it back.
///
/// Asked twice: once giving every argument an operation may be asked with, once giving only the ones
/// it requires, which is what says an argument left out leaves the query it would have filled empty.
fn reachesEveryOperation(optionals: bool) !void {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);
    const declared = try surface.declaredOperations(io, held);

    // What the API answers, worked out before it is started: it answers in the order it is asked,
    // and it is asked in the order the operations were found.
    var scripted: std.ArrayList(helper.Scripted) = .empty;
    inline for (group_names) |group_name| {
        const Group = @field(hook0.api, group_name);
        inline for (comptime operationsOf(Group)) |name| {
            const Value = comptime Answered(@TypeOf(@field(Group, name)));
            if (Value) |V| {
                const answered = try surface.valueOf(V, held, lists, optionals, group_name, name, surface.model_text);
                const body = try stringified(held, try hook0.runtime.written(held, answered));
                try scripted.append(held, .{ .status = 200, .body = body });
            } else {
                try scripted.append(held, .{ .status = 204, .body = "{}" });
            }
        }
    }

    const api = try helper.FakeApi.init(io, held, scripted.items);
    defer api.deinit();

    var transport: hook0.Transport = .{ .io = io, .base_url = try api.baseUrl(held), .token = "token-xyz" };
    var driven: std.ArrayList([]const u8) = .empty;
    var reached: std.ArrayList([]const u8) = .empty;
    var at: usize = 0;

    inline for (group_names) |group_name| {
        const Group = @field(hook0.api, group_name);
        inline for (comptime operationsOf(Group)) |name| {
            var group: Group = .init(transport.any());
            const Fn = @TypeOf(@field(Group, name));

            var args: std.meta.ArgsTuple(Fn) = undefined;
            args[0] = &group;
            args[1] = held;
            inline for (@typeInfo(Fn).@"fn".params, 0..) |param, index| {
                if (index >= 2) {
                    args[index] = try surface.valueOf(
                        param.type.?,
                        held,
                        lists,
                        optionals,
                        group_name,
                        name,
                        surface.argument_text,
                    );
                }
            }

            const named = group_name ++ "." ++ name;
            if (comptime Answered(Fn)) |_| {
                const owned = try @call(.auto, @field(Group, name), args);
                defer owned.deinit();

                const back = try stringified(held, try hook0.runtime.written(held, owned.value));
                std.testing.expectEqualStrings(scripted.items[at].body, back) catch |failed| {
                    std.debug.print("{s} did not read back what the API answered\n", .{named});
                    return failed;
                };
            } else {
                try @call(.auto, @field(Group, name), args);
            }

            const request = api.at(at) orelse return error.AnOperationIssuedNoRequest;
            try reached.append(held, try heldTo(held, declared, request, named, optionals));
            try driven.append(held, named);
            at += 1;
        }
    }

    try std.testing.expectEqual(operation_names.len, api.count());
    try expectSameNames(operation_names, driven.items);
    try expectSameNames(try templatesOf(held, declared), reached.items);
}

/// What one operation put on the wire, held to what the document declares it as, and which
/// operation of the document that was.
fn heldTo(
    held: std.mem.Allocator,
    declared: []const surface.Declared,
    request: helper.Received,
    named: []const u8,
    optionals: bool,
) ![]const u8 {
    var matched: ?surface.Declared = null;
    var found: usize = 0;
    for (declared) |one| {
        if (one.matches(request.method, request.target)) {
            matched = one;
            found += 1;
        }
    }
    std.testing.expectEqual(@as(usize, 1), found) catch |failed| {
        std.debug.print("`{s} {s}` is {d} of the operations the document declares\n", .{
            request.method,
            request.target,
            found,
        });
        return failed;
    };
    const one = matched.?;

    try std.testing.expectEqualStrings("Bearer token-xyz", request.get("authorization").?);
    try std.testing.expectEqualStrings("application/json", request.get("accept").?);

    // The value lands in the path escaped, so that nothing in it can name a segment the operation
    // never had.
    var wanted = std.mem.splitScalar(u8, one.template, '/');
    var sent = std.mem.splitScalar(u8, surface.pathOf(request.target), '/');
    while (wanted.next()) |segment| {
        const carried = sent.next() orelse return error.APathLostASegment;
        if (!std.mem.startsWith(u8, segment, "{")) continue;
        std.testing.expectEqualStrings(surface.escaped_argument_text, carried) catch |failed| {
            std.debug.print("{s} left `{s}` unescaped\n", .{ named, segment });
            return failed;
        };
    }

    // The query is what the document declares and nothing else, and what it carries is what the
    // operation was asked with rather than something the transport altered on the way.
    const carried = try surface.queryOf(held, request.target);
    var names: std.ArrayList([]const u8) = .empty;
    for (carried) |pair| {
        try names.append(held, pair[0]);
        std.testing.expectEqualStrings(surface.argument_text, pair[1]) catch |failed| {
            std.debug.print("{s} carried `{s}` altered\n", .{ named, pair[0] });
            return failed;
        };
    }
    try expectSameSet(try one.queryNames(held, optionals), names.items);

    return std.fmt.allocPrint(held, "{s} {s}", .{ one.verb, one.template });
}

/// Every operation the document declares, spelled the way a request names one.
fn templatesOf(held: std.mem.Allocator, declared: []const surface.Declared) ![]const []const u8 {
    var named: std.ArrayList([]const u8) = .empty;
    for (declared) |one| {
        try named.append(held, try std.fmt.allocPrint(held, "{s} {s}", .{ one.verb, one.template }));
    }
    return named.toOwnedSlice(held);
}

test "every operation the API declares is reached the way it declares it, with everything it may carry" {
    try reachesEveryOperation(true);
}

test "every operation the API declares is reached the way it declares it, with only what it requires" {
    try reachesEveryOperation(false);
}

/// The errors this package declares, which is one per problem and one for the rest.
const failure_names: []const []const u8 = found: {
    var held: []const []const u8 = &.{};
    for (@typeInfo(hook0.errors.Failure).error_set.?) |declared| {
        held = held ++ [_][]const u8{declared.name};
    }
    break :found held;
};

test "the errors this package declares are the problems the API names, and one more for the rest" {
    // A Zig error carries no value, so the name is the whole of it: a problem the API grows has to
    // arrive as an error of its own rather than folded into the one that stands for the rest.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    var declared: std.ArrayList([]const u8) = .empty;
    try declared.appendSlice(held, &hook0.models.ProblemId.values);
    try declared.append(held, "Unreadable");

    try expectSameNames(declared.items, failure_names);
}

test "every problem the API names is raised as the error of that name, and says what it answered" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);

    // One operation, found rather than named: which one it is does not matter, since what these
    // cases are about is what a failure does rather than which request drew it.
    const Group = @field(hook0.api, group_names[0]);
    const name = comptime operationsOf(Group)[0];
    const Fn = @TypeOf(@field(Group, name));

    var scripted: std.ArrayList(helper.Scripted) = .empty;
    for (hook0.models.ProblemId.values) |problem| {
        try scripted.append(held, .{ .status = 400, .body = try std.fmt.allocPrint(held,
            \\{{"id":"{s}","status":400,"title":"refused","detail":"what this case scripted","type":"https://hook0.com/documentation/errors/{s}"}}
        , .{ problem, problem }) });
    }

    const api = try helper.FakeApi.init(io, held, scripted.items);
    defer api.deinit();

    var transport: hook0.Transport = .{ .io = io, .base_url = try api.baseUrl(held), .token = "token-xyz" };

    for (hook0.models.ProblemId.values) |problem| {
        var group: Group = .init(transport.any());

        var args: std.meta.ArgsTuple(Fn) = undefined;
        args[0] = &group;
        args[1] = held;
        inline for (@typeInfo(Fn).@"fn".params, 0..) |param, index| {
            if (index >= 2) {
                args[index] = try surface.valueOf(
                    param.type.?,
                    held,
                    lists,
                    false,
                    group_names[0],
                    name,
                    surface.argument_text,
                );
            }
        }

        const answered = @call(.auto, @field(Group, name), args);
        if (answered) |owned| {
            owned.deinit();
            std.debug.print("`{s}` was read as a success\n", .{problem});
            return error.AProblemWasReadAsASuccess;
        } else |raised| {
            std.testing.expectEqualStrings(problem, @errorName(raised)) catch |failed| {
                std.debug.print("`{s}` was raised as `{s}`\n", .{ problem, @errorName(raised) });
                return failed;
            };
            try std.testing.expectEqual(@as(u16, 400), group.reported.status);
            try std.testing.expectEqualStrings(problem, group.reported.problem.?.id);
            try std.testing.expectEqualStrings("what this case scripted", group.reported.problem.?.detail);
        }
    }
}

test "a problem the API grew after this package is still reported as a whole problem document" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);
    const Group = @field(hook0.api, group_names[0]);
    const name = comptime operationsOf(Group)[0];
    const Fn = @TypeOf(@field(Group, name));

    const scripted = [_]helper.Scripted{.{
        .status = 500,
        .body =
        \\{"id":"AProblemThisClientHasNeverHeardOf","status":500,"title":"refused","detail":"what this case scripted","type":"https://hook0.com/documentation/errors/AProblemThisClientHasNeverHeardOf"}
        ,
    }};
    const api = try helper.FakeApi.init(io, held, &scripted);
    defer api.deinit();

    var transport: hook0.Transport = .{ .io = io, .base_url = try api.baseUrl(held), .token = "token-xyz" };
    var group: Group = .init(transport.any());

    var args: std.meta.ArgsTuple(Fn) = undefined;
    args[0] = &group;
    args[1] = held;
    inline for (@typeInfo(Fn).@"fn".params, 0..) |param, index| {
        if (index >= 2) {
            args[index] = try surface.valueOf(
                param.type.?,
                held,
                lists,
                false,
                group_names[0],
                name,
                surface.argument_text,
            );
        }
    }

    try std.testing.expectError(error.Unreadable, @call(.auto, @field(Group, name), args));
    try std.testing.expectEqual(@as(u16, 500), group.reported.status);
    // The catalogue is what this package was generated from, and the API outlives it: a problem it
    // grows still arrives whole rather than as a failure that dropped what the API took the trouble
    // to say.
    try std.testing.expect(std.mem.indexOf(u8, group.reported.detail, "AProblemThisClientHasNeverHeardOf") != null);
}

test "every operation frees what it had already taken when the API refuses it" {
    // The operations are asked with the testing allocator rather than an arena of the case's own,
    // so what an operation took before the API refused it and did not give back is a leak this case
    // fails on. That is the whole of what the cleanup on the failing path is for.
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const lists = try surface.listsOf(io, held);

    var scripted: std.ArrayList(helper.Scripted) = .empty;
    for (0..operation_names.len) |_| {
        try scripted.append(held, .{
            .status = 404,
            .body =
            \\{"id":"NotFound","status":404,"title":"Not found","detail":"what this case scripted","type":"https://hook0.com/documentation/errors/NotFound"}
            ,
        });
    }

    const api = try helper.FakeApi.init(io, held, scripted.items);
    defer api.deinit();

    var transport: hook0.Transport = .{ .io = io, .base_url = try api.baseUrl(held), .token = "token-xyz" };
    var refused: std.ArrayList([]const u8) = .empty;

    inline for (group_names) |group_name| {
        const Group = @field(hook0.api, group_name);
        inline for (comptime operationsOf(Group)) |name| {
            var group: Group = .init(transport.any());
            const Fn = @TypeOf(@field(Group, name));

            var args: std.meta.ArgsTuple(Fn) = undefined;
            args[0] = &group;
            args[1] = allocator;
            inline for (@typeInfo(Fn).@"fn".params, 0..) |param, index| {
                if (index >= 2) {
                    args[index] = try surface.valueOf(
                        param.type.?,
                        held,
                        lists,
                        false,
                        group_name,
                        name,
                        surface.argument_text,
                    );
                }
            }

            const named = group_name ++ "." ++ name;
            const answered = @call(.auto, @field(Group, name), args);
            if (comptime Answered(Fn)) |_| {
                if (answered) |owned| {
                    owned.deinit();
                    std.debug.print("{s} read a problem as a success\n", .{named});
                    return error.AProblemWasReadAsASuccess;
                } else |raised| {
                    try std.testing.expectEqual(error.NotFound, raised);
                }
            } else {
                try std.testing.expectError(error.NotFound, answered);
            }

            try std.testing.expectEqual(@as(u16, 404), group.reported.status);
            try refused.append(held, named);
        }
    }

    try expectSameNames(operation_names, refused.items);
}
