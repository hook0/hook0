//! Feeds arbitrary bytes to the snapshot parser: it may refuse them, it may never panic, and
//! whatever model it builds out of them holds its invariants and emits without panicking.
//!
//! The committed corpus under `tests/__fuzz__/snapshot/corpus` is replayed on every run, then a
//! bounded number of random inputs is drawn. Longer campaigns go through `cargo bolero`.

use std::time::Duration;

use bolero::check;
use hook0_sdkgen::{ApiModel, EntityModel, Limits, MCP_TAG, SDK_TAG, Snapshot, mcp};

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

            for tag in [SDK_TAG, MCP_TAG] {
                let Ok(snapshot) = Snapshot::from_bytes(input, tag, &limits) else {
                    continue;
                };
                // Reading the types the document declares may refuse it, and owes the caller a
                // reason rather than a panic whatever it was handed.
                let read = ApiModel::from_snapshot(&snapshot, &limits);
                if let Ok(api) = read.as_ref() {
                    common::assert_model_invariants(&snapshot, &api.entities, &limits);
                    assert!(
                        !api.errors.catalogue.is_empty(),
                        "a discovered catalogue lists no problem"
                    );
                }
                assert_eq!(
                    read,
                    ApiModel::from_snapshot(&snapshot, &limits),
                    "two reads of the same snapshot differ"
                );

                let Ok(model) = EntityModel::from_snapshot(&snapshot, &limits) else {
                    continue;
                };

                common::assert_model_invariants(&snapshot, &model, &limits);

                // Emission may refuse the model it is handed, but it owes the caller a reason
                // rather than a panic, and the same model always writes the same bytes.
                let emitted = mcp::tool_definitions(&model);
                assert_eq!(
                    emitted,
                    mcp::tool_definitions(&model),
                    "two emissions of the same model differ"
                );
            }
        });
}
