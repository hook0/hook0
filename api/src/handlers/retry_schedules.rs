//! Retry schedule CRUD with cross-field validator.
//!
//! End-user sees 422 with field errors on invalid payloads. 404 hides cross-organization access.
//!
//! - Flat `RetrySchedulePost` / `RetrySchedulePut` drive the `validator` crate.
//! - Business bounds come from `State`. Attribute ranges stay wide.
//! - Per-organization quota: atomic `INSERT … WHERE count(*) < $limit`.
//! - Soft cap under contention. One extra row acceptable.
//! - Auth failure on get/edit/delete returns 404.
//! - Hides cross-organization existence oracle.

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, query_scalar};
use std::time::Duration;
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

const FIELD_LINEAR_DELAY: &str = "linear_delay";
const FIELD_CUSTOM_INTERVALS: &str = "custom_intervals";
const FIELD_INCREASING_BASE_DELAY: &str = "increasing_base_delay";
const FIELD_INCREASING_WAIT_FACTOR: &str = "increasing_wait_factor";

/// Retry pacing family that picks how delays between retries are computed.
#[non_exhaustive]
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
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RetryStrategy {
    ExponentialIncreasing,
    Linear,
    Custom,
}

/// Persisted retry schedule owned by one organization.
#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow, Clone)]
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

