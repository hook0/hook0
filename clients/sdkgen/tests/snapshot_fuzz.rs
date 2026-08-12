//! Feeds arbitrary bytes to the snapshot parser: it may refuse them, it may never panic, and
//! whatever model it builds out of them holds its invariants.
//!
//! The committed corpus under `tests/__fuzz__/snapshot/corpus` is replayed on every run, then a
//! bounded number of random inputs is drawn. Longer campaigns go through `cargo bolero`.

use std::time::Duration;

use bolero::check;
use hook0_sdkgen::{EntityModel, Limits, Snapshot};

mod common;

/// Random inputs drawn on top of the corpus when the suite runs under `cargo test`.
const SMOKE_ITERATIONS: usize = 4_000;

/// Longest random input drawn.
const SMOKE_INPUT_BYTES: usize = 8 * 1024;

/// Wall clock the smoke run may spend.
const SMOKE_BUDGET: Duration = Duration::from_secs(30);

#[test]
fn parsing_a_snapshot_never_panics_and_yields_a_sound_model() {
    check!(name = "snapshot")
        .with_iterations(SMOKE_ITERATIONS)
        .with_max_len(SMOKE_INPUT_BYTES)
        .with_test_time(SMOKE_BUDGET)
        .for_each(|input: &[u8]| {
            let limits = Limits::default();

            let Ok(snapshot) = Snapshot::from_bytes(input, &limits) else {
                return;
            };
            let Ok(model) = EntityModel::from_snapshot(&snapshot, &limits) else {
                return;
            };

            common::assert_model_invariants(&snapshot, &model, &limits);
        });
}
