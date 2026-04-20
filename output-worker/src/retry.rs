//! Per-attempt retry delay computation.
//!
//! Operator-visible effect: delays returned here land in `request_attempt.delay_until`.
//!
//! - `Strategy` enum mirrors the API enum. Unknown DB values fail loud.
//! - `ScheduleConfig` mirrors `webhook.retry_schedule` (3 strategies, nullable fields)
//! - `compute_delay_from_schedule` returns `None` when the retry budget is exhausted
//! - `with_jitter` applies multiplicative jitter to spread retries after recovery
//! - Shared between custom path and built-in fallback for one jitter policy
//! - Per-retry delay capped at 7 days for DoS resistance and i64 safety

use std::time::Duration;

use rand::Rng;
use tracing::warn;

/// Hard cap on a single retry delay (7 days). Matches API validator.
pub const MAX_RETRY_DELAY_SECS: u64 = 7 * 24 * 3600;

/// Retry pacing family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    ExponentialIncreasing,
    Linear,
    Custom,
}

impl Strategy {
    /// Parse the persisted string; None on unknown value (caller must log).
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "exponential_increasing" => Some(Self::ExponentialIncreasing),
            "linear" => Some(Self::Linear),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Retry schedule fields as persisted; strategy-specific columns are mutually exclusive.
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub strategy: Strategy,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Delay before retry `retry_count+1`, or `None` if the budget is spent.
/// `retry_count` = zero-indexed count of attempts already failed.
/// `jitter_factor` in `[0.0, 1.0]`: 0.0 disables jitter, 0.2 = ±20% random spread.
pub fn compute_delay_from_schedule(
    schedule: &ScheduleConfig,
    retry_count: i16,
    jitter_factor: f64,
) -> Option<Duration> {
    // Defensive: negative retry_count would wrap to a huge usize in custom indexing.
    if retry_count < 0 {
        warn!(
            retry_count,
            "negative retry_count; refusing to schedule retry"
        );
        return None;
    }
    if i32::from(retry_count) >= schedule.max_retries {
        return None;
    }

    let cap_f = MAX_RETRY_DELAY_SECS as f64;
    let raw_secs: u64 = match schedule.strategy {
        Strategy::ExponentialIncreasing => {
            // `?` on null columns is defense-in-depth: DB CHECK enforces non-null.
            let base = schedule.increasing_base_delay?;
            let factor = schedule.increasing_wait_factor?;
            let secs = f64::from(base) * factor.powi(i32::from(retry_count));
            // Clamp before casting so NaN / +Inf / >u64::MAX saturate cleanly.
            secs.clamp(0.0, cap_f) as u64
        }
        Strategy::Linear => {
            let delay = schedule.linear_delay?;
            if delay <= 0 {
                return None;
            }
            // Bounded by DB CHECK 1..=604800.
            delay as u64
        }
        Strategy::Custom => {
            let intervals = schedule.custom_intervals.as_ref()?;
            let v = intervals.get(retry_count as usize).copied()?;
            if v <= 0 {
                return None;
            }
            // Bounded by DB CHECK 1..=604800.
            v as u64
        }
    };

    let capped = raw_secs.min(MAX_RETRY_DELAY_SECS);
    Some(with_jitter(Duration::from_secs(capped), jitter_factor))
}

/// Multiplicative jitter: `delay * random([1.0, 1.0 + factor))`.
/// Factor ≤ 0 or NaN disables jitter. Output clamped at MAX_RETRY_DELAY_SECS.
pub fn with_jitter(delay: Duration, jitter_factor: f64) -> Duration {
    if jitter_factor <= 0.0 || jitter_factor.is_nan() {
        return delay;
    }
    let random: f64 = rand::rng().random_range(0.0..1.0);
    let multiplier = 1.0 + random * jitter_factor;
    let jittered = delay.mul_f64(multiplier);
    jittered.min(Duration::from_secs(MAX_RETRY_DELAY_SECS))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(base: i32, factor: f64, max: i32) -> ScheduleConfig {
        ScheduleConfig {
            strategy: Strategy::ExponentialIncreasing,
            max_retries: max,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(base),
            increasing_wait_factor: Some(factor),
        }
    }

    fn linear(delay: i32, max: i32) -> ScheduleConfig {
        ScheduleConfig {
            strategy: Strategy::Linear,
            max_retries: max,
            custom_intervals: None,
            linear_delay: Some(delay),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    fn custom(intervals: Vec<i32>) -> ScheduleConfig {
        let len = intervals.len() as i32;
        ScheduleConfig {
            strategy: Strategy::Custom,
            max_retries: len,
            custom_intervals: Some(intervals),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    #[test]
    fn exponential_capped_at_7d_per_retry() {
        let s = exp(1, 10.0, 10);
        let d = compute_delay_from_schedule(&s, 9, 0.0).unwrap();
        assert!(d.as_secs() <= MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn exponential_over_max_retries_is_none() {
        let s = exp(60, 2.0, 3);
        assert!(compute_delay_from_schedule(&s, 3, 0.0).is_none());
        assert!(compute_delay_from_schedule(&s, 10, 0.0).is_none());
    }

    #[test]
    fn linear_rejects_non_positive_delay() {
        let s = linear(0, 5);
        assert!(compute_delay_from_schedule(&s, 0, 0.0).is_none());
    }

    #[test]
    fn custom_indexes_into_intervals() {
        let s = custom(vec![5, 10, 20]);
        assert!(compute_delay_from_schedule(&s, 0, 0.0).unwrap().as_secs() == 5);
        assert!(compute_delay_from_schedule(&s, 2, 0.0).unwrap().as_secs() == 20);
        assert!(compute_delay_from_schedule(&s, 3, 0.0).is_none());
    }

    #[test]
    fn jitter_multiplier_within_factor_bound() {
        let base = Duration::from_secs(100);
        for _ in 0..100 {
            let d = with_jitter(base, 0.2);
            let secs = d.as_secs_f64();
            assert!((100.0..120.0).contains(&secs), "out of range: {secs}");
        }
    }

    #[test]
    fn jitter_zero_factor_is_noop() {
        let base = Duration::from_secs(42);
        assert_eq!(with_jitter(base, 0.0), base);
    }

    #[test]
    fn negative_retry_count_refuses() {
        let s = linear(60, 5);
        assert!(compute_delay_from_schedule(&s, -1, 0.0).is_none());
    }

    #[test]
    fn exponential_clamps_positive_infinity() {
        // Factor * huge base overflows to +Inf; clamp protects the cast to u64.
        let s = exp(i32::MAX, 20.0, 5);
        let d = compute_delay_from_schedule(&s, 4, 0.0).unwrap();
        assert!(d.as_secs() <= MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn exponential_clamps_nan_factor() {
        // retry_count >= 1 so factor.powi(…) is NaN (NaN^0 = 1 by IEEE, so skip 0).
        let s = exp(60, f64::NAN, 5);
        let d = compute_delay_from_schedule(&s, 1, 0.0).unwrap();
        assert_eq!(d.as_secs(), 0);
    }
}
