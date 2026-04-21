//! Per-attempt retry delay computation for the output worker.
//!
//! Operator-visible effect: the Duration returned here lands in `request_attempt.delay_until`.
//!
//! - `Strategy` mirrors the API enum (strum + sqlx::Type derives), fetched typed from the DB
//! - `SubscriptionRetrySchedule` is the `query_as!` target: nullable fields because the LEFT JOIN may not find a schedule
//! - `compute_next_retry` orchestrates: fetch row, decide scheduled vs. built-in, apply jitter
//! - `with_jitter` is shared between scheduled and built-in paths for one jitter policy
//! - Per-retry delay capped at 7 days for DoS resistance and i64 safety

use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use strum::{Display, EnumString};
use tracing::{error, warn};

use hook0_protobuf::RequestAttempt;

use crate::work::{Response, ResponseError};

/// Hard cap on a single retry delay (7 days). Matches API validator.
pub const MAX_RETRY_DELAY_SECS: u64 = 7 * 24 * 3600;

/// Same cap as [`MAX_RETRY_DELAY_SECS`], pre-computed as f64 for exponential math.
const MAX_RETRY_DELAY_SECS_F64: f64 = 7.0 * 24.0 * 3600.0;

/// Retry pacing family. Derives mirror the API enum so the DB round-trip is typed.
#[non_exhaustive]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Strategy {
    ExponentialIncreasing,
    Linear,
    Custom,
}

/// Row shape from the `subscription ⟕ retry_schedule` join.
/// All schedule-side fields are nullable because the LEFT JOIN may not find a match.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SubscriptionRetrySchedule {
    pub subscription_retry_schedule_id: Option<uuid::Uuid>,
    pub strategy: Option<Strategy>,
    pub max_retries: Option<i32>,
    pub custom_intervals_secs: Option<Vec<i32>>,
    pub linear_delay_secs: Option<i32>,
    pub increasing_base_delay_secs: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Decide the delay before the next retry attempt, or `None` to give up.
///
/// 1. Bail on unrecoverable signing errors.
/// 2. Load the subscription + its attached schedule (if any).
/// 3. Compute the scheduled delay when a schedule is present, else the built-in table.
/// 4. Apply multiplicative jitter.
pub async fn compute_next_retry(
    conn: &mut PgConnection,
    attempt: &RequestAttempt,
    response: &Response,
    max_retries: u8,
    jitter_factor: f64,
) -> Result<Option<Duration>, sqlx::Error> {
    if let Some(ResponseError::InvalidHeader) = response.response_error {
        let body = response_body_text(response);
        error!(
            request_attempt_id = %attempt.request_attempt_id,
            "Could not construct signature ({body}); giving up",
        );
        return Ok(None);
    }

    if let Some(ResponseError::InvalidTarget) = response.response_error {
        let body = response_body_text(response);
        warn!(
            request_attempt_id = %attempt.request_attempt_id,
            "Invalid target ({body}); continuing as normal",
        );
    }

    // Disabled / soft-deleted subscriptions short-circuit to None.
    // Prevents scheduling follow-ups on mid-flight state.
    let row = sqlx::query_as!(
        SubscriptionRetrySchedule,
        r#"
            select
                s.retry_schedule__id as "subscription_retry_schedule_id?",
                rs.strategy as "strategy?: Strategy",
                rs.max_retries as "max_retries?",
                rs.custom_intervals as custom_intervals_secs,
                rs.linear_delay as linear_delay_secs,
                rs.increasing_base_delay as increasing_base_delay_secs,
                rs.increasing_wait_factor
            from webhook.subscription as s
            inner join event.application as a on a.application__id = s.application__id
            left join webhook.retry_schedule as rs on rs.retry_schedule__id = s.retry_schedule__id
            where s.subscription__id = $1
                and s.deleted_at is null
                and s.is_enabled
                and a.deleted_at is null
        "#,
        attempt.subscription_id,
    )
    .fetch_optional(conn)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    // Subscription points to a schedule but the LEFT JOIN found no row. FK has
    // ON DELETE SET NULL so this should not happen; log loud and fall back.
    if row.subscription_retry_schedule_id.is_some() && row.strategy.is_none() {
        warn!(
            subscription_id = %attempt.subscription_id,
            schedule_id = ?row.subscription_retry_schedule_id,
            "Subscription references a retry_schedule that could not be joined; falling back to built-in backoff",
        );
    }

    let delay = compute_scheduled_retry_delay(&row, attempt.retry_count)
        .or_else(|| built_in_retry_delay(max_retries, attempt.retry_count));

    Ok(delay.map(|d| with_jitter(d, jitter_factor)))
}

