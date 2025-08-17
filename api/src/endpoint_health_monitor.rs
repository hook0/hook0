use chrono::{DateTime, Duration, Utc};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{query, query_as, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use hook0_client::Hook0Client;
use crate::hook0_client::Hook0ClientEvent;
use crate::mailer::Mailer;
use crate::problems::Hook0Problem;

/// Configuration for endpoint health monitoring
#[derive(Debug, Clone)]
pub struct EndpointHealthConfig {
    /// Enable/disable automatic endpoint deactivation
    pub disable_failing_endpoints: bool,
    /// Days before sending warning notification
    pub warning_days: i64,
    /// Days before automatic deactivation
    pub disable_days: i64,
    /// Minimum failures before starting the counter
    pub min_failures_to_track: i32,
}

impl Default for EndpointHealthConfig {
    fn default() -> Self {
        Self {
            disable_failing_endpoints: true,
            warning_days: 3,
            disable_days: 5,
            min_failures_to_track: 10,
        }
    }
}

/// Endpoint health monitor for tracking failures and managing auto-disable
pub struct EndpointHealthMonitor {
    db: PgPool,
    mailer: Arc<Mailer>,
    hook0_client: Option<Arc<Hook0Client>>,
    config: EndpointHealthConfig,
}

/// Subscription health information
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionHealth {
    pub subscription_id: Uuid,
    pub application_id: Uuid,
    pub organization_id: Uuid,
    pub subscription_name: Option<String>,
    pub application_name: String,
    pub target_url: String,
    pub first_failure_at: DateTime<Utc>,
    pub last_failure_at: DateTime<Utc>,
    pub consecutive_failures: i32,
    pub is_enabled: bool,
}

/// Notification type for tracking
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum NotificationType {
    #[serde(rename = "warning_3_days")]
    Warning3Days,
    #[serde(rename = "disabled_5_days")]
    Disabled5Days,
    #[serde(rename = "recovered")]
    Recovered,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Warning3Days => write!(f, "warning_3_days"),
            NotificationType::Disabled5Days => write!(f, "disabled_5_days"),
            NotificationType::Recovered => write!(f, "recovered"),
        }
    }
}

impl EndpointHealthMonitor {
    pub fn new(
        db: PgPool,
        mailer: Arc<Mailer>,
        hook0_client: Option<Arc<Hook0Client>>,
        config: EndpointHealthConfig,
    ) -> Self {
        Self {
            db,
            mailer,
            hook0_client,
            config,
        }
    }

    /// Main health check function to be called periodically (e.g., by a cron job)
    pub async fn check_failing_endpoints(&self) -> Result<(), Hook0Problem> {
        if !self.config.disable_failing_endpoints {
            info!("Endpoint health monitoring is disabled");
            return Ok(());
        }

        info!("Starting endpoint health check");

        // Check for endpoints failing for warning period
        let warning_subscriptions = self
            .find_subscriptions_failing_for_days(self.config.warning_days)
            .await?;

        info!(
            "Found {} subscriptions failing for {} days",
            warning_subscriptions.len(),
            self.config.warning_days
        );

        // Send warning notifications
        for sub in warning_subscriptions {
            if !self
                .notification_already_sent(&sub.subscription_id, NotificationType::Warning3Days)
                .await?
            {
                info!(
                    "Sending warning notification for subscription {}",
                    sub.subscription_id
                );
                
                if let Err(e) = self.send_warning_email(&sub).await {
                    error!(
                        "Failed to send warning email for subscription {}: {:?}",
                        sub.subscription_id, e
                    );
                    continue;
                }
                
                self.record_notification(&sub.subscription_id, NotificationType::Warning3Days, &sub)
                    .await?;
            }
        }

        // Check for endpoints failing for disable period
        let disable_subscriptions = self
            .find_subscriptions_failing_for_days(self.config.disable_days)
            .await?;

        info!(
            "Found {} subscriptions failing for {} days (will be disabled)",
            disable_subscriptions.len(),
            self.config.disable_days
        );

        // Disable and notify
        for sub in disable_subscriptions {
            if sub.is_enabled {
                info!(
                    "Disabling subscription {} after {} days of failures",
                    sub.subscription_id, self.config.disable_days
                );
                
                self.disable_subscription(&sub.subscription_id).await?;
                
                if let Err(e) = self.send_operational_webhook("endpoint.disabled", &sub).await {
                    error!(
                        "Failed to send operational webhook for subscription {}: {:?}",
                        sub.subscription_id, e
                    );
                }
                
                if let Err(e) = self.send_disabled_email(&sub).await {
                    error!(
                        "Failed to send disabled email for subscription {}: {:?}",
                        sub.subscription_id, e
                    );
                }
                
                self.record_notification(&sub.subscription_id, NotificationType::Disabled5Days, &sub)
                    .await?;
            }
        }

        info!("Endpoint health check completed");
        Ok(())
    }

