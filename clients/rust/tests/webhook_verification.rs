#![cfg(feature = "consumer")]

//! What a consumer of a Hook0 webhook observes when it verifies one: whether the webhook is
//! accepted, and which error it is refused with.

use chrono::DateTime;
use hook0_client::{
    Hook0ClientError, verify_webhook_signature, verify_webhook_signature_with_current_time,
};
use std::time::Duration as StdDuration;

#[test]
fn verifying_valid_signature_v0() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    assert!(
        verify_webhook_signature::<&str, &str>(
            signature,
            payload,
            &[],
            subscription_secret,
            tolerance
        )
        .is_ok()
    );
}

#[test]
fn verifying_valid_signature_v0_with_current_time() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    assert!(
        verify_webhook_signature::<&str, &str>(
            signature,
            payload,
            &[],
            subscription_secret,
            tolerance
        )
        .is_ok()
    );
}

#[test]
fn verifying_expired_signature_v0() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs(300);

    assert!(
        verify_webhook_signature::<&str, &str>(
            signature,
            payload,
            &[],
            subscription_secret,
            tolerance
        )
        .is_err()
    );
}

#[test]
fn a_timestamp_later_than_the_tolerance_is_refused() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs(300);
    // The webhook claims to have been signed one hour after the moment it is received.
    let current_time = DateTime::from_timestamp(1636936200 - 3600, 0).unwrap();

    let result = verify_webhook_signature_with_current_time::<&str, &str>(
        signature,
        payload,
        &[],
        subscription_secret,
        tolerance,
        current_time,
    );

    assert!(
        matches!(&result, Err(Hook0ClientError::ExpiredWebhook { .. })),
        "expected an ExpiredWebhook error, got {result:?}"
    );
}

#[test]
fn a_timestamp_slightly_ahead_but_within_the_tolerance_is_accepted() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs(300);
    // A minute of clock drift between the producer and the consumer.
    let current_time = DateTime::from_timestamp(1636936200 - 60, 0).unwrap();

    assert!(
        verify_webhook_signature_with_current_time::<&str, &str>(
            signature,
            payload,
            &[],
            subscription_secret,
            tolerance,
            current_time,
        )
        .is_ok()
    );
}

#[test]
fn a_timestamp_earlier_than_the_tolerance_is_refused() {
    let signature =
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs(300);
    let current_time = DateTime::from_timestamp(1636936200 + 3600, 0).unwrap();

    let result = verify_webhook_signature_with_current_time::<&str, &str>(
        signature,
        payload,
        &[],
        subscription_secret,
        tolerance,
        current_time,
    );

    assert!(
        matches!(&result, Err(Hook0ClientError::ExpiredWebhook { .. })),
        "expected an ExpiredWebhook error, got {result:?}"
    );
}

#[test]
fn a_header_named_in_the_signature_but_absent_from_the_request_is_reported_as_missing() {
    let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
    let payload = "hello !".as_bytes();
    let header_values = [("x-test", "val1")];
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    let result = verify_webhook_signature::<&str, &str>(
        signature,
        payload,
        &header_values,
        subscription_secret,
        tolerance,
    );

    assert!(
        matches!(&result, Err(Hook0ClientError::MissingHeader(name)) if name.as_str() == "x-test2"),
        "expected a MissingHeader error naming x-test2, got {result:?}"
    );
}

#[test]
fn a_header_named_in_the_signature_is_required_even_when_only_v0_is_signed() {
    let signature =
        "t=1636936200,h=x-test,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98";
    let payload = "hello !".as_bytes();
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    let result = verify_webhook_signature::<&str, &str>(
        signature,
        payload,
        &[],
        subscription_secret,
        tolerance,
    );

    assert!(
        matches!(&result, Err(Hook0ClientError::MissingHeader(name)) if name.as_str() == "x-test"),
        "expected a MissingHeader error naming x-test, got {result:?}"
    );
}

#[test]
fn verifying_valid_signature_v1() {
    let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
    let payload = "hello !".as_bytes();
    let header_values = [("x-test", "val1"), ("x-test2", "val2")];
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    assert!(
        verify_webhook_signature::<&str, &str>(
            signature,
            payload,
            &header_values,
            subscription_secret,
            tolerance
        )
        .is_ok()
    );
}

