use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{Action, authorize_for_organization};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

/// Retry strategy for webhook delivery
#[derive(Debug, Clone, Serialize, Deserialize, Apiv2Schema, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum RetryStrategy {
    /// Exponential backoff with increasing delays
    Exponential,
    /// Linear intervals with fixed delays
    Linear,
    /// Custom intervals defined by the user
    Custom,
}

impl std::fmt::Display for RetryStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryStrategy::Exponential => write!(f, "exponential"),
            RetryStrategy::Linear => write!(f, "linear"),
            RetryStrategy::Custom => write!(f, "custom"),
        }
    }
}

/// Retry schedule configuration for webhook delivery
#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct RetrySchedule {
    pub retry_schedule_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub strategy: RetryStrategy,
    /// Intervals in seconds between retry attempts
    pub intervals: Vec<i32>,
    /// Maximum number of retry attempts
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a retry schedule
#[derive(Debug, Deserialize, Validate, Apiv2Schema)]
pub struct RetryScheduleInput {
    pub organization_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    pub strategy: RetryStrategy,
    #[validate(length(min = 1, max = 100, message = "Intervals must have between 1 and 100 elements"))]
    pub intervals: Vec<i32>,
    #[validate(range(min = 1, max = 100, message = "Max attempts must be between 1 and 100"))]
    pub max_attempts: i32,
}

