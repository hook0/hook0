# Health Notification Emails - Diff

## api/src/mailer.rs

```rust
@@ -1,17 +1,18 @@
 use html2text::from_read;
 use lettre::message::{Mailbox, MultiPart};
 use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
 use std::string::String;
 use std::time::Duration;
 use tracing::{info, warn};
 use url::Url;
+use uuid::Uuid;

 use crate::problems::Hook0Problem;
```

```rust
@@ -41,44 +42,94 @@ pub enum Mail {
         current_events_per_day: i32,
         events_per_days_limit: i32,
         extra_variables: Vec<(String, String)>,
     },
     QuotaEventsPerDayReached {
         pricing_url_hash: String,
         current_events_per_day: i32,
         events_per_days_limit: i32,
         extra_variables: Vec<(String, String)>,
     },
+    SubscriptionWarning {
+        organization_id: Uuid,
+        application_id: Uuid,
+        application_name: String,
+        subscription_description: String,
+        subscription_id: Uuid,
+        target_url: String,
+        failure_percent: f64,
+        evaluation_window: String,
+    },
+    SubscriptionDisabled {
+        organization_id: Uuid,
+        application_id: Uuid,
+        application_name: String,
+        subscription_description: String,
+        subscription_id: Uuid,
+        target_url: String,
+        failure_percent: f64,
+        evaluation_window: String,
+        disabled_at: String,
+    },
+    SubscriptionResolved {
+        organization_id: Uuid,
+        application_id: Uuid,
+        application_name: String,
+        subscription_description: String,
+        subscription_id: Uuid,
+        target_url: String,
+    },
 }

 impl Mail {
     pub fn template(&self) -> &'static str {
         match self {
             Mail::VerifyUserEmail { .. } => include_str!("mail_templates/verify_user_email.mjml"),
             Mail::ResetPassword { .. } => include_str!("mail_templates/reset_password.mjml"),
             Mail::QuotaEventsPerDayWarning { .. } => {
                 include_str!("mail_templates/quotas/events_per_day_warning.mjml")
             }
             Mail::QuotaEventsPerDayReached { .. } => {
                 include_str!("mail_templates/quotas/events_per_day_reached.mjml")
             }
+            Mail::SubscriptionWarning { .. } => {
+                include_str!("mail_templates/subscriptions/warning.mjml")
+            }
+            Mail::SubscriptionDisabled { .. } => {
+                include_str!("mail_templates/subscriptions/disabled.mjml")
+            }
+            Mail::SubscriptionResolved { .. } => {
+                include_str!("mail_templates/subscriptions/resolved.mjml")
+            }
         }
     }

     pub fn subject(&self) -> String {
         match self {
             Mail::VerifyUserEmail { .. } => "[Hook0] Verify your email address".to_owned(),
             Mail::ResetPassword { .. } => "[Hook0] Reset your password".to_owned(),
             Mail::QuotaEventsPerDayWarning { .. } => "[Hook0] Quota Warning".to_owned(),
             Mail::QuotaEventsPerDayReached { .. } => "[Hook0] Quota Reached".to_owned(),
+            Mail::SubscriptionWarning {
+                subscription_description,
+                ..
+            } => format!("[Hook0] Subscription failing: {subscription_description}"),
+            Mail::SubscriptionDisabled {
+                subscription_description,
+                ..
+            } => format!("[Hook0] Subscription disabled: {subscription_description}"),
+            Mail::SubscriptionResolved {
+                subscription_description,
+                ..
+            } => format!("[Hook0] Subscription resolved: {subscription_description}"),
         }
     }
```

