# Rust SDK

The official Hook0 SDK for Rust applications, providing a safe, performant, and idiomatic interface to the Hook0 API.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
hook0-client = "1"
```

## Quick Start

The Rust SDK (`hook0-client`) supports both webhook production (sending events) and consumption (verifying webhook signatures).

### Send Events (Producer)

```rust
use hook0_client::{Hook0Client, Event};
use reqwest::Url;
use uuid::Uuid;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client
    let api_url = Url::parse("http://localhost:8081/api/v1")?;
    let application_id = Uuid::parse_str("{APP_ID}-here")?;
    let token = "{YOUR_TOKEN}";

    let client = Hook0Client::new(api_url, application_id, token)?;

    // Create an event
    let event = Event {
        event_id: &None,
        event_type: "user.account.created",
        payload: Cow::Borrowed(r#"{"user_id": "123", "email": "john@example.com"}"#),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![
            ("environment".to_string(), "production".to_string()),
        ],
    };

    // Send the event
    let event_id = client.send_event(&event).await?;
    println!("Event sent: {}", event_id);

    Ok(())
}
```

### Verify Webhook Signatures (Consumer)

```rust
use hook0_client::verify_webhook_signature;
use std::time::Duration;

fn verify_incoming_webhook(
    signature_header: &str,
    body: &[u8],
    headers: &[(&str, &str)],
    subscription_secret: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 5 minutes tolerance for timestamp validation
    let tolerance = Duration::from_secs(300);

    verify_webhook_signature(
        signature_header,
        body,
        headers,
        subscription_secret,
        tolerance,
    )?;

    println!("Webhook signature verified successfully");
    Ok(())
}
```

## Features

The SDK supports optional features that can be enabled in your `Cargo.toml`:

```toml
[dependencies]
hook0-client = { version = "1", features = ["producer", "consumer"] }
```

### Available Features

- **`producer`** (default): Enable features for sending events to Hook0 and upserting event types
- **`consumer`** (default): Enable features for verifying webhook signatures

### Minimal Producer-Only Installation

```toml
[dependencies]
hook0-client = { version = "1", default-features = false, features = ["producer"] }
```

### Minimal Consumer-Only Installation

```toml
[dependencies]
hook0-client = { version = "1", default-features = false, features = ["consumer"] }
```

## Configuration

Initialize the client with your Hook0 credentials:

```rust
use hook0_client::Hook0Client;
use reqwest::Url;
use uuid::Uuid;

let api_url = Url::parse("http://localhost:8081/api/v1")?;
let application_id = Uuid::parse_str("your-application-id")?;
let token = std::env::var("HOOK0_TOKEN")?;

let client = Hook0Client::new(api_url, application_id, &token)?;
```

## Upserting Event Types

Ensure your application has the required event types defined:

```rust
let event_types = vec![
    "user.account.created",
    "user.account.updated",
    "user.account.deleted",
    "order.checkout.completed",
    "order.shipped",
];

let created_types = client.upsert_event_types(&event_types).await?;
println!("Created {} new event types", created_types.len());
```

## Webhook Verification

The SDK provides built-in webhook signature verification:

### Example: Actix-web Integration

```rust
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use hook0_client::{verify_webhook_signature, Hook0ClientError};
use std::time::Duration;

