//! Integration tests for the subscription health evaluation pipeline.
//!
//! Split by behavioral focus rather than by the file the code lives in:
//! - [`bucket_tests`] — bucket population, closing, retention cleanup.
//! - [`window_tests`] — adaptive-window flow (two-pass warning then
//!   recovery/anti-flap evaluated across `failure_rate_evaluation_window`).
//! - [`threshold_tests`] — UNION behavior in
//!   `find_subscriptions_pending_health_evaluation` that keeps a
//!   previously-warned subscription in the candidate set even after its
//!   bucket failure rate drops, which is what lets the state machine fire
//!   the Resolved transition.
//! - [`cursor_tests`] — cursor advancement semantics.
//! - [`subscription_state_tests`] — failure_percent cache reset for
//!   recovered subscriptions.

mod bucket_tests;
mod cursor_tests;
mod subscription_state_tests;
mod threshold_tests;
mod window_tests;
