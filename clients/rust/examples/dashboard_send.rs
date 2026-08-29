//! What the dashboard shows under "Send an event", for Rust.
//!
//! This file exists so that the snippet is compiled against the real client. A renamed method, a
//! changed signature or a dropped field turns `client.rust.check` red on the day it happens, which
//! is the whole reason the snippet lives here rather than in the dashboard: one written by hand
//! over there is backed by nothing and drifts in silence.
//!
//! Two pairs of markers say how it is read. `hook0:snippet` delimits what a reader is shown, so
//! that anything this file needs only in order to compile stays out of it. `hook0:label` delimits
//! the one rendering of a label, which the dashboard repeats once per label the form carries and
//! joins with the separator its manifest declares — the region carries no trailing separator of its
//! own, and sits inside its container, so no label at all leaves a valid empty one.
//!
//! The `__HOOK0_*__` words are string literals, which is what lets a file full of them compile.
//! They never resolve to anything: this example is built, never run.

// hook0:snippet:begin
use hook0_client::{Event, Hook0Client};
use std::borrow::Cow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parsed through the types the client's own signature names rather than through `Url::parse`
    // and `Uuid::parse_str`. Rust requires the crate a `use` line names to be a direct dependency,
    // and a reader who installed this client has one direct dependency: this client. Naming
    // `reqwest` and `uuid` here would ask them for two more.
    let client = Hook0Client::new(
        "__HOOK0_API_URL__".parse()?,
        "__HOOK0_APPLICATION_ID__".parse()?,
        "__HOOK0_TOKEN__",
    )?;

    let event_id = client
        .send_event(&Event {
            event_id: None,
            event_type: "__HOOK0_EVENT_TYPE__",
            payload: Cow::Borrowed("__HOOK0_PAYLOAD__"),
            payload_content_type: "application/json",
            metadata: None,
            occurred_at: None,
            labels: vec![
                // hook0:label:begin
                (
                    "__HOOK0_LABEL_KEY__".to_owned(),
                    "__HOOK0_LABEL_VALUE__".to_owned(),
                ), // hook0:label:end
            ],
        })
        .await?;

    println!("ingested as {event_id}");
    Ok(())
}
// hook0:snippet:end
