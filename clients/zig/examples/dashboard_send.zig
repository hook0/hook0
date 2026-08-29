//! What the dashboard shows under "Send an event", for Zig.
//!
//! This file exists so that the snippet is compiled against the real client. A renamed method, a
//! changed signature or a dropped field turns `clients.zig.check` red on the day it happens, which
//! is the whole reason the snippet lives here rather than in the dashboard: one written by hand
//! over there is backed by nothing and drifts in silence.
//!
//! Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so
//! that anything this file needs only in order to compile stays out of it. `hook0:label` delimits
//! the one rendering of a label, which the dashboard repeats once per label the form carries and
//! joins with the separator its manifest declares — the region carries no trailing separator of its
//! own, and sits inside its container, so no label at all leaves a valid empty one.
//!
//! The `__HOOK0_*__` words are string literals, which is what lets a file full of them compile.
//! They never resolve to anything: this example is built, never run.

// hook0:snippet:begin
const std = @import("std");
const hook0 = @import("hook0");

pub fn send(io: std.Io, allocator: std.mem.Allocator) !void {
    var client: hook0.Client = .init(
        io,
        "__HOOK0_API_URL__",
        "__HOOK0_APPLICATION_ID__",
        "__HOOK0_TOKEN__",
        .{},
    );

    // The answer owns the arena the identifier points into, so one `deinit` frees the identifier,
    // the body that was sent and everything read back.
    const sent = try client.sendEvent(allocator, .{
        .event_type = "__HOOK0_EVENT_TYPE__",
        .payload = "__HOOK0_PAYLOAD__",
        .payload_content_type = "application/json",
        .labels = &.{
            // hook0:label:begin
            .{ .key = "__HOOK0_LABEL_KEY__", .value = "__HOOK0_LABEL_VALUE__" }, // hook0:label:end
        },
    });
    defer sent.deinit();

    std.log.info("ingested as {s}", .{sent.value});
}
// hook0:snippet:end

/// What makes this file a program rather than a fragment, which is what the build compiles it as.
///
/// The clock, the source of randomness and the sockets all reach this client through the `Io` the
/// caller hands in, and Zig 0.16 hands one to `main` along with the allocators, so the example
/// above takes both rather than reaching for a global.
pub fn main(init: std.process.Init) !void {
    try send(init.io, init.gpa);
}