```rust
@@ -118,20 +169,75 @@ impl Mail {
                         current_events_per_day.to_string(),
                     ),
                     (
                         "events_per_days_limit".to_owned(),
                         events_per_days_limit.to_string(),
                     ),
                 ];
                 vars.extend(extra_variables.clone());
                 vars
             }
+            Mail::SubscriptionWarning {
+                organization_id,
+                application_id,
+                application_name,
+                subscription_description,
+                subscription_id,
+                target_url,
+                failure_percent,
+                evaluation_window,
+            } => vec![
+                ("organization_id".to_owned(), organization_id.to_string()),
+                ("application_id".to_owned(), application_id.to_string()),
+                ("application_name".to_owned(), application_name.to_owned()),
+                ("subscription_description".to_owned(), subscription_description.to_owned()),
+                ("subscription_id".to_owned(), subscription_id.to_string()),
+                ("target_url".to_owned(), target_url.to_owned()),
+                ("failure_percent".to_owned(), format!("{:.1}", failure_percent)),
+                ("evaluation_window".to_owned(), evaluation_window.to_owned()),
+            ],
+            Mail::SubscriptionDisabled {
+                organization_id,
+                application_id,
+                application_name,
+                subscription_description,
+                subscription_id,
+                target_url,
+                failure_percent,
+                evaluation_window,
+                disabled_at,
+            } => vec![
+                ("organization_id".to_owned(), organization_id.to_string()),
+                ("application_id".to_owned(), application_id.to_string()),
+                ("application_name".to_owned(), application_name.to_owned()),
+                ("subscription_description".to_owned(), subscription_description.to_owned()),
+                ("subscription_id".to_owned(), subscription_id.to_string()),
+                ("target_url".to_owned(), target_url.to_owned()),
+                ("failure_percent".to_owned(), format!("{:.1}", failure_percent)),
+                ("evaluation_window".to_owned(), evaluation_window.to_owned()),
+                ("disabled_at".to_owned(), disabled_at.to_owned()),
+            ],
+            Mail::SubscriptionResolved {
+                organization_id,
+                application_id,
+                application_name,
+                subscription_description,
+                subscription_id,
+                target_url,
+            } => vec![
+                ("organization_id".to_owned(), organization_id.to_string()),
+                ("application_id".to_owned(), application_id.to_string()),
+                ("application_name".to_owned(), application_name.to_owned()),
+                ("subscription_description".to_owned(), subscription_description.to_owned()),
+                ("subscription_id".to_owned(), subscription_id.to_string()),
+                ("target_url".to_owned(), target_url.to_owned()),
+            ],
         }
     }
```

```rust
@@ -246,20 +352,49 @@ mod tests {
                 current_events_per_day: 0,
                 events_per_days_limit: 0,
                 extra_variables: vec![("test".to_owned(), "test".to_owned())],
             },
             Mail::QuotaEventsPerDayReached {
                 pricing_url_hash: "test".to_owned(),
                 current_events_per_day: 0,
                 events_per_days_limit: 0,
                 extra_variables: vec![("test".to_owned(), "test".to_owned())],
             },
+            Mail::SubscriptionWarning {
+                organization_id: Uuid::nil(),
+                application_id: Uuid::nil(),
+                application_name: "Test App".to_owned(),
+                subscription_description: "Test Subscription".to_owned(),
+                subscription_id: Uuid::nil(),
+                target_url: "https://example.com/webhook".to_owned(),
+                failure_percent: 85.5,
+                evaluation_window: "1 hour".to_owned(),
+            },
+            Mail::SubscriptionDisabled {
+                organization_id: Uuid::nil(),
+                application_id: Uuid::nil(),
+                application_name: "Test App".to_owned(),
+                subscription_description: "Test Subscription".to_owned(),
+                subscription_id: Uuid::nil(),
+                target_url: "https://example.com/webhook".to_owned(),
+                failure_percent: 95.0,
+                evaluation_window: "1 hour".to_owned(),
+                disabled_at: "2024-01-15T10:30:00Z".to_owned(),
+            },
+            Mail::SubscriptionResolved {
+                organization_id: Uuid::nil(),
+                application_id: Uuid::nil(),
+                application_name: "Test App".to_owned(),
+                subscription_description: "Test Subscription".to_owned(),
+                subscription_id: Uuid::nil(),
+                target_url: "https://example.com/webhook".to_owned(),
+            },
         ];
```

---

## api/src/main.rs

```rust
@@ -1178,20 +1178,21 @@ async fn main() -> anyhow::Result<()> {
             config.website_url,
             config.app_url.clone(),
             config.support_email_address.clone(),
         )
         .await
         .expect("Could not initialize mailer; check SMTP configuration");

         if config.enable_subscription_health_monitor {
             let subscription_health_db = housekeeping_pool.clone();
             let subscription_health_semaphore = housekeeping_semaphore.clone();
+            let subscription_health_mailer = mailer.clone();
             let subscription_health_config =
                 subscription_health_monitor::SubscriptionHealthMonitorConfig {
```

