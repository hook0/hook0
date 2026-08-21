//! What this smoke builds: one executable, and the client it is held against.
//!
//! The client is a module rooted at its own source rather than a dependency fetched by URL and
//! hash, because what is under test is the package in this repository. The package declares no
//! dependencies, so there is nothing else to bring in — but it does expect one thing of whoever
//! builds it: an options module named `manifest`, carrying the version its own `build.zig` hands
//! it. That version is read out of the client's manifest here rather than written down again, so
//! this smoke does not become a second place a release has to be remembered.

const std = @import("std");

/// Where the client sits, relative to this file.
const client = "../../../clients/zig";

/// The most bytes of the client's manifest read. It is a few dozen lines; anything past this is not
/// that file.
const max_manifest_bytes = 64 * 1024;

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const declared = b.addOptions();
    declared.addOption([]const u8, "version", declaredVersion(b));

    const hook0 = b.createModule(.{
        .root_source_file = b.path(client ++ "/src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    hook0.addOptions("manifest", declared);

    const smoke = b.addModule("smoke", .{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    smoke.addImport("hook0", hook0);

    const exe = b.addExecutable(.{ .name = "zig-smoke", .root_module = smoke });
    const run = b.addRunArtifact(exe);

    b.step("run", "Hold the client against a running instance").dependOn(&run.step);
}

/// The version the client's manifest declares.
///
/// Read as text rather than imported: `@import` of a manifest reaches only inside the package doing
/// the importing, and this one belongs to the package next door.
fn declaredVersion(b: *std.Build) []const u8 {
    const manifest = std.Io.Dir.cwd().readFileAlloc(
        b.graph.io,
        b.pathFromRoot(client ++ "/build.zig.zon"),
        b.allocator,
        .limited(max_manifest_bytes),
    ) catch @panic("the client's build.zig.zon is not readable from this smoke");

    const marker = ".version = \"";
    const at = std.mem.indexOf(u8, manifest, marker) orelse
        @panic("the client's build.zig.zon declares no version");
    const rest = manifest[at + marker.len ..];
    const end = std.mem.indexOfScalar(u8, rest, '"') orelse
        @panic("the version in the client's build.zig.zon is not closed");
    return rest[0..end];
}
