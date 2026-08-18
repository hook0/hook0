---
title: "Rust webhook SDK — hook0-client crate"
description: "Send Hook0 events and verify webhook signatures from Rust. Async on tokio and reqwest, idempotent event IDs, retries and payload bounds built in. Producer and consumer are separate features."
keywords: [Rust webhook SDK, Hook0 Rust client, hook0-client crate, verify webhook signature Rust, actix-web webhook, async webhook client Rust]
sdkTarget: rust
---

# Rust SDK

The Hook0 SDK for Rust sends events and verifies webhook signatures. Sending is `async` on `tokio` and `reqwest`; verifying is a plain function that computes an HMAC and touches nothing.

The crate is split in two features, both on by default. `producer` is everything that reaches the API — the client, the retry policy, the generated API groups. `consumer` is signature verification alone, and pulls in no HTTP stack at all.

## Installation

```toml
[dependencies]
hook0-client = "1"
```

A consumer that only verifies deliveries takes the half it needs and nothing else:

```toml
[dependencies]
hook0-client = { version = "1", default-features = false, features = ["consumer"] }
```

That drops `reqwest`, `tokio`, `serde_json`, `url` and `uuid` from the dependency graph. `producer` alone is the mirror image, for an emitter that never receives a webhook. Enabling neither is a compile error rather than a crate that builds and does nothing.

## Send an event

```rust example=send
use hook0_client::{Event, Hook0Client};
use reqwest::Url;
use std::borrow::Cow;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Hook0Client::new(
        Url::parse("https://app.hook0.com/api/v1")?,
        Uuid::parse_str("0d0ea1e0-8b1f-4f4d-9c2a-1b5c9a3d7e21")?,
        &std::env::var("HOOK0_TOKEN")?,
    )?;

    let event_id = client
        .send_event(&Event {
            event_id: None,
            event_type: "billing.invoice.paid",
            payload: Cow::Borrowed(r#"{"invoice": "in_123"}"#),
            payload_content_type: "application/json",
            metadata: None,
            occurred_at: None,
            labels: vec![("environment".to_owned(), "production".to_owned())],
        })
        .await?;

    println!("ingested as {event_id}");
    Ok(())
}
```

`Event` has no builder and no `Default`: every field is written at every call site, which is what keeps a new one from being silently left out the day the API grows another.

```rust example=event
Event {
    event_id: Some(chosen), // None means the client mints a UUIDv7
    event_type: "billing.invoice.paid",
    payload: Cow::Borrowed(r#"{"invoice": "in_123"}"#),
    payload_content_type: "application/json",
    metadata: Some(vec![("emitter".to_owned(), "billing-worker".to_owned())]),
    occurred_at: None, // the current moment when absent
    labels: vec![("environment".to_owned(), "production".to_owned())],
}
```

`payload` is a `Cow`, so a payload borrowed from a literal costs no allocation and one built by `serde_json::to_string` is moved in as `Cow::Owned`. Labels and metadata are pairs rather than a map because the API reads them in the order they were written.

The token goes in without a `Bearer` prefix; the client adds it.

## Sending an event is idempotent, and retried

`send_event` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `send_event` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is returned as an error.

## Bounds, and how to change them

Every builder method takes the client by value and gives it back, so the bounds are set where the client is built:

```rust example=bounds
use hook0_client::{Hook0Client, RetryPolicy};
use std::time::Duration;

let client = Hook0Client::new(api_url, application_id, &token)?
    .with_retry_policy(RetryPolicy {
        max_attempts: 4,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(2),
        max_total_delay: Duration::from_secs(5),
    })
    .with_request_timeout(Duration::from_secs(10))
    .with_max_payload_bytes(1024 * 1024)
    .with_max_response_bytes(8 * 1024 * 1024);
```

Those are the defaults: the first four are what `RetryPolicy::default()` returns, and the rest are public constants of the crate.

| Bound | Default |
|-------|---------|
| `max_attempts` (the first attempt included) | `4`, capped at `MAX_ATTEMPTS_CAP` = 16 |
| `initial_backoff` | 100 ms |
| `max_backoff` | 2 s |
| `max_total_delay`, the budget all delays of one send share | 5 s |
| `DEFAULT_REQUEST_TIMEOUT`, per attempt | 10 s |
| `DEFAULT_MAX_PAYLOAD_BYTES` | 1 MiB |
| `DEFAULT_MAX_RESPONSE_BYTES` | 8 MiB |
| `MAX_RESPONSE_HEADERS` | 64 |
| `MAX_HEADER_BYTES` | 64 KiB |
| `MAX_HEAD_BYTES` | 16 KiB |

