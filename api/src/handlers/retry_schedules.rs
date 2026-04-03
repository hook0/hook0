//! CRUD handlers for retry schedules — the per-org timing configs that control how failed webhook
//! deliveries are retried.
//!
//! How it works:
//! 1. Each schedule picks a strategy (increasing, linear, custom) that determines wait times.
//! 2. Strategy-specific fields are cross-validated at the request level (`validate_strategy_fields`).
//! 3. The CREATE path uses an atomic INSERT…WHERE subquery to enforce the per-org quota without TOCTOU races.
//! 4. All mutations emit Hook0 client events for internal observability.

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use strum::{Display, EnumString};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventRetryScheduleCreated, EventRetryScheduleRemoved, EventRetryScheduleUpdated,
    Hook0ClientEvent,
};
use crate::iam::{Action, authorize};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

/// Maximum interval between retries across all strategies, in seconds (7 days).
pub const MAX_INTERVAL_SECS: i32 = 3600 * 24 * 7;

/// Minimum base delay for the increasing retry strategy, in seconds.
const MIN_BASE_DELAY_SECS: i32 = 1;

/// Maximum base delay for the increasing retry strategy, in seconds (1 hour).
const MAX_BASE_DELAY_SECS: i32 = 3600;

/// Minimum wait factor (multiplier) for the increasing retry strategy.
/// Example: with base_delay=3s and wait_factor=1.5, delays are: 3s, 4.5s, 6.75s, 10.1s, 15.2s...
const MIN_WAIT_FACTOR: f64 = 1.5;

/// Maximum wait factor (multiplier) for the increasing retry strategy.
/// Example: with base_delay=3s and wait_factor=10, delays are: 3s, 30s, 5min, 50min, 8.3h...
const MAX_WAIT_FACTOR: f64 = 10.0;

/// Minimum delay for linear and custom retry strategies, in seconds.
const MIN_DELAY_SECS: i32 = 1;

/// The algorithm that determines wait times between retry attempts.
///
/// "Increasing" = exponential backoff, "Linear" = fixed interval, "Custom" = caller-supplied array.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    Apiv2Schema,
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum RetryStrategy {
    Increasing,
    Linear,
    Custom,
}

/// API response for a retry schedule — the timing config a subscription needs to retry failed deliveries.
#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: RetryStrategy,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create request. Strategy-specific fields are cross-validated by `validate_strategy_fields`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[validate(schema(function = "RetrySchedulePost::validate_strategy"))]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(non_control_character, length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

impl RetrySchedulePost {
    fn validate_strategy(&self) -> Result<(), validator::ValidationError> {
        validate_strategy_fields(
            self.strategy,
            self.max_retries,
            self.linear_delay,
            self.custom_intervals.as_deref(),
            self.increasing_base_delay,
            self.increasing_wait_factor,
        )
    }
}

/// Update request. Same cross-field rules as `RetrySchedulePost`, minus `organization_id`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[validate(schema(function = "RetrySchedulePut::validate_strategy"))]
pub struct RetrySchedulePut {
    #[validate(non_control_character, length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

impl RetrySchedulePut {
    fn validate_strategy(&self) -> Result<(), validator::ValidationError> {
        validate_strategy_fields(
            self.strategy,
            self.max_retries,
            self.linear_delay,
            self.custom_intervals.as_deref(),
            self.increasing_base_delay,
            self.increasing_wait_factor,
        )
    }
}

/// Query string that scopes every operation to one organization.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}

fn strategy_error(message: &str) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("strategy_fields");
    err.message = Some(std::borrow::Cow::Owned(message.to_owned()));
    err
}

fn require_none<T>(
    field: &str,
    value: &Option<T>,
    strategy: &str,
) -> Result<(), validator::ValidationError> {
    if value.is_some() {
        Err(strategy_error(&format!(
            "{field} must be None for {strategy} strategy"
        )))
    } else {
        Ok(())
    }
}

fn require_some<'a, T>(
    field: &str,
    value: &'a Option<T>,
    strategy: &str,
) -> Result<&'a T, validator::ValidationError> {
    value
        .as_ref()
        .ok_or_else(|| strategy_error(&format!("{field} is required for {strategy} strategy")))
}

fn require_range(
    field: &str,
    value: i32,
    min: i32,
    max: i32,
) -> Result<(), validator::ValidationError> {
    if !(min..=max).contains(&value) {
        Err(strategy_error(&format!(
            "{field} must be between {min} and {max}"
        )))
    } else {
        Ok(())
    }
}

