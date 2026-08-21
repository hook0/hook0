// The rest of the file, for every Zig example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the import when a neighbouring snippet
// already showed it, it assumes a client or an event is already built, and it names a token or an
// application ID without saying where it came from. Each region below is the file that snippet
// would live in, with a hole where it goes. The page points at one by name on the fence, so what a
// snippet is standing on is one word away from the snippet itself.
//
// Every region becomes its own file under `examples/`, imported by a generated root that exists
// only to hold `std.testing.refAllDecls` on each of them — Zig analyses a declaration only once
// something needs it, and a `pub fn` nobody calls is otherwise never even looked at. `build.zig`
// writes that root itself, from whatever it finds under `examples/`, so a page gaining its ninth
// example costs this project nothing.

// HARNESS program
const std = @import("std");

/// What a consuming project's own `build.zig` already has by the time it reaches this: the
/// builder, the target and the optimize mode it resolved, and the executable the dependency is
/// wired into. Proven for real, against a throwaway project that depends on this client by path —
/// see `install-check/` — rather than by the `refAllDecls` sweep every other region here goes
/// through, since this is `build.zig` code, not code the client's own module can be imported into.
pub fn wire(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
    exe: *std.Build.Step.Compile,
) void {
    EXAMPLE
}

// END HARNESS

// HARNESS send
const std = @import("std");

pub fn run(io: std.Io, allocator: std.mem.Allocator, application_id: []const u8, token: []const u8) !void {
    EXAMPLE
}

// END HARNESS

// HARNESS event
const hook0 = @import("hook0");

// The value the page shows, held so that every field of it is checked against the client.
pub const configured: hook0.Event =
    EXAMPLE
;

// END HARNESS

// HARNESS configure
const std = @import("std");
const hook0 = @import("hook0");

pub fn configured(io: std.Io, api_url: []const u8, application_id: []const u8, token: []const u8) void {
    EXAMPLE

    // What the page's client is handed to next; a local Zig refuses a file over a variable it
    // declared and never read again.
    _ = &client;
}

// END HARNESS

// HARNESS verify
const std = @import("std");
const hook0 = @import("hook0");

/// What the reader's own HTTP layer would have handed this handler; not part of the client.
const Delivered = struct {
    body: []const u8,
    headers: []const hook0.signature.Header,

    fn header(self: Delivered, name: []const u8) ?[]const u8 {
        for (self.headers) |candidate| {
            if (std.mem.eql(u8, candidate.name, name)) return candidate.value;
        }
        return null;
    }
};

/// What the reader's own handler would answer; not part of the client.
const Response = struct { status: u16 };

pub fn handleWebhook(
    io: std.Io,
    delivered: Delivered,
    headers: [2]hook0.signature.Header,
    subscription_secret: []const u8,
) Response {
    EXAMPLE

    return .{ .status = 200 };
}

// END HARNESS

// HARNESS errors
EXAMPLE

comptime {
    // Every member declared above has to be one `hook0.SignatureError` actually raises, and
    // every one it raises has to be declared above: this is what stops the page's own
    // redeclaration of the error set from drifting the day a refusal is added or renamed.
    const hook0 = @import("hook0");
    const shown = @typeInfo(SignatureError).error_set.?;
    const real = @typeInfo(hook0.SignatureError).error_set.?;
    if (shown.len != real.len) {
        @compileError("the page's SignatureError does not name as many members as hook0.SignatureError");
    }
    for (shown) |member| {
        if (@field(SignatureError, member.name) != @field(hook0.SignatureError, member.name)) {
            @compileError("the page's SignatureError does not match hook0.SignatureError");
        }
    }
}

// END HARNESS

// HARNESS upsert
const std = @import("std");
const hook0 = @import("hook0");

pub fn declare(client: *hook0.Client, allocator: std.mem.Allocator) !void {
    EXAMPLE
}

// END HARNESS

// HARNESS api_group
const std = @import("std");
const hook0 = @import("hook0");

pub fn list(client: *hook0.Client, allocator: std.mem.Allocator, application_id: []const u8) !void {
    EXAMPLE
}

// END HARNESS
