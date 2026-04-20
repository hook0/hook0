//! Per-attempt retry delay computation.
//!
//! Operator-visible effect: delays returned here land in `request_attempt.delay_until`
//! and become the wait between the current failure and the next auto-retry.
//!
//! - `ScheduleConfig` mirrors `webhook.retry_schedule` (3 strategies, nullable fields)
//! - `compute_delay_from_schedule` returns `None` when the retry budget is exhausted
//! - Jitter adds positive random smear; prevents thundering-herd when many subs retry together
//! - Per-retry delay capped at 7 days to guarantee DoS resistance and i64 safety

use std::time::Duration;

use rand::Rng;

/// Hard cap on a single retry delay (7 days). Matches API validator.
pub const MAX_RETRY_DELAY_SECS: u64 = 7 * 24 * 3600;

/// Upper bound for jitter in milliseconds (keeps smear subtle).
const JITTER_MAX_MS: u64 = 1_000;

/// Retry schedule fields as persisted; strategy columns are mutually exclusive.
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    pub strategy: String,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Return the delay before retry `retry_count+1`, or `None` if the budget is spent.
/// `retry_count` is zero-indexed on the attempt that just failed.
pub fn compute_delay_from_schedule(
    schedule: &ScheduleConfig,
    retry_count: i16,
) -> Option<Duration> {
    let max_retries = schedule.max_retries;
    if retry_count as i32 >= max_retries {
        return None;
    }

    let raw_secs = match schedule.strategy.as_str() {
        "exponential_increasing" => {
            let base = schedule.increasing_base_delay?;
            let factor = schedule.increasing_wait_factor?;
            let base_f = f64::from(base);
            let secs = base_f * factor.powi(i32::from(retry_count));
            secs.max(0.0) as u64
        }
        "linear" => {
            let delay = schedule.linear_delay?;
            if delay <= 0 {
                return None;
            }
            delay as u64
        }
        "custom" => {
            let intervals = schedule.custom_intervals.as_ref()?;
            let idx = retry_count as usize;
            let v = intervals.get(idx).copied()?;
            if v <= 0 {
                return None;
            }
            v as u64
        }
        _ => return None,
    };

    let capped = raw_secs.min(MAX_RETRY_DELAY_SECS);
    Some(with_jitter(Duration::from_secs(capped)))
}

/// Add small positive jitter; spreads retries that would otherwise align.
fn with_jitter(base: Duration) -> Duration {
    let jitter_ms = rand::rng().random_range(0..=JITTER_MAX_MS);
    base + Duration::from_millis(jitter_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(base: i32, factor: f64, max: i32) -> ScheduleConfig {
        ScheduleConfig {
            strategy: "exponential_increasing".to_owned(),
            max_retries: max,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(base),
            increasing_wait_factor: Some(factor),
        }
    }

    fn linear(delay: i32, max: i32) -> ScheduleConfig {
        ScheduleConfig {
            strategy: "linear".to_owned(),
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
            strategy: "custom".to_owned(),
            max_retries: len,
            custom_intervals: Some(intervals),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    #[test]
    fn exponential_grows_by_factor() {
        let s = exp(60, 2.0, 5);
        let d0 = compute_delay_from_schedule(&s, 0).unwrap();
        let d1 = compute_delay_from_schedule(&s, 1).unwrap();
        assert!(d1 >= d0);
    }

    #[test]
    fn exponential_capped_at_7d_per_retry() {
        let s = exp(1, 10.0, 10);
        let d = compute_delay_from_schedule(&s, 9).unwrap();
        assert!(d.as_secs() <= MAX_RETRY_DELAY_SECS + JITTER_MAX_MS);
    }

    #[test]
    fn exponential_over_max_retries_is_none() {
        let s = exp(60, 2.0, 3);
        assert!(compute_delay_from_schedule(&s, 3).is_none());
        assert!(compute_delay_from_schedule(&s, 10).is_none());
    }

    #[test]
    fn linear_returns_same_delay_each_retry() {
        let s = linear(60, 5);
        let d0 = compute_delay_from_schedule(&s, 0).unwrap().as_secs();
        let d1 = compute_delay_from_schedule(&s, 1).unwrap().as_secs();
        assert!(d0.abs_diff(d1) <= 1);
    }

    #[test]
    fn linear_rejects_non_positive_delay() {
        let s = linear(0, 5);
        assert!(compute_delay_from_schedule(&s, 0).is_none());
    }

    #[test]
    fn custom_indexes_into_intervals() {
        let s = custom(vec![5, 10, 20]);
        assert!(compute_delay_from_schedule(&s, 0).unwrap().as_secs() >= 5);
        assert!(compute_delay_from_schedule(&s, 2).unwrap().as_secs() >= 20);
        assert!(compute_delay_from_schedule(&s, 3).is_none());
    }

    #[test]
    fn jitter_stays_within_bound() {
        let s = linear(1, 5);
        for _ in 0..100 {
            let d = compute_delay_from_schedule(&s, 0).unwrap();
            let ms = d.as_millis() as u64;
            assert!((1_000..=1_000 + JITTER_MAX_MS).contains(&ms), "ms={ms}");
        }
    }

    #[test]
    fn unknown_strategy_yields_none() {
        let mut s = linear(1, 5);
        s.strategy = "mystery".to_owned();
        assert!(compute_delay_from_schedule(&s, 0).is_none());
    }
}