#[test]
fn verifying_valid_signature_v1_with_current_time() {
    let signature = "t=1636936200,h=x-test x-test2,v1=493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";
    let payload = "hello !".as_bytes();
    let header_values = [("x-test", "val1"), ("x-test2", "val2")];
    let subscription_secret = "secret";
    let tolerance = StdDuration::from_secs((i64::MAX / 1000) as u64);

    assert!(
        verify_webhook_signature::<&str, &str>(
            signature,
            payload,
            &header_values,
            subscription_secret,
            tolerance
        )
        .is_ok()
    );
}

/// The payload, secret and moment the fixed vectors below were computed with.
const A_PAYLOAD: &[u8] = b"hello !";
const A_SECRET: &str = "secret";
const SIGNED_AT: i64 = 1636936200;

/// A `v1` signature, which covers the moment, the names and values of the headers it names, and
/// the payload.
const A_V1: &str = "493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a";

fn at_signing_time() -> DateTime<chrono::Utc> {
    DateTime::from_timestamp(SIGNED_AT, 0).expect("the moment the vectors were signed at")
}

#[test]
fn a_header_whose_name_is_not_one_is_refused_rather_than_looked_up() {
    let refused = verify_webhook_signature_with_current_time(
        &format!("t={SIGNED_AT},h=x-test x-test2,v1={A_V1}"),
        A_PAYLOAD,
        // A request cannot carry this, and a client that let it through would be signing over a
        // name no server can have sent.
        &[("x-test(2)", "val1"), ("x-test2", "val2")],
        A_SECRET,
        StdDuration::from_secs(300),
        at_signing_time(),
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::InvalidHeaderName { .. })),
        "a header whose name is not one was read as {refused:?}"
    );
}

#[test]
fn a_header_whose_value_is_not_text_is_refused_rather_than_signed_over() {
    let refused = verify_webhook_signature_with_current_time(
        &format!("t={SIGNED_AT},h=x-test x-test2,v1={A_V1}"),
        A_PAYLOAD,
        &[
            ("x-test", [0xff, 0xfe].as_slice()),
            ("x-test2", b"val2".as_slice()),
        ],
        A_SECRET,
        StdDuration::from_secs(300),
        at_signing_time(),
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::InvalidHeaderValue { .. })),
        "a header whose value is not text was read as {refused:?}"
    );
}

#[test]
fn a_code_that_is_not_the_one_the_secret_produces_is_refused() {
    let refused = verify_webhook_signature_with_current_time(
        &format!("t={SIGNED_AT},h=x-test x-test2,v1={A_V1}"),
        A_PAYLOAD,
        &[("x-test", "val1"), ("x-test2", "val2")],
        "another secret entirely",
        StdDuration::from_secs(300),
        at_signing_time(),
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::InvalidSignature)),
        "a code computed with another secret was read as {refused:?}"
    );
}

#[test]
fn a_tolerance_no_window_can_be_made_of_is_refused() {
    // A window is compared against a moment as a signed count of milliseconds, and a duration past
    // what that count reaches is not a window a webhook can be accepted inside of.
    let refused = verify_webhook_signature_with_current_time::<&str, &str>(
        "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98",
        A_PAYLOAD,
        &[],
        A_SECRET,
        StdDuration::MAX,
        at_signing_time(),
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::InvalidTolerance(_))),
        "a tolerance no window can be made of was read as {refused:?}"
    );
}

#[test]
fn a_moment_no_clock_reaches_is_refused_even_when_the_code_over_it_is_right() {
    // A signature may name any whole number of seconds, and the code below really is the one the
    // secret produces over this one. What decides the webhook is that no date can be made of the
    // moment, so there is no window it could be compared against.
    let refused = verify_webhook_signature_with_current_time::<&str, &str>(
        &format!(
            "t={},v0=76bf7fdcca5ceb6bddfa79dfc6cc2edaa4f735251473d898615fa6eaf54cf36c",
            i64::MAX
        ),
        A_PAYLOAD,
        &[],
        A_SECRET,
        StdDuration::from_secs(300),
        at_signing_time(),
    );

    assert!(
        matches!(refused, Err(Hook0ClientError::InvalidSignature)),
        "a moment no clock reaches was read as {refused:?}"
    );
}
