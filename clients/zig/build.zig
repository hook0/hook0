//! What this package builds, and what checks it.
//!
//! One module, `hook0`, rooted at `src/root.zig`: the generated half under `src/generated` and the
//! hand-written half beside it are one module rather than two, which is what lets a generated file
//! reach the runtime above it by relative path and keeps the seam between them a matter of imports
//! rather than of packaging.
//!
//! `zig build test` runs the suite under `tests`, which is hand-written and never regenerated. Every
//! case of it drives this client against a socket on the loopback interface.

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

    b.getInstallStep().dependOn(&b.addTest(.{ .root_module = hook0 }).step);
}