async fn handle_webhook(
    subscription_secret: web::Data<String>,
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    let signature = req.headers().get("X-Hook0-Signature");

    if let Some(signature) = signature {
        let signature: &str = signature.to_str().unwrap();
        let tolerance = Duration::from_secs(300);

        // Collect headers as a Vec of tuples for the verification function
        match verify_webhook_signature(
            signature,
            &body,
            &req.headers().iter().collect::<Vec<_>>(),
            subscription_secret.into_inner().as_str(),
            tolerance,
        ) {
            Ok(_) => println!("Signature verification successful!"),
            Err(Hook0ClientError::InvalidSignature) => {
                println!("Signature verification failed: Invalid signature.")
            }
            Err(Hook0ClientError::ExpiredWebhook {
                signed_at,
                tolerance,
                current_time,
            }) => {
                println!(
                    "Signature verification failed: expired (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})"
                )
            }
            Err(e) => {
                println!("Signature verification failed: {e}")
            }
        }
    }

    "Ok"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscription_secret = std::env::var("SUBSCRIPTION_SECRET")
        .expect("You must define a SUBSCRIPTION_SECRET environment variable");

    HttpServer::new(move || {
        App::new()
            .route("/webhook", web::post().to(handle_webhook))
            .app_data(web::Data::new(subscription_secret.to_owned()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

See the `examples/actix-web.rs` file in the [repository](https://github.com/hook0/hook0/tree/master/clients/rust) for a complete example with logging.

## Error Handling

The SDK uses the `Hook0ClientError` enum for comprehensive error handling:

```rust
use hook0_client::{Hook0Client, Hook0ClientError, Event};

async fn send_event_with_handling(client: &Hook0Client, event: &Event<'_>) {
    match client.send_event(event).await {
        Ok(event_id) => {
            println!("Event sent successfully: {}", event_id);
        }
        Err(Hook0ClientError::EventSending { event_id, error, body }) => {
            eprintln!("Failed to send event {}: {}", event_id, error);
            if let Some(body) = body {
                eprintln!("Response body: {}", body);
            }
        }
        Err(Hook0ClientError::InvalidEventType(event_type)) => {
            eprintln!("Invalid event type: {}", event_type);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
}
```

### Consumer Errors

```rust
use hook0_client::{Hook0ClientError, verify_webhook_signature};
use std::time::Duration;

fn handle_webhook_verification(
    signature: &str,
    payload: &[u8],
    headers: &[(&str, &str)],
    subscription_secret: &str,
) {
    match verify_webhook_signature(signature, payload, headers, subscription_secret, Duration::from_secs(300)) {
        Ok(()) => println!("Valid webhook"),
        Err(Hook0ClientError::InvalidSignature) => {
            eprintln!("Invalid signature - webhook may be forged");
        }
        Err(Hook0ClientError::ExpiredWebhook { signed_at, tolerance, current_time }) => {
            eprintln!("Webhook expired: signed at {}, current time {}, tolerance {}",
                signed_at, current_time, tolerance);
        }
        Err(Hook0ClientError::MissingHeader(header_name)) => {
            eprintln!("Missing required header: {}", header_name);
        }
        Err(e) => {
            eprintln!("Verification error: {}", e);
        }
    }
}
```

## Type Safety

Use strongly-typed payloads with serde:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use hook0_client::{Hook0Client, Event};
use std::borrow::Cow;

// Define strongly-typed payloads
#[derive(Serialize, Deserialize, Debug)]
struct UserCreatedPayload {
    user_id: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderPlacedPayload {
    order_id: String,
    customer_id: String,
    total: f64,
    items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderItem {
    product_id: String,
    quantity: u32,
    price: f64,
}

// Type-safe event creation
async fn create_and_send_user_event(
    client: &Hook0Client,
    user: UserCreatedPayload,
) -> Result<uuid::Uuid, Box<dyn std::error::Error>> {
    let payload = serde_json::to_string(&user)?;

    let event = Event {
        event_id: &None,
        event_type: "user.account.created",
        payload: Cow::Owned(payload),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![
            ("source".to_string(), "api".to_string()),
            ("version".to_string(), "1.0".to_string()),
        ],
    };

    let event_id = client.send_event(&event).await?;
    Ok(event_id)
}
```

## Testing

### Unit Testing Webhook Verification

```rust
#[cfg(test)]
mod tests {
    use hook0_client::verify_webhook_signature;
    use std::time::Duration;

    #[test]
    fn test_valid_signature_v1() {
        // v1 signature includes headers in the HMAC computation
        let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
        let payload = "hello !".as_bytes();
        let subscription_secret = "secret";
        let tolerance = Duration::from_secs((i64::MAX / 1000) as u64);

        let headers = vec![
            ("x-test", "val1"),
            ("x-test2", "val2"),
        ];

        assert!(verify_webhook_signature(
            signature,
            payload,
            &headers,
            subscription_secret,
            tolerance,
        ).is_ok());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use hook0_client::{Hook0Client, Event};
    use reqwest::Url;
    use uuid::Uuid;
    use std::borrow::Cow;

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_end_to_end() {
        let api_url = Url::parse("http://localhost:8081/api/v1").unwrap();
        let application_id = Uuid::parse_str(&std::env::var("TEST_APP_ID").unwrap()).unwrap();
        let token = std::env::var("TEST_HOOK0_TOKEN").expect("TEST_HOOK0_TOKEN not set");

        let client = Hook0Client::new(api_url, application_id, &token).unwrap();

        // Ensure event type exists
        client.upsert_event_types(&["test.integration"]).await.unwrap();

        // Send test event
        let event = Event {
            event_id: &None,
            event_type: "test.integration",
            payload: Cow::Borrowed(r#"{"test": true}"#),
            payload_content_type: "application/json",
            metadata: None,
            occurred_at: None,
            labels: vec![("test".to_string(), "integration".to_string())],
        };

        let event_id = client.send_event(&event).await.unwrap();
        assert!(!event_id.to_string().is_empty());
    }
}
```

## Performance Optimization

### Connection Reuse

The `Hook0Client` uses `reqwest::Client` internally, which maintains a connection pool. Reuse the client instance:

```rust
use hook0_client::Hook0Client;
use std::sync::Arc;

// Create once, share across threads
let client = Arc::new(Hook0Client::new(api_url, application_id, &token)?);

// Clone Arc for different async tasks
let client_clone = client.clone();
tokio::spawn(async move {
    let event = /* ... */;
    client_clone.send_event(&event).await.unwrap();
});
```

### Parallel Event Processing

```rust
use futures::future::join_all;
use hook0_client::{Hook0Client, Event};
use std::sync::Arc;

async fn send_events_parallel(
    client: Arc<Hook0Client>,
    events: Vec<Event<'_>>,
) -> Vec<Result<uuid::Uuid, hook0_client::Hook0ClientError>> {
    let futures: Vec<_> = events
        .iter()
        .map(|event| {
            let client = client.clone();
            async move {
                client.send_event(event).await
            }
        })
        .collect();

    join_all(futures).await
}
```

## Best Practices

### 1. Reuse Client Instances

```rust
// Initialize once at application startup
let client = Hook0Client::new(api_url, application_id, &token)?;

// Share across your application (use Arc for thread-safety)
let client = Arc::new(client);
```

### 2. Use Strong Types

```rust
use serde::Serialize;

#[derive(Serialize)]
struct UserPayload {
    user_id: String,
    email: String,
}

let payload = UserPayload {
    user_id: "123".to_string(),
    email: "test@example.com".to_string(),
};

let payload_str = serde_json::to_string(&payload)?;
```

### 3. Handle Errors Properly

```rust
use log::{info, error};

match client.send_event(&event).await {
    Ok(event_id) => {
        info!("Event sent: {}", event_id);
    }
    Err(e) => {
        error!("Failed to send event: {:?}", e);
        // Implement retry or fallback logic
    }
}
```

### 4. Use Environment Variables

```rust
let token = std::env::var("HOOK0_TOKEN")
    .expect("HOOK0_TOKEN environment variable not set");
let application_id = std::env::var("HOOK0_APP_ID")
    .expect("HOOK0_APP_ID environment variable not set");
```

### 5. Provide Custom Event IDs for Idempotency

```rust
use uuid::Uuid;
use std::borrow::Cow;
use hook0_client::Event;

let custom_event_id = Uuid::new_v4();
let event_id_opt = Some(&custom_event_id);

let event = Event {
    event_id: &event_id_opt,
    event_type: "payment.processed",
    payload: Cow::Borrowed(r#"{"amount": 100.00}"#),
    payload_content_type: "application/json",
    metadata: None,
    occurred_at: None,
    labels: vec![],
};
```

## Troubleshooting

### Common Issues

**Lifetime Issues with Async**
```rust
use std::sync::Arc;
use hook0_client::Hook0Client;

// Wrap client in Arc for sharing across async tasks
let client = Arc::new(Hook0Client::new(api_url, application_id, &token)?);
let client_clone = client.clone();

tokio::spawn(async move {
    let event = /* ... */;
    client_clone.send_event(&event).await.unwrap();
});
```

**Payload Content Type Mismatch**
```rust
// Ensure payload string matches content type
let event = Event {
    event_id: &None,
    event_type: "user.account.created",
    payload: Cow::Borrowed(r#"{"user_id": "123"}"#),  // JSON string
    payload_content_type: "application/json",  // Must match
    metadata: None,
    occurred_at: None,
    labels: vec![],
};
```

**Async Runtime Issues**
```rust
// Use tokio::main for simple cases
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = /* ... */;
    client.send_event(&event).await?;
    Ok(())
}
```

## Links

- **Crate**: [hook0-client on crates.io](https://crates.io/crates/hook0-client)
- **Documentation**: [docs.rs/hook0-client](https://docs.rs/hook0-client)
- **Source Code**: [GitHub Repository](https://github.com/hook0/hook0/tree/master/clients/rust)
- **Examples**: [examples/actix-web.rs](https://github.com/hook0/hook0/tree/master/clients/rust/examples)
- **API Docs**: [Hook0 API Reference](../../openapi/intro)
- **Issues**: [GitHub Issues](https://github.com/hook0/hook0/issues)
- **Discord**: [Join Community](https://www.hook0.com/community)
