//! Background subsystem that auto-warns and auto-disables unhealthy webhook
//! subscriptions based on their rolling failure rate.
//!
//! Module graph:
//!
//!   ┌──────────┐     ┌──────────────┐     ┌───────────┐
//!   │  runner  │────▶│  evaluation  │────▶│  queries  │
//!   └────┬─────┘     └──────────────┘     └───────────┘
//!        │                                       ▲
//!        │           ┌───────────────┐           │
//!        └──────────▶│ state_machine │           │
//!                    └───────┬───────┘           │
//!                            │                   │
//!                            ▼                   │
//!                    ┌───────────────┐           │
//!                    │     types     │◀──────────┘
//!                    └───────────────┘
//!
//! - [`runner`]: background loop, per-tick orchestrator, and `PlannedAction`
//!   dispatch to the DB layer. Only place in the subsystem that holds a
//!   transaction scope.
//! - [`evaluation`]: pipeline façade (`run_subscription_health_monitor_tick`) that produces
//!   the list of subscriptions the state machine should judge.
//! - [`queries`]: SQL layer, split by domain (buckets, cursor, deltas,
//!   events, subscription_state).
//! - [`state_machine`]: pure decision function — takes a subscription's
//!   current state, returns a list of `PlannedAction`. No I/O.
//! - [`types`]: `HealthStatus` and `HealthEventCause` enums shared across
//!   the subsystem and the API layer.

pub mod errors;
mod evaluation;
mod queries;
mod runner;
mod state_machine;
pub mod types;

pub use runner::{SubscriptionHealthMonitorConfig, run_subscription_health_monitor};