```rust
@@ -1201,20 +1202,21 @@ async fn main() -> anyhow::Result<()> {
                     bucket_max_request_attempts: config
                         .subscription_health_monitor_bucket_max_request_attempts,
                     bucket_retention: config.subscription_health_monitor_bucket_retention,
                     request_attempt_scan_cap_per_tick: config
                         .subscription_health_monitor_request_attempt_scan_cap_per_tick,
                 };
             actix_web::rt::spawn(async move {
                 subscription_health_monitor::run_subscription_health_monitor(
                     &subscription_health_semaphore,
                     &subscription_health_db,
+                    &subscription_health_mailer,
                     &subscription_health_config,
                 )
                 .await;
             });
         }
```

---

## api/src/subscription_health_monitor.rs

```rust
@@ -1,20 +1,25 @@
+use std::collections::HashMap;
+use std::str::FromStr;
+use std::time::{Duration, Instant};
+
 use chrono::{DateTime, Duration as ChronoDuration, Utc};
+use lettre::{Address, message::Mailbox};
 use paperclip::actix::Apiv2Schema;
 use serde::Serialize;
 use sqlx::{PgPool, query, query_as, query_scalar};
-use std::collections::HashMap;
-use std::time::{Duration, Instant};
 use tokio::sync::Semaphore;
 use tracing::{debug, error, info, warn};
 use uuid::Uuid;

+use crate::mailer::{Mail, Mailer};
+
 #[derive(Clone)]
 pub struct SubscriptionHealthMonitorConfig {
```

```rust
@@ -40,50 +45,70 @@ pub enum HealthStatus {
 #[serde(rename_all = "lowercase")]
 pub enum HealthEventCause {
     Auto,
     Manual,
 }

 #[derive(Debug)]
 struct SubscriptionHealth {
     subscription_id: Uuid,
+    organization_id: Uuid,
+    application_id: Uuid,
+    application_name: String,
+    description: Option<String>,
+    target_url: String,
     failure_percent: f64,
     last_health_status: Option<HealthStatus>,
     last_health_at: Option<DateTime<Utc>>,
     #[allow(dead_code)]
     last_health_cause: Option<HealthEventCause>,
     #[allow(dead_code)]
     last_health_user_id: Option<Uuid>,
 }

+/// Info collected during transaction for sending emails after commit.
+#[derive(Debug, Clone)]
+struct HealthEmailAction {
+    status: HealthStatus,
+    subscription_id: Uuid,
+    organization_id: Uuid,
+    application_id: Uuid,
+    application_name: String,
+    description: Option<String>,
+    target_url: String,
+    failure_percent: f64,
+    disabled_at: Option<DateTime<Utc>>,
+}
+
 const CLEANUP_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

-/// Evaluates subscription failure rates, emits health events.
+/// Evaluates subscription failure rates, emits health events, sends notification emails.
 pub async fn run_subscription_health_monitor(
     housekeeping_semaphore: &Semaphore,
     db: &PgPool,
+    mailer: &Mailer,
     config: &SubscriptionHealthMonitorConfig,
 ) {
```

```rust
@@ -142,25 +167,29 @@ enum SubscriptionAction {

 const ADVISORY_LOCK_ID: i64 = 42_000_001;

 const TICK_STATEMENT_TIMEOUT: &str = "5min";

-/// Entire tick runs in one transaction with advisory lock.
+/// Phase 1: transaction with advisory lock evaluates health and emits events.
+/// Phase 2: sends notification emails after commit (best-effort).
 async fn run_one_tick(
     db: &PgPool,
+    mailer: Option<&Mailer>,
     config: &SubscriptionHealthMonitorConfig,
 ) -> Result<(), sqlx::Error> {
+    let mut email_actions: Vec<HealthEmailAction> = Vec::new();
+
     let mut tx = db.begin().await?;
```

