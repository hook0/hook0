//! Health monitor side-effects: emails and Hook0 events.
//!
//! Dispatched after the database transaction commits so a rollback never
//! leaves phantom notifications. All side-effects are best-effort — failures
//! are logged but never propagated.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use lettre::{Address, message::Mailbox};
use sqlx::PgPool;
use tracing::warn;
use uuid::Uuid;

use hook0_client::Hook0Client;

use crate::hook0_client::{
    EventSubscriptionDisabled, Hook0ClientEvent, RetrySchedulePayload, SubscriptionDisabledPayload,
};
use crate::mailer::{Mail, Mailer};

use super::HealthMonitorConfig;
use super::evaluation::SubscriptionHealth;

/// Describes a side-effect (email / Hook0 event) to perform after the
/// transaction has been committed.
pub enum HealthAction {
    /// Emitted when a subscription's failure rate crosses warning_failure_percent
    /// for the first time (or again after cooldown expires).
    Warning(HealthActionInfo),
    /// Emitted when a subscription's failure rate crosses disable_failure_percent —
    /// the subscription has been disabled in the same transaction.
    Disabled(HealthActionInfo),
    /// Emitted when a previously warned subscription's failure rate drops back
    /// below warning_failure_percent — the endpoint recovered on its own.
    Recovered(HealthActionInfo),
}

/// Maps 1:1 to `HealthAction` but without payload — selects the email template.
pub enum EmailKind {
    Warning,
    Disabled,
    Recovered,
}

/// Data needed to send emails and Hook0 events outside the transaction.
pub struct HealthActionInfo {
    pub subscription_id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub application_name: Option<String>,
    pub description: Option<String>,
    pub target_url: String,
    pub failure_percent: f64,
    /// `Some` only for Disabled — the timestamp when is_enabled was flipped to false.
    pub disabled_at: Option<DateTime<Utc>>,
    /// Included so emails can show retry config (schedule name, strategy, delays).
    pub retry_schedule: Option<RetrySchedulePayload>,
}

impl HealthActionInfo {
    /// Builds a `HealthActionInfo` from a `SubscriptionHealth` row and an optional disabled timestamp.
    pub fn from_subscription(
        subscription: &SubscriptionHealth,
        disabled_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            subscription_id: subscription.subscription_id,
            organization_id: subscription.organization_id,
            application_id: subscription.application_id,
            application_name: subscription.application_name.clone(),
            description: subscription.description.clone(),
            target_url: subscription.target_url.clone(),
            failure_percent: subscription.failure_percent,
            disabled_at,
            retry_schedule: subscription
                .retry_schedule_id
                .map(|id| RetrySchedulePayload {
                    retry_schedule_id: id,
                    name: subscription.retry_schedule_name.clone().unwrap_or_default(),
                    strategy: subscription.retry_strategy.clone().unwrap_or_default(),
                    max_retries: subscription.retry_max_retries.unwrap_or(0),
                    custom_intervals: subscription.retry_custom_intervals.clone(),
                    linear_delay: subscription.retry_linear_delay,
                    increasing_base_delay: subscription.retry_increasing_base_delay,
                    increasing_wait_factor: subscription.retry_increasing_wait_factor,
                }),
        }
    }
}

