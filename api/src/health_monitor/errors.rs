//! Error types for the health monitor subsystem.

use thiserror::Error;

/// Errors from the health monitor evaluation and state machine.
#[derive(Debug, Error)]
pub enum HealthMonitorError {
    /// Only error type because side-effects (emails, Hook0 events) are best-effort
    /// and swallowed — all load-bearing failures are DB.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
