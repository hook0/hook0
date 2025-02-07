# Hook0 Rust Client

This is the Rust SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0. (**producer**)
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0. (**producer**)
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks. (**consumer**)

## Getting Started

To add the Hook0 client to your Rust project, run `cargo add hook0-client` in your project's directory.

## Usage Examples

### Initializing the `Hook0Client`
```rust
use hook0_client::{Hook0Client, Hook0ClientError};
use url::Url;
use uuid::Uuid;

fn main() -> Result<(), Hook0ClientError> {
    let api_url = Url::parse("https://api.hook0.com")?;
    let application_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;
    let token = "your-auth-token";

    let client = Hook0Client::new(api_url, application_id, token)?;
    
    println!("Hook0 client initialized successfully");
    Ok(())
}
```

### Creating an `EventType` Normally and From String
```rust
use hook0_client::{EventType, Hook0ClientError};
use std::str::FromStr;

fn main() -> Result<(), Hook0ClientError> {
    // Creating an event type normally
    let event_type = EventType::from_str("billing.invoice.paid")?;
    println!("EventType created successfully: {}", event_type);

    // Handling invalid event type
    let event_type_from_string = EventType::from_str("invalid_event_type")
        .map_err(|_| Hook0ClientError::InvalidEventType("invalid_event_type".to_string()))?;

    println!("EventType from string: {}", event_type_from_string);

    Ok(())
}
```

### Upserting Event Types
```rust
use hook0_client::{Hook0Client, Hook0ClientError};
use url::Url;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Hook0ClientError> {
    let api_url = Url::parse("https://api.hook0.com")?;
    let application_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;
    let token = "your-auth-token";

    let client = Hook0Client::new(api_url, application_id, token)?;

    let event_types = vec!["auth.user.create", "billing.invoice.paid"];

    let added_types = client.upsert_event_types(&event_types).await?;
    println!("Successfully upserted event types: {:?}", added_types);

    Ok(())
}
```

### Sending an Event with Error Handling
```rust
use hook0_client::{Event, Hook0Client, Hook0ClientError};
use url::Url;
use uuid::Uuid;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> Result<(), Hook0ClientError> {
    let api_url = Url::parse("https://api.hook0.com")?;
    let application_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000")?;
    let token = "your-auth-token";

    let client = Hook0Client::new(api_url, application_id, token)?;

    let event = Event {
        event_id: &None,
        event_type: "billing.invoice.paid",
        payload: Cow::Borrowed("{\"user_id\": \"123\", \"amount\": 100}"),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![("production".to_string(), "true".into())],
    };

    let event_id = client.send_event(&event).await?;
    println!("Event sent successfully with ID: {}", event_id);

    Ok(())
}
```

### Verifying Webhook Signature with Current Time
```rust
use hook0_client::{verify_webhook_signature_with_current_time, Hook0ClientError};
use chrono::Utc;
use std::time::Duration;

fn main() -> Result<(), Hook0ClientError> {
    let signature = "t=1636936200,v0=abc";
    let payload = b"hello !";
    let secret = "my_secret";
    let current_time = Utc::now();
    let tolerance = Duration::from_secs(300);

    verify_webhook_signature_with_current_time(signature, payload, secret, tolerance, current_time)?;

    println!("Webhook signature is valid");
    Ok(())
}
```

### Verifying Webhook Signature
```rust
use hook0_client::{verify_webhook_signature, Hook0ClientError};
use std::time::Duration;

fn main() -> Result<(), Hook0ClientError> {
    let signature = "t=1636936200,v0=abc";
    let payload = b"hello !";
    let secret = "my_secret";
    let tolerance = Duration::from_secs(300);

    verify_webhook_signature(signature, payload, secret, tolerance)?;

    println!("Webhook signature is valid");
    Ok(())
}
```

### Enabling Features

The client supports several optional features:

- `reqwest-rustls-tls-webpki-roots` (**default**): Uses Rustls with WebPKI roots for TLS. This includes Mozilla's root certificates from [webpki-roots](https://github.com/rustls/webpki-roots).
- `reqwest-rustls-tls-native-roots`: Uses Rustls with the system's root certificates, relying on [rustls-native-certs](https://github.com/rustls/webpki-roots).
- `consumer` (**default**): Enable features related to webhook signature verification.
- `producer` (**default**): Enable features related to upserting event types and sending events to Hook0.

## Examples

### Actix Web Example

The `examples/actix-web.rs` file demonstrates how to set up a simple [Actix Web](https://actix.rs/) server to handle webhooks signature verification.

This example starts a server listening on `127.0.0.1:8081` and handles incoming POST requests to the `/webhook` route. It checks webhooks signatures of incoming HTTP requests and displays information in the standard output.

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)
- Thomas Tartrau - [thomas@tartrau.fr](mailto:thomas@tartrau.fr)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)