/// Compute the delay from an attached retry schedule. Returns `None` when:
/// - no schedule attached
/// - retry_count exceeds the schedule's max_retries
/// - retry_count is negative
/// - required strategy fields are missing (defense-in-depth; DB CHECK already enforces)
pub fn compute_scheduled_retry_delay(
    row: &SubscriptionRetrySchedule,
    retry_count: i16,
) -> Option<Duration> {
    let strategy = row.strategy?;
    let max_retries = row.max_retries?;

    // Defensive: negative retry_count would wrap to a huge usize in custom indexing.
    if retry_count < 0 {
        warn!(
            retry_count,
            "negative retry_count; refusing to schedule retry"
        );
        return None;
    }
    if i32::from(retry_count) >= max_retries {
        return None;
    }

    let raw_delay_secs: u64 = match strategy {
        Strategy::ExponentialIncreasing => {
            let base = row.increasing_base_delay_secs?;
            let factor = row.increasing_wait_factor?;
            let projected = f64::from(base) * factor.powi(i32::from(retry_count));
            // Pre-clamped to [0, MAX_RETRY_DELAY_SECS_F64]; residual NaN saturates to 0 via `as u64`.
            projected.clamp(0.0, MAX_RETRY_DELAY_SECS_F64) as u64
        }
        Strategy::Linear => {
            let delay = row.linear_delay_secs?;
            u64::try_from(delay).ok()?
        }
        Strategy::Custom => {
            let intervals = row.custom_intervals_secs.as_ref()?;
            let index = usize::try_from(retry_count).ok()?;
            let value = intervals.get(index).copied()?;
            u64::try_from(value).ok()?
        }
    };

    if raw_delay_secs == 0 && !matches!(strategy, Strategy::ExponentialIncreasing) {
        // A zero or negative persisted delay is effectively "give up" — DB CHECK forbids it.
        return None;
    }

    let capped = raw_delay_secs.min(MAX_RETRY_DELAY_SECS);
    Some(Duration::from_secs(capped))
}

