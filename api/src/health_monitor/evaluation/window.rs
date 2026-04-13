//! Adaptive-window state machine flow tests: two-pass warning then
//! recovery/cooldown evaluated across the `time_window`.
//!
//! This sub-module currently hosts no production code. The actual flow logic
//! lives in [`super::fetch_subscription_health_stats`] and the state machine
//! in [`crate::health_monitor::state_machine`]. Its sibling test file
//! [`super::window_tests`] exercises that flow end-to-end.
