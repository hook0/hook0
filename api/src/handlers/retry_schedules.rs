//! Retry schedule CRUD with cross-field validator.
//!
//! End-user sees 422 with field errors on invalid payloads.
//!
//! - Flat `RetrySchedulePost` / `RetrySchedulePut` drive the `validator` crate.
//! - Business bounds come from `State`. Attribute ranges stay wide.
//! - Per-organization quota: atomic `INSERT … WHERE count(*) < $limit`.
//! - Soft cap under contention. One extra row acceptable.
//! - Status codes are uniform across handlers: authz failure returns 403, missing
//!   or cross-organization schedules return 404.

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use std::time::Duration;
use strum::{AsRefStr, Display, EnumString};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventRetryScheduleCreated, EventRetryScheduleRemoved, EventRetryScheduleUpdated,
    Hook0ClientEvent,
};
use crate::iam::{Action, authorize, get_retry_schedule_owner_organization};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

const FIELD_LINEAR_DELAY: &str = "linear_delay_secs";
const FIELD_CUSTOM_INTERVALS: &str = "custom_intervals_secs";
const FIELD_INCREASING_BASE_DELAY: &str = "increasing_base_delay_secs";
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
    AsRefStr,
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
    pub custom_intervals_secs: Option<Vec<i32>>,
    pub linear_delay_secs: Option<i32>,
    pub increasing_base_delay_secs: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create request. Business bounds enforced at runtime in `validate_against_limits`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate, Clone)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "RetrySchedulePost::validate_strategy"))]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(
        non_control_character,
        length(min = 1, max = 200),
        custom(function = "validate_name_trimmed")
    )]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    #[validate(custom(function = "validate_custom_intervals_range"))]
    pub custom_intervals_secs: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay_secs: Option<i32>,
    #[validate(range(min = 1, max = 604_800))]
    pub increasing_base_delay_secs: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Update request. Same rules as `RetrySchedulePost`, minus `organization_id`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate, Clone)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "RetrySchedulePut::validate_strategy"))]
pub struct RetrySchedulePut {
    #[validate(
        non_control_character,
        length(min = 1, max = 200),
        custom(function = "validate_name_trimmed")
    )]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 25))]
    pub max_retries: i32,
    #[validate(custom(function = "validate_custom_intervals_range"))]
    pub custom_intervals_secs: Option<Vec<i32>>,
    #[validate(range(min = 1, max = 604_800))]
    pub linear_delay_secs: Option<i32>,
    #[validate(range(min = 1, max = 604_800))]
    pub increasing_base_delay_secs: Option<i32>,
    pub increasing_wait_factor: Option<f64>,
}

/// Reject whitespace-only names up front so validation and the eventual trim at INSERT stay aligned.
fn validate_name_trimmed(name: &str) -> Result<(), validator::ValidationError> {
    let trimmed_length = name.trim().chars().count();
    if !(1..=200).contains(&trimmed_length) {
        let mut err = validator::ValidationError::new("length");
        err.message = Some(std::borrow::Cow::Borrowed(
            "name must contain 1 to 200 non-whitespace characters",
        ));
        return Err(err);
    }
    Ok(())
}

/// Attribute-level guard for custom interval arrays. Stays wide (1..=7 days per entry);
/// business bounds are enforced in `validate_against_limits`.
fn validate_custom_intervals_range(intervals: &[i32]) -> Result<(), validator::ValidationError> {
    if let Some((index, _)) = intervals
        .iter()
        .enumerate()
        .find(|(_, value)| !(1..=604_800).contains(*value))
    {
        return Err(range_error(
            &format!("{FIELD_CUSTOM_INTERVALS}[{index}]"),
            1,
            604_800,
        ));
    }
    Ok(())
}

macro_rules! impl_validate_strategy {
    ($payload:ty) => {
        impl $payload {
            fn validate_strategy(&self) -> Result<(), validator::ValidationError> {
                validate_strategy_fields(
                    self.strategy,
                    self.max_retries,
                    self.linear_delay_secs,
                    self.custom_intervals_secs.as_deref(),
                    self.increasing_base_delay_secs,
                    self.increasing_wait_factor,
                )
            }
        }
    };
}

