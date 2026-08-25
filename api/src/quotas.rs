use actix_web::web::Data;
use lettre::{Address, message::Mailbox};
use paperclip::actix::web::Json;
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::Serialize;
use sqlx::{Acquire, AssertSqlSafe, Postgres, Transaction, query, query_as, query_scalar};
use std::str::FromStr;
use std::time::Duration;
use strum::Display;
use tracing::error;
use uuid::Uuid;

use crate::mailer::Mail;
use crate::problems::Hook0Problem;

/// Bounds how long a quota-enforcing transaction waits for the row lock it needs.
///
/// Must stay strictly below `DB_STATEMENT_TIMEOUT` when that one is set, otherwise the
/// statement timeout fires first and the wait surfaces as a `500` instead of a retryable
/// `503`. `main` warns at startup when that is the case.
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quota {
    MembersPerOrganization,
    ApplicationsPerOrganization,
    EventsPerDay,
    DaysOfEventsRetention,
    SubscriptionsPerApplication,
    EventTypesPerApplication,
}

impl Quota {
    fn get_name(&self) -> String {
        match self {
            Quota::MembersPerOrganization => "members_per_organization".to_string(),
            Quota::ApplicationsPerOrganization => "applications_per_organization".to_string(),
            Quota::EventsPerDay => "events_per_day".to_string(),
            Quota::DaysOfEventsRetention => "days_of_events_retention".to_string(),
            Quota::SubscriptionsPerApplication => "subscriptions_per_application".to_string(),
            Quota::EventTypesPerApplication => "event_types_per_application".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum QuotaNotificationType {
    Warning,
    Reached,
}

pub type QuotaValue = i32;

#[derive(Debug, Clone)]
struct QueryResult {
    val: Option<QuotaValue>,
}

#[derive(Debug, Clone, Serialize, Apiv2Schema, Copy)]
pub struct QuotaLimits {
    pub global_members_per_organization_limit: QuotaValue,
    pub global_applications_per_organization_limit: QuotaValue,
    pub global_events_per_day_limit: QuotaValue,
    pub global_days_of_events_retention_limit: QuotaValue,
    pub global_subscriptions_per_application_limit: QuotaValue,
    pub global_event_types_per_application_limit: QuotaValue,
}

#[derive(Debug, Clone, Serialize, Copy)]
pub struct Quotas {
    enabled: bool,
    limits: QuotaLimits,
}

#[derive(Debug, Clone, Serialize, Copy, Apiv2Schema)]
pub struct QuotasResponse {
    enabled: bool,
    limits: QuotaLimits,
}

impl Quotas {
    pub fn new(enabled: bool, limits: QuotaLimits) -> Self {
        Self { enabled, limits }
    }

    pub async fn get_limit_for_organization<'a, A: Acquire<'a, Database = Postgres>>(
        &self,
        db: A,
        quota: Quota,
        organization_id: &Uuid,
    ) -> Result<QuotaValue, sqlx::Error> {
        if self.enabled {
            let mut db = db.acquire().await?;

            let plan_value = match quota {
                Quota::MembersPerOrganization => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT p.members_per_organization_limit AS val
                            FROM iam.organization AS o
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE o.organization__id = $1
                        ",
                        organization_id,
                    )
                    .fetch_optional(&mut *db)
                    .await
                }
                Quota::ApplicationsPerOrganization => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT p.applications_per_organization_limit AS val
                            FROM iam.organization AS o
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE o.organization__id = $1
                        ",
                        organization_id,
                    )
                    .fetch_optional(&mut *db)
                    .await
                }
                Quota::EventsPerDay => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT p.events_per_day_limit AS val
                            FROM iam.organization AS o
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE o.organization__id = $1
                        ",
                        organization_id,
                    )
                    .fetch_optional(&mut *db)
                    .await
                }
                Quota::DaysOfEventsRetention => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT p.days_of_events_retention_limit AS val
                            FROM iam.organization AS o
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE o.organization__id = $1
                        ",
                        organization_id,
                    )
                    .fetch_optional(&mut *db)
                    .await
                }
                Quota::SubscriptionsPerApplication => Ok(None),
                Quota::EventTypesPerApplication => Ok(None),
            }?
            .and_then(|r| r.val);
            Ok(plan_value.unwrap_or(match quota {
                Quota::MembersPerOrganization => self.limits.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.limits.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.limits.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.limits.global_days_of_events_retention_limit,
                Quota::SubscriptionsPerApplication => {
                    self.limits.global_subscriptions_per_application_limit
                }
                Quota::EventTypesPerApplication => {
                    self.limits.global_event_types_per_application_limit
                }
            }))
        } else {
            Ok(QuotaValue::MAX)
        }
    }

    pub async fn get_limit_for_application<'a, A: Acquire<'a, Database = Postgres>>(
        &self,
        db: A,
        quota: Quota,
        application_id: &Uuid,
    ) -> Result<QuotaValue, sqlx::Error> {
        if self.enabled {
            let mut db = db.acquire().await?;

            let app_value = match quota {
                Quota::MembersPerOrganization => None,
                Quota::ApplicationsPerOrganization => None,
                Quota::EventsPerDay => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT a.events_per_day_limit AS val
                            FROM event.application AS a
                            INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE a.application__id = $1
                        ",
                        application_id,
                    )
                    .fetch_optional(&mut *db)
                    .await?
                }
                Quota::DaysOfEventsRetention => {
                    query_as!(
                        QueryResult,
                        "
                            SELECT a.days_of_events_retention_limit AS val
                            FROM event.application AS a
                            INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                            LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                            LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                            WHERE a.application__id = $1
                        ",
                        application_id,
                    )
                    .fetch_optional(&mut *db)
                    .await?
                }
                Quota::SubscriptionsPerApplication => None,
                Quota::EventTypesPerApplication => None,
            };
            let plan_value = match app_value {
                Some(QueryResult { val: Some(val) }) => Some(val),
                _ => match quota {
                    Quota::MembersPerOrganization => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.members_per_organization_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    }
                    Quota::ApplicationsPerOrganization => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.applications_per_organization_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    }
                    Quota::EventsPerDay => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.events_per_day_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    }
                    Quota::DaysOfEventsRetention => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.days_of_events_retention_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    },
                    Quota::SubscriptionsPerApplication => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.subscriptions_per_application_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    },
                    Quota::EventTypesPerApplication => {
                        query_as!(
                            QueryResult,
                            "
                                SELECT p.event_types_per_application_limit AS val
                                FROM event.application AS a
                                INNER JOIN iam.organization AS o ON o.organization__id = a.organization__id
                                LEFT JOIN pricing.price AS pr ON pr.price__id = o.price__id
                                LEFT JOIN pricing.plan AS p ON p.plan__id = pr.plan__id
                                WHERE a.application__id = $1
                            ",
                            application_id,
                        )
                        .fetch_optional(&mut *db)
                        .await
                    },
                }?
                .and_then(|r| r.val),
            };
            Ok(plan_value.unwrap_or(match quota {
                Quota::MembersPerOrganization => self.limits.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.limits.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.limits.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.limits.global_days_of_events_retention_limit,
                Quota::SubscriptionsPerApplication => {
                    self.limits.global_subscriptions_per_application_limit
                }
                Quota::EventTypesPerApplication => {
                    self.limits.global_event_types_per_application_limit
                }
            }))
        } else {
            Ok(QuotaValue::MAX)
        }
    }

    /// Bounds how long the current transaction may wait for the row locks it is about
    /// to take.
    ///
    /// `SET LOCAL` is reverted at COMMIT and at ROLLBACK, so this cannot leak to the
    /// next request that reuses the pooled connection.
    ///
    /// The setting stays in effect for the rest of the transaction, so it also bounds
    /// the lock waits of the insert that follows. That is intentional.
    async fn set_lock_timeout(tx: &mut Transaction<'_, Postgres>) -> Result<(), Hook0Problem> {
        // GUC values cannot be bound as `$1` parameters, so the value is interpolated. It is a
        // compile-time constant rendered as an integer number of milliseconds, never user input.
        query(AssertSqlSafe(format!(
            "SET LOCAL lock_timeout = {}",
            LOCK_TIMEOUT.as_millis()
        )))
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Serializes concurrent quota checks that target the same organization.
    ///
    /// `FOR NO KEY UPDATE` conflicts with itself, so two transactions running this
    /// against the same organization cannot proceed at the same time. It does not
    /// conflict with the `FOR KEY SHARE` lock that foreign key checks take, so
    /// unrelated inserts referencing this organization are not blocked.
    ///
    /// It does conflict with a plain `UPDATE` of that row.
    ///
    /// The lock is released at COMMIT/ROLLBACK, so this only means anything inside
    /// the transaction that also performs the insert.
    async fn lock_organization(
        tx: &mut Transaction<'_, Postgres>,
        organization_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        Self::set_lock_timeout(&mut *tx).await?;

        query_scalar!(
            "
                SELECT organization__id
                FROM iam.organization
                WHERE organization__id = $1
                FOR NO KEY UPDATE
            ",
            organization_id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(Hook0Problem::NotFound)?;
        Ok(())
    }

    /// Serializes concurrent quota checks that target the same application.
    ///
    /// See [`Quotas::lock_organization`] for why `FOR NO KEY UPDATE` is used. Soft
    /// deleted applications are deliberately not filtered out: the lock exists only
    /// to serialize, never to decide whether the application is usable.
    ///
    /// As with organizations, this conflicts with a plain `UPDATE` of that row.
    async fn lock_application(
        tx: &mut Transaction<'_, Postgres>,
        application_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        Self::set_lock_timeout(&mut *tx).await?;

        query_scalar!(
            "
                SELECT application__id
                FROM event.application
                WHERE application__id = $1
                FOR NO KEY UPDATE
            ",
            application_id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(Hook0Problem::NotFound)?;
        Ok(())
    }

    /// Rejects the call if the organization already reached its applications limit.
    ///
    /// Must be called inside the transaction that performs the insert, and must be
    /// the first statement of it: the row lock it takes only lives as long as that
    /// transaction, and taking it first is what keeps lock ordering consistent.
    ///
    /// Taking a [`Transaction`] rather than a connection is what makes that contract a
    /// compile error to violate: outside a transaction the row lock would be released
    /// immediately and the `SET LOCAL` of [`Quotas::set_lock_timeout`] would be a silent
    /// no-op, so both guarantees would vanish without a trace.
    pub async fn enforce_applications_per_organization(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        organization_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        if !self.enabled {
            return Ok(());
        }

        Self::lock_organization(&mut *tx, organization_id).await?;

        let limit = self
            .get_limit_for_organization(
                &mut **tx,
                Quota::ApplicationsPerOrganization,
                organization_id,
            )
            .await?;

        let current = query_scalar!(
            r#"
                SELECT COUNT(application__id) AS "val!"
                FROM event.application
                WHERE organization__id = $1
                AND deleted_at IS NULL
            "#,
            organization_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        if current >= i64::from(limit) {
            Err(Hook0Problem::TooManyApplicationsPerOrganization(limit))
        } else {
            Ok(())
        }
    }

    /// Rejects the call if the organization already reached its members limit.
    ///
    /// Same transactional contract as [`Quotas::enforce_applications_per_organization`].
    pub async fn enforce_members_per_organization(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        organization_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        if !self.enabled {
            return Ok(());
        }

        Self::lock_organization(&mut *tx, organization_id).await?;

        let limit = self
            .get_limit_for_organization(&mut **tx, Quota::MembersPerOrganization, organization_id)
            .await?;

        let current = query_scalar!(
            r#"
                SELECT COUNT(user__id) AS "val!"
                FROM iam.user__organization
                WHERE organization__id = $1
            "#,
            organization_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        if current >= i64::from(limit) {
            Err(Hook0Problem::TooManyMembersPerOrganization(limit))
        } else {
            Ok(())
        }
    }

    /// Rejects the call if the application already reached its subscriptions limit.
    ///
    /// Same transactional contract as [`Quotas::enforce_applications_per_organization`].
    pub async fn enforce_subscriptions_per_application(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        application_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        if !self.enabled {
            return Ok(());
        }

        Self::lock_application(&mut *tx, application_id).await?;

        let limit = self
            .get_limit_for_application(
                &mut **tx,
                Quota::SubscriptionsPerApplication,
                application_id,
            )
            .await?;

        let current = query_scalar!(
            r#"
                SELECT COUNT(subscription__id) AS "val!"
                FROM webhook.subscription
                WHERE application__id = $1
                    and deleted_at IS NULL
            "#,
            application_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        if current >= i64::from(limit) {
            Err(Hook0Problem::TooManySubscriptionsPerApplication(limit))
        } else {
            Ok(())
        }
    }

    /// Rejects the call if the application already reached its event types limit.
    ///
    /// Same transactional contract as [`Quotas::enforce_applications_per_organization`].
    pub async fn enforce_event_types_per_application(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        application_id: &Uuid,
    ) -> Result<(), Hook0Problem> {
        if !self.enabled {
            return Ok(());
        }

        Self::lock_application(&mut *tx, application_id).await?;

        let limit = self
            .get_limit_for_application(&mut **tx, Quota::EventTypesPerApplication, application_id)
            .await?;

        let current = query_scalar!(
            r#"
                SELECT COUNT(*) AS "val!"
                FROM event.event_type
                WHERE application__id = $1
                    AND deactivated_at IS NULL
            "#,
            application_id,
        )
        .fetch_one(&mut **tx)
        .await?;

        if current >= i64::from(limit) {
            Err(Hook0Problem::TooManyEventTypesPerApplication(limit))
        } else {
            Ok(())
        }
    }

    pub async fn send_organization_email_notification(
        &self,
        state: &Data<crate::State>,
        quota: Quota,
        notification_type: QuotaNotificationType,
        organization_id: &Uuid,
        application_id: Option<Uuid>,
        mail: Mail,
    ) -> Result<(), Hook0Problem> {
        let can_send_notification = query!(
            r#"
                SELECT 1 AS ONE
                FROM pricing.quota_notifications
                WHERE organization__id = $1
                    AND type = $2
                    AND name = $3
                    AND DATE(executed_at) = CURRENT_DATE
            "#,
            organization_id,
            notification_type.to_string(),
            quota.get_name(),
        )
        .fetch_optional(&state.db)
        .await?
        .is_none();

        if can_send_notification {
            struct User {
                first_name: String,
                last_name: String,
                email: String,
            }

            let emails_from_organization = query_as!(
                User,
                r#"
                    SELECT u.first_name, u.last_name, u.email
                    FROM iam.user u
                    INNER JOIN iam.user__organization ou ON u.user__id = ou.user__id
                    WHERE ou.organization__id = $1
                "#,
                organization_id,
            )
            .fetch_all(&state.db)
            .await
            .map_err(Hook0Problem::from)?
            .into_iter()
            .collect::<Vec<_>>();

            let mut tx = state.db.begin().await?;

            query!(
                r#"
                    INSERT INTO pricing.quota_notifications
                        (organization__id, type, name)
                    VALUES
                        ($1, $2, $3)
                "#,
                organization_id,
                notification_type.to_string(),
                quota.get_name(),
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            let email_sending_result: Result<(), Hook0Problem> = async {
                for user in emails_from_organization {
                    let recipient_address = match Address::from_str(&user.email) {
                        Ok(address) => address,
                        Err(e) => {
                            error!("Error trying to parse email address: {e}");
                            continue;
                        }
                    };

                    let recipient = Mailbox::new(
                        Some(format!("{} {}", user.first_name, user.last_name)),
                        recipient_address,
                    );

                    let dashboard_path = match application_id {
                        Some(application_id) => format!("/organizations/{organization_id}/applications/{application_id}/dashboard"),
                        None => format!("/organizations/{organization_id}/dashboard"),
                    };

                    let mut mail = mail.clone();
                    // Per-recipient personalization (boost CTR via first-name).
                    // Quota mails are constructed as a template with
                    // `recipient_first_name: None` in `handlers/events.rs`
                    // and MUST be hydrated here before `send_mail`, otherwise
                    // `Mail::render` fails fast.
                    match &mut mail {
                        Mail::QuotaEventsPerDayWarning { recipient_first_name, .. }
                        | Mail::QuotaEventsPerDayReached { recipient_first_name, .. } => {
                            *recipient_first_name = Some(user.first_name.clone());
                        }
                        _ => {}
                    }
                    let dashboard_url_tracked = state
                        .mailer
                        .build_tracked_app_url(&mail, &dashboard_path);
                    mail.add_variable("dashboard_url_tracked".to_owned(), dashboard_url_tracked);

                    if let Err(e) = &state.mailer
                        .send_mail(
                            mail,
                            recipient,
                        )
                        .await
                    {
                        error!("Error trying to send email: {e}");
                    }
                }

                Ok(())
            }
            .await;

            if let Err(e) = email_sending_result {
                error!("Error trying to send email: {e}");
                tx.rollback().await?;
            } else {
                tx.commit().await?;
            }
        }

        Ok(())
    }

    pub async fn send_application_email_notification(
        &self,
        state: &Data<crate::State>,
        quota: Quota,
        notification_type: QuotaNotificationType,
        application_id: Uuid,
        mail: Mail,
    ) -> Result<(), Hook0Problem> {
        let organization_id = query_scalar!(
            r#"
                SELECT organization__id
                FROM event.application
                WHERE application__id = $1
            "#,
            application_id,
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

        self.send_organization_email_notification(
            state,
            quota,
            notification_type,
            &organization_id,
            Some(application_id),
            mail,
        )
        .await
    }
}

#[api_v2_operation(
    summary = "Get quotas",
    description = "Get the current quotas limitations on the instance.",
    operation_id = "quotas.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0", "sdk")
)]
pub async fn get(state: Data<crate::State>) -> Result<Json<QuotasResponse>, Hook0Problem> {
    Ok(Json(QuotasResponse {
        enabled: state.quotas.enabled,
        limits: state.quotas.limits,
    }))
}

#[cfg(test)]
mod call_site_contract_tests {
    use std::fs;
    use std::path::Path;

    /// Every `enforce_*` above takes a row lock and counts under it. That count
    /// is only true for as long as the lock is held, so the write it guards has
    /// to run in the same transaction. Opening a second one in between drops the
    /// first, which releases the lock before anything is written, and two
    /// callers arriving together can then both clear a limit only one of them
    /// should have passed.
    ///
    /// This reads the sources rather than the behaviour, on purpose. sqlx sends
    /// the rollback of a dropped transaction only when the connection goes back
    /// to the pool, so the lock currently outlives the transaction that took it
    /// and a concurrency test stays green whether or not the contract is kept.
    /// The property test belongs elsewhere; what is checked here is the shape
    /// that made the property true by accident rather than by design.
    #[test]
    fn no_handler_opens_a_second_transaction_between_a_quota_check_and_its_commit() {
        let handlers = Path::new("src/handlers");
        let mut offenders = Vec::new();

        let mut sources: Vec<_> = fs::read_dir(handlers)
            .expect("read src/handlers")
            .map(|entry| entry.expect("read a handler directory entry").path())
            .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        sources.sort();

        assert!(
            !sources.is_empty(),
            "no handler source was read: this guard would pass by finding nothing"
        );

        for path in sources {
            let source = fs::read_to_string(&path).expect("read a handler source");
            let lines: Vec<&str> = source.lines().collect();

            for (start, line) in lines.iter().enumerate() {
                if !line.contains(".enforce_") {
                    continue;
                }
                for (offset, later) in lines.iter().enumerate().skip(start + 1) {
                    // The write this check guards has been committed: contract kept.
                    if later.contains(".commit()") {
                        break;
                    }
                    // A new item starts: the check guarded nothing that commits
                    // here, and what follows belongs to another call site.
                    if later.starts_with("pub async fn ")
                        || later.starts_with("async fn ")
                        || later.starts_with("pub fn ")
                        || later.starts_with("fn ")
                    {
                        break;
                    }
                    if later.contains("db.begin()") {
                        offenders.push(format!(
                            "{}:{} opens a transaction after the quota check on line {}",
                            path.display(),
                            offset + 1,
                            start + 1
                        ));
                        break;
                    }
                }
            }
        }

        assert!(
            offenders.is_empty(),
            "a quota check no longer covers the write it guards:\n  {}",
            offenders.join("\n  ")
        );
    }
}
