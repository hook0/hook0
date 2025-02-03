# Hook0 Rust Client

This is the Rust SDK for Hook0, an open-source webhooks-as-a-service platform designed for SaaS applications.

## Features

- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Send Events**: Send events to Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.

## Examples

### Actix Web Example

The `examples/actix-web.rs` file demonstrates how to set up a simple [Actix Web](https://actix.rs/) server to handle webhooks signature verification.

```rust
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/webhook", web::post().to(handle_webhook)))
        .bind("127.0.0.1:8081")?
        .run()
        .await
}
```

This example starts a server listening on `127.0.0.1:8081` and handles incoming POST requests to the `/webhook` route. It checks webhooks signatures of incoming HTTP requests and displays information in the standard output.

## Getting Started

To use the Hook0 client in your Rust project, run `cargo add hook0-client` in your project.
Or you can add the following to your `Cargo.toml`:

```toml
[dependencies]
hook0-client = "hook0-client-version"
```

### Enabling Features

The client supports several optional features:

- `reqwest-rustls-tls-webpki-roots`: Use Rustls with WebPKI roots for TLS.
- `consumer`: Enable features related to webhook signature verification.
- `producer`: Enable features related to upserting event types and sending events to Hook0.

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)
