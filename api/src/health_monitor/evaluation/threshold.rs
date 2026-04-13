//! Threshold-driven suspect tracking responsibility.
//!
//! The threshold comparison logic itself (failure_percent vs
//! `warning_failure_percent` / `disable_failure_percent`) lives in
//! [`crate::health_monitor::queries::find_suspects`]. This sub-module currently
//! hosts no production code; its sibling test file [`super::threshold_tests`]
//! exercises the UNION behavior that keeps a previously-warned subscription in
//! the suspect set even after its bucket failure rate drops — that is what
//! lets the state machine fire the Recovered transition.
