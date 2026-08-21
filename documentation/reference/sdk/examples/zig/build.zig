//! What this project builds, to prove every Zig example of the SDK reference against the real
//! client.
//!
//! `clients/zig` is read directly by absolute path — substituted in below by the checker that
//! assembles this project — rather than depended on through `zig fetch` or `b.dependency`: that
//! is the ordinary way a Zig package pulls in another one, and it is also exactly what the one
//! example under "Installation" teaches a reader to write for themselves, which makes it the one
//! example this project cannot prove by importing the client the way every other one does. Proving
//! it instead means running it: `install-check/` is a second, throwaway package generated below,
//! whose own `build.zig` is the reader's own snippet, depending on `clients/zig` by path the way a
//! real consumer would.
//!
//! Every other example becomes a file under `examples/`, and Zig only analyses a declaration once
//! something needs it — a `pub fn` nobody calls is otherwise never even looked at, so building the
//! client's own package proves nothing about a page beside it. The loop below writes a small
//! generated root that does `std.testing.refAllDecls` on every file it finds there, which is what
//! forces each one's declarations, bodies included, through full analysis.

const std = @import("std");

/// Where the client this project proves its examples against lives, substituted by the checker
/// that assembles this project.
const client_root = "{{client}}";

/// Directory, relative to this project's root, that every non-`install-check` example lands in.
const examples_dir = "examples";

/// The function name that marks the one example `install-check/` builds for real instead of
/// importing — see the module doc comment above. Chosen once, here, rather than kept in step with
/// the example's position on the page, so reordering the page's fences cannot silently stop this
/// project from finding it.
const install_marker = "pub fn wire(";

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const io = b.graph.io;

    const hook0_mod = clientModule(b, target, optimize);

    const discovered = discoverExamples(b, io) catch |err|
        std.debug.panic("could not read {s}: {t}", .{ examples_dir, err });

    const suite = b.createModule(.{
        .root_source_file = writeGeneratedIndex(b, io, discovered.swept),
        .target = target,
        .optimize = optimize,
    });
    suite.addImport("hook0", hook0_mod);

    const run_suite = b.addRunArtifact(b.addTest(.{ .root_module = suite }));
    const test_step = b.step("test", "Prove every example against the real client");
    test_step.dependOn(&run_suite.step);
    b.default_step.dependOn(test_step);

    writeInstallCheck(b, io, discovered.install_wiring);
}

/// The `hook0` module every non-`install-check` example is checked against: `clients/zig`'s own
/// root, read by absolute path rather than fetched, with the `manifest` options module its
/// `root.zig` imports supplied a documentation-only version rather than the real one — this
/// project proves the client's API, not what it announces itself as on the wire.
fn clientModule(b: *std.Build, target: std.Build.ResolvedTarget, optimize: std.builtin.OptimizeMode) *std.Build.Module {
    const declared = b.addOptions();
    declared.addOption([]const u8, "version", "0.0.0-documentation-examples");

    const hook0_mod = b.createModule(.{
        .root_source_file = .{ .cwd_relative = client_root ++ "/src/root.zig" },
        .target = target,
        .optimize = optimize,
    });
    hook0_mod.addOptions("manifest", declared);
    return hook0_mod;
}

const Discovered = struct {
    /// Every `examples/*.zig` file proven by import, none of them the one `build.zig` teaches.
    swept: std.ArrayList([]const u8),
    /// The body of the one example that shows a consuming project's own `build.zig`, when the page
    /// carries one; proven by `install-check/` instead of by import.
    install_wiring: ?[]const u8,
};

/// Every assembled example under `examples/`, split into what an import can prove and the one
/// thing it cannot.
fn discoverExamples(b: *std.Build, io: std.Io) !Discovered {
    var swept: std.ArrayList([]const u8) = .empty;
    var install_wiring: ?[]const u8 = null;

    var dir = try std.Io.Dir.cwd().openDir(io, examples_dir, .{ .iterate = true });
    defer dir.close(io);

    var names: std.ArrayList([]const u8) = .empty;
    var it = dir.iterate();
    while (try it.next(io)) |entry| {
        if (entry.kind != .file) continue;
        if (!std.mem.endsWith(u8, entry.name, ".zig")) continue;
        try names.append(b.allocator, b.dupe(entry.name));
    }
    // Sorted so that a rebuild that changed nothing writes the same generated root: content
    // determines what this project proves, never the order a directory happened to iterate in.
    std.mem.sort([]const u8, names.items, {}, lessThan);

    for (names.items) |name| {
        const path = std.fmt.allocPrint(b.allocator, "{s}/{s}", .{ examples_dir, name }) catch @panic("OOM");
        const content = std.Io.Dir.cwd().readFileAlloc(io, path, b.allocator, .limited(1 << 20)) catch |err|
            std.debug.panic("could not read {s}: {t}", .{ path, err });

        if (std.mem.indexOf(u8, content, install_marker) != null) {
            install_wiring = content;
            continue;
        }
        try swept.append(b.allocator, path);
    }

    return .{ .swept = swept, .install_wiring = install_wiring };
}

fn lessThan(_: void, lhs: []const u8, rhs: []const u8) bool {
    return std.mem.lessThan(u8, lhs, rhs);
}