impl RetryScheduleInput {
    /// Validate that intervals are positive and reasonable
    pub fn validate_intervals(&self) -> Result<(), Hook0Problem> {
        for interval in &self.intervals {
            if *interval < 1 {
                return Err(Hook0Problem::EventInvalidJsonPayload(
                    "All intervals must be at least 1 second".to_string(),
                ));
            }
            if *interval > 604800 {
                // 1 week
                return Err(Hook0Problem::EventInvalidJsonPayload(
                    "Intervals cannot exceed 1 week (604800 seconds)".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Query parameters for listing retry schedules
#[derive(Debug, Deserialize, Apiv2Schema)]
pub struct RetrySchedulesQuery {
    pub organization_id: Uuid,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub after: Option<Uuid>,
}

fn default_limit() -> i64 {
    100
}

/// Create a new retry schedule
#[api_v2_operation(
    summary = "Create a retry schedule",
    description = "Create a new retry schedule configuration for webhook delivery",
    operation_id = "retry_schedules.create",
    consumes = "application/json",
    produces = "application/json",
    tags("RetrySchedules")
)]
pub async fn create(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    input: Json<RetryScheduleInput>,
) -> Result<CreatedJson<RetrySchedule>, Hook0Problem> {
    let organization_id = input.organization_id;
    
    // Authorize access to organization
    authorize_for_organization(&biscuit, &organization_id, Action::RetryScheduleWrite)?;
    
    // Validate input
    input.validate()?;
    input.validate_intervals()?;
    
    // Check if a schedule with the same name already exists
    let existing = query!(
        r#"
        SELECT retry_schedule__id
        FROM webhook.retry_schedule
        WHERE organization__id = $1 AND name = $2
        "#,
        organization_id,
        input.name
    )
    .fetch_optional(db.as_ref())
    .await?;
    
    if existing.is_some() {
        return Err(Hook0Problem::EventInvalidJsonPayload(
            "A retry schedule with this name already exists".to_string(),
        ));
    }
    
    // Create the retry schedule
    let retry_schedule = query_as!(
        RetrySchedule,
        r#"
        INSERT INTO webhook.retry_schedule (
            organization__id,
            name,
            strategy,
            intervals,
            max_attempts
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            retry_schedule__id as retry_schedule_id,
            organization__id as organization_id,
            name,
            strategy as "strategy: RetryStrategy",
            intervals,
            max_attempts,
            created_at,
            updated_at
        "#,
        organization_id,
        input.name,
        input.strategy.to_string(),
        &input.intervals,
        input.max_attempts
    )
    .fetch_one(db.as_ref())
    .await?;
    
    Ok(CreatedJson(retry_schedule))
}

/// List retry schedules for an organization
#[api_v2_operation(
    summary = "List retry schedules",
    description = "List all retry schedules for an organization",
    operation_id = "retry_schedules.list",
    produces = "application/json",
    tags("RetrySchedules")
)]
pub async fn list(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    query: Query<RetrySchedulesQuery>,
) -> Result<Json<Vec<RetrySchedule>>, Hook0Problem> {
    let organization_id = query.organization_id;
    
    // Authorize access to organization
    authorize_for_organization(&biscuit, &organization_id, Action::RetryScheduleRead)?;
    
    let schedules = if let Some(after) = query.after {
        query_as!(
            RetrySchedule,
            r#"
            SELECT 
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                intervals,
                max_attempts,
                created_at,
                updated_at
            FROM webhook.retry_schedule
            WHERE organization__id = $1 AND retry_schedule__id > $2
            ORDER BY retry_schedule__id
            LIMIT $3
            "#,
            organization_id,
            after,
            query.limit
        )
        .fetch_all(db.as_ref())
        .await?
    } else {
        query_as!(
            RetrySchedule,
            r#"
            SELECT 
                retry_schedule__id as retry_schedule_id,
                organization__id as organization_id,
                name,
                strategy as "strategy: RetryStrategy",
                intervals,
                max_attempts,
                created_at,
                updated_at
            FROM webhook.retry_schedule
            WHERE organization__id = $1
            ORDER BY retry_schedule__id
            LIMIT $2
            "#,
            organization_id,
            query.limit
        )
        .fetch_all(db.as_ref())
        .await?
    };
    
    Ok(Json(schedules))
}

/// Get a specific retry schedule
#[api_v2_operation(
    summary = "Get a retry schedule",
    description = "Get a specific retry schedule by ID",
    operation_id = "retry_schedules.get",
    produces = "application/json",
    tags("RetrySchedules")
)]
pub async fn get(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    schedule_id: Path<Uuid>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let schedule_id = schedule_id.into_inner();
    
    // Fetch the schedule and check organization access
    let schedule = query_as!(
        RetrySchedule,
        r#"
        SELECT 
            retry_schedule__id as retry_schedule_id,
            organization__id as organization_id,
            name,
            strategy as "strategy: RetryStrategy",
            intervals,
            max_attempts,
            created_at,
            updated_at
        FROM webhook.retry_schedule
        WHERE retry_schedule__id = $1
        "#,
        schedule_id
    )
    .fetch_optional(db.as_ref())
    .await?
    .ok_or(Hook0Problem::NotFound)?;
    
    // Authorize access to the organization
    authorize_for_organization(&biscuit, &schedule.organization_id, Action::RetryScheduleRead)?;
    
    Ok(Json(schedule))
}

/// Update a retry schedule
#[api_v2_operation(
    summary = "Update a retry schedule",
    description = "Update an existing retry schedule",
    operation_id = "retry_schedules.update",
    consumes = "application/json",
    produces = "application/json",
    tags("RetrySchedules")
)]
pub async fn update(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    schedule_id: Path<Uuid>,
    input: Json<RetryScheduleInput>,
) -> Result<Json<RetrySchedule>, Hook0Problem> {
    let schedule_id = schedule_id.into_inner();
    
    // Validate input
    input.validate()?;
    input.validate_intervals()?;
    
    // Fetch the existing schedule
    let existing = query!(
        r#"
        SELECT organization__id
        FROM webhook.retry_schedule
        WHERE retry_schedule__id = $1
        "#,
        schedule_id
    )
    .fetch_optional(db.as_ref())
    .await?
    .ok_or(Hook0Problem::NotFound)?;
    
    // Authorize access to the organization
    authorize_for_organization(&biscuit, &existing.organization__id, Action::RetryScheduleWrite)?;
    
    // Update the schedule
    let schedule = query_as!(
        RetrySchedule,
        r#"
        UPDATE webhook.retry_schedule
        SET 
            name = $2,
            strategy = $3,
            intervals = $4,
            max_attempts = $5,
            updated_at = statement_timestamp()
        WHERE retry_schedule__id = $1
        RETURNING 
            retry_schedule__id as retry_schedule_id,
            organization__id as organization_id,
            name,
            strategy as "strategy: RetryStrategy",
            intervals,
            max_attempts,
            created_at,
            updated_at
        "#,
        schedule_id,
        input.name,
        input.strategy.to_string(),
        &input.intervals,
        input.max_attempts
    )
    .fetch_one(db.as_ref())
    .await?;
    
    Ok(Json(schedule))
}

