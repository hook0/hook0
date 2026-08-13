#![cfg(feature = "consumer")]

//! How the value of the `X-Hook0-Signature` header is read.
//!
//! The parsed form of that header is internal to the crate; what a consumer observes is whether a
//! webhook carrying a given header verifies, and which error it is refused with. Each case below
//! therefore states a property of the header format through the verification entry point, over a
//! fixed payload and secret, and reads the outcome.

use chrono::{DateTime, Utc};
use hook0_client::{Hook0ClientError, verify_webhook_signature_with_current_time};
use std::time::Duration as StdDuration;

/// The payload every signature below was computed over.
const PAYLOAD: &[u8] = b"hello !";

/// The subscription secret every signature below was computed with.
const SECRET: &str = "secret";

/// The moment every signature below claims to have been signed at.
const SIGNED_AT: i64 = 1636936200;

/// A `v0` signature, which covers the timestamp and the payload.
const V0: &str = "1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";

/// A `v1` signature, which covers the timestamp, the names and values of [`SIGNED_HEADERS`], and
/// the payload.
const V1: &str = "493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";

/// The headers, in the order [`V1`] covers them.
const SIGNED_HEADERS: [(&str, &str); 2] = [("x-test", "val1"), ("x-test2", "val2")];

/// Every webhook here is received at the moment it claims to have been signed, so the tolerance
/// window never decides these cases; it has a file of its own.
const TOLERANCE: StdDuration = StdDuration::from_secs(300);

fn signing_time() -> DateTime<Utc> {
    DateTime::from_timestamp(SIGNED_AT, 0).expect("SIGNED_AT is a representable timestamp")
}

/// Verifies a webhook received at the very moment it claims to have been signed.
fn verify(signature: &str, headers: &[(&str, &str)]) -> Result<(), Hook0ClientError> {
    verify_webhook_signature_with_current_time(
        signature,
        PAYLOAD,
        headers,
        SECRET,
        TOLERANCE,
        signing_time(),
    )
}

#[test]
fn a_v0_only_header_carries_a_timestamp_and_a_signature_and_requires_no_header() {
    // The timestamp is part of the signed material, so a webhook that verifies proves it was read
    // as `SIGNED_AT`; the request carries no header and is not refused, so the `h` list was read as
    // empty; and the `v0` bytes were decoded, since they are what the computed MAC was compared to.
    let result = verify(&format!("t={SIGNED_AT},v0={V0}"), &[]);

    assert!(
        result.is_ok(),
        "expected the webhook to verify, got {result:?}"
    );
}

#[test]
fn a_timestamp_that_is_not_a_number_is_refused() {
    let result = verify("t=error,v0=def", &[]);

    assert!(
        matches!(&result, Err(Hook0ClientError::InvalidSignature)),
        "expected an InvalidSignature error, got {result:?}"
    );
}

#[test]
fn a_header_whose_fields_carry_no_signature_is_refused() {
    let result = verify("t=error,h=x-test,foo=bar", &[]);

    assert!(
        matches!(&result, Err(Hook0ClientError::InvalidSignature)),
        "expected an InvalidSignature error, got {result:?}"
    );
}

#[test]
fn a_signature_header_carrying_no_signature_at_all_is_refused() {
    for header in ["t=123", "t=123,h=x-test", "t=123,foo=bar"] {
        let result = verify(header, &[]);

        assert!(
            matches!(&result, Err(Hook0ClientError::InvalidSignature)),
            "`{header}` carries no v0 nor v1 field and must not verify, got {result:?}"
        );
    }
}

#[test]
fn a_signature_whose_hex_is_not_decodable_is_refused() {
    // `zz` is not hex at all, `abcz` and `abc` would decode to a shorter value than announced,
    // and `ab=cd` is what an extra assignator inside a value looks like.
    for header in [
        "t=123,v0=zz",
        "t=123,v0=abcz",
        "t=123,v0=abc",
        "t=123,v0=ab=cd",
        "t=123,h=x-test,v1=zz",
        "t=123,h=x-test,v1=abcz",
    ] {
        let result = verify(header, &[]);

        assert!(
            matches!(&result, Err(Hook0ClientError::InvalidSignature)),
            "`{header}` does not carry decodable hex and must not verify, got {result:?}"
        );
    }
}

#[test]
fn the_header_names_a_v1_signature_covers_are_read_in_order() {
    // The names are signed in the order the `h` field lists them, and so are the values they
    // resolve to: naming the very same two headers the other way round no longer matches.
    let listed_in_order = verify(
        &format!("t={SIGNED_AT},h=x-test x-test2,v1={V1}"),
        &SIGNED_HEADERS,
    );
    let listed_reversed = verify(
        &format!("t={SIGNED_AT},h=x-test2 x-test,v1={V1}"),
        &SIGNED_HEADERS,
    );

    assert!(
        listed_in_order.is_ok(),
        "expected the webhook to verify, got {listed_in_order:?}"
    );
    assert!(
        matches!(&listed_reversed, Err(Hook0ClientError::InvalidSignature)),
        "expected an InvalidSignature error, got {listed_reversed:?}"
    );
}

#[test]
fn a_header_carrying_both_v0_and_v1_is_verified_against_v1() {
    let both_valid = verify(
        &format!("t={SIGNED_AT},v0={V0},h=x-test x-test2,v1={V1}"),
        &SIGNED_HEADERS,
    );
    // Same header, with a `v1` that does not match: the valid `v0` beside it does not save it, so
    // `v1` is what was verified.
    let v1_does_not_match = verify(
        &format!("t={SIGNED_AT},v0={V0},h=x-test x-test2,v1={V0}"),
        &SIGNED_HEADERS,
    );

    assert!(
        both_valid.is_ok(),
        "expected the webhook to verify, got {both_valid:?}"
    );
    assert!(
        matches!(&v1_does_not_match, Err(Hook0ClientError::InvalidSignature)),
        "expected an InvalidSignature error, got {v1_does_not_match:?}"
    );
}

#[test]
fn a_v0_signature_computed_with_the_subscription_secret_verifies() {
    let result = verify(&format!("t={SIGNED_AT},v0={V0}"), &[]);

    assert!(
        result.is_ok(),
        "expected the webhook to verify, got {result:?}"
    );
}

#[test]
fn a_v0_signature_computed_with_another_secret_is_refused() {
    let result = verify_webhook_signature_with_current_time::<&str, &str>(
        &format!("t={SIGNED_AT},v0={V0}"),
        PAYLOAD,
        &[],
        "another secret",
        TOLERANCE,
        signing_time(),
    );

    assert!(
        matches!(&result, Err(Hook0ClientError::InvalidSignature)),
        "expected an InvalidSignature error, got {result:?}"
    );
}

#[test]
fn a_v0_header_padded_with_spaces_still_verifies() {
    // Field names and values are trimmed as the header is read, so a producer that pads them does
    // not produce a webhook the consumer refuses.
    let result = verify(&format!(" t = {SIGNED_AT} , v0 = {V0} "), &[]);

    assert!(
        result.is_ok(),
        "expected the webhook to verify, got {result:?}"
    );
}

#[test]
fn a_v1_signature_over_the_headers_it_names_verifies() {
    let result = verify(
        &format!("t={SIGNED_AT},h=x-test x-test2,v1={V1}"),
        &SIGNED_HEADERS,
    );

    assert!(
        result.is_ok(),
        "expected the webhook to verify, got {result:?}"
    );
}
