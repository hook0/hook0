//! Retry schedule CRUD endpoints with cross-field validator.
//!
//! Caller-visible effect: 422 on invalid payload (validator errors), 404 on cross-org access.
//!
//! - Flat `RetrySchedulePost` / `RetrySchedulePut` use the `validator` crate for field + cross-field checks
//! - Business bounds come from `State` (CLI args / env); attribute ranges are anti-corruption only
//! - Per-org quota enforced via atomic `INSERT … WHERE count(*) < $limit` (soft — one extra row acceptable under contention)
//! - Auth failure on get/edit/delete collapses to 404 to close the cross-org existence oracle

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use strum::{Display, EnumString};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{Action, authorize};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

/// Retry pacing family that picks how delays between retries are computed.
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
/// enforced by the runtime check in `validate_against_state`.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
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

/// Query string that scopes list operations to one organization.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
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
/// or the runtime `validate_against_state` pass.
pub fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationError> {
    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            require_none(
                "custom_intervals",
                &custom_intervals,
                "exponential_increasing",
            )?;
            require_none("linear_delay", &linear_delay, "exponential_increasing")?;
            let _ = require_some(
                "increasing_base_delay",
                &increasing_base_delay,
                "exponential_increasing",
            )?;
            let factor = require_some(
                "increasing_wait_factor",
                &increasing_wait_factor,
                "exponential_increasing",
            )?;
            if factor.is_nan() {
                return Err(strategy_error(
                    "increasing_wait_factor must be a finite number",
                ));
            }
        }
        RetryStrategy::Linear => {
            require_none("custom_intervals", &custom_intervals, "linear")?;
            require_none("increasing_base_delay", &increasing_base_delay, "linear")?;
            require_none("increasing_wait_factor", &increasing_wait_factor, "linear")?;
            let _ = require_some("linear_delay", &linear_delay, "linear")?;
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
        }
    }
    Ok(())
}

