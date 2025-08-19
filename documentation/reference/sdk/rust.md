# Rust SDK

The official Hook0 SDK for Rust applications, providing a safe, performant, and idiomatic interface to the Hook0 API.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
hook0 = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
```

## Quick Start

:::note Rust SDK Implementation
The Rust SDK is currently in development. While the crate is published, the full implementation matching the API is not yet complete. Please use the REST API directly or the TypeScript SDK for production use.
:::

```rust
// Example of using the REST API directly with reqwest
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let event = json!({
        "event_type": "user.created",
        "payload": {
            "user_id": "user_123",
            "email": "john.doe@example.com"
        },
        "labels": {
            "environment": "production"
        }
    });
    
    let response = client
        .post("https://app.hook0.com/api/v1/event")
        .header("Authorization", "Bearer biscuit:YOUR_TOKEN_HERE")
        .header("Content-Type", "application/json")
        .json(&event)
        .send()
        .await?;
    
    if response.status().is_success() {
        let result: serde_json::Value = response.json().await?;
        println!("Event sent: {}", result["event_id"]);
    }
    
    Ok(())
}
```

## Configuration

:::warning SDK Configuration Not Yet Available
The configuration options shown below are planned features. Currently, use the REST API directly with your preferred HTTP client.
:::

## Using the REST API with Rust

Until the official Rust SDK is complete, you can use the REST API directly:

### Send Event

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct EventRequest {
    event_type: String,
    payload: serde_json::Value,
    labels: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct EventResponse {
    event_id: String,
    status: String,
}

async fn send_event(
    client: &Client,
    token: &str,
    event: EventRequest,
) -> Result<EventResponse, reqwest::Error> {
    let response = client
        .post("https://app.hook0.com/api/v1/event")
        .header("Authorization", format!("Bearer {}", token))
        .json(&event)
        .send()
        .await?
        .json::<EventResponse>()
        .await?;
    
    Ok(response)
}

// Usage
let client = Client::new();
let event = EventRequest {
    event_type: "user.created".to_string(),
    payload: json!({
        "user_id": "user_123",
        "email": "john@example.com"
    }),
    labels: Some(json!({
        "environment": "production"
    })),
};

let result = send_event(&client, "biscuit:YOUR_TOKEN", event).await?;
println!("Event sent: {}", result.event_id);
```

### List Applications

```rust
#[derive(Deserialize)]
struct Application {
    id: String,
    name: String,
    description: Option<String>,
}

async fn list_applications(
    client: &Client,
    token: &str,
) -> Result<Vec<Application>, reqwest::Error> {
    let response = client
        .get("https://app.hook0.com/api/v1/applications")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    let apps: Vec<Application> = serde_json::from_value(
        response["data"].clone()
    ).unwrap_or_default();
    
    Ok(apps)
}
```

## Webhook Verification

Implement webhook signature verification manually until the SDK is complete:

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

fn verify_webhook_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> bool {
    // Parse signature format: "sha256=..."
    if !signature.starts_with("sha256=") {
        return false;
    }
    
    let sig_hex = &signature[7..];
    
    // Calculate expected signature
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());
    
    // Constant-time comparison
    sig_hex == expected
}

// Usage in Actix-web
use actix_web::{web, HttpRequest, HttpResponse};

async fn webhook_handler(
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::Error> {
    let signature = req
        .headers()
        .get("hook0-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            actix_web::error::ErrorUnauthorized("Missing signature")
        })?;
    
    let secret = std::env::var("WEBHOOK_SECRET")
        .expect("WEBHOOK_SECRET not set");
    
    if !verify_webhook_signature(&body, signature, &secret) {
        return Err(actix_web::error::ErrorUnauthorized("Invalid signature"));
    }
    
    let payload: serde_json::Value = serde_json::from_slice(&body)?;
    println!("Webhook received: {:?}", payload);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "processed"
    })))
}
```

### Error Handling with REST API

```rust
use reqwest::{Client, StatusCode};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
enum ApiError {
    RateLimit(u64),
    Unauthorized,
    ValidationError(String),
    NetworkError(reqwest::Error),
}

async fn send_event_with_retry(
    client: &Client,
    token: &str,
    event: &EventRequest,
) -> Result<EventResponse, ApiError> {
    let mut retries = 0;
    const MAX_RETRIES: u32 = 3;
    
    loop {
        let response = client
            .post("https://app.hook0.com/api/v1/event")
            .header("Authorization", format!("Bearer {}", token))
            .json(event)
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                match resp.status() {
                    StatusCode::OK | StatusCode::CREATED => {
                        return resp.json::<EventResponse>()
                            .await
                            .map_err(ApiError::NetworkError);
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        if let Some(retry_after) = resp.headers()
                            .get("X-RateLimit-Reset")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            return Err(ApiError::RateLimit(retry_after));
                        }
                    }
                    StatusCode::UNAUTHORIZED => {
                        return Err(ApiError::Unauthorized);
                    }
                    StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                        let error_body = resp.text().await.unwrap_or_default();
                        return Err(ApiError::ValidationError(error_body));
                    }
                    _ => {
                        if retries >= MAX_RETRIES {
                            return Err(ApiError::NetworkError(
                                reqwest::Error::from(resp.error_for_status().unwrap_err())
                            ));
                        }
                    }
                }
            }
            Err(e) if retries < MAX_RETRIES => {
                eprintln!("Network error, retrying... (attempt {}/{})", retries + 1, MAX_RETRIES);
            }
            Err(e) => return Err(ApiError::NetworkError(e)),
        }
        
        retries += 1;
        sleep(Duration::from_secs(2_u64.pow(retries))).await;
    }
}
```

## Type Safety

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
fn create_user_event(user: UserCreatedPayload) -> EventRequest {
    EventRequest {
        event_type: "user.created".to_string(),
        payload: serde_json::to_value(user).unwrap(),
        labels: Some(json!({
            "source": "api",
            "version": "1.0"
        })),
    }
}

// Custom error types
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("API error: {0}")]
    Api(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Custom error: {0}")]
    Custom(String),
}
```

