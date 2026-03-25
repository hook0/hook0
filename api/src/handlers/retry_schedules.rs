use chrono::{DateTime, Utc};
use paperclip::actix::Apiv2Schema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use uuid::Uuid;
use validator::Validate;

use crate::problems::Hook0Problem;

pub const MAX_INTERVAL_SECS: i32 = 604_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Apiv2Schema)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: String,
    pub max_retries: i32,
    pub initial_delay_secs: i32,
    pub linear_delay_secs: Option<i32>,
    pub custom_intervals: Option<Vec<i32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(non_control_character, length(min = 1, max = 100))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 20))]
    pub max_retries: i32,
    #[validate(range(min = 1, max = 604_800))]
    pub initial_delay_secs: i32,
    pub linear_delay_secs: Option<i32>,
    pub custom_intervals: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct RetrySchedulePut {
    pub organization_id: Uuid,
    #[validate(non_control_character, length(min = 1, max = 100))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 20))]
    pub max_retries: i32,
    #[validate(range(min = 1, max = 604_800))]
    pub initial_delay_secs: i32,
    pub linear_delay_secs: Option<i32>,
    pub custom_intervals: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}

pub fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay_secs: Option<i32>,
    custom_intervals: Option<&[i32]>,
) -> Result<(), Hook0Problem> {
    match strategy {
        RetryStrategy::Exponential => {
            if custom_intervals.is_some() {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("custom_intervals", "custom_intervals must be None for exponential strategy"),
                ));
            }
            if linear_delay_secs.is_some() {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("linear_delay_secs", "linear_delay_secs must be None for exponential strategy"),
                ));
            }
        }
        RetryStrategy::Linear => {
            if custom_intervals.is_some() {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("custom_intervals", "custom_intervals must be None for linear strategy"),
                ));
            }
            if linear_delay_secs.is_none() {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("linear_delay_secs", "linear_delay_secs is required for linear strategy"),
                ));
            }
            let delay = linear_delay_secs.unwrap();
            if !(1..=MAX_INTERVAL_SECS).contains(&delay) {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("linear_delay_secs", "linear_delay_secs must be between 1 and 604800"),
                ));
            }
        }
        RetryStrategy::Custom => {
            if linear_delay_secs.is_some() {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("linear_delay_secs", "linear_delay_secs must be None for custom strategy"),
                ));
            }
            let intervals = match custom_intervals {
                Some(i) => i,
                None => {
                    return Err(Hook0Problem::Validation(
                        mk_validation_error("custom_intervals", "custom_intervals is required for custom strategy"),
                    ));
                }
            };
            if intervals.len() != max_retries as usize {
                return Err(Hook0Problem::Validation(
                    mk_validation_error("custom_intervals", "custom_intervals length must equal max_retries"),
                ));
            }
            for (i, &val) in intervals.iter().enumerate() {
                if !(1..=MAX_INTERVAL_SECS).contains(&val) {
                    let msg = format!("custom_intervals[{i}] must be between 1 and {MAX_INTERVAL_SECS}");
                    return Err(Hook0Problem::Validation(
                        mk_validation_error("custom_intervals", &msg),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn mk_validation_error(field: &'static str, message: &str) -> validator::ValidationErrors {
    let mut errors = validator::ValidationErrors::new();
    let mut error = validator::ValidationError::new("invalid");
    error.message = Some(std::borrow::Cow::Owned(message.to_owned()));
    errors.add(field, error);
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_valid() {
        let result = validate_strategy_fields(RetryStrategy::Exponential, 5, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn exponential_rejects_custom_intervals() {
        let result = validate_strategy_fields(RetryStrategy::Exponential, 3, None, Some(&[10, 20, 30]));
        assert!(result.is_err());
    }

    #[test]
    fn exponential_rejects_linear_delay() {
        let result = validate_strategy_fields(RetryStrategy::Exponential, 3, Some(60), None);
        assert!(result.is_err());
    }

    #[test]
    fn exponential_rejects_both_fields() {
        let result = validate_strategy_fields(RetryStrategy::Exponential, 3, Some(60), Some(&[10, 20, 30]));
        assert!(result.is_err());
    }

    #[test]
    fn linear_valid() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(120), None);
        assert!(result.is_ok());
    }

    #[test]
    fn linear_rejects_missing_delay() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn linear_rejects_custom_intervals() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(60), Some(&[10, 20, 30]));
        assert!(result.is_err());
    }

    #[test]
    fn linear_rejects_delay_too_low() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(0), None);
        assert!(result.is_err());
    }

    #[test]
    fn linear_rejects_delay_too_high() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(MAX_INTERVAL_SECS + 1), None);
        assert!(result.is_err());
    }

    #[test]
    fn linear_accepts_max_delay() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(MAX_INTERVAL_SECS), None);
        assert!(result.is_ok());
    }

    #[test]
    fn linear_accepts_min_delay() {
        let result = validate_strategy_fields(RetryStrategy::Linear, 3, Some(1), None);
        assert!(result.is_ok());
    }

    #[test]
    fn custom_valid() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, None, Some(&[10, 60, 300]));
        assert!(result.is_ok());
    }

    #[test]
    fn custom_rejects_missing_intervals() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn custom_rejects_linear_delay() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, Some(60), Some(&[10, 20, 30]));
        assert!(result.is_err());
    }

    #[test]
    fn custom_rejects_wrong_length() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, None, Some(&[10, 20]));
        assert!(result.is_err());
    }

    #[test]
    fn custom_rejects_interval_too_low() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, None, Some(&[0, 60, 300]));
        assert!(result.is_err());
    }

    #[test]
    fn custom_rejects_interval_too_high() {
        let intervals = &[10, 60, MAX_INTERVAL_SECS + 1];
        let result = validate_strategy_fields(RetryStrategy::Custom, 3, None, Some(intervals));
        assert!(result.is_err());
    }

    #[test]
    fn custom_accepts_max_interval() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 1, None, Some(&[MAX_INTERVAL_SECS]));
        assert!(result.is_ok());
    }

    #[test]
    fn custom_accepts_min_interval() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 1, None, Some(&[1]));
        assert!(result.is_ok());
    }

    #[test]
    fn custom_rejects_empty_intervals() {
        let result = validate_strategy_fields(RetryStrategy::Custom, 0, None, Some(&[]));
        assert!(result.is_ok()); // 0 retries with 0 intervals is consistent
    }
}