impl_validate_strategy!(RetrySchedulePost);
impl_validate_strategy!(RetrySchedulePut);

/// Query string that scopes operations to one organization.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[serde(deny_unknown_fields)]
pub struct RetryScheduleQuery {
    pub organization_id: Uuid,
}

fn strategy_error(message: &str) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("strategy_fields");
    err.message = Some(std::borrow::Cow::Owned(message.to_owned()));
    err
}

fn range_error<T: std::fmt::Display>(field: &str, min: T, max: T) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("range");
    err.message = Some(std::borrow::Cow::Owned(format!(
        "{field} must be within {min}..={max}"
    )));
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
    linear_delay_secs: Option<i32>,
    custom_intervals_secs: Option<&[i32]>,
    increasing_base_delay_secs: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationError> {
    let strategy_name: &str = strategy.as_ref();
    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            require_none(
                FIELD_CUSTOM_INTERVALS,
                &custom_intervals_secs,
                strategy_name,
            )?;
            require_none(FIELD_LINEAR_DELAY, &linear_delay_secs, strategy_name)?;
            require_some(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay_secs,
                strategy_name,
            )?;
            let factor = require_some(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                strategy_name,
            )?;
            if !factor.is_finite() {
                return Err(strategy_error(
                    "increasing_wait_factor must be a finite number",
                ));
            }
        }
        RetryStrategy::Linear => {
            require_none(
                FIELD_CUSTOM_INTERVALS,
                &custom_intervals_secs,
                strategy_name,
            )?;
            require_none(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay_secs,
                strategy_name,
            )?;
            require_none(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                strategy_name,
            )?;
            require_some(FIELD_LINEAR_DELAY, &linear_delay_secs, strategy_name)?;
        }
        RetryStrategy::Custom => {
            require_none(FIELD_LINEAR_DELAY, &linear_delay_secs, strategy_name)?;
            require_none(
                FIELD_INCREASING_BASE_DELAY,
                &increasing_base_delay_secs,
                strategy_name,
            )?;
            require_none(
                FIELD_INCREASING_WAIT_FACTOR,
                &increasing_wait_factor,
                strategy_name,
            )?;
            let intervals = require_some(
                FIELD_CUSTOM_INTERVALS,
                &custom_intervals_secs,
                strategy_name,
            )?;
            let expected_length = usize::try_from(max_retries).unwrap_or(0);
            if intervals.len() != expected_length {
                return Err(strategy_error(
                    "custom_intervals_secs length must equal max_retries",
                ));
            }
        }
    }
    Ok(())
}