```rust
@@ -236,41 +265,75 @@ async fn run_one_tick(
             let action_result = match new_action {
                 SubscriptionAction::UpdateFailurePercent => query!(
                     "update webhook.subscription set failure_percent = $1 where subscription__id = $2",
                     candidate_subscription.failure_percent,
                     candidate_subscription.subscription_id,
                 )
                 .execute(&mut *tx)
                 .await
                 .map(|_| ()),

-                SubscriptionAction::EmitWarning => query!(
-                    r#"
-                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
-                        values ($1, 'warning', 'auto', null)
-                    "#,
-                    candidate_subscription.subscription_id,
-                )
-                .execute(&mut *tx)
-                .await
-                .map(|_| ()),
+                SubscriptionAction::EmitWarning => {
+                    let result = query!(
+                        r#"
+                            insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
+                            values ($1, 'warning', 'auto', null)
+                        "#,
+                        candidate_subscription.subscription_id,
+                    )
+                    .execute(&mut *tx)
+                    .await
+                    .map(|_| ());
+
+                    if result.is_ok() {
+                        email_actions.push(HealthEmailAction {
+                            status: HealthStatus::Warning,
+                            subscription_id: candidate_subscription.subscription_id,
+                            organization_id: candidate_subscription.organization_id,
+                            application_id: candidate_subscription.application_id,
+                            application_name: candidate_subscription.application_name.clone(),
+                            description: candidate_subscription.description.clone(),
+                            target_url: candidate_subscription.target_url.clone(),
+                            failure_percent: candidate_subscription.failure_percent,
+                            disabled_at: None,
+                        });
+                    }
+                    result
+                }

-                SubscriptionAction::EmitResolved => query!(
-                    r#"
-                        insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
-                        values ($1, 'resolved', 'auto', null)
-                    "#,
-                    candidate_subscription.subscription_id,
-                )
-                .execute(&mut *tx)
-                .await
-                .map(|_| ()),
+                SubscriptionAction::EmitResolved => {
+                    let result = query!(
+                        r#"
+                            insert into webhook.subscription_health_event (subscription__id, status, cause, user__id)
+                            values ($1, 'resolved', 'auto', null)
+                        "#,
+                        candidate_subscription.subscription_id,
+                    )
+                    .execute(&mut *tx)
+                    .await
+                    .map(|_| ());
+
+                    if result.is_ok() {
+                        email_actions.push(HealthEmailAction {
+                            status: HealthStatus::Resolved,
+                            subscription_id: candidate_subscription.subscription_id,
+                            organization_id: candidate_subscription.organization_id,
+                            application_id: candidate_subscription.application_id,
+                            application_name: candidate_subscription.application_name.clone(),
+                            description: candidate_subscription.description.clone(),
+                            target_url: candidate_subscription.target_url.clone(),
+                            failure_percent: candidate_subscription.failure_percent,
+                            disabled_at: None,
+                        });
+                    }
+                    result
+                }
```

```rust
@@ -281,46 +344,169 @@ async fn run_one_tick(
                                 returning created_at
                             )
                             select created_at as "created_at!" from inserted
                         "#,
                         candidate_subscription.subscription_id,
                     )
                     .fetch_optional(&mut *tx)
                     .await;

                     match disabled_at {
-                        Ok(Some(_)) => {
+                        Ok(Some(at)) => {
                             info!(
                                 "Subscription health monitor: disabled subscription {}",
                                 candidate_subscription.subscription_id
                             );
+                            email_actions.push(HealthEmailAction {
+                                status: HealthStatus::Disabled,
+                                subscription_id: candidate_subscription.subscription_id,
+                                organization_id: candidate_subscription.organization_id,
+                                application_id: candidate_subscription.application_id,
+                                application_name: candidate_subscription.application_name.clone(),
+                                description: candidate_subscription.description.clone(),
+                                target_url: candidate_subscription.target_url.clone(),
+                                failure_percent: candidate_subscription.failure_percent,
+                                disabled_at: Some(at),
+                            });
                             Ok(())
                         }
                         Ok(None) => Ok(()),
                         Err(e) => Err(e),
                     }
                 }
             };
```

