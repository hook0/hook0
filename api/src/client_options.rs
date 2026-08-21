//! What a Hook0 SDK states about itself on every request, and what this API records of it.
//!
//! The default root span already carries the user agent, so which SDK and which version is asking
//! is answerable. What it cannot answer is how that SDK is configured, and one setting shows
//! through in the traffic strongly enough to be worth reading: the retry policy. A client repeating
//! one send and a client stuck in a loop look identical from here, and a client left at a single
//! attempt reports a flaky API for failures one retry would have absorbed. Both become legible the
//! moment the policy is recorded beside the request it produced.
//!
//! The grammar is the one `clients/conformance/request.json` pins and every SDK emits:
//! `attempts=4,backoff=100,ceiling=2000,budget=5000`. Nothing here parses it. The value is recorded
//! whole, because a field that keeps the client's own words stays readable when the grammar grows,
//! and because parsing an untrusted header to record it would be work done for no reader.

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, http::header::HeaderName};
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, root_span};

/// Header a Hook0 SDK states its retry policy in.
pub const CLIENT_OPTIONS_HEADER: HeaderName = HeaderName::from_static("hook0-client-options");

/// Longest a recorded value may be, in bytes.
///
/// A header is whatever the far end chose to send. The SDKs cut what they compose to 256 bytes, but
/// nothing obliges a caller to be an SDK, so the ceiling that matters is the one held here rather
/// than the one the clients agreed among themselves. 256 leaves room for several times the parts
/// the grammar has today and still bounds what one request can put in a trace.
const MAX_RECORDED_BYTES: usize = 256;

/// The stated options, cut to what this API is willing to record.
///
/// The cut lands on a character boundary, so a value that was too long is still a string rather
/// than a fragment of one. Byte zero is always a boundary, so the search always ends.
fn recorded(stated: &str) -> &str {
    if stated.len() <= MAX_RECORDED_BYTES {
        return stated;
    }

    let mut end = MAX_RECORDED_BYTES;
    while !stated.is_char_boundary(end) {
        end -= 1;
    }
    &stated[..end]
}

/// The default span, plus what the client said about itself.
pub struct Hook0RootSpanBuilder;

impl RootSpanBuilder for Hook0RootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        // A header that is not valid UTF-8 is not something an SDK sent, and there is nothing to
        // record about it that a reader could act on.
        let stated = request
            .headers()
            .get(CLIENT_OPTIONS_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(recorded)
            .unwrap_or_default();

        root_span!(request, hook0_client_options = stated)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_within_the_ceiling_is_recorded_whole() {
        let stated = "attempts=4,backoff=100,ceiling=2000,budget=5000";

        assert_eq!(recorded(stated), stated);
    }

    #[test]
    fn a_value_above_the_ceiling_is_cut_to_it() {
        let stated = "a".repeat(MAX_RECORDED_BYTES + 100);

        assert_eq!(recorded(&stated).len(), MAX_RECORDED_BYTES);
    }

    #[test]
    fn a_cut_lands_on_a_character_boundary() {
        // The character has to be one whose width does not divide the ceiling, or the ceiling
        // lands on a boundary by accident and a naive cut passes. `€` is three bytes and the
        // ceiling is not a multiple of three, so byte 256 falls inside a character and cutting
        // there would produce something that is not a string.
        const WIDTH: usize = "€".len();
        assert_eq!(WIDTH, 3);
        assert_ne!(MAX_RECORDED_BYTES % WIDTH, 0);

        let stated = "€".repeat(MAX_RECORDED_BYTES);

        let kept = recorded(&stated);

        assert!(kept.len() <= MAX_RECORDED_BYTES);
        assert!(stated.starts_with(kept));
        assert_eq!(kept.chars().count(), MAX_RECORDED_BYTES / WIDTH);
    }

    #[test]
    fn a_value_exactly_at_the_ceiling_is_recorded_whole() {
        let stated = "a".repeat(MAX_RECORDED_BYTES);

        assert_eq!(recorded(&stated), stated);
    }
}
