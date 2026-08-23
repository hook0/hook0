//! Holds the bound the harness waits under to the worker it is waiting for.
//!
//! `smoke/src/worker.rs` quotes two of the output worker's defaults and derives the delivery
//! deadline from them. A quote drifts silently: raise the worker's polling sleep to a minute and
//! nothing here stops compiling — the harness simply starts calling a healthy instance broken, and
//! the number in the refusal, which claims to be the worker's own cadence, becomes a lie about it.
//!
//! So both are read back out of the worker's source. What fails is not the run, it is this, with
//! the two numbers side by side.
//!
//! The same holds for the session lifetime `smoke/src/api.rs` quotes out of the API: shorten it
//! there and a run stops halfway down the language list, on a token that expired under it.

use std::fs;
use std::path::Path;
use std::time::Duration;

use hook0_smoke::worker;

/// The most bytes of a source file read looking for one number.
const MAX_SOURCE_BYTES: u64 = 512 * 1024;

fn source(path: &str) -> String {
    let path = Path::new(path);
    let length = fs::metadata(path)
        .unwrap_or_else(|cause| panic!("{} is not there: {cause}", path.display()))
        .len();
    assert!(
        length <= MAX_SOURCE_BYTES,
        "{} is {length} bytes, which is not a source file this test should be reading",
        path.display()
    );
    fs::read_to_string(path)
        .unwrap_or_else(|cause| panic!("{} unreadable: {cause}", path.display()))
}

/// The seconds in `Duration::from_secs(N)` on the line declaring `name`.
fn constant(source: &str, name: &str) -> Duration {
    let line = source
        .lines()
        .find(|line| line.contains(&format!("const {name}:")))
        .unwrap_or_else(|| panic!("the worker no longer declares {name}"));
    let seconds = line
        .split_once("from_secs(")
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(seconds, _)| seconds)
        .unwrap_or_else(|| panic!("{name} is no longer a whole number of seconds: {line}"));
    Duration::from_secs(whole_seconds(name, seconds))
}

/// The value of a `from_secs` argument written either as a number or as a product of them.
///
/// `api/src/iam.rs` writes its five minutes as `60 * 5`, which reads better than `300` and is the
/// form this has to accept for the quote of it to be checkable at all.
fn whole_seconds(name: &str, written: &str) -> u64 {
    written
        .split('*')
        .map(|factor| {
            factor
                .trim()
                .parse::<u64>()
                .unwrap_or_else(|cause| panic!("{name} reads `{written}`: {cause}"))
        })
        .product()
}

/// The `default_value` of the command line option backing a field declared exactly as `field`.
///
/// Searched upwards from the field rather than downwards from a name, because the worker declares a
/// dozen fields whose names end in `_timeout` and the one that matters is the one that is not
/// prefixed at all.
fn option_default(source: &str, field: &str) -> Duration {
    let lines: Vec<&str> = source.lines().collect();
    let declared = lines
        .iter()
        .position(|line| line.trim() == format!("{field}: Duration,"))
        .unwrap_or_else(|| panic!("the worker no longer takes a `{field}` option"));

    let attribute = lines[..declared]
        .iter()
        .rev()
        .take(4)
        .find(|line| line.contains("default_value = \""))
        .unwrap_or_else(|| panic!("`{field}` no longer declares a default"));
    let written = attribute
        .split_once("default_value = \"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(written, _)| written)
        .unwrap_or_else(|| panic!("`{field}` declares an unreadable default: {attribute}"));

    let seconds = written
        .strip_suffix('s')
        .unwrap_or_else(|| panic!("`{field}` defaults to `{written}`, which is not seconds"));
    Duration::from_secs(
        seconds
            .parse()
            .unwrap_or_else(|cause| panic!("`{field}` defaults to `{written}`: {cause}")),
    )
}

#[test]
fn the_sleep_the_harness_waits_out_is_the_sleep_the_worker_takes() {
    let declared = constant(&source("../output-worker/src/pg.rs"), "MAX_POLLING_SLEEP");

    assert_eq!(
        declared,
        worker::IDLE_SLEEP,
        "the output worker now sleeps up to {declared:?} between looks for work, and the harness \
         still derives its deadline from {:?}",
        worker::IDLE_SLEEP
    );
}

#[test]
fn the_attempt_the_harness_allows_for_is_the_attempt_the_worker_allows_itself() {
    let declared = option_default(&source("../output-worker/src/main.rs"), "timeout");

    assert_eq!(
        declared,
        worker::ATTEMPT,
        "the output worker now gives one delivery attempt {declared:?}, and the harness still \
         derives its deadline from {:?}",
        worker::ATTEMPT
    );
}

#[test]
fn the_deadline_is_longer_than_one_cycle_and_says_how_it_was_reached() {
    assert!(
        worker::DELIVERS_WITHIN > worker::IDLE_SLEEP + worker::ATTEMPT,
        "the deadline is under one whole cycle of the worker's own, so a working instance can miss it"
    );

    let said = worker::expectation();
    for number in [
        worker::DELIVERS_WITHIN.as_secs(),
        worker::IDLE_SLEEP.as_secs(),
        worker::ATTEMPT.as_secs(),
    ] {
        assert!(
            said.contains(&number.to_string()),
            "the refusal claims to derive its bound but does not show {number}s: {said}"
        );
    }
}

/// Holds the harness's quote of the session lifetime to the API that mints it.
///
/// A run provisions twelve languages one after another, and each one needs three calls only a user
/// session can make. The harness logs the account back in before the session in hand lapses, and it
/// can only know when that is by quoting the API. Shorten the session there and this fails here,
/// rather than a run failing partway down the list on a token that expired under it.
#[test]
fn the_harness_quotes_the_session_the_api_really_mints() {
    let declared = constant(&source("../api/src/iam.rs"), "USER_ACCESS_TOKEN_EXPIRATION");
    let quoted = constant(&source("src/api.rs"), "SESSION_LASTS");

    assert_eq!(
        declared, quoted,
        "the API mints a session lasting {declared:?} and the harness still assumes {quoted:?}"
    );
}

/// The margin has to leave the run somewhere to stand.
///
/// At or above the lifetime every call would re-mint, and at zero a call could be handed a session
/// that expires while it is in flight — which is the failure this margin exists to remove.
#[test]
fn the_session_margin_sits_inside_the_session() {
    let lasts = constant(&source("src/api.rs"), "SESSION_LASTS");
    let margin = constant(&source("src/api.rs"), "SESSION_MARGIN");

    assert!(
        margin > Duration::ZERO && margin < lasts,
        "a margin of {margin:?} against a session of {lasts:?} leaves the run nowhere to stand"
    );
}