/// Create request. Ranges here are anti-corruption bounds; business limits are
/// enforced by the runtime check in `validate_against_limits`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "RetrySchedulePost::validate_strategy"))]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(non_control_character, length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Update request. Same cross-field rules as `RetrySchedulePost`, minus `organization_id`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "RetrySchedulePut::validate_strategy"))]
pub struct RetrySchedulePut {
    #[validate(non_control_character, length(min = 1, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub increasing_base_delay: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Dispatch validator-crate schema hooks to the shared strategy checker.
/// Structs share field names so the body is identical; macro avoids the duplication.
macro_rules! impl_strategy_schema_validator {
    ($struct_name:ident) => {
        impl $struct_name {
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
    };
}

impl_strategy_schema_validator!(RetrySchedulePost);
impl_strategy_schema_validator!(RetrySchedulePut);

/// Query string that scopes list operations to one organization.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(deny_unknown_fields)]
pub struct RetryScheduleListQuery {
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

/// Cross-field validation for a retry schedule strategy.
///
/// Wired by the `validator` crate via `#[validate(schema(function = "…"))]`.
/// Only checks mutual exclusion + presence; numeric ranges are handled by attributes
/// or the runtime `validate_against_limits` pass.
pub fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationError> {
    let strategy_name = strategy.to_string();
    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            require_none(FIELD_CUSTOM_INTERVALS, &custom_intervals, &strategy_name)?;
            require_none(FIELD_LINEAR_DELAY, &linear_delay, &strategy_name)?;
            require_some(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay,
                &strategy_name,
            )?;
            let factor = require_some(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                &strategy_name,
            )?;
            if factor.is_nan() {
                return Err(strategy_error(
                    "increasing_wait_factor must be a finite number",
                ));
            }
        }
        RetryStrategy::Linear => {
            require_none(FIELD_CUSTOM_INTERVALS, &custom_intervals, &strategy_name)?;
            require_none(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay,
                &strategy_name,
            )?;
            require_none(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                &strategy_name,
            )?;
            require_some(FIELD_LINEAR_DELAY, &linear_delay, &strategy_name)?;
        }
        RetryStrategy::Custom => {
            require_none(FIELD_LINEAR_DELAY, &linear_delay, &strategy_name)?;
            require_none(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay,
                &strategy_name,
            )?;
            require_none(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                &strategy_name,
            )?;
            let intervals =
                require_some(FIELD_CUSTOM_INTERVALS, &custom_intervals, &strategy_name)?;
            let expected_length = usize::try_from(max_retries).unwrap_or(0);
            if intervals.len() != expected_length {
                return Err(strategy_error(
                    "custom_intervals length must equal max_retries",
                ));
            }
        }
    }
    Ok(())
}

/// Borrowed view of the six strategy-specific payload fields.
/// Implemented by `RetrySchedulePost` and `RetrySchedulePut` so validators run on either.
trait StrategyFields {
    fn strategy(&self) -> RetryStrategy;
    fn max_retries(&self) -> i32;
    fn linear_delay(&self) -> Option<i32>;
    fn custom_intervals(&self) -> Option<&[i32]>;
    fn increasing_base_delay(&self) -> Option<i32>;
    fn increasing_wait_factor(&self) -> Option<f64>;
}

macro_rules! impl_strategy_fields {
    ($struct_name:ident) => {
        impl StrategyFields for $struct_name {
            fn strategy(&self) -> RetryStrategy {
                self.strategy
            }
            fn max_retries(&self) -> i32 {
                self.max_retries
            }
            fn linear_delay(&self) -> Option<i32> {
                self.linear_delay
            }
            fn custom_intervals(&self) -> Option<&[i32]> {
                self.custom_intervals.as_deref()
            }
            fn increasing_base_delay(&self) -> Option<i32> {
                self.increasing_base_delay
            }
            fn increasing_wait_factor(&self) -> Option<f64> {
                self.increasing_wait_factor
            }
        }
    };
}

impl_strategy_fields!(RetrySchedulePost);
impl_strategy_fields!(RetrySchedulePut);

/// Saturating Duration → i32 seconds conversion (bounds derive from clap-parsed humantime).
fn duration_to_i32_secs(duration: Duration) -> i32 {
    i32::try_from(duration.as_secs()).unwrap_or(i32::MAX)
}

/// Saturating Duration → i64 seconds conversion (bounds derive from clap-parsed humantime).
fn duration_to_i64_secs(duration: Duration) -> i64 {
    i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
}

/// Runtime bounds check against the instance-level limits from `State`. Rejects payloads
/// that passed validator attributes (anti-corruption) but breach the tighter business caps.
fn validate_against_limits(
    state: &crate::State,
    fields: &impl StrategyFields,
) -> Result<(), validator::ValidationErrors> {
    let mut errors = validator::ValidationErrors::new();

    if fields.max_retries() > state.retry_schedule_max_retries {
        let cap = state.retry_schedule_max_retries;
        errors.add("max_retries", range_error("max_retries", 1, i64::from(cap)));
    }

    match fields.strategy() {
        RetryStrategy::ExponentialIncreasing => {
            if let Some(base) = fields.increasing_base_delay() {
                let min = duration_to_i32_secs(state.retry_schedule_exponential_base_delay_min);
                let max = duration_to_i32_secs(state.retry_schedule_exponential_base_delay_max);
                if !(min..=max).contains(&base) {
                    errors.add(
                        FIELD_INCREASING_BASE_DELAY,
                        range_error(FIELD_INCREASING_BASE_DELAY, i64::from(min), i64::from(max)),
                    );
                }
            }
            if let Some(factor) = fields.increasing_wait_factor() {
                let min = state.retry_schedule_exponential_wait_factor_min;
                let max = state.retry_schedule_exponential_wait_factor_max;
                if factor < min || factor > max {
                    errors.add(
                        FIELD_INCREASING_WAIT_FACTOR,
                        range_error(FIELD_INCREASING_WAIT_FACTOR, min, max),
                    );
                }
            }
        }
        RetryStrategy::Linear => {
            if let Some(delay) = fields.linear_delay() {
                let min = duration_to_i32_secs(state.retry_schedule_min_single_delay);
                let max = duration_to_i32_secs(state.retry_schedule_max_single_delay);
                if !(min..=max).contains(&delay) {
                    errors.add(
                        FIELD_LINEAR_DELAY,
                        range_error(FIELD_LINEAR_DELAY, i64::from(min), i64::from(max)),
                    );
                }
            }
        }
        RetryStrategy::Custom => {
            if let Some(intervals) = fields.custom_intervals() {
                let min = duration_to_i32_secs(state.retry_schedule_min_single_delay);
                let max = duration_to_i32_secs(state.retry_schedule_max_single_delay);
                if let Some((index, _)) = intervals
                    .iter()
                    .enumerate()
                    .find(|(_, value)| !(min..=max).contains(value))
                {
                    errors.add(
                        FIELD_CUSTOM_INTERVALS,
                        range_error(
                            &format!("{FIELD_CUSTOM_INTERVALS}[{index}]"),
                            i64::from(min),
                            i64::from(max),
                        ),
                    );
                }
            }
        }
    }

    let single_max_secs = duration_to_i32_secs(state.retry_schedule_max_single_delay);
    let total_cap_secs = duration_to_i64_secs(state.retry_schedule_max_total_duration);
    if compute_total_duration(fields, single_max_secs) > total_cap_secs {
        errors.add(
            "total_duration",
            range_error("total_duration", 0, total_cap_secs),
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn range_error<T: std::fmt::Display>(field: &str, min: T, max: T) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("range");
    err.message = Some(std::borrow::Cow::Owned(format!(
        "{field} must be within {min}..={max}"
    )));
    err
}

/// Sum of retry delays in seconds. Per-retry delay pre-clamped to single_max_secs so
/// hostile inputs cannot overflow i64 before the total-cap check.
fn compute_total_duration(fields: &impl StrategyFields, single_max_secs: i32) -> i64 {
    match fields.strategy() {
        RetryStrategy::ExponentialIncreasing => {
            let (Some(base), Some(factor)) = (
                fields.increasing_base_delay(),
                fields.increasing_wait_factor(),
            ) else {
                return 0;
            };
            let cap_float = f64::from(single_max_secs);
            let mut total: i64 = 0;
            for index in 0..fields.max_retries().max(0) {
                let base_float = f64::from(base);
                let factor_power = factor.powi(index);
                let projected = base_float * factor_power;
                let clamped = projected.clamp(0.0, cap_float);
                // Pre-clamped finite in [0, single_max_secs ≤ i32::MAX]; NaN saturates to 0 via `as i64`.
                let term = clamped as i64;
                total = total.saturating_add(term);
            }
            total
        }
        RetryStrategy::Linear => match fields.linear_delay() {
            Some(delay) => i64::from(fields.max_retries()).saturating_mul(i64::from(delay)),
            None => 0,
        },
        RetryStrategy::Custom => fields
            .custom_intervals()
            .map(|values| {
                values
                    .iter()
                    .map(|value| i64::from(*value).min(i64::from(single_max_secs)))
                    .fold(0i64, |accumulator, value| accumulator.saturating_add(value))
            })
            .unwrap_or(0),
    }
}

#[api_v2_operation(
    summary = "List retry schedules",
    description = "List retry schedules available in an organization. Editors and viewers can read; only editors can create/update/delete.",
    operation_id = "retrySchedules.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<RetryScheduleListQuery>,
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

    let rows = query_as!(
        RetrySchedule,
        r#"
            select
                retry_schedule__id as "retry_schedule_id!",
                organization__id as "organization_id!",
                name as "name!",
                strategy as "strategy!: RetryStrategy",
                max_retries as "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at as "created_at!",
                updated_at as "updated_at!"
            from webhook.retry_schedule
            where organization__id = $1
            order by name asc
        "#,
        &organization_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(rows))
}

#[api_v2_operation(
    summary = "Get a retry schedule by its ID",
    description = "Returns a single retry schedule. Cross-org access collapses to 404.",
    operation_id = "retrySchedules.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = lookup_organization(&state.db, &retry_schedule_id).await?;

    // Auth failure collapses to NotFound so cross-org existence does not leak.
    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleGet {
            retry_schedule_id: &retry_schedule_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::NotFound);
    }