/// Runtime bounds check against the API State config. Rejects payloads that passed
/// validator attributes (anti-corruption) but breach the tighter business limits.
fn validate_against_state(
    state: &crate::State,
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
) -> Result<(), validator::ValidationErrors> {
    let mut errors = validator::ValidationErrors::new();

    if max_retries > state.retry_schedule_max_retries {
        errors.add(
            "max_retries",
            range_error("max_retries", 1, state.retry_schedule_max_retries.into()),
        );
    }

    let single_min = state.retry_schedule_min_single_delay_secs;
    let single_max = state.retry_schedule_max_single_delay_secs;

    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            if let Some(base) = increasing_base_delay
                && (base < state.retry_schedule_exponential_base_delay_min_secs
                    || base > state.retry_schedule_exponential_base_delay_max_secs)
            {
                errors.add(
                    "increasing_base_delay",
                    range_error(
                        "increasing_base_delay",
                        state.retry_schedule_exponential_base_delay_min_secs.into(),
                        state.retry_schedule_exponential_base_delay_max_secs.into(),
                    ),
                );
            }
            if let Some(factor) = increasing_wait_factor
                && (factor < state.retry_schedule_exponential_wait_factor_min
                    || factor > state.retry_schedule_exponential_wait_factor_max)
            {
                errors.add(
                    "increasing_wait_factor",
                    range_error(
                        "increasing_wait_factor",
                        state.retry_schedule_exponential_wait_factor_min as i64,
                        state.retry_schedule_exponential_wait_factor_max as i64,
                    ),
                );
            }
        }
        RetryStrategy::Linear => {
            if let Some(delay) = linear_delay
                && !(single_min..=single_max).contains(&delay)
            {
                errors.add(
                    "linear_delay",
                    range_error("linear_delay", single_min.into(), single_max.into()),
                );
            }
        }
        RetryStrategy::Custom => {
            if let Some(intervals) = custom_intervals {
                for (index, value) in intervals.iter().enumerate() {
                    if !(single_min..=single_max).contains(value) {
                        errors.add(
                            "custom_intervals",
                            range_error(
                                &format!("custom_intervals[{index}]"),
                                single_min.into(),
                                single_max.into(),
                            ),
                        );
                        break;
                    }
                }
            }
        }
    }

    let total = compute_total_duration(
        strategy,
        max_retries,
        linear_delay,
        custom_intervals,
        increasing_base_delay,
        increasing_wait_factor,
        single_max,
    );
    if total > state.retry_schedule_max_total_duration_secs {
        errors.add(
            "total_duration",
            range_error(
                "total_duration",
                0,
                state.retry_schedule_max_total_duration_secs,
            ),
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn range_error(field: &str, min: i64, max: i64) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("range");
    err.message = Some(std::borrow::Cow::Owned(format!(
        "{field} must be within {min}..={max}"
    )));
    err
}

/// Sum of retry delays in seconds. Per-retry delay pre-clamped to single_max
/// so hostile inputs cannot overflow i64 before the total-cap check.
fn compute_total_duration(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
    single_max: i32,
) -> i64 {
    match strategy {
        RetryStrategy::ExponentialIncreasing => {
            let (Some(base), Some(factor)) = (increasing_base_delay, increasing_wait_factor) else {
                return 0;
            };
            let cap = f64::from(single_max);
            let mut total: i64 = 0;
            for index in 0..max_retries.max(0) {
                let term = (f64::from(base) * factor.powi(index)).clamp(0.0, cap) as i64;
                total = total.saturating_add(term);
            }
            total
        }
        RetryStrategy::Linear => match linear_delay {
            Some(delay) => i64::from(max_retries).saturating_mul(i64::from(delay)),
            None => 0,
        },
        RetryStrategy::Custom => custom_intervals
            .map(|values| {
                values
                    .iter()
                    .map(|value| i64::from(*value).min(i64::from(single_max)))
                    .fold(0i64, |acc, value| acc.saturating_add(value))
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
    if let Err(errors) = validate_against_state(
        &state,
        body.strategy,
        body.max_retries,
        body.linear_delay,
        body.custom_intervals.as_deref(),
        body.increasing_base_delay,
        body.increasing_wait_factor,
    ) {
        return Err(Hook0Problem::Validation(errors));
    }

    let name = body.name.trim();

    // Atomic INSERT with per-org quota: the subquery short-circuits when the org is at capacity.
    // Under concurrent creates two inserts may both see count=limit-1 and land — acceptable soft limit.
    let created = query_as!(
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
        name,
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

    created
        .map(CreatedJson)
        .ok_or(Hook0Problem::TooManyRetrySchedulesPerOrganization(
            state.max_retry_schedules_per_organization,
        ))
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
    if let Err(errors) = validate_against_state(
        &state,
        body.strategy,
        body.max_retries,
        body.linear_delay,
        body.custom_intervals.as_deref(),
        body.increasing_base_delay,
        body.increasing_wait_factor,
    ) {
        return Err(Hook0Problem::Validation(errors));
    }

    let name = body.name.trim();

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
        name,
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

    updated.map(Json).ok_or(Hook0Problem::NotFound)
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

    Ok(NoContent)
}

/// Owning organization for a retry schedule; 404 if missing.
async fn lookup_organization(
    db: &sqlx::PgPool,
    retry_schedule_id: &Uuid,
) -> Result<Uuid, Hook0Problem> {
    sqlx::query_scalar!(
        "select organization__id from webhook.retry_schedule where retry_schedule__id = $1",
        retry_schedule_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)
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

    fn linear_post(name: &str, max_retries: i32, delay: i32) -> RetrySchedulePost {
        RetrySchedulePost {
            organization_id: Uuid::nil(),
            name: name.to_owned(),
            strategy: RetryStrategy::Linear,
            max_retries,
            custom_intervals: None,
            linear_delay: Some(delay),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        }
    }

    fn custom_post(name: &str, intervals: Vec<i32>) -> RetrySchedulePost {
        let max_retries = intervals.len() as i32;
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
    fn exponential_ok_passes_validator_attributes() {
        let post = exponential_post("good", 5, 60, 2.0);
        assert!(post.validate().is_ok());
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
    fn name_length_enforced_by_validator_attribute() {
        let mut post = linear_post(&"x".repeat(201), 3, 60);
        assert!(post.validate().is_err());
        post.name = String::new();
        assert!(post.validate().is_err());
    }

    #[test]
    fn compute_total_duration_clamps_per_retry() {
        let total = compute_total_duration(
            RetryStrategy::ExponentialIncreasing,
            10,
            None,
            None,
            Some(1),
            Some(10.0),
            7 * 24 * 3600,
        );
        assert!(total > 0);
        assert!(total <= 10 * (7 * 24 * 3600));
    }
}
