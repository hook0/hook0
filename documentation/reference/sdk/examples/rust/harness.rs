// The rest of the file, for every Rust example of the SDK reference.
//
// A snippet on a page is written for a reader: it leaves out the imports, it assumes a client or
// an event is already built, and it names a token or an application ID without saying where they
// came from. Each region below is the file that snippet would live in, with a hole where it goes.
// The page points at one by name on the fence, so what a snippet is standing on is one word away
// from the snippet itself.
//
// Every region becomes its own file under `src/bin`, which is why every region that is not
// already a complete program supplies its own `fn main() {}`: a Rust binary refuses to build
// without one, and none of these functions is ever meant to run.

// HARNESS send
EXAMPLE
// END HARNESS

// HARNESS bounds
use reqwest::Url;
use uuid::Uuid;

fn main() {}

// What this configuration was handed before it started.
fn configure_bounds(
    api_url: Url,
    application_id: Uuid,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS verify
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS configure
fn main() {}

fn configured() -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS upsert
use hook0_client::Hook0Client;

fn main() {}

// The client this page assumes is already built and reachable.
async fn declare(client: &Hook0Client) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS actix
EXAMPLE
// END HARNESS

// HARNESS error_handling
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS consumer_errors
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS type_safety
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS unit_test
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS integration_test
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS arc_share
use reqwest::Url;
use uuid::Uuid;

fn main() {}

// What this client was handed before it started.
fn share(api_url: Url, application_id: Uuid, token: String) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS parallel
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS reuse_client
use hook0_client::Hook0Client;
use reqwest::Url;
use std::sync::Arc;
use uuid::Uuid;

fn main() {}

// What this page was handed before it started.
fn reuse(api_url: Url, application_id: Uuid, token: String) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS strong_types
fn main() {}

fn strong_types() -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS handle_errors
use hook0_client::{Event, Hook0Client};

fn main() {}

// The client and the event this page assumes already exist.
async fn handle(client: &Hook0Client, event: Event<'_>) {
    EXAMPLE
}
// END HARNESS

// HARNESS env_vars
fn main() {
    EXAMPLE
}
// END HARNESS

// HARNESS custom_event_id
fn main() {
    EXAMPLE
}
// END HARNESS

// HARNESS payload_mismatch
use hook0_client::Event;
use std::borrow::Cow;

fn main() {
    EXAMPLE
}
// END HARNESS

// HARNESS async_runtime
use hook0_client::Hook0Client;

EXAMPLE
// END HARNESS
