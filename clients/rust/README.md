# Hook0 Rust Client

This is the Rust SDK for Hook0, an open source Webhooks-as-a-Service platform designed for SaaS applications.

## What is Hook0?

**Hook0** is a product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Features

- **Send Events**: Send events to Hook0. (**producer**)
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0. (**producer**)
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks. (**consumer**)

## Examples

### Actix Web Example

The `examples/actix-web.rs` file demonstrates how to set up a simple [Actix Web](https://actix.rs/) server to handle webhooks signature verification.

This example starts a server listening on `127.0.0.1:8081` and handles incoming POST requests to the `/webhook` route. It checks webhooks signatures of incoming HTTP requests and displays information in the standard output.

## Getting Started

To use the Hook0 client in your Rust project, run `cargo add hook0-client` in your project.
You can also manually add the following to your `Cargo.toml`.

### Enabling Features

The client supports several optional features:

- `reqwest-rustls-tls-webpki-roots`: Uses Rustls with WebPKI roots for TLS. This includes Mozilla's root certificates from [webpki-roots](https://github.com/rustls/webpki-roots).
- `reqwest-rustls-tls-native-roots`: Uses Rustls with the system's root certificates, relying on [rustls-native-certs](https://github.com/rustls/webpki-roots).
- `consumer`: Enable features related to webhook signature verification.
- `producer`: Enable features related to upserting event types and sending events to Hook0.

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)