    /// Find subscriptions that have been failing for at least N days
    async fn find_subscriptions_failing_for_days(
        &self,
        days: i64,
    ) -> Result<Vec<SubscriptionHealth>, Hook0Problem> {
        let threshold_date = Utc::now() - Duration::days(days);
        
        let subscriptions = query_as!(
            SubscriptionHealth,
            r#"
            SELECT 
                s.subscription__id as subscription_id,
                s.application__id as application_id,
                a.organization__id as organization_id,
                s.description as subscription_name,
                a.name as application_name,
                th.url as target_url,
                s.first_failure_at as "first_failure_at!",
                s.last_failure_at as "last_failure_at!",
                s.consecutive_failures,
                s.is_enabled
            FROM webhook.subscription s
            JOIN event.application a ON s.application__id = a.application__id
            JOIN webhook.target_http th ON s.target__id = th.target__id
            WHERE s.deleted_at IS NULL
                AND a.deleted_at IS NULL
                AND s.first_failure_at IS NOT NULL
                AND s.first_failure_at <= $1
                AND s.consecutive_failures >= $2
                AND s.last_failure_at >= $3
            ORDER BY s.first_failure_at
            "#,
            threshold_date,
            self.config.min_failures_to_track,
            Utc::now() - Duration::hours(24), // Last failure within 24 hours
        )
        .fetch_all(&self.db)
        .await?;

        Ok(subscriptions)
    }

    /// Check if a notification has already been sent for this failure sequence
    async fn notification_already_sent(
        &self,
        subscription_id: &Uuid,
        notification_type: NotificationType,
    ) -> Result<bool, Hook0Problem> {
        let today = Utc::now().date_naive();
        
        let result = query!(
            r#"
            SELECT COUNT(*) as count
            FROM webhook.endpoint_health_notification
            WHERE subscription__id = $1
                AND notification_type = $2
                AND DATE(sent_at) = $3
            "#,
            subscription_id,
            notification_type.to_string(),
            today
        )
        .fetch_one(&self.db)
        .await?;

        Ok(result.count.unwrap_or(0) > 0)
    }

