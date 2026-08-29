//! What this package builds, and what checks it.
//!
//! One module, `hook0`, rooted at `src/root.zig`: the generated half under `src/generated` and the
//! hand-written half beside it are one module rather than two, which is what lets a generated file
//! reach the runtime above it by relative path and keeps the seam between them a matter of imports
//! rather than of packaging.
//!
//! `zig build test` runs the suite under `tests`, which is hand-written and never regenerated. Every
//! case of it drives this client against a socket on the loopback interface. The same step compiles
//! what `examples` holds: those are what the dashboard shows under "Send an event", and a snippet
//! nothing compiles is one that drifts the day a method is renamed.

const std = @import("std");

/// What this package declares itself to be, read here so that the source never says it twice.
///
/// The version is on the wire — every request names it — and the manifest is the one place it is
/// written down: it is handed to the module below rather than repeated in a constant somebody would
/// have to remember to move.
const manifest = @import("build.zig.zon");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const declared = b.addOptions();
    declared.addOption([]const u8, "version", manifest.version);

    const hook0 = b.addModule("hook0", .{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    hook0.addOptions("manifest", declared);

    const suite = b.addModule("tests", .{
        .root_source_file = b.path("tests/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    suite.addImport("hook0", hook0);

    const run_suite = b.addRunArtifact(b.addTest(.{ .root_module = suite }));
    const run_module = b.addRunArtifact(b.addTest(.{ .root_module = hook0 }));

    const test_step = b.step("test", "Run the suite against a loopback socket");
    test_step.dependOn(&run_suite.step);
    test_step.dependOn(&run_module.step);

    addExamples(b, target, optimize, hook0, test_step);

    b.getInstallStep().dependOn(&b.addTest(.{ .root_module = hook0 }).step);
}

/// Compiles every example against this package, without running any of them.
///
/// The examples are what the dashboard renders: the values in them are markers it substitutes, so
/// running one would reach a host that does not exist. Compiling is the whole point — a method
/// renamed in `src` fails here rather than reaching a reader as a snippet that does not build.
///
/// What is compiled is found by looking rather than by naming, so an example added beside these two
/// is held to the same thing without this file being touched.
fn addExamples(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    package: *std.Build.Module,
    test_step: *std.Build.Step,
) void {
    const io = b.graph.io;
    var directory = b.build_root.handle.openDir(io, "examples", .{ .iterate = true }) catch |unopenable|
        std.debug.panic("`examples` cannot be read: {t}", .{unopenable});
    defer directory.close(io);

    var found: usize = 0;
    var walked = directory.iterate();
    while (walked.next(io) catch |unwalkable|
        std.debug.panic("`examples` cannot be walked: {t}", .{unwalkable})) |entry|
    {
        if (entry.kind != .file) continue;
        if (!std.mem.endsWith(u8, entry.name, ".zig")) continue;

        const name = b.dupe(entry.name);
        const example = b.createModule(.{
            .root_source_file = b.path(b.pathJoin(&.{ "examples", name })),
            .target = target,
            .optimize = optimize,
        });
        example.addImport("hook0", package);

        const built = b.addExecutable(.{
            .name = b.fmt("example-{s}", .{std.fs.path.stem(name)}),
            .root_module = example,
        });
        test_step.dependOn(&built.step);
        found += 1;
    }

    // An empty directory would leave this step holding nothing while still reporting success, which
    // is a guard that stopped existing without anything red saying so.
    if (found == 0) std.debug.panic("`examples` holds no example to compile", .{});
}