/// Cross-field validation for retry schedule strategy. Called by the validator
/// crate via `#[validate(schema(function = "..."))]` on Post and Put structs.
/// This is a free function (not a trait) because the `validator` crate's
/// `#[validate(schema(function = "Self::validate_strategy"))]` resolves to an
/// inherent method — it does not support trait method paths.
pub fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationError> {
    match strategy {
        RetryStrategy::Increasing => {
            require_none("custom_intervals", &custom_intervals, "increasing")?;
            require_none("linear_delay", &linear_delay, "increasing")?;
            let base = require_some(
                "increasing_base_delay",
                &increasing_base_delay,
                "increasing",
            )?;
            require_range(
                "increasing_base_delay",
                *base,
                MIN_BASE_DELAY_SECS,
                MAX_BASE_DELAY_SECS,
            )?;
            let factor = require_some(
                "increasing_wait_factor",
                &increasing_wait_factor,
                "increasing",
            )?;
            if !(*factor >= MIN_WAIT_FACTOR && *factor <= MAX_WAIT_FACTOR) {
                return Err(strategy_error(&format!(
                    "increasing_wait_factor must be between {MIN_WAIT_FACTOR} and {MAX_WAIT_FACTOR}"
                )));
            }
        }
        RetryStrategy::Linear => {
            require_none("custom_intervals", &custom_intervals, "linear")?;
            require_none("increasing_base_delay", &increasing_base_delay, "linear")?;
            require_none("increasing_wait_factor", &increasing_wait_factor, "linear")?;
            let delay = require_some("linear_delay", &linear_delay, "linear")?;
            require_range("linear_delay", *delay, MIN_DELAY_SECS, MAX_INTERVAL_SECS)?;
        }
        RetryStrategy::Custom => {
            require_none("linear_delay", &linear_delay, "custom")?;
            require_none("increasing_base_delay", &increasing_base_delay, "custom")?;
            require_none("increasing_wait_factor", &increasing_wait_factor, "custom")?;
            let intervals = require_some("custom_intervals", &custom_intervals, "custom")?;
            if intervals.len() != max_retries as usize {
                return Err(strategy_error(
                    "custom_intervals length must equal max_retries",
                ));
            }
            for (i, &val) in intervals.iter().enumerate() {
                if !(MIN_DELAY_SECS..=MAX_INTERVAL_SECS).contains(&val) {
                    return Err(strategy_error(&format!(
                        "custom_intervals[{i}] must be between {MIN_DELAY_SECS} and {MAX_INTERVAL_SECS}"
                    )));
                }
            }
        }
    }
    Ok(())
}

#[api_v2_operation(
    summary = "List retry schedules",
    operation_id = "retry_schedules.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
/// Return all retry schedules belonging to the given organization, newest first.
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<Vec<RetrySchedule>>, Hook0Problem> {
    let organization_id = qs.organization_id;

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleList,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let schedules = sqlx::query_as::<_, RetrySchedule>(
        "
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                increasing_base_delay, increasing_wait_factor,
                created_at, updated_at
            FROM webhook.retry_schedule
            WHERE organization__id = $1
            ORDER BY created_at DESC
        ",
    )
    .bind(organization_id)
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(schedules))
}

#[api_v2_operation(
    summary = "Create a retry schedule",
    operation_id = "retry_schedules.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
/// Validate, insert a new retry schedule (with atomic quota check), and emit a creation event.
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<RetrySchedulePost>,
) -> Result<CreatedJson<RetrySchedule>, Hook0Problem> {
    let organization_id = body.organization_id;

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleCreate,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    // Atomic INSERT with quota check: the WHERE subquery counts existing rows for this org
    // and only proceeds if under the limit, preventing TOCTOU race conditions.
    let max_per_org = i64::from(state.max_retry_schedules_per_org);
    let schedule = sqlx::query_as::<_, RetrySchedule>(
        "
            INSERT INTO webhook.retry_schedule (organization__id, name, strategy, max_retries, custom_intervals, linear_delay, increasing_base_delay, increasing_wait_factor)
            SELECT $1, $2, $3, $4, $5, $6, $7, $8
            WHERE (
                SELECT COUNT(*)
                FROM webhook.retry_schedule
                WHERE organization__id = $1
            ) < $9
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                increasing_base_delay, increasing_wait_factor,
                created_at, updated_at
        ",
    )
    .bind(organization_id)
    .bind(&body.name)
    .bind(body.strategy.to_string())
    .bind(body.max_retries)
    .bind(body.custom_intervals.as_deref())
    .bind(body.linear_delay)
    .bind(body.increasing_base_delay)
    .bind(body.increasing_wait_factor)
    .bind(max_per_org)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // The INSERT's WHERE clause checks the quota — if no row returned, the org hit its limit
    let Some(s) = schedule else {
        return Err(Hook0Problem::TooManyRetrySchedulesPerOrganization(
            state.max_retry_schedules_per_org,
        ));
    };

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleCreated {
            organization_id,
            retry_schedule_id: s.retry_schedule_id,
            name: s.name.to_owned(),
            strategy: s.strategy.to_string(),
            max_retries: s.max_retries,
            custom_intervals: s.custom_intervals.to_owned(),
            linear_delay: s.linear_delay,
            increasing_base_delay: s.increasing_base_delay,
            increasing_wait_factor: s.increasing_wait_factor,
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(s))
}