/// Delete a retry schedule
#[api_v2_operation(
    summary = "Delete a retry schedule",
    description = "Delete a retry schedule (subscriptions using it will fall back to default)",
    operation_id = "retry_schedules.delete",
    tags("RetrySchedules")
)]
pub async fn delete(
    _req: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    db: Data<PgPool>,
    schedule_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    let schedule_id = schedule_id.into_inner();
    
    // Fetch the schedule to check organization access
    let existing = query!(
        r#"
        SELECT organization__id
        FROM webhook.retry_schedule
        WHERE retry_schedule__id = $1
        "#,
        schedule_id
    )
    .fetch_optional(db.as_ref())
    .await?
    .ok_or(Hook0Problem::NotFound)?;
    
    // Authorize access to the organization
    authorize_for_organization(&biscuit, &existing.organization__id, Action::RetryScheduleWrite)?;
    
    // Check if any subscriptions are using this schedule
    let subscription_count = query!(
        r#"
        SELECT COUNT(*) as count
        FROM webhook.subscription
        WHERE retry_schedule__id = $1
        "#,
        schedule_id
    )
    .fetch_one(db.as_ref())
    .await?;
    
    if subscription_count.count.unwrap_or(0) > 0 {
        // Set subscriptions to null (they will use default)
        query!(
            r#"
            UPDATE webhook.subscription
            SET retry_schedule__id = NULL
            WHERE retry_schedule__id = $1
            "#,
            schedule_id
        )
        .execute(db.as_ref())
        .await?;
    }
    
    // Delete the schedule
    query!(
        r#"
        DELETE FROM webhook.retry_schedule
        WHERE retry_schedule__id = $1
        "#,
        schedule_id
    )
    .execute(db.as_ref())
    .await?;
    
    Ok(NoContent)
}

/// Compute the next retry delay based on retry schedule
pub fn compute_delay_from_schedule(
    strategy: &RetryStrategy,
    intervals: &[i32],
    max_attempts: i32,
    retry_count: i16,
) -> Option<std::time::Duration> {
    if retry_count >= max_attempts as i16 {
        return None;
    }
    
    let delay_seconds = match strategy {
        RetryStrategy::Exponential => {
            // Use provided intervals or calculate exponentially
            if let Some(&interval) = intervals.get(retry_count as usize) {
                interval
            } else if let Some(&last) = intervals.last() {
                // Use the last interval for attempts beyond the array
                last
            } else {
                // Fallback to default exponential calculation
                let base_delay = 5;
                let max_delay = 36000; // 10 hours
                std::cmp::min(base_delay * 2_i32.pow(retry_count as u32), max_delay)
            }
        }
        RetryStrategy::Linear => {
            // Use the first interval for all retries or a default
            intervals.first().copied().unwrap_or(300) // Default 5 minutes
        }
        RetryStrategy::Custom => {
            // Use the exact interval from the array
            if let Some(&interval) = intervals.get(retry_count as usize) {
                interval
            } else if let Some(&last) = intervals.last() {
                // Use the last interval for attempts beyond the array
                last
            } else {
                return None;
            }
        }
    };
    
    Some(std::time::Duration::from_secs(delay_seconds as u64))
}