/// The root every example under `examples/` — but the one `install-check/` builds instead — is
/// referenced from, so that none of them is skipped by Zig's lazy analysis.
///
/// Written fresh on every configure rather than kept in the scaffold: the set of examples is
/// whatever the page shows today, and a file listing them by name would be the one thing here that
/// could fall out of step with it.
fn writeGeneratedIndex(b: *std.Build, io: std.Io, swept: std.ArrayList([]const u8)) std.Build.LazyPath {
    var out: std.ArrayList(u8) = .empty;
    out.appendSlice(b.allocator, "// Generated at configure time from whatever `examples/` holds; not part of the scaffold.\n") catch @panic("OOM");
    out.appendSlice(b.allocator, "const std = @import(\"std\");\n\ntest {\n") catch @panic("OOM");
    for (swept.items) |path| {
        const line = std.fmt.allocPrint(b.allocator, "    std.testing.refAllDecls(@import(\"{s}\"));\n", .{path}) catch @panic("OOM");
        out.appendSlice(b.allocator, line) catch @panic("OOM");
    }
    out.appendSlice(b.allocator, "}\n") catch @panic("OOM");

    const generated_path = "generated_index.zig";
    std.Io.Dir.cwd().writeFile(io, .{ .sub_path = generated_path, .data = out.items }) catch |err|
        std.debug.panic("could not write {s}: {t}", .{ generated_path, err });
    return b.path(generated_path);
}

/// The throwaway package that proves the one example an import cannot: a `build.zig` depending on
/// `clients/zig` by path, exactly as a real consumer's would.
///
/// `install-check/src/main.zig` is the one part of this that is scaffold, since it never has
/// anything to do with what the page shows. `build.zig` and `build.zig.zon` are generated: the
/// dependency path has to be relative to this file, and this project's own location is not known
/// until it is assembled.
fn writeInstallCheck(b: *std.Build, io: std.Io, wiring: ?[]const u8) void {
    const body = wiring orelse
        "pub fn wire(b: *std.Build, target: std.Build.ResolvedTarget, optimize: std.builtin.OptimizeMode, exe: *std.Build.Step.Compile) void {\n    _ = .{ b, target, optimize, exe };\n}\n";

    var source: std.ArrayList(u8) = .empty;
    source.appendSlice(b.allocator, body) catch @panic("OOM");
    source.appendSlice(b.allocator,
        \\
        \\pub fn build(b: *std.Build) void {
        \\    const target = b.standardTargetOptions(.{});
        \\    const optimize = b.standardOptimizeOption(.{});
        \\
        \\    const exe = b.addExecutable(.{
        \\        .name = "install_check",
        \\        .root_module = b.createModule(.{
        \\            .root_source_file = b.path("src/main.zig"),
        \\            .target = target,
        \\            .optimize = optimize,
        \\        }),
        \\    });
        \\
        \\    wire(b, target, optimize, exe);
        \\
        \\    b.installArtifact(exe);
        \\}
        \\
    ) catch @panic("OOM");

    write(io, "install-check/build.zig", source.items);
    write(io, "install-check/build.zig.zon", zonFor(b, relativeToInstallCheck(b)));
}

/// The path from `install-check/` back to the client, computed rather than written down: both are
/// absolute, and only one of the two is known before this project is assembled.
fn relativeToInstallCheck(b: *std.Build) []const u8 {
    const install_check_dir = std.fmt.allocPrint(b.allocator, "{s}/install-check", .{b.build_root.path orelse "."}) catch @panic("OOM");
    return relativeBetweenAbsolute(b.allocator, install_check_dir, client_root);
}

/// The relative path from one absolute POSIX path to another. `std.fs.path.relative` exists, but
/// resolves its inputs against a working directory this project never needs, since both of these
/// already are absolute; this is the same walk-the-shared-prefix-then-`..`-out algorithm with that
/// step removed.
fn relativeBetweenAbsolute(allocator: std.mem.Allocator, from: []const u8, to: []const u8) []const u8 {
    var from_it = std.mem.tokenizeScalar(u8, from, '/');
    var to_it = std.mem.tokenizeScalar(u8, to, '/');
    while (true) {
        const from_component = from_it.next() orelse {
            const rest = to_it.rest();
            return if (rest.len == 0) "." else allocator.dupe(u8, rest) catch @panic("OOM");
        };
        const to_rest = to_it.rest();
        if (to_it.next()) |to_component| {
            if (std.mem.eql(u8, from_component, to_component)) continue;
        }

        var up_count: usize = 1;
        while (from_it.next()) |_| up_count += 1;

        var out: std.ArrayList(u8) = .empty;
        for (0..up_count) |_| out.appendSlice(allocator, "../") catch @panic("OOM");
        out.appendSlice(allocator, to_rest) catch @panic("OOM");
        return out.items;
    }
}

fn zonFor(b: *std.Build, dependency_path: []const u8) []const u8 {
    return std.fmt.allocPrint(b.allocator,
        \\.{{
        \\    .name = .hook0_docs_install_check,
        \\    .version = "0.0.0",
        \\    .minimum_zig_version = "0.16.0",
        \\    .fingerprint = 0xe4e2d81340db1559,
        \\    .dependencies = .{{
        \\        .hook0 = .{{ .path = "{s}" }},
        \\    }},
        \\    .paths = .{{ "build.zig", "build.zig.zon", "src" }},
        \\}}
        \\
    , .{dependency_path}) catch @panic("OOM");
}

fn write(io: std.Io, sub_path: []const u8, data: []const u8) void {
    std.Io.Dir.cwd().writeFile(io, .{ .sub_path = sub_path, .data = data }) catch |err|
        std.debug.panic("could not write {s}: {t}", .{ sub_path, err });
}
