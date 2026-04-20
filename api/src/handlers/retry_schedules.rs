//! Retry schedule types and payload validator.
//!
//! Caller-visible effect: rejects payloads breaching business bounds with 400 InvalidPayload.
//!
//! - Entity, strategy enum, discriminated-union payload
//! - Business constants: single-delay / total-duration / max-retries / name / quota caps
//! - `validate_payload` + `compute_total_duration` enforced pre-persist

// CRUD handlers land in task 3 and consume these items.
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::problems::Hook0Problem;

pub const MAX_RETRIES: i32 = 15;
pub const MIN_SINGLE_DELAY_SECS: i32 = 1;
pub const MAX_SINGLE_DELAY_SECS: i32 = 7 * 24 * 3600;
pub const MAX_TOTAL_DURATION_SECS: i64 = 7 * 24 * 3600;
pub const EXP_BASE_MIN_SECS: i32 = 1;
pub const EXP_BASE_MAX_SECS: i32 = 3600;
pub const EXP_FACTOR_MIN: f64 = 1.5;
pub const EXP_FACTOR_MAX: f64 = 10.0;
pub const MAX_NAME_LEN: usize = 200;
pub const MIN_NAME_LEN: usize = 1;
pub const MAX_PER_ORG: i64 = 50;

/// Persisted retry schedule owned by one organization.
#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow, Clone)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: Strategy,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Retry pacing family selected when schedule is created.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, sqlx::Type, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum Strategy {
    ExponentialIncreasing,
    Linear,
    Custom,
}

/// Create/update payload; strategy discriminator picks the variant shape.
#[derive(Debug, Deserialize, Apiv2Schema, Clone)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum RetrySchedulePayload {
    ExponentialIncreasing {
        name: String,
        max_retries: i32,
        base_delay: i32,
        wait_factor: f64,
    },
    Linear {
        name: String,
        max_retries: i32,
        delay: i32,
    },
    Custom {
        name: String,
        intervals: Vec<i32>,
    },
}

/// Reject payload breaching business bounds.
pub fn validate_payload(payload: &RetrySchedulePayload) -> Result<(), Hook0Problem> {
    validate_name(payload_name(payload))?;
    validate_strategy_fields(payload)?;

    let total = compute_total_duration(payload);
    if total > MAX_TOTAL_DURATION_SECS {
        return Err(invalid(format!(
            "Total retry duration ({total}s) exceeds cap of {MAX_TOTAL_DURATION_SECS}s."
        )));
    }

    Ok(())
}

/// Sum of retry delays in seconds; per-retry delay pre-clamped.
pub fn compute_total_duration(payload: &RetrySchedulePayload) -> i64 {
    match payload {
        RetrySchedulePayload::ExponentialIncreasing {
            max_retries,
            base_delay,
            wait_factor,
            ..
        } => {
            let retries = (*max_retries).max(0);
            let base = f64::from(*base_delay);
            let cap = f64::from(MAX_SINGLE_DELAY_SECS);
            let mut total: i64 = 0;
            for i in 0..retries {
                // Per-retry clamp prevents i64 overflow on hostile inputs.
                let term = (base * wait_factor.powi(i)).min(cap).max(0.0);
                total = total.saturating_add(term as i64);
            }
            total
        }
        RetrySchedulePayload::Linear {
            max_retries, delay, ..
        } => i64::from(*max_retries).saturating_mul(i64::from(*delay)),
        RetrySchedulePayload::Custom { intervals, .. } => intervals
            .iter()
            .map(|x| i64::from(*x).min(i64::from(MAX_SINGLE_DELAY_SECS)))
            .fold(0i64, |acc, v| acc.saturating_add(v)),
    }
}

fn payload_name(payload: &RetrySchedulePayload) -> &str {
    match payload {
        RetrySchedulePayload::ExponentialIncreasing { name, .. }
        | RetrySchedulePayload::Linear { name, .. }
        | RetrySchedulePayload::Custom { name, .. } => name,
    }
}

fn validate_name(name: &str) -> Result<(), Hook0Problem> {
    if name.trim().is_empty() {
        return Err(invalid(
            "Name must not be empty or whitespace-only.".to_owned(),
        ));
    }
    let len = name.len();
    if !(MIN_NAME_LEN..=MAX_NAME_LEN).contains(&len) {
        return Err(invalid(format!(
            "Name length ({len}) must be within {MIN_NAME_LEN}..={MAX_NAME_LEN}."
        )));
    }
    Ok(())
}

