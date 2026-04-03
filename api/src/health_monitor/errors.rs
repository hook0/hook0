use thiserror::Error;

/// Errors from the health monitor evaluation and state machine.
#[derive(Debug, Error)]
pub enum HealthMonitorError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
