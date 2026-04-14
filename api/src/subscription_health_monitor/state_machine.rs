//! Subscription health state machine.
//!
//! Pure decision function: evaluates a subscription's current failure rate
//! against its previous health status and returns a list of `PlannedAction`s
//! describing what the caller should do. This module has no I/O and no DB
//! access — persistence lives in `queries`, orchestration in `runner`.
//!
//! State transitions:
//!
//!   ┌─────────┐  failure ≥ warning & < disable   ┌─────────┐
//!   │  None   │─────────────────────────────────▶│ Warning │
//!   └────┬────┘                                  └────┬────┘
//!        │                                            │
//!        │ failure ≥ disable                          │ failure < warning
//!        ▼                                            ▼
//!   ┌──────────┐                                 ┌──────────┐
//!   │ Disabled │◀──── failure ≥ disable ─────────│ Resolved │
//!   └──────────┘                                 └──────────┘
//!
//!   Disabled                    → do nothing (user must re-enable manually).
//!   Resolved + within anti-flap → do nothing (avoids warning↔resolved
//!                                 oscillations in the audit trail when the
//!                                 failure rate hovers around the threshold).

use chrono::{DateTime, Duration as ChronoDuration, Utc};

use super::queries::SubscriptionHealth;
use super::runner::SubscriptionHealthMonitorConfig;
use super::types::HealthStatus;

/// Persistent side-effect the caller should apply, decided by the pure
/// `plan_health_actions` function. The caller is expected to dispatch
/// each action in order, inside a database transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum PlannedAction {
    /// Cache the current failure rate on the subscription row. Always
    /// emitted first so API consumers see the latest number on every tick.
    UpdateFailurePercent,
    /// Insert a `warning` health event row.
    EmitWarning,
    /// Insert a `resolved` health event row.
    EmitResolved,
    /// Disable the subscription (atomic UPDATE + INSERT).
    EmitDisabled,
}