/// Built-in escalating table used when no retry schedule is attached.
/// Pure and deterministic so planning helpers stay reproducible.
///
/// WARNING: keep in sync with `api::handlers::instance::DEFAULT_SCHEDULE_DELAYS`.
/// Same values must live in both crates until we have a shared crate for retry policy.
pub fn built_in_retry_delay(max_retries: u8, retry_count: i16) -> Option<Duration> {
    if retry_count >= i16::from(max_retries) {
        return None;
    }
    match retry_count {
        0 => Some(Duration::from_secs(3)),
        1 => Some(Duration::from_secs(10)),
        2 => Some(Duration::from_secs(3 * 60)),
        3 => Some(Duration::from_secs(30 * 60)),
        4 => Some(Duration::from_hours(1)),
        5 => Some(Duration::from_hours(3)),
        6 => Some(Duration::from_hours(5)),
        _ => Some(Duration::from_hours(10)),
    }
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

fn response_body_text(response: &Response) -> &str {
    response
        .body
        .as_ref()
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .unwrap_or("???")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exponential(base: i32, factor: f64, max: i32) -> SubscriptionRetrySchedule {
        SubscriptionRetrySchedule {
            subscription_retry_schedule_id: Some(uuid::Uuid::nil()),
            strategy: Some(Strategy::ExponentialIncreasing),
            max_retries: Some(max),
            custom_intervals_secs: None,
            linear_delay_secs: None,
            increasing_base_delay_secs: Some(base),
            increasing_wait_factor: Some(factor),
        }
    }

    fn linear(delay_seconds: i32, max: i32) -> SubscriptionRetrySchedule {
        SubscriptionRetrySchedule {
            subscription_retry_schedule_id: Some(uuid::Uuid::nil()),
            strategy: Some(Strategy::Linear),
            max_retries: Some(max),
            custom_intervals_secs: None,
            linear_delay_secs: Some(delay_seconds),
            increasing_base_delay_secs: None,
            increasing_wait_factor: None,
        }
    }

    fn custom(intervals: Vec<i32>) -> SubscriptionRetrySchedule {
        let max = i32::try_from(intervals.len()).expect("test intervals fit in i32");
        SubscriptionRetrySchedule {
            subscription_retry_schedule_id: Some(uuid::Uuid::nil()),
            strategy: Some(Strategy::Custom),
            max_retries: Some(max),
            custom_intervals_secs: Some(intervals),
            linear_delay_secs: None,
            increasing_base_delay_secs: None,
            increasing_wait_factor: None,
        }
    }

    #[test]
    fn exponential_capped_at_7d_per_retry() {
        let row = exponential(1, 10.0, 10);
        let delay = compute_scheduled_retry_delay(&row, 9).unwrap();
        assert!(delay.as_secs() <= MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn exponential_over_max_retries_is_none() {
        let row = exponential(60, 2.0, 3);
        assert!(compute_scheduled_retry_delay(&row, 3).is_none());
        assert!(compute_scheduled_retry_delay(&row, 10).is_none());
    }

    #[test]
    fn linear_rejects_non_positive_delay() {
        let row = linear(0, 5);
        assert!(compute_scheduled_retry_delay(&row, 0).is_none());
    }

    #[test]
    fn custom_indexes_into_intervals() {
        let row = custom(vec![5, 10, 20]);
        let first_delay = compute_scheduled_retry_delay(&row, 0).unwrap();
        let last_delay = compute_scheduled_retry_delay(&row, 2).unwrap();
        assert_eq!(first_delay.as_secs(), 5);
        assert_eq!(last_delay.as_secs(), 20);
        assert!(compute_scheduled_retry_delay(&row, 3).is_none());
    }

    #[test]
    fn jitter_multiplier_within_factor_bound() {
        let base = Duration::from_secs(100);
        for _ in 0..100 {
            let delay = with_jitter(base, 0.2);
            let secs = delay.as_secs_f64();
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
        let row = linear(60, 5);
        assert!(compute_scheduled_retry_delay(&row, -1).is_none());
    }

    #[test]
    fn exponential_clamps_positive_infinity() {
        // Factor * huge base overflows to +Inf; clamp protects the cast to u64.
        let row = exponential(i32::MAX, 20.0, 5);
        let delay = compute_scheduled_retry_delay(&row, 4).unwrap();
        assert!(delay.as_secs() <= MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn exponential_clamps_nan_factor() {
        // retry_count >= 1 so factor.powi(…) is NaN (NaN^0 = 1 by IEEE, so skip 0).
        let row = exponential(60, f64::NAN, 5);
        let delay = compute_scheduled_retry_delay(&row, 1).unwrap();
        assert_eq!(delay.as_secs(), 0);
    }

    #[test]
    fn built_in_delay_respects_max_retries() {
        assert!(built_in_retry_delay(3, 3).is_none());
        assert!(built_in_retry_delay(3, 0).is_some());
    }

    #[test]
    fn built_in_schedule_matches_published_defaults() {
        // These values must match `api::handlers::instance::DEFAULT_SCHEDULE_DELAYS_SECS`.
        // Changing the table here requires updating the `/instance` endpoint so the frontend
        // displays accurate built-in delays.
        let expected = [3u64, 10, 180, 1800, 3600, 10800, 18000, 36000];
        for (retry_count, &want) in expected.iter().enumerate() {
            let got = built_in_retry_delay(u8::MAX, i16::try_from(retry_count).unwrap()).unwrap();
            assert_eq!(got.as_secs(), want, "retry_count={retry_count}");
        }
    }
}