```rust
     tx.commit().await?;
+
+    // Phase 2: send notification emails (best-effort, after commit)
+    if let Some(mailer) = mailer {
+        for action in &email_actions {
+            send_health_notification_emails(db, mailer, action, config).await;
+        }
+    }
+
     Ok(())
 }

+/// Sends health notification emails to all members of an organization.
+/// Best-effort: failures are logged but do not fail the tick. Emails are not retried.
+async fn send_health_notification_emails(
+    db: &PgPool,
+    mailer: &Mailer,
+    action: &HealthEmailAction,
+    config: &SubscriptionHealthMonitorConfig,
+) {
+    let description = action
+        .description
+        .clone()
+        .unwrap_or_else(|| action.subscription_id.to_string());
+    let evaluation_window = humantime::format_duration(config.failure_rate_window).to_string();
+
+    let mail = match action.status {
+        HealthStatus::Warning => Mail::SubscriptionWarning {
+            organization_id: action.organization_id,
+            application_id: action.application_id,
+            application_name: action.application_name.clone(),
+            subscription_description: description,
+            subscription_id: action.subscription_id,
+            target_url: action.target_url.clone(),
+            failure_percent: action.failure_percent,
+            evaluation_window,
+        },
+        HealthStatus::Disabled => Mail::SubscriptionDisabled {
+            organization_id: action.organization_id,
+            application_id: action.application_id,
+            application_name: action.application_name.clone(),
+            subscription_description: description,
+            subscription_id: action.subscription_id,
+            target_url: action.target_url.clone(),
+            failure_percent: action.failure_percent,
+            evaluation_window,
+            disabled_at: action
+                .disabled_at
+                .unwrap_or_else(Utc::now)
+                .to_rfc3339(),
+        },
+        HealthStatus::Resolved => Mail::SubscriptionResolved {
+            organization_id: action.organization_id,
+            application_id: action.application_id,
+            application_name: action.application_name.clone(),
+            subscription_description: description,
+            subscription_id: action.subscription_id,
+            target_url: action.target_url.clone(),
+        },
+    };
+
+    struct OrganizationUser {
+        first_name: String,
+        last_name: String,
+        email: String,
+    }
+
+    let users = match query_as!(
+        OrganizationUser,
+        r#"
+        select u.first_name, u.last_name, u.email
+        from iam.user u
+        inner join iam.user__organization uo on u.user__id = uo.user__id
+        where uo.organization__id = $1
+        "#,
+        action.organization_id,
+    )
+    .fetch_all(db)
+    .await
+    {
+        Ok(users) => users,
+        Err(e) => {
+            warn!(
+                "Subscription health monitor: failed to query org users for email (org {}): {e}",
+                action.organization_id
+            );
+            return;
+        }
+    };
+
+    for user in users {
+        let address = match Address::from_str(&user.email) {
+            Ok(a) => a,
+            Err(e) => {
+                warn!(
+                    "Subscription health monitor: invalid email address {}: {e}",
+                    user.email
+                );
+                continue;
+            }
+        };
+
+        let recipient = Mailbox::new(
+            Some(format!("{} {}", user.first_name, user.last_name)),
+            address,
+        );
+
+        if let Err(e) = mailer.send_mail(mail.clone(), recipient).await {
+            warn!(
+                "Subscription health monitor: failed to send email to {}: {e}",
+                user.email
+            );
+        }
+    }
+}
```

```rust
@@ -491,29 +677,36 @@ async fn list_health_evaluation_subscriptions_candidates(
                 union
                 select subscription__id
                 from (
                     select distinct on (subscription__id) subscription__id, status
                     from webhook.subscription_health_event
                     order by subscription__id, created_at desc
                 ) latest
                 where latest.status = 'warning'
             )
             select
-                bs.subscription__id as "subscription_id!",
-                bs.failure_percent as "failure_percent!",
-                lh.status as "last_health_status?: HealthStatus",
-                lh.created_at as "last_health_at?",
-                lh.cause as "last_health_cause?: HealthEventCause",
-                lh.user__id as "last_health_user_id?"
+                bs.subscription__id as subscription_id,
+                app.organization__id as organization_id,
+                s.application__id as application_id,
+                app.name as "application_name!",
+                s.description,
+                coalesce(th.url, '') as "target_url!",
+                coalesce(bs.failure_percent, 0.0) as "failure_percent!",
+                lh.status as "last_health_status: HealthStatus",
+                lh.created_at as last_health_at,
+                lh.cause as "last_health_cause: HealthEventCause",
+                lh.user__id as last_health_user_id
             from candidates c
             inner join bucket_stats bs on bs.subscription__id = c.subscription__id
             inner join webhook.subscription s on s.subscription__id = c.subscription__id
+            inner join event.application app on app.application__id = s.application__id
+            left join webhook.target_http th on th.target__id = s.target__id
             left join lateral (
```

```rust
@@ -623,21 +816,21 @@ mod tests {
-        run_one_tick(&pool, &config).await.unwrap();
+        run_one_tick(&pool, None, &config).await.unwrap();
```

(même changement pour tous les appels `run_one_tick` dans les tests)
