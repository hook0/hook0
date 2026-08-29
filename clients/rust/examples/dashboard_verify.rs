//! What the dashboard shows under "Verify a webhook", for Rust.
//!
//! Sending is only half of what a reader has come to do, and it is the easier half. This is the one
//! the SDK reference calls the half most often got wrong by hand, so the dashboard shows it beside
//! the send rather than leaving it to be found later.
//!
//! The secret is read from the environment on purpose. The dashboard cannot know which subscription
//! a reader means — outside the onboarding it loads none, and an application may have several — so
//! it points at the subscription instead of guessing one, and no second secret is put on screen.
//!
//! Read the markers as in `dashboard_send.rs`: `hook0:snippet` is what is displayed, everything
//! outside it is what makes the file compile.

// hook0:snippet:begin
use hook0_client::verify_webhook_signature;
use std::time::Duration;

// Verify against the *raw* body: one that has been parsed and serialised again no longer hashes to
// what was signed. The tolerance is bilateral, so a delivery dated too far ahead is refused exactly
// like one dated too far behind.
fn accept(signature: &str, body: &[u8], headers: &[(&str, &str)]) -> bool {
    // The secret of the subscription being verified, which the dashboard links to rather than
    // prints: it cannot know which subscription a reader means, and an application may have several.
    // Reading configuration is the one place a panic beats a default, and a variable nobody
    // exported and one exported empty are the same defect: either hashes every genuine delivery to
    // the wrong code, so each one comes back refused as forged while saying nothing at all.
    let secret = std::env::var("HOOK0_SUBSCRIPTION_SECRET")
        .ok()
        .filter(|secret| !secret.is_empty())
        .expect("HOOK0_SUBSCRIPTION_SECRET is not set");
    verify_webhook_signature(signature, body, headers, &secret, Duration::from_secs(300)).is_ok()
}
// hook0:snippet:end

fn main() {
    // Nothing here is ever run: this file exists to be compiled against the real client.
    let accepted = accept("", b"", &[("x-hook0-signature", "")]);
    println!("accepted: {accepted}");
}