/// Evaluates a single subscription's health and returns the list of planned
/// side effects. Pure — no DB, no I/O.
pub fn plan_health_actions(
    subscription: &SubscriptionHealth,
    config: &SubscriptionHealthMonitorConfig,
    now: DateTime<Utc>,
) -> Vec<PlannedAction> {
    let failure_percent = subscription.failure_percent;
    let warning_percent = f64::from(config.failure_percent_for_warning);
    let disable_percent = f64::from(config.failure_percent_for_disable);

    // Pre-computed once so the match arms below read as a single condition.
    // from_std fails only when std::Duration exceeds chrono's i64-millisecond
    // range (~292 billion years) — we fall back to zero, which effectively
    // disables the anti-flap.
    let anti_flap = ChronoDuration::from_std(config.anti_flap_window)
        .unwrap_or_else(|_| ChronoDuration::zero());

    // Always cache the current failure rate first so API consumers see the
    // latest number regardless of which transition branch we take next.
    let mut actions = vec![PlannedAction::UpdateFailurePercent];

    match subscription.last_health_status {
        // Already disabled — user must re-enable manually. We don't touch
        // the state again to avoid overriding a deliberate user action.
        Some(HealthStatus::Disabled) => {}

        // Recently resolved (within the anti-flap window) — skip to avoid
        // polluting the audit trail with warning→resolved→warning flips
        // when the failure rate oscillates around the threshold.
        Some(HealthStatus::Resolved)
            if subscription
                .last_health_at
                .is_some_and(|at| (now - at) < anti_flap) => {}

        // Already warned and still in the warning band (still bad but not
        // bad enough to disable) — nothing new to record.
        Some(HealthStatus::Warning)
            if failure_percent >= warning_percent && failure_percent < disable_percent => {}

        // Was warned but failure rate dropped below the warning threshold —
        // the endpoint recovered.
        Some(HealthStatus::Warning) if failure_percent < warning_percent => {
            actions.push(PlannedAction::EmitResolved);
        }

        // Was warned and failure rate climbed above the disable threshold —
        // shut it down.
        Some(HealthStatus::Warning) => {
            actions.push(PlannedAction::EmitDisabled);
        }

        // No previous health state, or resolved long enough ago that the
        // anti-flap window no longer applies: start from scratch.
        None | Some(HealthStatus::Resolved) if failure_percent >= disable_percent => {
            actions.push(PlannedAction::EmitDisabled);
        }
        None | Some(HealthStatus::Resolved) if failure_percent >= warning_percent => {
            actions.push(PlannedAction::EmitWarning);
        }
        None | Some(HealthStatus::Resolved) => {}
    }

    actions
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::Utc;
    use uuid::Uuid;

    use super::*;

    fn test_config() -> SubscriptionHealthMonitorConfig {
        SubscriptionHealthMonitorConfig {
            interval: Duration::from_secs(60),
            failure_percent_for_warning: 50,
            failure_percent_for_disable: 90,
            failure_rate_window: Duration::from_secs(3_600),
            min_deliveries: 10,
            anti_flap_window: Duration::from_secs(3_600),
            resolved_event_retention: Duration::from_secs(30 * 86_400),
            bucket_duration: Duration::from_secs(300),
            bucket_max_messages: 1_000,
            bucket_retention: Duration::from_secs(30 * 86_400),
            max_request_attempts_per_tick: 10_000,
        }
    }

    fn make_subscription_health(
        failure_percent: f64,
        last_status: Option<HealthStatus>,
        last_at: Option<DateTime<Utc>>,
    ) -> SubscriptionHealth {
        SubscriptionHealth {
            subscription_id: Uuid::nil(),
            failure_percent,
            last_health_status: last_status,
            last_health_at: last_at,
            last_health_cause: None,
            last_health_user_id: None,
        }
    }

    fn evaluate(
        failure_percent: f64,
        last_status: Option<HealthStatus>,
        last_at: Option<DateTime<Utc>>,
    ) -> Vec<PlannedAction> {
        let subscription = make_subscription_health(failure_percent, last_status, last_at);
        plan_health_actions(&subscription, &test_config(), Utc::now())
    }

    #[test]
    fn healthy_below_warning_does_nothing() {
        // No prior state + failure below warning -> only the UpdateFailurePercent
        // cache write, no event.
        let actions = evaluate(10.0, None, None);
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn healthy_crosses_warning_emits_warning() {
        // No prior state + failure crosses warning (but not disable) -> emit Warning.
        let actions = evaluate(60.0, None, None);
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
        // separate warning step first.
        let actions = evaluate(95.0, None, None);
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
        );
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn warning_recovers_when_failure_drops() {
        // Previous Warning + failure dropped below warning -> EmitResolved.
        let actions = evaluate(
            5.0,
            Some(HealthStatus::Warning),
            Some(Utc::now() - chrono::Duration::minutes(30)),
        );
        assert_eq!(
            actions,
            vec![
                PlannedAction::UpdateFailurePercent,
                PlannedAction::EmitResolved,
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
            );
            assert_eq!(
                actions,
                vec![PlannedAction::UpdateFailurePercent],
                "disabled should be terminal at failure {failure}"
            );
        }
    }

    #[test]
    fn resolved_within_anti_flap_does_not_re_warning() {
        // Resolved within the anti-flap window + new spike -> skip, even
        // though failure is above warning.
        let actions = evaluate(
            70.0,
            Some(HealthStatus::Resolved),
            Some(Utc::now() - chrono::Duration::minutes(10)),
        );
        assert_eq!(actions, vec![PlannedAction::UpdateFailurePercent]);
    }

    #[test]
    fn resolved_after_anti_flap_can_re_warning() {
        // Resolved long enough ago that the anti-flap window expired ->
        // same-as-fresh behavior, new Warning is emitted.
        let actions = evaluate(
            70.0,
            Some(HealthStatus::Resolved),
            Some(Utc::now() - chrono::Duration::hours(2)),
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
