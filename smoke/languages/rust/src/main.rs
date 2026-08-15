//! The Rust client against a Hook0 that is really running.
//!
//! Three things the loopback suite cannot ask: whether an application secret the API minted is
//! accepted, whether a second send under an identifier already ingested is reported as the
//! conflict it is, and whether a signature the output worker computed verifies. Everything else
//! about this client is settled by `clients/rust/tests`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use hook0_client::{Event, Hook0Client, verify_webhook_signature};

/// The conflict the API answers a duplicated ingestion with.
const ALREADY_INGESTED: &str = "EventAlreadyIngested";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), String> {
    let api_url = setting("HOOK0_API_URL")?;
    let application_id = setting("HOOK0_APPLICATION_ID")?;
    let token = setting("HOOK0_TOKEN")?;
    let event_type = setting("HOOK0_EVENT_TYPE")?;
    let delivery = PathBuf::from(setting("HOOK0_DELIVERY")?);

    let client = Hook0Client::new(
        api_url
            .parse()
            .map_err(|cause| format!("{api_url}: {cause}"))?,
        application_id
            .parse()
            .map_err(|cause| format!("{application_id}: {cause}"))?,
        &token,
    )
    .map_err(|cause| format!("building the client: {cause}"))?;

    let sent = client
        .send_event(&event(&event_type, None))
        .await
        .map_err(|cause| format!("the instance refused the first send: {cause}"))?;
    println!("ingested {sent}");

    let refused = client
        .send_event(&event(&event_type, Some(&sent)))
        .await
        .err()
        .ok_or("sending the same event twice was accepted twice")?;
    let said = format!("{refused}");
    if !said.contains(ALREADY_INGESTED) {
        return Err(format!(
            "the second send failed without naming {ALREADY_INGESTED}: {said}"
        ));
    }
    println!("the second send reported {ALREADY_INGESTED}");

    verify(&delivery)?;
    println!("the signature the instance produced verifies");
    Ok(())
}

/// The event both sends carry, under the identifier the caller names.
fn event<'a>(event_type: &'a str, event_id: Option<&'a uuid::Uuid>) -> Event<'a> {
    Event {
        event_id,
        event_type,
        payload: r#"{"from":"the rust smoke"}"#.into(),
        payload_content_type: "application/json",
        metadata: None,
        occurred_at: None,
        labels: vec![("language".to_owned(), "rust".to_owned())],
    }
}

/// Verifies what the output worker really delivered, with this client's own verification.
fn verify(delivery: &Path) -> Result<(), String> {
    let signature = read(delivery, "signature")?;
    let secret = read(delivery, "secret")?;
    let tolerance: u64 = read(delivery, "tolerance")?
        .trim()
        .parse()
        .map_err(|cause| format!("the tolerance is not a number of seconds: {cause}"))?;
    let body = fs::read(delivery.join("body"))
        .map_err(|cause| format!("reading the delivered body: {cause}"))?;

    let delivered = read(delivery, "headers")?;
    let headers: Vec<(&str, &str)> = delivered
        .lines()
        .filter_map(|line| line.split_once(": "))
        .collect();

    verify_webhook_signature(
        signature.trim(),
        &body,
        &headers,
        secret.trim(),
        Duration::from_secs(tolerance),
    )
    .map_err(|refused| format!("the signature the instance produced was refused: {refused}"))
}

/// One part of the delivery, as the harness wrote it down.
fn read(delivery: &Path, part: &str) -> Result<String, String> {
    fs::read_to_string(delivery.join(part))
        .map_err(|cause| format!("reading the delivered {part}: {cause}"))
}

/// A setting the harness passes, or a refusal naming it: a smoke that ran without one would report
/// a failure of the client for something the harness never handed it.
fn setting(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is not set"))
}