    let row = query_as!(
        RetrySchedule,
        r#"
            select
                retry_schedule__id as "retry_schedule_id!",
                organization__id as "organization_id!",
                name as "name!",
                strategy as "strategy!: RetryStrategy",
                max_retries as "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at as "created_at!",
                updated_at as "updated_at!"
            from webhook.retry_schedule
            where retry_schedule__id = $1
        "#,
        &retry_schedule_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    row.map(Json).ok_or(Hook0Problem::NotFound)
}

#[api_v2_operation(
    summary = "Create a retry schedule",
    description = "Create a retry schedule in the given organization. Rejects payloads that breach validator ranges or business bounds.",
    operation_id = "retrySchedules.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
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

    if let Err(errors) = body.validate() {
        return Err(Hook0Problem::Validation(errors));
    }
    if let Err(errors) = validate_against_limits(&state, &*body) {
        return Err(Hook0Problem::Validation(errors));
    }

    // Atomic INSERT with per-org quota: the subquery short-circuits when the org is at capacity.
    // Under concurrent creates two inserts may both see count=limit-1 and land — acceptable soft limit.
    let inserted_schedule = query_as!(
        RetrySchedule,
        r#"
            insert into webhook.retry_schedule
                (organization__id, name, strategy, max_retries,
                 custom_intervals, linear_delay,
                 increasing_base_delay, increasing_wait_factor)
            select $1, $2, $3, $4, $5, $6, $7, $8
            where (
                select count(*) from webhook.retry_schedule where organization__id = $1
            ) < $9
            returning
                retry_schedule__id as "retry_schedule_id!",
                organization__id as "organization_id!",
                name as "name!",
                strategy as "strategy!: RetryStrategy",
                max_retries as "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at as "created_at!",
                updated_at as "updated_at!"
        "#,
        &organization_id,
        body.name.trim(),
        body.strategy as RetryStrategy,
        body.max_retries,
        body.custom_intervals.as_deref(),
        body.linear_delay,
        body.increasing_base_delay,
        body.increasing_wait_factor,
        state.max_retry_schedules_per_organization,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    // None here means the quota WHERE clause short-circuited the INSERT. Name-uniqueness
    // violations surface as `Hook0Problem::RetryScheduleNameAlreadyExist` via `problems.rs`,
    // so this branch is reached only when the organization is already at the schedule cap.
    let Some(inserted_schedule) = inserted_schedule else {
        return Err(Hook0Problem::TooManyRetrySchedulesPerOrganization(
            state.max_retry_schedules_per_organization,
        ));
    };

    let event: Hook0ClientEvent = EventRetryScheduleCreated {
        organization_id: inserted_schedule.organization_id,
        retry_schedule_id: inserted_schedule.retry_schedule_id,
        name: inserted_schedule.name.to_owned(),
        strategy: inserted_schedule.strategy.to_string(),
        max_retries: inserted_schedule.max_retries,
        custom_intervals: inserted_schedule.custom_intervals.to_owned(),
        linear_delay: inserted_schedule.linear_delay,
        increasing_base_delay: inserted_schedule.increasing_base_delay,
        increasing_wait_factor: inserted_schedule.increasing_wait_factor,
    }
    .into();
    emit_retry_schedule_event(&state, event).await;

    Ok(CreatedJson(inserted_schedule))
}

#[api_v2_operation(
    summary = "Update a retry schedule",
    description = "Replaces the retry schedule definition. Same bounds as create.",
    operation_id = "retrySchedules.edit",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = lookup_organization(&state.db, &retry_schedule_id).await?;

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleEdit {
            retry_schedule_id: &retry_schedule_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::NotFound);
    }

    if let Err(errors) = body.validate() {
        return Err(Hook0Problem::Validation(errors));
    }
    if let Err(errors) = validate_against_limits(&state, &*body) {
        return Err(Hook0Problem::Validation(errors));
    }

    let updated = query_as!(
        RetrySchedule,
        r#"
            update webhook.retry_schedule
            set
                name = $2,
                strategy = $3,
                max_retries = $4,
                custom_intervals = $5,
                linear_delay = $6,
                increasing_base_delay = $7,
                increasing_wait_factor = $8,
                updated_at = statement_timestamp()
            where retry_schedule__id = $1
            returning
                retry_schedule__id as "retry_schedule_id!",
                organization__id as "organization_id!",
                name as "name!",
                strategy as "strategy!: RetryStrategy",
                max_retries as "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at as "created_at!",
                updated_at as "updated_at!"
        "#,
        &retry_schedule_id,
        body.name.trim(),
        body.strategy as RetryStrategy,
        body.max_retries,
        body.custom_intervals.as_deref(),
        body.linear_delay,
        body.increasing_base_delay,
        body.increasing_wait_factor,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    let Some(updated) = updated else {
        return Err(Hook0Problem::NotFound);
    };

    let event: Hook0ClientEvent = EventRetryScheduleUpdated {
        organization_id: updated.organization_id,
        retry_schedule_id: updated.retry_schedule_id,
        name: updated.name.to_owned(),
        strategy: updated.strategy.to_string(),
        max_retries: updated.max_retries,
        custom_intervals: updated.custom_intervals.to_owned(),
        linear_delay: updated.linear_delay,
        increasing_base_delay: updated.increasing_base_delay,
        increasing_wait_factor: updated.increasing_wait_factor,
    }
    .into();
    emit_retry_schedule_event(&state, event).await;

    Ok(Json(updated))
}