## Testing

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};
    
    #[tokio::test]
    async fn test_send_event() {
        let _m = mock("POST", "/api/v1/event")
            .match_header("authorization", "Bearer biscuit:test_token")
            .match_body(Matcher::Json(json!({
                "event_type": "test.event",
                "payload": {"test": true},
                "labels": null
            })))
            .with_status(201)
            .with_body(r#"{"event_id":"evt_123","status":"accepted"}"#)
            .create();
        
        let client = Client::new();
        let event = EventRequest {
            event_type: "test.event".to_string(),
            payload: json!({"test": true}),
            labels: None,
        };
        
        let result = send_event(
            &client,
            "biscuit:test_token",
            event
        ).await.unwrap();
        
        assert_eq!(result.event_id, "evt_123");
    }
    
    #[test]
    fn test_webhook_verification() {
        let body = b"test payload";
        let secret = "webhook_secret";
        
        // Generate signature
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let result = mac.finalize();
        let signature = format!("sha256={}", hex::encode(result.into_bytes()));
        
        assert!(verify_webhook_signature(body, &signature, secret));
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_end_to_end() {
        let token = std::env::var("TEST_HOOK0_TOKEN")
            .expect("TEST_HOOK0_TOKEN not set");
        
        let client = Client::new();
        let event = EventRequest {
            event_type: "test.integration".to_string(),
            payload: json!({"test": true}),
            labels: Some(json!({"test": "integration"})),
        };
        
        let result = send_event(&client, &token, event).await.unwrap();
        assert!(!result.event_id.is_empty());
        
        // Verify event was created by checking the API
        let response = client
            .get(format!("https://app.hook0.com/api/v1/events/{}", result.event_id))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

## Performance Optimization

### Connection Pooling with reqwest

```rust
use reqwest::Client;
use std::time::Duration;

// Create a client with connection pooling
let client = Client::builder()
    .pool_idle_timeout(Duration::from_secs(60))
    .pool_max_idle_per_host(100)
    .timeout(Duration::from_secs(30))
    .build()?;

// Reuse the client for all requests
let event_sender = EventSender::new(client, token);
```

### Parallel Event Processing

```rust
use futures::future::join_all;
use std::sync::Arc;

async fn process_events_parallel(
    client: Arc<Client>,
    token: &str,
    events: Vec<EventRequest>,
) -> Vec<Result<EventResponse, ApiError>> {
    const CONCURRENT_REQUESTS: usize = 10;
    
    let mut results = Vec::new();
    
    for chunk in events.chunks(CONCURRENT_REQUESTS) {
        let futures: Vec<_> = chunk
            .iter()
            .map(|event| {
                let client = client.clone();
                let token = token.to_string();
                let event = event.clone();
                
                async move {
                    send_event(&client, &token, event).await
                }
            })
            .collect();
        
        let chunk_results = join_all(futures).await;
        results.extend(chunk_results);
    }
    
    results
}
```

## Best Practices

### 1. Use Strong Types

```rust
// Bad
let payload = json!({
    "user_id": "123",
    "email": "test@example.com"
});

// Good
#[derive(Serialize)]
struct UserPayload {
    user_id: String,
    email: String,
}

let payload = UserPayload {
    user_id: "123".to_string(),
    email: "test@example.com".to_string(),
};
```

### 2. Handle Errors Properly

```rust
// Bad
let result = client.send_event(event).await.unwrap();

// Good
match client.send_event(event).await {
    Ok(result) => {
        info!("Event sent: {}", result.event_id);
    }
    Err(e) => {
        error!("Failed to send event: {:?}", e);
        // Implement retry or fallback logic
    }
}
```

### 3. Use Environment Variables

```rust
// Bad
let token = "biscuit:hardcoded_token";

// Good
let token = std::env::var("HOOK0_TOKEN")
    .expect("HOOK0_TOKEN environment variable not set");
```

### 4. Implement Idempotency

```rust
use uuid::Uuid;

let idempotency_key = Uuid::new_v4().to_string();

let event = Event {
    event_type: "payment.processed".to_string(),
    payload: json!({"amount": 100.00}),
    labels: None,
    idempotency_key: Some(idempotency_key),
};
```

## Troubleshooting

### Common Issues

**Lifetime Issues**
```rust
// Use Arc for shared ownership
use std::sync::Arc;

let client = Arc::new(Hook0Client::new("biscuit:YOUR_TOKEN_HERE"));
let client_clone = client.clone();

tokio::spawn(async move {
    client_clone.send_event(event).await.unwrap();
});
```

**Serialization Errors**
```rust
// Ensure all types are serializable
#[derive(Serialize, Deserialize)]
struct CustomPayload {
    #[serde(rename = "userId")]
    user_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
}
```

**Async Runtime Issues**
```rust
// Use tokio::main for simple cases
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here
    Ok(())
}

// Or configure runtime explicitly
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .build()?;
```

## Support

- **Documentation**: [Hook0 API Docs](https://app.hook0.com/api/v1/docs)
- **GitHub Issues**: [Report Issues](https://github.com/hook0/hook0/issues)
- **Discord**: [Join Community](https://www.hook0.com/community)

:::info Rust SDK Development
The official Rust SDK is under development. For now, use the REST API directly with reqwest or your preferred HTTP client. Watch the GitHub repository for updates on the official SDK release.
:::