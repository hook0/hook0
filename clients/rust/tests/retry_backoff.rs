#![cfg(feature = "producer")]

//! Invariants a retry schedule holds, whatever policy it was built from and whatever randomness it
//! was jittered with.
//!
//! Rust's `Duration` is unsigned, so a delay being non-negative is a property of the type rather
//! than something a case can observe; what is checked here is everything a wrong schedule could
//! still get away with: a delay above its ceiling, a ceiling that shrinks as retries pile up, more
//! attempts than the policy allows, and delays that add up to more than the budget they share.

use hook0_client::{MAX_ATTEMPTS_CAP, RetryPolicy};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use std::time::Duration;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/retry_backoff.txt"
);

/// Retries a schedule is inspected over, well past what any policy is allowed to make.
const INSPECTED_RETRIES: u32 = 64;

/// Fixed so that a run explores the same number of cases on every machine.
fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 512,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

prop_compose! {
    /// A policy spanning what a caller can configure: no retry at all, more attempts than the cap
    /// allows, a backoff larger than its own ceiling, and budgets from nothing to a minute.
    fn a_policy()(
        max_attempts in 0u32..40,
        initial_backoff_ms in 0u64..5_000,
        max_backoff_ms in 0u64..10_000,
        max_total_delay_ms in 0u64..60_000,
    ) -> RetryPolicy {
        RetryPolicy {
            max_attempts,
            initial_backoff: Duration::from_millis(initial_backoff_ms),
            max_backoff: Duration::from_millis(max_backoff_ms),
            max_total_delay: Duration::from_millis(max_total_delay_ms),
        }
    }
}

prop_compose! {
    /// One random draw per retry, plus the values a broken source of randomness could hand over.
    fn some_draws()(
        draws in prop::collection::vec(
            prop_oneof![
                90 => 0.0f64..1.0,
                10 => prop_oneof![
                    Just(f64::NAN),
                    Just(f64::INFINITY),
                    Just(-1.0),
                    Just(2.0),
                ],
            ],
            0..48,
        ),
    ) -> Vec<f64> {
        draws
    }
}

proptest! {
    #![proptest_config(config())]

    /// However many attempts a policy asks for, it makes at least one and never more than the cap.
    #[test]
    fn a_policy_makes_between_one_attempt_and_the_cap(policy in a_policy()) {
        let attempts = policy.attempts();

        prop_assert!(
            (1..=MAX_ATTEMPTS_CAP).contains(&attempts),
            "{policy:?} makes {attempts} attempts"
        );
    }

    /// Ceilings grow with the retry count and stop at `max_backoff`, so the delay a retry is drawn
    /// from — uniform between zero and its ceiling, hence half of it on average — never gets
    /// smaller as retries pile up.
    #[test]
    fn ceilings_never_shrink_and_never_pass_the_maximum_backoff(policy in a_policy()) {
        for retry in 1..=INSPECTED_RETRIES {
            let ceiling = policy.backoff_ceiling(retry);
            let next = policy.backoff_ceiling(retry + 1);

            prop_assert!(
                ceiling <= policy.max_backoff,
                "the ceiling of retry {retry} of {policy:?} is {ceiling:?}"
            );
            prop_assert!(
                ceiling <= next,
                "the ceiling of {policy:?} shrinks from {ceiling:?} at retry {retry} to {next:?}"
            );
        }
    }

    /// A schedule fits in the attempts the policy allows, every delay stays under the ceiling of
    /// its own retry, and the whole schedule stays inside the budget the delays share.
    #[test]
    fn a_schedule_stays_inside_every_bound_of_its_policy(
        policy in a_policy(),
        draws in some_draws(),
    ) {
        let delays = policy.delays(&draws);

        prop_assert!(
            delays.len() < policy.attempts() as usize,
            "{policy:?} scheduled {} delays for {} attempts",
            delays.len(),
            policy.attempts()
        );

        let mut total = Duration::ZERO;
        for (index, delay) in delays.iter().enumerate() {
            let retry = index as u32 + 1;
            let ceiling = policy.backoff_ceiling(retry);

            prop_assert!(
                *delay <= ceiling,
                "retry {retry} of {policy:?} waits {delay:?}, above its {ceiling:?} ceiling"
            );
            total = total.saturating_add(*delay);
        }

        prop_assert!(
            total <= policy.max_total_delay,
            "{policy:?} waits {total:?} in total, above its {:?} budget",
            policy.max_total_delay
        );
    }

    /// A policy that does not retry schedules nothing to wait for.
    #[test]
    fn a_disabled_policy_schedules_no_delay(draws in some_draws()) {
        let policy = RetryPolicy::disabled();

        prop_assert_eq!(policy.attempts(), 1);
        prop_assert!(policy.delays(&draws).is_empty());
    }
}