/// Runtime bounds check against the instance-level limits from `State`. Rejects payloads
/// that passed validator attributes (anti-corruption) but breach the tighter business caps.
/// Operates on raw fields so both `RetrySchedulePost` and `RetrySchedulePut` feed the same pass.
fn validate_against_limits(
    state: &crate::State,
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay_secs: Option<i32>,
    custom_intervals_secs: Option<&[i32]>,
    increasing_base_delay_secs: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationErrors> {
    let mut errors = validator::ValidationErrors::new();

    if max_retries > state.retry_schedule_max_retries {
        let cap = state.retry_schedule_max_retries;
        errors.add("max_retries", range_error("max_retries", 1, i64::from(cap)));
    }

    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            if let Some(base) = increasing_base_delay_secs {
                let candidate = Duration::from_secs(u64::try_from(base).unwrap_or(0));
                let min = state.retry_schedule_exponential_base_delay_min;
                let max = state.retry_schedule_exponential_base_delay_max;
                if !(min..=max).contains(&candidate) {
                    errors.add(
                        FIELD_INCREASING_BASE_DELAY,
                        range_error(FIELD_INCREASING_BASE_DELAY, min.as_secs(), max.as_secs()),
                    );
                }
            }
            if let Some(factor) = increasing_wait_factor {
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
            if let Some(delay) = linear_delay_secs {
                let candidate = Duration::from_secs(u64::try_from(delay).unwrap_or(0));
                let min = state.retry_schedule_min_single_delay;
                let max = state.retry_schedule_max_single_delay;
                if !(min..=max).contains(&candidate) {
                    errors.add(
                        FIELD_LINEAR_DELAY,
                        range_error(FIELD_LINEAR_DELAY, min.as_secs(), max.as_secs()),
                    );
                }
            }
        }
        RetryStrategy::Custom => {
            if let Some(intervals) = custom_intervals_secs {
                let min = state.retry_schedule_min_single_delay;
                let max = state.retry_schedule_max_single_delay;
                if let Some((index, _)) = intervals.iter().enumerate().find(|(_, value)| {
                    let candidate = Duration::from_secs(u64::try_from(**value).unwrap_or(0));
                    !(min..=max).contains(&candidate)
                }) {
                    errors.add(
                        FIELD_CUSTOM_INTERVALS,
                        range_error(
                            &format!("{FIELD_CUSTOM_INTERVALS}[{index}]"),
                            min.as_secs(),
                            max.as_secs(),
                        ),
                    );
                }
            }
        }
    }

    let single_max = state.retry_schedule_max_single_delay;
    let total_cap = state.retry_schedule_max_total_duration;
    let total = compute_total_duration(
        strategy,
        max_retries,
        linear_delay_secs,
        custom_intervals_secs,
        increasing_base_delay_secs,
        increasing_wait_factor,
        single_max,
    );
    if total > total_cap {
        errors.add(
            "total_duration_secs",
            range_error(
                "total_duration_secs",
                Duration::ZERO.as_secs(),
                total_cap.as_secs(),
            ),
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Sum of retry delays. Per-retry value pre-clamped to `single_max` so hostile
/// inputs cannot overflow before the total-cap check.
fn compute_total_duration(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay_secs: Option<i32>,
    custom_intervals_secs: Option<&[i32]>,
    increasing_base_delay_secs: Option<i32>,
    increasing_wait_factor: Option<f64>,
    single_max: Duration,
) -> Duration {
    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            let (Some(base), Some(factor)) = (increasing_base_delay_secs, increasing_wait_factor)
            else {
                return Duration::ZERO;
            };
            let cap_float = single_max.as_secs() as f64;
            let mut total = Duration::ZERO;
            for index in 0..max_retries.max(0) {
                let base_float = f64::from(base);
                let factor_power = factor.powi(index);
                let projected = base_float * factor_power;
                let clamped = projected.clamp(0.0, cap_float);
                // Clamp + NaN-safe: `as u64` saturates to 0 for NaN, cap_float for >cap.
                let term = Duration::from_secs(clamped as u64);
                total = total.saturating_add(term);
            }
            total
        }
        RetryStrategy::Linear => match linear_delay_secs {
            Some(delay) => {
                let per_retry =
                    Duration::from_secs(u64::try_from(delay).unwrap_or(0)).min(single_max);
                let count = u32::try_from(max_retries.max(0)).unwrap_or(0);
                per_retry.saturating_mul(count)
            }
            None => Duration::ZERO,
        },
        RetryStrategy::Custom => custom_intervals_secs
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        Duration::from_secs(u64::try_from(*value).unwrap_or(0)).min(single_max)
                    })
                    .fold(Duration::ZERO, Duration::saturating_add)
            })
            .unwrap_or(Duration::ZERO),
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
    qs: Query<RetryScheduleQuery>,
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
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                max_retries,
                custom_intervals as custom_intervals_secs,
                linear_delay as linear_delay_secs,
                increasing_base_delay as increasing_base_delay_secs,
                increasing_wait_factor,
                created_at,
                updated_at
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
    qs: Query<RetryScheduleQuery>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = qs.organization_id;

    // Derive the authz organization from the row itself rather than trusting the query string.
    // Unknown schedule → NotFound (no existence oracle). Mismatch between qs and the real
    // owner also collapses to NotFound for the same reason.
    let owner_organization_id =
        match get_retry_schedule_owner_organization(&state.db, &retry_schedule_id).await {
            Some(id) if id == organization_id => id,
            _ => return Err(Hook0Problem::NotFound),
        };

    if authorize(
        &biscuit,
        Some(owner_organization_id),
        Action::RetryScheduleGet {
            retry_schedule_id: &retry_schedule_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let row = query_as!(
        RetrySchedule,
        r#"
            select
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                max_retries,
                custom_intervals as custom_intervals_secs,
                linear_delay as linear_delay_secs,
                increasing_base_delay as increasing_base_delay_secs,
                increasing_wait_factor,
                created_at,
                updated_at
            from webhook.retry_schedule
            where retry_schedule__id = $1
              and organization__id = $2
        "#,
        &retry_schedule_id,
        &owner_organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)?;

    Ok(Json(row))
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
    if let Err(errors) = validate_against_limits(
        &state,
        body.strategy,
        body.max_retries,
        body.linear_delay_secs,
        body.custom_intervals_secs.as_deref(),
        body.increasing_base_delay_secs,
        body.increasing_wait_factor,
    ) {
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
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                max_retries,
                custom_intervals as custom_intervals_secs,
                linear_delay as linear_delay_secs,
                increasing_base_delay as increasing_base_delay_secs,
                increasing_wait_factor,
                created_at,
                updated_at
        "#,
        &organization_id,
        body.name.trim(),
        body.strategy as RetryStrategy,
        body.max_retries,
        body.custom_intervals_secs.as_deref(),
        body.linear_delay_secs,
        body.increasing_base_delay_secs,
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
        custom_intervals_secs: inserted_schedule.custom_intervals_secs.to_owned(),
        linear_delay_secs: inserted_schedule.linear_delay_secs,
        increasing_base_delay_secs: inserted_schedule.increasing_base_delay_secs,
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
    qs: Query<RetryScheduleQuery>,
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = qs.organization_id;

    let owner_organization_id =
        match get_retry_schedule_owner_organization(&state.db, &retry_schedule_id).await {
            Some(id) if id == organization_id => id,
            _ => return Err(Hook0Problem::NotFound),
        };

    if authorize(
        &biscuit,
        Some(owner_organization_id),
        Action::RetryScheduleEdit {
            retry_schedule_id: &retry_schedule_id,
        },
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
    if let Err(errors) = validate_against_limits(
        &state,
        body.strategy,
        body.max_retries,
        body.linear_delay_secs,
        body.custom_intervals_secs.as_deref(),
        body.increasing_base_delay_secs,
        body.increasing_wait_factor,
    ) {
        return Err(Hook0Problem::Validation(errors));
    }

    let updated = query_as!(
        RetrySchedule,
        r#"
            update webhook.retry_schedule
            set
                name = $3,
                strategy = $4,
                max_retries = $5,
                custom_intervals = $6,
                linear_delay = $7,
                increasing_base_delay = $8,
                increasing_wait_factor = $9,
                updated_at = statement_timestamp()
            where retry_schedule__id = $1
              and organization__id = $2
            returning
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                max_retries,
                custom_intervals as custom_intervals_secs,
                linear_delay as linear_delay_secs,
                increasing_base_delay as increasing_base_delay_secs,
                increasing_wait_factor,
                created_at,
                updated_at
        "#,
        &retry_schedule_id,
        &owner_organization_id,
        body.name.trim(),
        body.strategy as RetryStrategy,
        body.max_retries,
        body.custom_intervals_secs.as_deref(),
        body.linear_delay_secs,
        body.increasing_base_delay_secs,
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
        custom_intervals_secs: updated.custom_intervals_secs.to_owned(),
        linear_delay_secs: updated.linear_delay_secs,
        increasing_base_delay_secs: updated.increasing_base_delay_secs,
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
    qs: Query<RetryScheduleQuery>,
) -> Result<NoContent, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = qs.organization_id;

    let owner_organization_id =
        match get_retry_schedule_owner_organization(&state.db, &retry_schedule_id).await {
            Some(id) if id == organization_id => id,
            _ => return Err(Hook0Problem::NotFound),
        };

    if authorize(
        &biscuit,
        Some(owner_organization_id),
        Action::RetryScheduleDelete {
            retry_schedule_id: &retry_schedule_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let deleted = query!(
        "delete from webhook.retry_schedule where retry_schedule__id = $1 and organization__id = $2 returning retry_schedule__id",
        &retry_schedule_id,
        &owner_organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    if deleted.is_none() {
        return Err(Hook0Problem::NotFound);
    }

    let event: Hook0ClientEvent = EventRetryScheduleRemoved {
        organization_id: owner_organization_id,
        retry_schedule_id,
    }
    .into();
    emit_retry_schedule_event(&state, event).await;

    Ok(NoContent)
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

    fn exponential_fields(
        name: &str,
        max_retries: i32,
        base: i32,
        factor: f64,
    ) -> RetrySchedulePut {
        RetrySchedulePut {
            name: name.to_owned(),
            strategy: RetryStrategy::ExponentialIncreasing,
            max_retries,
            custom_intervals_secs: None,
            linear_delay_secs: None,
            increasing_base_delay_secs: Some(base),
            increasing_wait_factor: Some(factor),
        }
    }

    fn linear_fields(name: &str, max_retries: i32, delay_seconds: i32) -> RetrySchedulePut {
        RetrySchedulePut {
            name: name.to_owned(),
            strategy: RetryStrategy::Linear,
            max_retries,
            custom_intervals_secs: None,
            linear_delay_secs: Some(delay_seconds),
            increasing_base_delay_secs: None,
            increasing_wait_factor: None,
        }
    }

    fn custom_fields(name: &str, intervals: Vec<i32>) -> RetrySchedulePut {
        let max_retries = i32::try_from(intervals.len()).expect("test intervals fit in i32");
        RetrySchedulePut {
            name: name.to_owned(),
            strategy: RetryStrategy::Custom,
            max_retries,
            custom_intervals_secs: Some(intervals),
            linear_delay_secs: None,
            increasing_base_delay_secs: None,
            increasing_wait_factor: None,
        }
    }

    #[test]
    fn exponential_rejects_nan_factor() {
        let post = exponential_fields("bad", 5, 60, f64::NAN);
        assert!(post.validate().is_err());
    }

    #[test]
    fn exponential_rejects_missing_base_delay() {
        let mut post = exponential_fields("bad", 5, 60, 2.0);
        post.increasing_base_delay_secs = None;
        assert!(post.validate().is_err());
    }

    #[test]
    fn linear_rejects_missing_delay() {
        let mut post = linear_fields("bad", 5, 60);
        post.linear_delay_secs = None;
        assert!(post.validate().is_err());
    }

    #[test]
    fn linear_rejects_extra_exponential_field() {
        let mut post = linear_fields("bad", 5, 60);
        post.increasing_base_delay_secs = Some(60);
        assert!(post.validate().is_err());
    }

    #[test]
    fn custom_rejects_length_mismatch() {
        let mut post = custom_fields("bad", vec![10, 20, 30]);
        post.max_retries = 5;
        assert!(post.validate().is_err());
    }

    #[test]
    fn custom_rejects_empty_intervals() {
        let post = custom_fields("bad", vec![]);
        // max_retries=0 is already rejected by #[validate(range(min=1))].
        assert!(post.validate().is_err());
    }

    #[test]
    fn compute_total_duration_clamps_per_retry() {
        let post = exponential_fields("clamp", 10, 1, 10.0);
        let single_max = Duration::from_secs(7 * 24 * 3600);
        let total = compute_total_duration(
            post.strategy,
            post.max_retries,
            post.linear_delay_secs,
            post.custom_intervals_secs.as_deref(),
            post.increasing_base_delay_secs,
            post.increasing_wait_factor,
            single_max,
        );
        assert!(total > Duration::ZERO);
        assert!(total <= single_max.saturating_mul(10));
    }
}