The last three are not configurable. They bound the head of an answer, which is written by whatever is on the other end: a line count and a size per line multiply, so the whole head is capped as well as each line of it, and the line that crosses either one stops the read before the body is touched.

`RetryPolicy::disabled()` sends each event exactly once. A payload above the maximum is refused before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

The delay before a retry doubles from `initial_backoff`, is capped by `max_backoff`, and the actual wait is drawn anywhere between zero and that ceiling — so emitters that failed at the same moment do not come back at the same moment. `RetryPolicy::delays` computes that series from the draws it is handed, which is what makes the schedule testable without waiting for it.

## Verify a webhook signature

```rust example=verify
use hook0_client::verify_webhook_signature;
use std::time::Duration;

fn accept(
    signature: &str,
    body: &[u8],
    headers: &[(&str, &str)],
    subscription_secret: &str,
) -> bool {
    verify_webhook_signature(
        signature,
        body,
        headers,
        subscription_secret,
        Duration::from_secs(300),
    )
    .is_ok()
}
```

It answers `Ok(())` when the delivery is genuine and a `Hook0ClientError` for every reason it is not. Pass the raw body: a body that has been parsed and re-serialised no longer hashes to what was signed.

`headers` is a slice of pairs, generic over anything that is `AsRef<[u8]>` on both sides, so a `Vec<(&str, &str)>`, an `actix_web::HeaderMap` collected into pairs and a list read off a socket all fit without being converted first.

The clock window is bilateral: a delivery dated too far ahead is refused exactly like one dated too far behind, since a window that only looked backwards is one a sender widens by dating its own delivery ahead. A header the signature covers but the request did not carry is refused before any code is computed.

`verify_webhook_signature_with_current_time` takes the same arguments followed by a `DateTime<Utc>`, for holding a signature against a moment you choose.

Refusals are variants rather than one opaque error, so a handler can tell a forgery from a clock:

```rust example=matching
match error {
    Hook0ClientError::InvalidSignature => {
        // the body or a covered header changed in flight, or the secret is wrong
    }
    Hook0ClientError::ExpiredWebhook { signed_at, tolerance, current_time } => {
        // too old, dated too far ahead, or a clock is off
        eprintln!("signed at {signed_at}, now {current_time}, tolerance {tolerance}");
    }
    Hook0ClientError::MissingHeader(name) => {
        // a header the signature covers did not arrive
        eprintln!("{name} was signed but not delivered");
    }
    Hook0ClientError::SignatureHeaderParsing(header) => {
        // the header is not a Hook0 signature
        eprintln!("unreadable: {header}");
    }
    other => eprintln!("refused: {other}"),
}
```

### Actix Web

```rust example=actix
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, web};
use hook0_client::verify_webhook_signature;
use std::time::Duration;

async fn handle_webhook(
    subscription_secret: web::Data<String>,
    request: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    let Some(signature) = request.headers().get("X-Hook0-Signature") else {
        return HttpResponse::BadRequest().finish();
    };
    let Ok(signature) = signature.to_str() else {
        return HttpResponse::BadRequest().finish();
    };

    let delivered: Vec<(&str, &[u8])> = request
        .headers()
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_bytes()))
        .collect();

    if verify_webhook_signature(
        signature,
        &body,
        &delivered,
        &subscription_secret,
        Duration::from_secs(300),
    )
    .is_err()
    {
        return HttpResponse::BadRequest().finish();
    }

    // act on the delivery
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscription_secret =
        std::env::var("SUBSCRIPTION_SECRET").expect("SUBSCRIPTION_SECRET must be set");

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

`web::Bytes` is the body as it arrived. Extracting into a type that deserialises it gives you something that no longer hashes to what was signed.

## Upsert event types

An event whose type the application does not declare is refused. `upsert_event_types` creates the ones that are missing and returns only those it created:

```rust example=upsert
let created = client
    .upsert_event_types(&["billing.invoice.paid", "billing.invoice.voided"])
    .await?;
```

An event type is written `service.resource_type.verb`, and one that is not is refused as `InvalidEventType` before anything is sent. Unlike the other clients, this one exposes no type for parsing an event type on its own: the check happens inside `upsert_event_types` and nowhere a caller can reach.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group under `hook0_client::generated` — `ApplicationsApi`, `SubscriptionsApi`, `EventsApi`, `RequestAttemptsApi` and nine more, one per entity, one method per operation.

They live under a module rather than at the crate root because the API document declares its own `Event` and `EventType`, which are the API's resources and not the `Event` an emitter fills in.

The groups declare the seam they issue requests through and implement none, so nothing in them carries a socket. Nine of the eleven clients hand you one anyway, built from the client you already have; this one and the TypeScript one do not. `Hook0Client` reaches the API for sending events, but it does not implement `Transport` and exposes nothing that does, so a caller of the generated groups writes one:

```rust example=transport
use hook0_client::generated::Transport;
use reqwest::{Client, Method, Url};