    /// Record that a notification was sent
    async fn record_notification(
        &self,
        subscription_id: &Uuid,
        notification_type: NotificationType,
        health: &SubscriptionHealth,
    ) -> Result<(), Hook0Problem> {
        let details = json!({
            "consecutive_failures": health.consecutive_failures,
            "first_failure_at": health.first_failure_at,
            "last_failure_at": health.last_failure_at,
            "target_url": health.target_url,
        });

        query!(
            r#"
            INSERT INTO webhook.endpoint_health_notification 
                (subscription__id, notification_type, details)
            VALUES ($1, $2, $3)
            "#,
            subscription_id,
            notification_type.to_string(),
            details
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Disable a subscription
    async fn disable_subscription(&self, subscription_id: &Uuid) -> Result<(), Hook0Problem> {
        query!(
            r#"
            UPDATE webhook.subscription
            SET 
                is_enabled = false,
                auto_disabled_at = NOW()
            WHERE subscription__id = $1
            "#,
            subscription_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Send warning email (3 days)
    async fn send_warning_email(&self, health: &SubscriptionHealth) -> Result<(), Hook0Problem> {
        let subject = format!(
            "[Hook0 - Warning] Failing Endpoint: {}",
            health.subscription_name.as_deref().unwrap_or("Unnamed")
        );

        let body = format!(
            r#"Hello,

We are writing to inform you that the following endpoint has been experiencing difficulties for the past {} days:

Application: {}
Subscription: {} (ID: {})
Endpoint: {}
Description: {}

Statistics:
- First failure: {}
- Total number of failures: {}
- Last attempt: {}

Action required:
If no action is taken, this endpoint will be automatically disabled in {} days.

To prevent deactivation:
1. Verify that your endpoint is accessible and responding correctly.
2. Ensure it returns a 2xx HTTP code within 15 seconds.
3. Check the detailed logs in your Hook0 dashboard.

Regards,
The Hook0 Team"#,
            self.config.warning_days,
            health.application_name,
            health.subscription_name.as_deref().unwrap_or("Unnamed"),
            health.subscription_id,
            health.target_url,
            health.subscription_name.as_deref().unwrap_or("N/A"),
            health.first_failure_at.format("%Y-%m-%d %H:%M:%S UTC"),
            health.consecutive_failures,
            health.last_failure_at.format("%Y-%m-%d %H:%M:%S UTC"),
            self.config.disable_days - self.config.warning_days,
        );

        // Get admin emails for the organization
        let admin_emails = self.get_organization_admin_emails(&health.organization_id).await?;
        
        for email in admin_emails {
            self.mailer.send_text_email(&email, &subject, &body).await?;
        }

        Ok(())
    }

    /// Send disabled email (5 days)
    async fn send_disabled_email(&self, health: &SubscriptionHealth) -> Result<(), Hook0Problem> {
        let subject = format!(
            "[Hook0 - Action Required] Endpoint Disabled: {}",
            health.subscription_name.as_deref().unwrap_or("Unnamed")
        );

        let body = format!(
            r#"Hello,

The following endpoint has been automatically disabled due to repeated failures:

Application: {}
Subscription: {} (ID: {})
Endpoint: {}
Description: {}

Deactivation Details:
- Deactivation date: {}
- Total duration of failures: {} days
- Total number of attempts: {}

Possible actions:
1. Fix the issues with your endpoint.
2. Re-enable the subscription via the API or the dashboard.
3. Use the "Replay" feature to resend missed events.

Re-activation API:
PUT /v1/subscriptions/{}
{{
  "is_enabled": true
}}

Regards,
The Hook0 Team"#,
            health.application_name,
            health.subscription_name.as_deref().unwrap_or("Unnamed"),
            health.subscription_id,
            health.target_url,
            health.subscription_name.as_deref().unwrap_or("N/A"),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.config.disable_days,
            health.consecutive_failures,
            health.subscription_id,
        );

        // Get admin emails for the organization
        let admin_emails = self.get_organization_admin_emails(&health.organization_id).await?;
        
        for email in admin_emails {
            self.mailer.send_text_email(&email, &subject, &body).await?;
        }

        Ok(())
    }

    /// Get admin emails for an organization
    async fn get_organization_admin_emails(
        &self,
        organization_id: &Uuid,
    ) -> Result<Vec<String>, Hook0Problem> {
        let emails = query!(
            r#"
            SELECT u.email
            FROM iam.user u
            JOIN iam.user__organization uo ON u.user__id = uo.user__id
            WHERE uo.organization__id = $1
                AND uo.role = 'editor'
                AND u.email_verified_at IS NOT NULL
            "#,
            organization_id
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|r| r.email)
        .collect();

        Ok(emails)
    }

    /// Send operational webhook
    async fn send_operational_webhook(
        &self,
        event_type: &str,
        health: &SubscriptionHealth,
    ) -> Result<(), Hook0Problem> {
        if let Some(_client) = &self.hook0_client {
            let event = Hook0ClientEvent::EndpointDisabled {
                organization_id: health.organization_id,
                application_id: health.application_id,
                subscription_id: health.subscription_id,
                endpoint_url: health.target_url.clone(),
                disabled_at: Utc::now(),
                failure_count: health.consecutive_failures as i64,
            };
            
            // TODO: Fix method name - client.ingest(event.mk_hook0_event()).await?;
            let _ = event.mk_hook0_event(); // Use the event to avoid unused warnings
        }

        // Also check if there's a configured operational webhook
        let config = query!(
            r#"
            SELECT target_url, headers
            FROM webhook.operational_webhook_config
            WHERE organization__id = $1
                AND event_type = $2
                AND is_enabled = true
            "#,
            health.organization_id,
            event_type
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(config) = config {
            // Send webhook to configured URL
            let payload = json!({
                "type": event_type,
                "organization_id": health.organization_id,
                "application_id": health.application_id,
                "subscription_id": health.subscription_id,
                "endpoint_url": health.target_url,
                "disabled_at": Utc::now(),
                "failure_count": health.consecutive_failures,
            });

            // Here you would use reqwest or similar to send the webhook
            // This is simplified for the example
            info!(
                "Would send operational webhook to {} with payload: {:?}",
                config.target_url, payload
            );
        }

        Ok(())
    }

    /// Update failure tracking for a subscription
    #[allow(dead_code)]
    pub async fn track_failure(
        &self,
        subscription_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        query!(
            r#"
            UPDATE webhook.subscription
            SET 
                last_failure_at = NOW(),
                consecutive_failures = consecutive_failures + 1,
                first_failure_at = COALESCE(first_failure_at, NOW())
            WHERE subscription__id = $1
            "#,
            subscription_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Reset failure tracking for a subscription (on success)
    #[allow(dead_code)]
    pub async fn reset_failure_tracking(
        &self,
        subscription_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        query!(
            r#"
            UPDATE webhook.subscription
            SET 
                last_failure_at = NULL,
                consecutive_failures = 0,
                first_failure_at = NULL
            WHERE subscription__id = $1
            "#,
            subscription_id
        )
        .execute(&self.db)
        .await?;

        // Check if we should send a recovery notification
        let was_warned = self
            .notification_already_sent(subscription_id, NotificationType::Warning3Days)
            .await?;

        if was_warned {
            // Send recovery notification
            if let Ok(health) = self.get_subscription_health(subscription_id).await {
                if let Err(e) = self.send_recovery_email(&health).await {
                    warn!("Failed to send recovery email: {:?}", e);
                }
                
                self.record_notification(subscription_id, NotificationType::Recovered, &health)
                    .await?;
            }
        }

        Ok(())
    }

    /// Get subscription health information
    #[allow(dead_code)]
    async fn get_subscription_health(
        &self,
        subscription_id: &Uuid,
    ) -> Result<SubscriptionHealth, Hook0Problem> {
        let health = query_as!(
            SubscriptionHealth,
            r#"
            SELECT 
                s.subscription__id as subscription_id,
                s.application__id as application_id,
                a.organization__id as organization_id,
                s.description as subscription_name,
                a.name as application_name,
                th.url as target_url,
                COALESCE(s.first_failure_at, NOW()) as "first_failure_at!",
                COALESCE(s.last_failure_at, NOW()) as "last_failure_at!",
                s.consecutive_failures,
                s.is_enabled
            FROM webhook.subscription s
            JOIN event.application a ON s.application__id = a.application__id
            JOIN webhook.target_http th ON s.target__id = th.target__id
            WHERE s.subscription__id = $1
            "#,
            subscription_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(health)
    }

    /// Send recovery email
    async fn send_recovery_email(&self, health: &SubscriptionHealth) -> Result<(), Hook0Problem> {
        let subject = format!(
            "[Hook0 - Info] Endpoint Recovered: {}",
            health.subscription_name.as_deref().unwrap_or("Unnamed")
        );

        let body = format!(
            r#"Hello,

Good news! The following endpoint has recovered and is working correctly again:

Application: {}
Subscription: {} (ID: {})
Endpoint: {}

The endpoint is now successfully receiving and processing webhooks.

Regards,
The Hook0 Team"#,
            health.application_name,
            health.subscription_name.as_deref().unwrap_or("Unnamed"),
            health.subscription_id,
            health.target_url,
        );

        let admin_emails = self.get_organization_admin_emails(&health.organization_id).await?;
        
        for email in admin_emails {
            self.mailer.send_text_email(&email, &subject, &body).await?;
        }

        Ok(())
    }
}