#[api_v2_operation(
    summary = "Get a retry schedule",
    operation_id = "retry_schedules.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
/// Fetch a single retry schedule by ID, scoped to the caller's organization.
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let retry_schedule_id = retry_schedule_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleGet,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let schedule = sqlx::query_as::<_, RetrySchedule>(
        "
            SELECT
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                increasing_base_delay, increasing_wait_factor,
                created_at, updated_at
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
                AND organization__id = $2
        ",
    )
    .bind(retry_schedule_id)
    .bind(organization_id)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match schedule {
        Some(s) => Ok(Json(s)),
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Edit a retry schedule",
    operation_id = "retry_schedules.edit",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
/// Replace all fields of a retry schedule (full PUT). Emits an update event on success.
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let retry_schedule_id = retry_schedule_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleEdit,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let schedule = sqlx::query_as::<_, RetrySchedule>(
        "
            UPDATE webhook.retry_schedule
            SET name = $3, strategy = $4, max_retries = $5,
                custom_intervals = $6, linear_delay = $7,
                increasing_base_delay = $8, increasing_wait_factor = $9,
                updated_at = statement_timestamp()
            WHERE retry_schedule__id = $1
                AND organization__id = $2
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                increasing_base_delay, increasing_wait_factor,
                created_at, updated_at
        ",
    )
    .bind(retry_schedule_id)
    .bind(organization_id)
    .bind(&body.name)
    .bind(body.strategy.to_string())
    .bind(body.max_retries)
    .bind(body.custom_intervals.as_deref())
    .bind(body.linear_delay)
    .bind(body.increasing_base_delay)
    .bind(body.increasing_wait_factor)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match schedule {
        Some(s) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventRetryScheduleUpdated {
                    organization_id,
                    retry_schedule_id: s.retry_schedule_id,
                    name: s.name.to_owned(),
                    strategy: s.strategy.to_string(),
                    max_retries: s.max_retries,
                    custom_intervals: s.custom_intervals.to_owned(),
                    linear_delay: s.linear_delay,
                    increasing_base_delay: s.increasing_base_delay,
                    increasing_wait_factor: s.increasing_wait_factor,
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(Json(s))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Delete a retry schedule",
    operation_id = "retry_schedules.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
/// Delete a retry schedule and emit a removal event. Subscriptions referencing it get their FK NULLed by the DB cascade.
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<()>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let retry_schedule_id = retry_schedule_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleDelete,
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // DELETE...RETURNING captures the row data in one atomic operation,
    // so we can emit the event with the deleted schedule's fields.
    struct DeletedRow {
        name: String,
        strategy: String,
        max_retries: i32,
        custom_intervals: Option<Vec<i32>>,
        linear_delay: Option<i32>,
        increasing_base_delay: Option<i32>,
        increasing_wait_factor: Option<f64>,
    }

    let deleted = query_as!(
        DeletedRow,
        "
            DELETE FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
                AND organization__id = $2
            RETURNING name, strategy, max_retries, custom_intervals, linear_delay, increasing_base_delay, increasing_wait_factor
        ",
        &retry_schedule_id,
        &organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // No row returned — schedule didn't exist or wrong org
    let Some(d) = deleted else {
        return Err(Hook0Problem::NotFound);
    };

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventRetryScheduleRemoved {
            organization_id,
            retry_schedule_id,
            name: d.name,
            strategy: d.strategy,
            max_retries: d.max_retries,
            custom_intervals: d.custom_intervals,
            linear_delay: d.linear_delay,
            increasing_base_delay: d.increasing_base_delay,
            increasing_wait_factor: d.increasing_wait_factor,
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(Json(()))
}

/// Unit tests for strategy-dependent field validation logic.
#[cfg(test)]
mod tests {
    use super::*;

    /// Call sites pass all 6 strategy params positionally so each test reads as a one-liner
    fn v(
        strategy: RetryStrategy,
        max: i32,
        ld: Option<i32>,
        ci: Option<&[i32]>,
        ibd: Option<i32>,
        isf: Option<f64>,
    ) -> Result<(), validator::ValidationError> {
        validate_strategy_fields(strategy, max, ld, ci, ibd, isf)
    }

    #[test]
    fn increasing_valid() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3), Some(3.0)).is_ok());
    }

    #[test]
    fn increasing_rejects_missing_base_delay() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, None, Some(3.0)).is_err());
    }

    #[test]
    fn increasing_rejects_missing_wait_factor() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3), None).is_err());
    }

    #[test]
    fn increasing_rejects_custom_intervals() {
        assert!(
            v(
                RetryStrategy::Increasing,
                3,
                None,
                Some(&[10, 20, 30]),
                Some(3),
                Some(3.0)
            )
            .is_err()
        );
    }

    #[test]
    fn increasing_rejects_linear_delay() {
        assert!(
            v(
                RetryStrategy::Increasing,
                3,
                Some(60),
                None,
                Some(3),
                Some(3.0)
            )
            .is_err()
        );
    }

    #[test]
    fn increasing_rejects_factor_too_low() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3), Some(1.0)).is_err());
    }

    #[test]
    fn increasing_rejects_factor_too_high() {
        assert!(
            v(
                RetryStrategy::Increasing,
                5,
                None,
                None,
                Some(3),
                Some(11.0)
            )
            .is_err()
        );
    }

    #[test]
    fn increasing_accepts_boundary_factor() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3), Some(1.5)).is_ok());
        assert!(
            v(
                RetryStrategy::Increasing,
                5,
                None,
                None,
                Some(3),
                Some(10.0)
            )
            .is_ok()
        );
    }

    #[test]
    fn linear_valid() {
        assert!(v(RetryStrategy::Linear, 3, Some(120), None, None, None).is_ok());
    }

    #[test]
    fn linear_rejects_missing_delay() {
        assert!(v(RetryStrategy::Linear, 3, None, None, None, None).is_err());
    }

    #[test]
    fn linear_rejects_custom_intervals() {
        assert!(
            v(
                RetryStrategy::Linear,
                3,
                Some(60),
                Some(&[10, 20, 30]),
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn linear_rejects_increasing_fields() {
        assert!(v(RetryStrategy::Linear, 3, Some(60), None, Some(3), None).is_err());
        assert!(v(RetryStrategy::Linear, 3, Some(60), None, None, Some(2.0)).is_err());
    }

    #[test]
    fn linear_rejects_delay_too_low() {
        assert!(v(RetryStrategy::Linear, 3, Some(0), None, None, None).is_err());
    }

    #[test]
    fn linear_rejects_delay_too_high() {
        assert!(
            v(
                RetryStrategy::Linear,
                3,
                Some(MAX_INTERVAL_SECS + 1),
                None,
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn linear_accepts_boundary_delay() {
        assert!(v(RetryStrategy::Linear, 3, Some(1), None, None, None).is_ok());
        assert!(
            v(
                RetryStrategy::Linear,
                3,
                Some(MAX_INTERVAL_SECS),
                None,
                None,
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn custom_valid() {
        assert!(
            v(
                RetryStrategy::Custom,
                3,
                None,
                Some(&[10, 60, 300]),
                None,
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn custom_rejects_missing_intervals() {
        assert!(v(RetryStrategy::Custom, 3, None, None, None, None).is_err());
    }

    #[test]
    fn custom_rejects_linear_delay() {
        assert!(
            v(
                RetryStrategy::Custom,
                3,
                Some(60),
                Some(&[10, 20, 30]),
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn custom_rejects_increasing_fields() {
        assert!(
            v(
                RetryStrategy::Custom,
                3,
                None,
                Some(&[10, 20, 30]),
                Some(3),
                None
            )
            .is_err()
        );
    }

    #[test]
    fn custom_rejects_wrong_length() {
        assert!(v(RetryStrategy::Custom, 3, None, Some(&[10, 20]), None, None).is_err());
    }

    #[test]
    fn custom_rejects_interval_too_low() {
        assert!(
            v(
                RetryStrategy::Custom,
                3,
                None,
                Some(&[0, 60, 300]),
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn custom_rejects_interval_too_high() {
        let intervals = &[10, 60, MAX_INTERVAL_SECS + 1];
        assert!(v(RetryStrategy::Custom, 3, None, Some(intervals), None, None).is_err());
    }

    #[test]
    fn custom_accepts_max_interval() {
        assert!(
            v(
                RetryStrategy::Custom,
                1,
                None,
                Some(&[MAX_INTERVAL_SECS]),
                None,
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn custom_accepts_min_interval() {
        assert!(v(RetryStrategy::Custom, 1, None, Some(&[1]), None, None).is_ok());
    }

    #[test]
    fn custom_rejects_empty_intervals() {
        // max_retries=1 but empty intervals -> len mismatch
        assert!(v(RetryStrategy::Custom, 1, None, Some(&[]), None, None).is_err());
    }

    #[test]
    fn increasing_rejects_base_delay_zero() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(0), Some(3.0)).is_err());
    }

    #[test]
    fn increasing_rejects_base_delay_too_high() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3601), Some(3.0)).is_err());
    }

    #[test]
    fn increasing_accepts_base_delay_boundaries() {
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(1), Some(3.0)).is_ok());
        assert!(v(RetryStrategy::Increasing, 5, None, None, Some(3600), Some(3.0)).is_ok());
    }
}
