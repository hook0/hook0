//! The suite this client is held to, which is hand-written and never regenerated.
//!
//! Every case of it drives the real client against a Hook0 API on a loopback socket, and the ones
//! the shared conformance corpus dictates read that corpus rather than repeating it.

test {
    _ = @import("conformance_test.zig");
    _ = @import("signature_test.zig");
    _ = @import("client_test.zig");
    _ = @import("property_test.zig");
    _ = @import("generated_test.zig");
    _ = @import("packaging_test.zig");
}
