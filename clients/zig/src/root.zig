//! Hook0 from Zig: send events, declare the event types an application uses, verify the signature of
//! an incoming webhook, and call every operation the API declares.
//!
//! The package is two halves. What the API declares — one struct per schema, one namespace of
//! constants per closed list of strings, one error per problem, one method per operation — is
//! generated from the OpenAPI snapshot the API commits and lands under `src/generated`. What it does
//! not declare — how a request reaches the network, how a send is retried, how a signature is
//! verified — is hand-written beside it and never regenerated.
//!
//! ```zig
//! const hook0 = @import("hook0");
//!
//! var client: hook0.Client = .init(io, "https://app.hook0.com/api/v1", application_id, token, .{});
//! const sent = try client.sendEvent(allocator, .{
//!     .event_type = "billing.invoice.created",
//!     .payload = "{\"invoice\":\"in_1\"}",
//!     .payload_content_type = "application/json",
//! });
//! defer sent.deinit();
//! ```

const std = @import("std");

pub const runtime = @import("runtime.zig");
pub const signature = @import("signature.zig");
pub const transport = @import("transport.zig");
pub const client = @import("client.zig");
pub const generated = @import("generated/root.zig");

/// What this package is released as, which `build.zig.zon` declares and the build hands down.
pub const version = @import("manifest").version;

pub const Client = client.Client;
pub const Options = client.Options;
pub const RetryPolicy = client.RetryPolicy;
pub const Event = client.Event;
pub const EventType = client.EventType;
pub const generateEventId = client.generateEventId;

pub const Transport = transport.Transport;
pub const TransportError = transport.TransportError;
pub const Cause = transport.Cause;

pub const Signature = signature.Signature;
pub const SignatureError = signature.SignatureError;
pub const verifyWebhookSignature = signature.verify;
pub const verifyWebhookSignatureWithCurrentTime = signature.verifyWithCurrentTime;

pub const Owned = runtime.Owned;
pub const DecodeError = runtime.DecodeError;

pub const models = generated.models;
pub const errors = generated.errors;
pub const api = generated.api;

test {
    std.testing.refAllDecls(@This());
}
