//! Subscription health state machine.
//!
//! Pure decision function: evaluates a subscription's current failure rate
//! against its previous health status and returns a list of `PlannedAction`s
//! describing what the caller should do. This module has no I/O and no DB
//! access — persistence lives in `queries.rs`, orchestration in `mod.rs`.
//!
//! State transitions:
//!   No previous state + high failure        -> Warning
//!   No previous state + very high failure   -> Disable (skip Warning, endpoint already broken)
//!   Warning + still failing (same band)     -> do nothing (already warned)
//!   Warning + even higher failure           -> Disable
//!   Warning + recovered (failure dropped)   -> Resolved (notify unless previous cause was manual)
//!   Disabled                                -> do nothing (user must re-enable manually)
//!   Resolved + within cooldown              -> do nothing (avoid email spam)

use chrono::{DateTime, Utc};

use super::HealthMonitorConfig;
use super::evaluation::SubscriptionHealth;
use super::types::{HealthEventCause, HealthStatus};

/// Persistent side-effect the caller should apply, decided by the pure
/// `evaluate_health_transition` function. The caller is expected to dispatch
/// each action in order, inside a database transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum PlannedAction {
    /// Cache the current failure rate on the subscription row. Always emitted
    /// first so the frontend can display the latest rate.
    UpdateFailurePercent,
    /// Insert a `warning` health event and queue a warning notification.
    EmitWarning,
    /// Insert a `resolved` health event. When `notify` is true the caller also
    /// queues a recovery notification; when false (previous cause was manual)
    /// the event is persisted silently to avoid redundant emails.
    EmitResolved { notify: bool },
    /// Disable the subscription (atomic UPDATE + INSERT) and queue a disabled
    /// notification if the transition actually happened.
    EmitDisabled,
}

/// Evaluates a single subscription's health and returns the list of planned
/// side effects. Pure — no DB, no I/O.
pub fn evaluate_health_transition(
    failure_percent: f64,
    last_status: Option<HealthStatus>,
    last_at: Option<DateTime<Utc>>,
    last_cause: Option<HealthEventCause>,
    config: &HealthMonitorConfig,
    now: DateTime<Utc>,
) -> Vec<PlannedAction> {
    let warning_percent = config.warning_failure_percent as f64;
    let disable_percent = config.disable_failure_percent as f64;

    // Always cache the current failure rate first so the UI sees the latest
    // number regardless of which transition branch we take next.
    let mut actions = vec![PlannedAction::UpdateFailurePercent];

    match last_status {
        // Already disabled by the health monitor — user must re-enable manually.
        // We don't touch it again to avoid overriding a deliberate user action.
        Some(HealthStatus::Disabled) => {}

        // Recently resolved (within cooldown period) — skip to avoid spamming
        // the user with warning -> resolved -> warning -> resolved emails.
        Some(HealthStatus::Resolved)
            if last_at.is_some_and(|at| {
                (now - at)
                    < chrono::Duration::from_std(config.warning_cooldown).unwrap_or_else(|_| {
                        // from_std fails when std::Duration exceeds chrono's
                        // i64-millisecond range (~292 billion years). Treat as
                        // zero so cooldown effectively disables itself.
                        chrono::Duration::zero()
                    })
            }) => {}

        // Already warned and failure rate is in the same range (still bad but
        // not bad enough to disable) — nothing new to tell the user.
        Some(HealthStatus::Warning)
            if failure_percent >= warning_percent && failure_percent < disable_percent => {}

        // Was warned but failure rate dropped below the warning threshold — the
        // endpoint recovered. Skip notification if the last event was a manual
        // API action (re-enable via API), since the user already knows.
        Some(HealthStatus::Warning) if failure_percent < warning_percent => {
            let notify = last_cause != Some(HealthEventCause::Manual);
            actions.push(PlannedAction::EmitResolved { notify });
        }

        // Was warned and failure rate climbed above the disable threshold —
        // shut it down.
        Some(HealthStatus::Warning) => {
            actions.push(PlannedAction::EmitDisabled);
        }

        // No previous health state (or resolved outside cooldown) and failure
        // rate is extremely high — disable immediately (no warning step).
        _ if failure_percent >= disable_percent => {
            actions.push(PlannedAction::EmitDisabled);
        }

        // No previous health state and failure rate crossed the warning
        // threshold — send a warning email so the user can investigate before
        // we disable.
        _ if failure_percent >= warning_percent => {
            actions.push(PlannedAction::EmitWarning);
        }

        // Failure rate below warning threshold — healthy, nothing to do.
        _ => {}
    }

    actions
}

