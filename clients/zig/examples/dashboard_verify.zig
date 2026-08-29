//! What the dashboard shows under "Verify a webhook", for Zig.
//!
//! Sending is only half of what a reader has come to do, and it is the easier half. This is the one
//! the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
//! the send rather than leaving it to be found later.
//!
//! The secret is read from the environment on purpose. The dashboard cannot know which subscription
//! a reader means — outside the onboarding it loads none, and an application may have several — so
//! it points at the subscription instead of guessing one, and no second secret is put on screen.
//!
//! Read the markers as in `dashboard_send.zig`: `hook0:snippet` is what is displayed, everything
//! outside it is what makes the file compile.

// hook0:snippet:begin
const std = @import("std");
const hook0 = @import("hook0");

// Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
// what was signed. The tolerance is in seconds and bilateral, so a delivery dated too far ahead is
// refused exactly like one dated too far behind.
pub fn accept(
    io: std.Io,
    environ: *const std.process.Environ.Map,
    signature: []const u8,
    body: []const u8,
    headers: []const hook0.signature.Header,
) error{SubscriptionSecretNotSet}!bool {
    // The secret of the subscription being verified, which the dashboard links to rather than
    // prints: it cannot know which subscription a reader means, and an application may have several.
    // Zig hands the environment to `main` rather than exposing it ambiently, so it is passed in.
    //
    // Its absence is an error and not an empty secret, which is what the error union in the return
    // type is for: verifying against nothing hashes every genuine delivery to the wrong code, and a
    // plain `bool` has no room to tell that apart from a forgery. A variable exported with nothing
    // in it verifies against exactly the same nothing, so it takes the same exit.
    const secret = environ.get("HOOK0_SUBSCRIPTION_SECRET") orelse
        return error.SubscriptionSecretNotSet;
    if (secret.len == 0) return error.SubscriptionSecretNotSet;
    hook0.verifyWebhookSignature(io, signature, body, headers, secret, 300) catch return false;
    return true;
}
// hook0:snippet:end

/// What makes this file a program rather than a fragment, which is what the build compiles it as.
pub fn main(init: std.process.Init) !void {
    // Nothing here is ever run: this file exists to be compiled against the real client.
    const accepted = try accept(init.io, init.environ_map, "", "", &.{.{ .name = "x-hook0-signature", .value = "" }});
    std.log.info("accepted: {}", .{accepted});
}