fn validate_strategy_fields(payload: &RetrySchedulePayload) -> Result<(), Hook0Problem> {
    match payload {
        RetrySchedulePayload::ExponentialIncreasing {
            max_retries,
            base_delay,
            wait_factor,
            ..
        } => {
            check_retries(*max_retries)?;
            if !(EXP_BASE_MIN_SECS..=EXP_BASE_MAX_SECS).contains(base_delay) {
                return Err(invalid(format!(
                    "base_delay ({base_delay}s) must be within {EXP_BASE_MIN_SECS}..={EXP_BASE_MAX_SECS}."
                )));
            }
            if !(EXP_FACTOR_MIN..=EXP_FACTOR_MAX).contains(wait_factor) {
                return Err(invalid(format!(
                    "wait_factor ({wait_factor}) must be within {EXP_FACTOR_MIN}..={EXP_FACTOR_MAX}."
                )));
            }
            Ok(())
        }
        RetrySchedulePayload::Linear {
            max_retries, delay, ..
        } => {
            check_retries(*max_retries)?;
            if !(MIN_SINGLE_DELAY_SECS..=MAX_SINGLE_DELAY_SECS).contains(delay) {
                return Err(invalid(format!(
                    "delay ({delay}s) must be within {MIN_SINGLE_DELAY_SECS}..={MAX_SINGLE_DELAY_SECS}."
                )));
            }
            Ok(())
        }
        RetrySchedulePayload::Custom { intervals, .. } => {
            let len = intervals.len() as i32;
            if intervals.is_empty() || len > MAX_RETRIES {
                return Err(invalid(format!(
                    "intervals count ({len}) must be within 1..={MAX_RETRIES}."
                )));
            }
            for (i, v) in intervals.iter().enumerate() {
                if !(MIN_SINGLE_DELAY_SECS..=MAX_SINGLE_DELAY_SECS).contains(v) {
                    return Err(invalid(format!(
                        "intervals[{i}] ({v}s) must be within {MIN_SINGLE_DELAY_SECS}..={MAX_SINGLE_DELAY_SECS}."
                    )));
                }
            }
            Ok(())
        }
    }
}

fn check_retries(max_retries: i32) -> Result<(), Hook0Problem> {
    if !(1..=MAX_RETRIES).contains(&max_retries) {
        return Err(invalid(format!(
            "max_retries ({max_retries}) must be within 1..={MAX_RETRIES}."
        )));
    }
    Ok(())
}

fn invalid(reason: String) -> Hook0Problem {
    // TODO(task 4): swap for RetryScheduleValidationFailed
    Hook0Problem::InvalidPayload { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(
        name: &str,
        max_retries: i32,
        base_delay: i32,
        wait_factor: f64,
    ) -> RetrySchedulePayload {
        RetrySchedulePayload::ExponentialIncreasing {
            name: name.to_owned(),
            max_retries,
            base_delay,
            wait_factor,
        }
    }

    fn linear(name: &str, max_retries: i32, delay: i32) -> RetrySchedulePayload {
        RetrySchedulePayload::Linear {
            name: name.to_owned(),
            max_retries,
            delay,
        }
    }

    fn custom(name: &str, intervals: Vec<i32>) -> RetrySchedulePayload {
        RetrySchedulePayload::Custom {
            name: name.to_owned(),
            intervals,
        }
    }

    #[test]
    fn test_validate_exponential_ok() {
        let p = exp("good", 5, 60, 2.0);
        assert!(validate_payload(&p).is_ok());
    }

    #[test]
    fn test_validate_exponential_rejects_factor_too_low() {
        let p = exp("bad", 5, 60, 1.0);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_exponential_rejects_factor_too_high() {
        let p = exp("bad", 5, 60, 10.1);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_exponential_rejects_base_out_of_range() {
        assert!(validate_payload(&exp("bad", 5, 0, 2.0)).is_err());
        assert!(validate_payload(&exp("bad", 5, 3601, 2.0)).is_err());
    }

    #[test]
    fn test_validate_linear_ok() {
        let p = linear("good", 10, 60);
        assert!(validate_payload(&p).is_ok());
    }

    #[test]
    fn test_validate_linear_rejects_delay_too_low() {
        let p = linear("bad", 10, 0);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_linear_rejects_retries_over_cap() {
        let p = linear("bad", 16, 60);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_custom_ok() {
        let p = custom("good", vec![1, 2, 3]);
        assert!(validate_payload(&p).is_ok());
    }

    #[test]
    fn test_validate_custom_rejects_empty() {
        let p = custom("bad", vec![]);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_custom_rejects_over_cap() {
        let p = custom("bad", vec![1; 16]);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_custom_rejects_element_out_of_range() {
        let p = custom("bad", vec![0]);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_custom_rejects_total_over_7d() {
        let p = custom(
            "bad",
            vec![
                MAX_SINGLE_DELAY_SECS,
                MAX_SINGLE_DELAY_SECS,
                MAX_SINGLE_DELAY_SECS,
            ],
        );
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_name_rejects_whitespace_only() {
        let p = linear("   ", 5, 60);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_validate_name_rejects_too_long() {
        let name = "x".repeat(MAX_NAME_LEN + 1);
        let p = linear(&name, 5, 60);
        assert!(validate_payload(&p).is_err());
    }

    #[test]
    fn test_compute_total_duration_exponential_caps_per_retry() {
        let p = exp("cap", 10, 1, 10.0);
        let total = compute_total_duration(&p);
        let max_possible = 10i64 * i64::from(MAX_SINGLE_DELAY_SECS);
        assert!(total <= max_possible);
        assert!(total > 0);
    }
}
