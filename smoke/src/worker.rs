//! How long the output worker may take to deliver, worked out from what the worker itself says.
//!
//! The harness has to give up on a delivery at some point, and the number it gives up at is the
//! difference between a useful failure and a rumour. Chosen, it is indefensible in both directions:
//! too small and it accuses a working stack of being broken, too large and a real fault is
//! discovered minutes after it happened, which is exactly when nobody is still watching. So it is
//! not chosen. Both figures below are the output worker's own defaults, quoted from its source, and
//! `tests/cadence.rs` fails if either of them moves without this moving with it.

use std::time::Duration;

/// The longest the worker sleeps between looks for work once it has found none: `MAX_POLLING_SLEEP`
/// in `output-worker/src/pg.rs`. An event emitted a moment after a look began waits this out, and
/// no longer.
pub const IDLE_SLEEP: Duration = Duration::from_secs(10);

/// The longest one delivery attempt runs before the worker abandons it and records the failure: the
/// `--timeout` default in `output-worker/src/main.rs`, which covers the connect phase too.
pub const ATTEMPT: Duration = Duration::from_secs(15);

/// How many whole "sleep, then attempt" cycles the instance is given.
///
/// One is already the worst case for a stack that works: the sleep bounds how late the pick-up can
/// be, the attempt bounds how long the call can take, and neither can be exceeded without the
/// worker being at fault. The second is there because a cold container on a loaded machine may
/// spend its first cycle starting rather than working, and losing a whole run to that would teach
/// nobody anything.
const CYCLES: u64 = 2;

/// How long the instance is given to deliver the webhook every client then verifies.
pub const DELIVERS_WITHIN: Duration =
    Duration::from_secs((IDLE_SLEEP.as_secs() + ATTEMPT.as_secs()) * CYCLES);

/// The bound in words, so that a refusal says what it was waiting for and on what grounds — not
/// merely that it waited.
pub fn expectation() -> String {
    format!(
        "that bound is the worker's own cadence rather than a number picked here: at most {}s \
         asleep between looks for work plus at most {}s for one delivery attempt, {CYCLES} cycles \
         over, which is {}s",
        IDLE_SLEEP.as_secs(),
        ATTEMPT.as_secs(),
        DELIVERS_WITHIN.as_secs(),
    )
}