struct Reqwest {
    base: Url,
    token: String,
    http: Client,
}

impl Transport for Reqwest {
    type Error = reqwest::Error;

    fn request(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, String)],
        body: Option<Vec<u8>>,
    ) -> impl std::future::Future<Output = Result<(u16, Vec<u8>), Self::Error>> + Send {
        // The path arrives written whole, escaping included, so it is carried as it was built
        // rather than parsed apart and put back together.
        let target = format!("{}{path}", self.base.as_str().trim_end_matches('/'));
        let method = Method::from_bytes(method.as_bytes()).unwrap_or(Method::GET);
        let query: Vec<(String, String)> = query
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        let http = self.http.clone();
        let token = self.token.clone();

        async move {
            let mut issued = http
                .request(method, target)
                .bearer_auth(token)
                .header("Accept", "application/json")
                .query(&query);
            if let Some(body) = body {
                issued = issued.header("Content-Type", "application/json").body(body);
            }

            let answered = issued.send().await?;
            let status = answered.status().as_u16();
            let payload = answered.bytes().await?;
            Ok((status, payload.to_vec()))
        }
    }
}
```

That is the whole seam: one method, a status and some bytes. A test satisfies it without opening a socket, which is what the trait is for.

With one in hand, every group is one line:

```rust example=api
use hook0_client::generated::{ApplicationsApi, ProblemId, RequestError};

let applications = ApplicationsApi::new(transport);

match applications.get(application_id).await {
    Ok(application) => println!("{}", application.name),
    Err(RequestError::Api(failure)) if failure.kind == Some(ProblemId::NotFound) => {
        // the API named a problem, and this is which one
    }
    Err(other) => return Err(other.into()),
}
```

Every failure the API can report is one `ProblemError` carrying a `kind`, not a type of its own — the closed list of identifiers is the `ProblemId` enum, so a `match` over it is checked and an identifier the crate has never heard of fails to deserialise rather than arriving as a string nobody looks at. Beside `kind` sits the whole RFC 9457 `Problem` the API answered, when it answered one this crate can read.

`RequestError` is the four ways a call can end other than with what it asked for: `Transport` when the request never got an answer, `Api` when the API described a failure, `Unreadable` when the answer was not the shape the document declares, and `Unwritable` when the request body could not be written.

## Errors

`Hook0ClientError` is one enum across both halves of the crate, and the producer and consumer variants are behind their own features.

| Variant | Returned when |
|---------|---------------|
| `EventSending` | A send failed or ran out of attempts. Carries `event_id`, the last `error`, and a `body` that says why the client gave up |
| `InvalidEventType` | An event type is not `service.resource_type.verb` |
| `GetAvailableEventTypes`, `CreatingEventType` | Listing or creating event types failed |
| `InvalidSignature`, `ExpiredWebhook`, `MissingHeader` | A delivery was refused |
| `SignatureHeaderParsing`, `TimestampParsing`, `V0SignatureParsing`, `V1SignatureParsing`, `HeaderNameParsing` | The signature header, or something it names, could not be read |
| `InvalidHeaderName`, `InvalidHeaderValue`, `InvalidTolerance` | What the caller handed in could not be used |
| `AuthHeader`, `ReqwestClient`, `Url` | The client could not be built at all |

`event_id` on `EventSending` is always the ID the request went out under, whether the caller chose it or the client generated it, so a failed send is still traceable in the instance's logs:

```rust example=errors
match client.send_event(&event).await {
    Ok(event_id) => println!("ingested as {event_id}"),
    Err(Hook0ClientError::EventSending { event_id, error, body }) => {
        eprintln!("event {event_id:?} not sent: {error} ({})", body.unwrap_or_default());
    }
    Err(other) => eprintln!("refused before sending: {other}"),
}
```

`Hook0ClientError::log_and_return` writes the error through `tracing` and hands it back, for a `?` chain that should leave a trace on the way out.

## Sharing one client

`Hook0Client` holds a `reqwest::Client`, which is a connection pool. Build one for the life of the process and share it rather than building one per send:

```rust example=share
let client = Arc::new(Hook0Client::new(api_url, application_id, &token)?);

let sending = client.clone();
tokio::spawn(async move {
    let event = /* build the event to send */ todo!();
    let _ = sending.send_event(&event).await;
});
```

## Links

- **Crate**: [hook0-client on crates.io](https://crates.io/crates/hook0-client)
- **Documentation**: [docs.rs/hook0-client](https://docs.rs/hook0-client)
- **Source**: [clients/rust](https://gitlab.com/hook0/hook0/-/tree/master/clients/rust)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
