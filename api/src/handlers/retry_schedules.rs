use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson};
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
use crate::iam::{authorize, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

pub const MAX_INTERVAL_SECS: i32 = 604_800; // 7 days

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Apiv2Schema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum RetryStrategy {
    Exponential,
    Linear,
    Custom,
}

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: RetryStrategy,
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[validate(schema(function = "RetrySchedulePost::validate_strategy"))]
pub struct RetrySchedulePost {
    pub organization_id: Uuid,
    #[validate(non_control_character, length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
}

impl RetrySchedulePost {
    fn validate_strategy(&self) -> Result<(), validator::ValidationError> {
        validate_strategy_fields(self.strategy, self.max_retries, self.linear_delay, self.custom_intervals.as_deref())
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
#[validate(schema(function = "RetrySchedulePut::validate_strategy"))]
pub struct RetrySchedulePut {
    #[validate(non_control_character, length(min = 2, max = 200))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(range(min = 1, max = 100))]
    pub max_retries: i32,
    pub custom_intervals: Option<Vec<i32>>,
    pub linear_delay: Option<i32>,
}

impl RetrySchedulePut {
    fn validate_strategy(&self) -> Result<(), validator::ValidationError> {
        validate_strategy_fields(self.strategy, self.max_retries, self.linear_delay, self.custom_intervals.as_deref())
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RetryScheduleQs {
    pub organization_id: Uuid,
}

// --- Validation helpers ---

fn strategy_error(message: &str) -> validator::ValidationError {
    let mut err = validator::ValidationError::new("strategy_fields");
    err.message = Some(std::borrow::Cow::Owned(message.to_owned()));
    err
}

fn require_none<T>(field: &str, value: &Option<T>, strategy: &str) -> Result<(), validator::ValidationError> {
    if value.is_some() {
        Err(strategy_error(&format!("{field} must be None for {strategy} strategy")))
    } else {
        Ok(())
    }
}

fn require_some<'a, T>(field: &str, value: &'a Option<T>, strategy: &str) -> Result<&'a T, validator::ValidationError> {
    value.as_ref().ok_or_else(|| strategy_error(&format!("{field} is required for {strategy} strategy")))
}

fn require_range(field: &str, value: i32, min: i32, max: i32) -> Result<(), validator::ValidationError> {
    if !(min..=max).contains(&value) {
        Err(strategy_error(&format!("{field} must be between {min} and {max}")))
    } else {
        Ok(())
    }
}

/// Cross-field validation for retry schedule strategy. Called by the validator
/// crate via `#[validate(schema(function = "..."))]` on Post and Put structs.
pub fn validate_strategy_fields(
    strategy: RetryStrategy,
    max_retries: i32,
    linear_delay: Option<i32>,
    custom_intervals: Option<&[i32]>,
) -> Result<(), validator::ValidationError> {
    match strategy {
        RetryStrategy::Exponential => {
            require_none("custom_intervals", &custom_intervals, "exponential")?;
            require_none("linear_delay", &linear_delay, "exponential")?;
        }
        RetryStrategy::Linear => {
            require_none("custom_intervals", &custom_intervals, "linear")?;
            let delay = require_some("linear_delay", &linear_delay, "linear")?;
            require_range("linear_delay", *delay, 1, MAX_INTERVAL_SECS)?;
        }
        RetryStrategy::Custom => {
            require_none("linear_delay", &linear_delay, "custom")?;
            let intervals = require_some("custom_intervals", &custom_intervals, "custom")?;
            if intervals.len() != max_retries as usize {
                return Err(strategy_error("custom_intervals length must equal max_retries"));
            }
            for (i, &val) in intervals.iter().enumerate() {
                if !(1..=MAX_INTERVAL_SECS).contains(&val) {
                    return Err(strategy_error(&format!("custom_intervals[{i}] must be between 1 and {MAX_INTERVAL_SECS}")));
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
                created_at, updated_at
            FROM webhook.retry_schedule
            WHERE organization__id = $1
            ORDER BY created_at ASC
        ",
    )
    .bind(&organization_id)
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
    let strategy_str = body.strategy.to_string();
    let schedule = sqlx::query_as::<_, RetrySchedule>(
        "
            INSERT INTO webhook.retry_schedule (organization__id, name, strategy, max_retries, custom_intervals, linear_delay)
            SELECT $1, $2, $3, $4, $5, $6
            WHERE (
                SELECT COUNT(*)
                FROM webhook.retry_schedule
                WHERE organization__id = $1
            ) < $7
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                created_at, updated_at
        ",
    )
    .bind(&organization_id)
    .bind(&body.name)
    .bind(&strategy_str)
    .bind(&body.max_retries)
    .bind(body.custom_intervals.as_deref())
    .bind(body.linear_delay)
    .bind(&max_per_org)
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match schedule {
        Some(s) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventRetryScheduleCreated {
                    organization_id,
                    retry_schedule_id: s.retry_schedule_id,
                    name: s.name.to_owned(),
                    strategy: s.strategy.to_string(),
                    max_retries: s.max_retries,
                    custom_intervals: s.custom_intervals.to_owned(),
                    linear_delay: s.linear_delay,
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
        None => Err(Hook0Problem::TooManyRetrySchedulesPerOrganization(state.max_retry_schedules_per_org)),
    }
}

#[api_v2_operation(
    summary = "Get a retry schedule",
    operation_id = "retry_schedules.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let schedule_id = schedule_id.into_inner();

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
                created_at, updated_at
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
                AND organization__id = $2
        ",
    )
    .bind(&schedule_id)
    .bind(&organization_id)
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
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
    body: Json<RetrySchedulePut>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let schedule_id = schedule_id.into_inner();

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

    let strategy_str = body.strategy.to_string();
    let schedule = sqlx::query_as::<_, RetrySchedule>(
        "
            UPDATE webhook.retry_schedule
            SET name = $3, strategy = $4, max_retries = $5,
                custom_intervals = $6, linear_delay = $7,
                updated_at = statement_timestamp()
            WHERE retry_schedule__id = $1
                AND organization__id = $2
            RETURNING
                retry_schedule__id AS retry_schedule_id,
                organization__id AS organization_id,
                name, strategy, max_retries, custom_intervals, linear_delay,
                created_at, updated_at
        ",
    )
    .bind(&schedule_id)
    .bind(&organization_id)
    .bind(&body.name)
    .bind(&strategy_str)
    .bind(&body.max_retries)
    .bind(body.custom_intervals.as_deref())
    .bind(body.linear_delay)
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
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    schedule_id: Path<Uuid>,
    qs: Query<RetryScheduleQs>,
) -> Result<Json<()>, Hook0Problem> {
    let organization_id = qs.organization_id;
    let schedule_id = schedule_id.into_inner();

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
    }

    let deleted = query_as!(
        DeletedRow,
        "
            DELETE FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
                AND organization__id = $2
            RETURNING name, strategy, max_retries, custom_intervals, linear_delay
        ",
        &schedule_id,
        &organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match deleted {
        Some(d) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventRetryScheduleRemoved {
                    organization_id,
                    retry_schedule_id: schedule_id,
                    name: d.name,
                    strategy: d.strategy,
                    max_retries: d.max_retries,
                    custom_intervals: d.custom_intervals,
                    linear_delay: d.linear_delay,
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
        None => Err(Hook0Problem::NotFound),
    }
}

/// Unit tests for strategy-dependent field validation logic.
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
        // max_retries=1 but empty intervals -> len mismatch
        let result = validate_strategy_fields(RetryStrategy::Custom, 1, None, Some(&[]));
        assert!(result.is_err());
    }
}
