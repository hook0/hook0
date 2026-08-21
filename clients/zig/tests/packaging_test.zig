//! What this package claims to be, held against what it is.
//!
//! Zig has no central registry to reject a manifest that does not match its package, so the manifest
//! is checked here instead: that the version it releases is the version the code answers, that the
//! Zig it declares is the Zig the pipeline installs, that the files it ships are the files that are
//! here, and — the one worth the most — that it still depends on nothing.
//!
//! That last one is a promise a reader cannot verify by reading, since a dependency is one line in a
//! file nobody opens twice. Here it is a type: the manifest is parsed into a struct whose
//! `dependencies` has no fields, so a dependency appearing is a parse that fails.

const std = @import("std");
const builtin = @import("builtin");
const hook0 = @import("hook0");

const io = std.testing.io;
const allocator = std.testing.allocator;

/// Largest file this case reads back, in bytes.
const max_manifest_bytes: usize = 64 * 1024;

/// Where the pipeline that builds this package is defined, relative to the package root.
const pipeline_path = "../../.gitlab-ci.yml";

/// What names the Zig the pipeline installs, in that file.
const pipeline_zig_version = "ZIG_VERSION:";

/// What the build writes rather than what the package holds, and so what is not shipped.
///
/// Entries starting with a dot are left out too — the cache and the pipeline definition both are —
/// which is why this names only the one build output that does not.
const build_output = "zig-out";

/// `build.zig.zon`, as far as this case is concerned with it.
///
/// `dependencies` is deliberately a struct with no fields: `std.zon.parse` refuses a member it was
/// not declared to hold, so the day somebody adds one, this stops being a manifest this case can
/// read at all.
const Manifest = struct {
    name: enum { hook0_client },
    version: []const u8,
    minimum_zig_version: []const u8,
    fingerprint: u64,
    dependencies: struct {},
    paths: []const []const u8,
};

fn manifestOf(arena: std.mem.Allocator) !Manifest {
    const written = try std.Io.Dir.cwd().readFileAlloc(io, "build.zig.zon", arena, .limited(max_manifest_bytes));
    const terminated = try arena.dupeZ(u8, written);

    var diagnostics: std.zon.parse.Diagnostics = .{};
    return std.zon.parse.fromSliceAlloc(Manifest, arena, terminated, &diagnostics, .{}) catch |failed| {
        std.debug.print("build.zig.zon does not say what this package declares: {f}\n", .{diagnostics});
        return failed;
    };
}

test "the manifest declares no dependencies at all" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();

    // Reading it is the assertion: a manifest carrying a dependency is one `Manifest` cannot hold.
    const manifest = try manifestOf(arena.allocator());
    try std.testing.expectEqual(@as(usize, 0), @typeInfo(@TypeOf(manifest.dependencies)).@"struct".fields.len);
}

test "the manifest releases the version this package answers" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();

    const manifest = try manifestOf(arena.allocator());
    try std.testing.expectEqualStrings(hook0.version, manifest.version);
}

test "the Zig the manifest declares is the Zig the pipeline installs, and the one running this" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const manifest = try manifestOf(held);
    const declared = try std.SemanticVersion.parse(manifest.minimum_zig_version);

    const written = try std.Io.Dir.cwd().readFileAlloc(io, pipeline_path, held, .limited(max_manifest_bytes));
    var lines = std.mem.splitScalar(u8, written, '\n');
    var pinned: ?[]const u8 = null;
    while (lines.next()) |line| {
        const at = std.mem.indexOf(u8, line, pipeline_zig_version) orelse continue;
        pinned = std.mem.trim(u8, line[at + pipeline_zig_version.len ..], " \"'\r\t");
    }

    std.testing.expect(pinned != null) catch |failed| {
        std.debug.print("{s} names no {s}\n", .{ pipeline_path, pipeline_zig_version });
        return failed;
    };
    try std.testing.expectEqualStrings(manifest.minimum_zig_version, pinned.?);

    // The compiler running this case, held to the same two numbers. A patch release is a compiler
    // somebody may reasonably be on; a different minor is the one where `std.Io` changed shape,
    // which is what this package is written against.
    try std.testing.expectEqual(declared.major, builtin.zig_version.major);
    try std.testing.expectEqual(declared.minor, builtin.zig_version.minor);
}

test "the manifest ships every file of this package, and nothing that is not here" {
    var arena: std.heap.ArenaAllocator = .init(allocator);
    defer arena.deinit();
    const held = arena.allocator();

    const manifest = try manifestOf(held);

    // What is here, found by looking rather than by naming: a directory somebody adds joins this
    // case without anybody remembering to add it.
    var root = try std.Io.Dir.cwd().openDir(io, ".", .{ .iterate = true });
    defer root.close(io);

    var present: std.ArrayList([]const u8) = .empty;
    var walked = root.iterateAssumeFirstIteration();
    while (try walked.next(io)) |entry| {
        if (std.mem.startsWith(u8, entry.name, ".")) continue;
        if (std.mem.eql(u8, entry.name, build_output)) continue;
        try present.append(held, try held.dupe(u8, entry.name));
    }

    for (present.items) |name| {
        var shipped = false;
        for (manifest.paths) |path| shipped = shipped or std.mem.eql(u8, path, name);
        std.testing.expect(shipped) catch |failed| {
            std.debug.print("`{s}` is in this package and `build.zig.zon` does not ship it\n", .{name});
            return failed;
        };
    }

    for (manifest.paths) |path| {
        var here = false;
        for (present.items) |name| here = here or std.mem.eql(u8, path, name);
        std.testing.expect(here) catch |failed| {
            std.debug.print("`build.zig.zon` ships `{s}`, which is not in this package\n", .{path});
            return failed;
        };
    }
}