#[api_v2_operation(
    summary = "Delete a retry schedule",
    description = "Deletes a retry schedule. Subscriptions referencing it fall back to NULL (FK on-delete set null).",
    operation_id = "retrySchedules.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    retry_schedule_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = lookup_organization(&state.db, &retry_schedule_id).await?;

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::RetryScheduleDelete {
            retry_schedule_id: &retry_schedule_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::NotFound);
    }

    let deleted = query!(
        "delete from webhook.retry_schedule where retry_schedule__id = $1 returning retry_schedule__id",
        &retry_schedule_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if deleted.is_none() {
        return Err(Hook0Problem::NotFound);
    }

    let event: Hook0ClientEvent = EventRetryScheduleRemoved {
        organization_id,
        retry_schedule_id,
    }
    .into();
    emit_retry_schedule_event(&state, event).await;

    Ok(NoContent)
}

/// Owning organization for a retry schedule; 404 if missing.
async fn lookup_organization(
    db: &sqlx::PgPool,
    retry_schedule_id: &Uuid,
) -> Result<Uuid, Hook0Problem> {
    query_scalar!(
        "select organization__id from webhook.retry_schedule where retry_schedule__id = $1",
        retry_schedule_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)
}

/// Best-effort emit to the Hook0 client. Send errors become log lines so CRUD handlers
/// stay on the success path even when the outbound pipeline is degraded.
async fn emit_retry_schedule_event(state: &crate::State, event: Hook0ClientEvent) {
    let Some(hook0_client) = state.hook0_client.as_ref() else {
        return;
    };
    if let Err(e) = hook0_client.send_event(&event.mk_hook0_event()).await {
        error!("Hook0ClientError: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exponential_post(name: &str, max_retries: i32, base: i32, factor: f64) -> RetrySchedulePost {
        RetrySchedulePost {
            organization_id: Uuid::nil(),
            name: name.to_owned(),
            strategy: RetryStrategy::ExponentialIncreasing,
            max_retries,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(base),
            increasing_wait_factor: Some(factor),
        }
    }

    fn linear_post(name: &str, max_retries: i32, delay_seconds: i32) -> RetrySchedulePost {
        RetrySchedulePost {
            organization_id: Uuid::nil(),
            name: name.to_owned(),
            strategy: RetryStrategy::Linear,
            max_retries,
            custom_intervals: None,
            linear_delay: Some(delay_seconds),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    fn custom_post(name: &str, intervals: Vec<i32>) -> RetrySchedulePost {
        let max_retries = i32::try_from(intervals.len()).expect("test intervals fit in i32");
        RetrySchedulePost {
            organization_id: Uuid::nil(),
            name: name.to_owned(),
            strategy: RetryStrategy::Custom,
            max_retries,
            custom_intervals: Some(intervals),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    #[test]
    fn exponential_rejects_nan_factor() {
        let post = exponential_post("bad", 5, 60, f64::NAN);
        assert!(post.validate().is_err());
    }

    #[test]
    fn exponential_rejects_missing_base_delay() {
        let mut post = exponential_post("bad", 5, 60, 2.0);
        post.increasing_base_delay = None;
        assert!(post.validate().is_err());
    }

    #[test]
    fn linear_rejects_missing_delay() {
        let mut post = linear_post("bad", 5, 60);
        post.linear_delay = None;
        assert!(post.validate().is_err());
    }

    #[test]
    fn linear_rejects_extra_exponential_field() {
        let mut post = linear_post("bad", 5, 60);
        post.increasing_base_delay = Some(60);
        assert!(post.validate().is_err());
    }

    #[test]
    fn custom_rejects_length_mismatch() {
        let mut post = custom_post("bad", vec![10, 20, 30]);
        post.max_retries = 5;
        assert!(post.validate().is_err());
    }

    #[test]
    fn custom_rejects_empty_intervals() {
        let post = custom_post("bad", vec![]);
        // max_retries=0 is already rejected by #[validate(range(min=1))].
        assert!(post.validate().is_err());
    }

    #[test]
    fn compute_total_duration_clamps_per_retry() {
        let post = exponential_post("clamp", 10, 1, 10.0);
        let total = compute_total_duration(&post, 7 * 24 * 3600);
        assert!(total > 0);
        assert!(total <= 10 * (7 * 24 * 3600));
    }
}