/// Convenience wrapper: pulls the inputs out of a `SubscriptionHealth` row and
/// calls the pure function above. Keeps the caller in `mod.rs` terse.
pub fn plan_for_subscription(
    config: &HealthMonitorConfig,
    subscription: &SubscriptionHealth,
    now: DateTime<Utc>,
) -> Vec<PlannedAction> {
    evaluate_health_transition(
        subscription.failure_percent,
        subscription.last_health_status,
        subscription.last_health_at,
        subscription.last_health_cause,
        config,
        now,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_config() -> HealthMonitorConfig {
        HealthMonitorConfig {
            interval: Duration::from_secs(60),
            warning_failure_percent: 50,
            disable_failure_percent: 90,
            time_window: Duration::from_secs(3_600),
            min_sample_size: 10,
            warning_cooldown: Duration::from_secs(3_600),
            retention_period_days: 30,
            bucket_duration: Duration::from_secs(300),
            bucket_max_messages: 1_000,
            bucket_retention_days: 30,
            max_delta_rows_per_tick: 10_000,
        }
    }

    fn evaluate(
        failure_percent: f64,
        last_status: Option<HealthStatus>,
        last_at: Option<DateTime<Utc>>,
        last_cause: Option<HealthEventCause>,
    ) -> Vec<PlannedAction> {
        evaluate_health_transition(
            failure_percent,
            last_status,
            last_at,
            last_cause,
            &test_config(),
            Utc::now(),
        )
    }

    #[test]
    fn healthy_below_warning_does_nothing() {
        // No prior state + failure below warning -> only the UpdateFailurePercent
        // cache write, no event, no email.
        let actions = evaluate(10.0, None, None, None);
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn healthy_crosses_warning_emits_warning() {
        // No prior state + failure crosses warning (but not disable) -> emit Warning.
        let actions = evaluate(60.0, None, None, None);
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitWarning,
            ]
        );
    }

    #[test]
    fn healthy_crosses_disable_jumps_straight_to_disabled() {
        // No prior state + failure crosses disable threshold -> skip warning,
        // go straight to Disabled. A broken endpoint doesn't warrant a
        // separate warning email first.
        let actions = evaluate(95.0, None, None, None);
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitDisabled,
            ]
        );
    }

    #[test]
    fn warning_stays_warning_when_still_failing() {
        // Already in warning band and still in warning band -> no new event.
        let actions = evaluate(
            70.0,
            Some(HealthStatus::Warning),
            Some(Utc::now() - chrono::Duration::minutes(30)),
            Some(HealthEventCause::Auto),
        );
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn warning_recovers_when_failure_drops() {
        // Previous Warning (auto) + failure dropped below warning -> EmitResolved
        // with notify=true so the user gets a recovery email.
        let actions = evaluate(
            5.0,
            Some(HealthStatus::Warning),
            Some(Utc::now() - chrono::Duration::minutes(30)),
            Some(HealthEventCause::Auto),
        );
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitResolved { notify: true },
            ]
        );
    }

    #[test]
    fn warning_recovers_no_recovery_email_if_last_cause_manual() {
        // Previous Warning caused by a manual API action + failure dropped ->
        // persist Resolved but notify=false (user already knows).
        let actions = evaluate(
            5.0,
            Some(HealthStatus::Warning),
            Some(Utc::now() - chrono::Duration::minutes(30)),
            Some(HealthEventCause::Manual),
        );
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitResolved { notify: false },
            ]
        );
    }

    #[test]
    fn warning_escalates_to_disabled_when_failure_exceeds_disable() {
        // Previous Warning + failure climbed above disable threshold -> Disable.
        let actions = evaluate(
            95.0,
            Some(HealthStatus::Warning),
            Some(Utc::now() - chrono::Duration::minutes(30)),
            Some(HealthEventCause::Auto),
        );
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitDisabled,
            ]
        );
    }

    #[test]
    fn disabled_is_terminal_no_action_ever() {
        // Once Disabled, the state machine refuses to act regardless of current
        // failure percent. Only a manual re-enable can move the subscription
        // out of this state.
        for failure in [0.0, 30.0, 60.0, 95.0, 100.0] {
            let actions = evaluate(
                failure,
                Some(HealthStatus::Disabled),
                Some(Utc::now() - chrono::Duration::minutes(30)),
                Some(HealthEventCause::Auto),
            );
            assert_eq!(
                actions,
                vec![PlannedAction::UpdateFailurePercent],
                "disabled should be terminal at failure {failure}"
            );
        }
    }

    #[test]
    fn resolved_within_cooldown_does_not_re_warning() {
        // Resolved within the cooldown window + new spike -> skip, even though
        // failure is above warning. Protects users from warning/resolved churn.
        let actions = evaluate(
            70.0,
            Some(HealthStatus::Resolved),
            Some(Utc::now() - chrono::Duration::minutes(10)),
            Some(HealthEventCause::Auto),
        );
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn resolved_after_cooldown_can_re_warning() {
        // Resolved long enough ago that cooldown has expired -> same-as-fresh
        // behavior, new Warning is emitted.
        let actions = evaluate(
            70.0,
            Some(HealthStatus::Resolved),
            Some(Utc::now() - chrono::Duration::hours(2)),
            Some(HealthEventCause::Auto),
        );
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitWarning,
            ]
        );
    }
}