/// Dispatches side-effects (emails and Hook0 events) for a list of health actions.
/// Called after the database transaction has been committed.
/// Failures are logged but never propagated — all side-effects are best-effort.
pub async fn dispatch_health_actions(
    actions: &[HealthAction],
    mailer: &Mailer,
    db: &PgPool,
    hook0_client: &Option<Hook0Client>,
    config: &HealthMonitorConfig,
) {
    for action in actions {
        let (action_info, kind) = match action {
            HealthAction::Warning(action_info) => (action_info, EmailKind::Warning),
            HealthAction::Disabled(action_info) => (action_info, EmailKind::Disabled),
            HealthAction::Recovered(action_info) => (action_info, EmailKind::Recovered),
        };

        send_email_to_organization(mailer, db, action_info, kind, config).await;

        // Only disabled subscriptions emit a Hook0 event — warnings and recoveries
        // are email-only to avoid noise on transient fluctuations.
        if let HealthAction::Disabled(action_info) = action
            && let Some(client) = hook0_client
        {
            let disabled_at = action_info.disabled_at.unwrap_or_else(Utc::now);
            let event = EventSubscriptionDisabled {
                subscription: SubscriptionDisabledPayload {
                    subscription_id: action_info.subscription_id,
                    organization_id: action_info.organization_id,
                    application_id: action_info.application_id,
                    description: action_info.description.clone(),
                    target: action_info.target_url.clone(),
                    disabled_at,
                },
                retry_schedule: action_info.retry_schedule.clone(),
            };
            // Hook0ClientEvent is our internal envelope type that wraps domain events
            // into the format expected by the Hook0 ingestion API (event type + payload).
            let hook0_event: Hook0ClientEvent = event.into();
            if let Err(e) = client.send_event(&hook0_event.mk_hook0_event()).await {
                warn!("Health monitor: failed to send subscription.disabled Hook0 event: {e}");
            }
        }
    }
}

/// Sends a health notification email to all users of the subscription's organization.
/// Failures are logged but never propagated — email delivery is best-effort.
async fn send_email_to_organization(
    mailer: &Mailer,
    db: &PgPool,
    action_info: &HealthActionInfo,
    kind: EmailKind,
    config: &HealthMonitorConfig,
) {
    let description = action_info
        .description
        .clone()
        .unwrap_or_else(|| action_info.subscription_id.to_string());
    let application_name = action_info
        .application_name
        .clone()
        .unwrap_or_else(|| action_info.application_id.to_string());
    let evaluation_window = humantime::format_duration(config.time_window).to_string();

    let mail = match kind {
        EmailKind::Warning => Mail::SubscriptionWarning {
            organization_id: action_info.organization_id,
            application_id: action_info.application_id,
            application_name,
            subscription_description: description,
            subscription_id: action_info.subscription_id,
            target_url: action_info.target_url.clone(),
            failure_percent: action_info.failure_percent,
            evaluation_window,
        },
        EmailKind::Disabled => Mail::SubscriptionDisabled {
            organization_id: action_info.organization_id,
            application_id: action_info.application_id,
            application_name,
            subscription_description: description,
            subscription_id: action_info.subscription_id,
            target_url: action_info.target_url.clone(),
            failure_percent: action_info.failure_percent,
            evaluation_window,
            disabled_at: action_info
                .disabled_at
                .unwrap_or_else(Utc::now)
                .to_rfc3339(),
        },
        EmailKind::Recovered => Mail::SubscriptionRecovered {
            organization_id: action_info.organization_id,
            application_id: action_info.application_id,
            application_name,
            subscription_description: description,
            subscription_id: action_info.subscription_id,
            target_url: action_info.target_url.clone(),
        },
    };

    struct OrganizationUser {
        first_name: String,
        last_name: String,
        email: String,
    }

    let users = match sqlx::query_as!(
        OrganizationUser,
        r#"
        SELECT u.first_name, u.last_name, u.email
        FROM iam.user u
        INNER JOIN iam.user__organization ou ON u.user__id = ou.user__id
        WHERE ou.organization__id = $1
        "#,
        action_info.organization_id,
    )
    .fetch_all(db)
    .await
    {
        Ok(users) => users,
        Err(e) => {
            warn!(
                "Health monitor: failed to query org users for email (org {}): {e}",
                action_info.organization_id
            );
            return;
        }
    };

    for user in users {
        let address = match Address::from_str(&user.email) {
            Ok(a) => a,
            Err(e) => {
                warn!("Health monitor: invalid email address {}: {e}", user.email);
                continue;
            }
        };

        let recipient = Mailbox::new(
            Some(format!("{} {}", user.first_name, user.last_name)),
            address,
        );

        if let Err(e) = mailer.send_mail(mail.clone(), recipient).await {
            warn!(
                "Health monitor: failed to send email to {}: {e}",
                user.email
            );
        }
    }
}
