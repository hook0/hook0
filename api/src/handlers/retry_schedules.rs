//! Retry schedule CRUD endpoints + payload validator.
//!
//! Caller-visible effect: full CRUD on `/retry_schedules`; invalid payloads get 400 InvalidPayload, cross-org access collapses to 404.
//!
//! - Entity, strategy enum, discriminated-union payload, business caps
//! - `validate_payload` + `compute_total_duration` reject out-of-bounds pre-persist
//! - Handlers (list/get/create/edit/delete) with per-org advisory-locked quota
//! - Unique-name constraint surfaces as RetryScheduleNameConflict (409)

use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::query_as;
use uuid::Uuid;

use crate::iam::{Action, authorize};
use crate::openapi::OaBiscuit;
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
    // Validate on the trimmed form so persisted == validated (insert calls .trim() too).
    let trimmed_len = name.trim().len();
    if trimmed_len == 0 {
        return Err(invalid(
            "Name must not be empty or whitespace-only.".to_owned(),
        ));
    }
    if !(MIN_NAME_LEN..=MAX_NAME_LEN).contains(&trimmed_len) {
        return Err(invalid(format!(
            "Name length ({trimmed_len}) must be within {MIN_NAME_LEN}..={MAX_NAME_LEN}."
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
            // Compare on usize to avoid silent i32 truncation on massive payloads.
            let len = intervals.len();
            if len == 0 || len > MAX_RETRIES as usize {
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
    Hook0Problem::InvalidPayload { reason }
}

/// Query string for list scoped by organization.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ListQs {
    organization_id: Uuid,
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
    qs: Query<ListQs>,
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
            SELECT
                retry_schedule__id AS "retry_schedule_id!",
                organization__id AS "organization_id!",
                name AS "name!",
                strategy AS "strategy!: Strategy",
                max_retries AS "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at AS "created_at!",
                updated_at AS "updated_at!"
            FROM webhook.retry_schedule
            WHERE organization__id = $1
            ORDER BY name ASC
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
    description = "Returns a single retry schedule. The caller must belong to the owning organization.",
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
    let organization_id = lookup_org(&state.db, &retry_schedule_id).await?;

    // Collapse cross-org auth denial to NotFound so existence doesn't leak.
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
            SELECT
                retry_schedule__id AS "retry_schedule_id!",
                organization__id AS "organization_id!",
                name AS "name!",
                strategy AS "strategy!: Strategy",
                max_retries AS "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at AS "created_at!",
                updated_at AS "updated_at!"
            FROM webhook.retry_schedule
            WHERE retry_schedule__id = $1
        "#,
        &retry_schedule_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    row.map(Json).ok_or(Hook0Problem::NotFound)
}

/// Create payload — organization_id on top of `RetrySchedulePayload`.
#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetryScheduleCreatePayload {
    pub organization_id: Uuid,
    #[serde(flatten)]
    pub payload: RetrySchedulePayload,
}

#[api_v2_operation(
    summary = "Create a retry schedule",
    description = "Creates a retry schedule in the given organization. Rejects payloads breaching business bounds (max 15 retries, 7d total duration, per-retry 1s..=7d).",
    operation_id = "retrySchedules.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Retry Schedules Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<RetryScheduleCreatePayload>,
) -> Result<CreatedJson<RetrySchedule>, Hook0Problem> {
    let RetryScheduleCreatePayload {
        organization_id,
        payload,
    } = body.into_inner();

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

    validate_payload(&payload)?;
    let fields = payload_to_db_fields(&payload);

    // Serialize concurrent creates in the same org via a transaction-scoped advisory lock.
    // Lock is released on commit/rollback; ordinary inserts elsewhere are unaffected.
    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    sqlx::query!(
        "SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))",
        organization_id.to_string(),
    )
    .execute(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    let created = query_as!(
        RetrySchedule,
        r#"
            INSERT INTO webhook.retry_schedule
                (organization__id, name, strategy, max_retries,
                 custom_intervals, linear_delay,
                 increasing_base_delay, increasing_wait_factor)
            SELECT $1, $2, $3, $4, $5, $6, $7, $8
            WHERE (
                SELECT count(*) FROM webhook.retry_schedule WHERE organization__id = $1
            ) < $9
            RETURNING
                retry_schedule__id AS "retry_schedule_id!",
                organization__id AS "organization_id!",
                name AS "name!",
                strategy AS "strategy!: Strategy",
                max_retries AS "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at AS "created_at!",
                updated_at AS "updated_at!"
        "#,
        &organization_id,
        fields.name.trim(),
        fields.strategy as Strategy,
        fields.max_retries,
        fields.custom_intervals.as_deref(),
        fields.linear_delay,
        fields.increasing_base_delay,
        fields.increasing_wait_factor,
        MAX_PER_ORG,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    tx.commit().await.map_err(Hook0Problem::from)?;

    created
        .map(CreatedJson)
        .ok_or(Hook0Problem::TooManyRetrySchedulesPerOrganization(
            MAX_PER_ORG,
        ))
}

#[api_v2_operation(
    summary = "Update a retry schedule",
    description = "Replaces the retry schedule definition. Same business bounds as create.",
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
    body: Json<RetrySchedulePayload>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let retry_schedule_id = retry_schedule_id.into_inner();
    let organization_id = lookup_org(&state.db, &retry_schedule_id).await?;

    // Collapse cross-org auth denial to NotFound so existence doesn't leak.
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

    let payload = body.into_inner();
    validate_payload(&payload)?;
    let fields = payload_to_db_fields(&payload);

    let updated = query_as!(
        RetrySchedule,
        r#"
            UPDATE webhook.retry_schedule
            SET
                name = $2,
                strategy = $3,
                max_retries = $4,
                custom_intervals = $5,
                linear_delay = $6,
                increasing_base_delay = $7,
                increasing_wait_factor = $8,
                updated_at = statement_timestamp()
            WHERE retry_schedule__id = $1
            RETURNING
                retry_schedule__id AS "retry_schedule_id!",
                organization__id AS "organization_id!",
                name AS "name!",
                strategy AS "strategy!: Strategy",
                max_retries AS "max_retries!",
                custom_intervals,
                linear_delay,
                increasing_base_delay,
                increasing_wait_factor,
                created_at AS "created_at!",
                updated_at AS "updated_at!"
        "#,
        &retry_schedule_id,
        fields.name.trim(),
        fields.strategy as Strategy,
        fields.max_retries,
        fields.custom_intervals.as_deref(),
        fields.linear_delay,
        fields.increasing_base_delay,
        fields.increasing_wait_factor,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    updated.map(Json).ok_or(Hook0Problem::NotFound)
}

#[api_v2_operation(
    summary = "Delete a retry schedule",
    description = "Deletes a retry schedule. Subscriptions still referencing it are set to NULL (FK on-delete set null).",
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
    let organization_id = lookup_org(&state.db, &retry_schedule_id).await?;

    // Collapse cross-org auth denial to NotFound so existence doesn't leak.
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

    let deleted = sqlx::query!(
        "DELETE FROM webhook.retry_schedule WHERE retry_schedule__id = $1 RETURNING retry_schedule__id",
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
async fn lookup_org(db: &sqlx::PgPool, retry_schedule_id: &Uuid) -> Result<Uuid, Hook0Problem> {
    sqlx::query_scalar!(
        "SELECT organization__id FROM webhook.retry_schedule WHERE retry_schedule__id = $1",
        retry_schedule_id,
    )
    .fetch_optional(db)
    .await
    .map_err(Hook0Problem::from)?
    .ok_or(Hook0Problem::NotFound)
}

/// DB-shaped view of a payload. Strategy-specific columns null where unused.
struct DbFields<'a> {
    name: &'a str,
    strategy: Strategy,
    max_retries: i32,
    custom_intervals: Option<Vec<i32>>,
    linear_delay: Option<i32>,
    increasing_base_delay: Option<i32>,
    increasing_wait_factor: Option<f64>,
}

fn payload_to_db_fields(payload: &RetrySchedulePayload) -> DbFields<'_> {
    match payload {
        RetrySchedulePayload::ExponentialIncreasing {
            name,
            max_retries,
            base_delay,
            wait_factor,
        } => DbFields {
            name,
            strategy: Strategy::ExponentialIncreasing,
            max_retries: *max_retries,
            custom_intervals: None,
            linear_delay: None,
            increasing_base_delay: Some(*base_delay),
            increasing_wait_factor: Some(*wait_factor),
        },
        RetrySchedulePayload::Linear {
            name,
            max_retries,
            delay,
        } => DbFields {
            name,
            strategy: Strategy::Linear,
            max_retries: *max_retries,
            custom_intervals: None,
            linear_delay: Some(*delay),
            increasing_base_delay: None,
            increasing_wait_factor: None,
        },
        RetrySchedulePayload::Custom { name, intervals } => DbFields {
            // Custom strategy derives max_retries from the intervals array length.
            name,
            strategy: Strategy::Custom,
            max_retries: intervals.len() as i32,
            custom_intervals: Some(intervals.clone()),
            linear_delay: None,
            increasing_base_delay: None,
            increasing_wait_factor: None,
        },
    }
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
