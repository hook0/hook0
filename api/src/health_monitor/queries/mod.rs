//! SQL layer for the health monitor — cursor management, delta ingestion,
//! bucket upserts, suspect identification, failure rate computation, and the
//! state-machine side effects (insert health event, disable subscription,
//! cache failure percent).
//!
//! Split into sub-modules by responsibility:
//! - [`reads`] — read-only analytical queries (cursor, delta scan, suspect
//!   identification, failure rate computation).
//! - [`buckets`] — bucket lifecycle queries (upsert into open buckets, close
//!   full buckets).
//! - [`writes`] — health event insertion and subscription state writes
//!   (insert health event, disable subscription, cache failure percent).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::types::{HealthEventCause, HealthStatus};

mod buckets;
mod reads;
mod writes;

pub use buckets::*;
pub use reads::*;
pub use writes::*;

/// One row per subscription from the delta scan: how many deliveries completed
/// since the cursor, split into total vs failed.
#[derive(Debug)]
pub struct DeltaRow {
    pub subscription_id: Uuid,
    pub total: i64,
    pub failed: i64,
    /// The latest completion timestamp in this batch — used to advance the cursor
    /// after the transaction commits so the next tick skips these rows.
    pub max_completed_at: Option<DateTime<Utc>>,
}

/// All the data the state machine needs to decide whether to warn, disable,
/// or resolve a subscription: its failure rate and its latest health event.
#[derive(Debug)]
pub struct SubscriptionHealth {
    pub subscription_id: Uuid,
    pub failure_percent: f64,
    pub last_health_status: Option<HealthStatus>,
    pub last_health_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    pub last_health_cause: Option<HealthEventCause>,
    #[allow(dead_code)]
    pub last_health_user_id: Option<Uuid>,
}
