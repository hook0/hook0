//! Error types for the subscription health monitor subsystem.

use thiserror::Error;

/// Errors from the subscription health evaluation pipeline and state machine.
#[derive(Debug, Error)]
pub enum SubscriptionHealthError {
    /// Only error variant because all load-bearing failures in this
    /// subsystem are database operations inside a transaction.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}
