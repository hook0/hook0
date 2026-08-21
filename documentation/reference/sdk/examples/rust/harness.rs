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
//
// A region never brings a name the snippet dropped into it brings itself: two `use` lines naming
// the same item is a file rustc refuses, so a region used by a snippet that shows its own imports
// brings none of them.

// HARNESS send
EXAMPLE
// END HARNESS

// HARNESS event
use hook0_client::Event;
use std::borrow::Cow;
use uuid::Uuid;

fn main() {}

// The identifier a send may be given, when the caller has one of its own to give.
fn built(chosen: &Uuid) -> Event<'_> {
    EXAMPLE
}
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

// HARNESS matching
use hook0_client::Hook0ClientError;

fn main() {}

// What verification answered, when what it answered was not that the delivery is genuine.
fn refused(error: Hook0ClientError) {
    EXAMPLE
}
// END HARNESS

// HARNESS actix
EXAMPLE
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

// HARNESS transport
fn main() {}

EXAMPLE
// END HARNESS

// HARNESS api
fn main() {}

// The transport the section above writes, already built, and the application it is asked about.
async fn read<T: hook0_client::generated::Transport>(
    transport: T,
    application_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS

// HARNESS errors
use hook0_client::{Event, Hook0Client, Hook0ClientError};

fn main() {}

// The client and the event this page assumes already exist.
async fn attempt(client: &Hook0Client, event: Event<'_>) {
    EXAMPLE
}
// END HARNESS

// HARNESS share
use hook0_client::Hook0Client;
use reqwest::Url;
use std::sync::Arc;
use uuid::Uuid;

fn main() {}

// What this client was handed before it started.
fn share(
    api_url: Url,
    application_id: Uuid,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    EXAMPLE

    Ok(())
}
// END HARNESS
