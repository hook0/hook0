use actix_web::web::Data;

use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::Serialize;

use crate::{mailer::Mail, problems::Hook0Problem};

use std::{str::FromStr, time::Duration};

use lettre::{message::Mailbox, Address};
use log::error;
use sqlx::{postgres::types::PgInterval, query, query_as, query_scalar, Acquire, Postgres};
use strum::Display;
use uuid::Uuid;

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

    pub async fn send_organization_email_notification(
        &self,
        state: &Data<crate::State>,
        quota: Quota,
        notification_type: QuotaNotificationType,
        organization_id: &Uuid,
        application_id: Option<Uuid>,
        mail: Mail,
    ) -> Result<(), Hook0Problem> {
        match PgInterval::try_from(Duration::from_secs(86400)) {
            Ok(quota_notification_period) => {
                let can_send_notification = query!(
                    r#"
                        SELECT 1 AS ONE
                        FROM pricing.quota_notifications
                        WHERE organization__id = $1
                            AND type = $2
                            AND name = $3
                            AND executed_at > now() - $4::interval
                    "#,
                    organization_id,
                    notification_type.to_string(),
                    quota.get_name(),
                    quota_notification_period,
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

                            let entity_hash = match application_id {
                                Some(application_id) => format!("/organizations/{organization_id}/applications/{application_id}/dashboard"),
                                None => format!("/organizations/{organization_id}/dashboard"),
                            };

                            let mut mail = mail.clone();
                            mail.add_variable("entity_hash".to_owned(), entity_hash);

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
            Err(_) => Err(Hook0Problem::InternalServerError),
        }
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
    tags("Hook0")
)]
pub async fn get(state: Data<crate::State>) -> Result<Json<QuotasResponse>, Hook0Problem> {
    Ok(Json(QuotasResponse {
        enabled: state.quotas.enabled,
        limits: state.quotas.limits,
    }))
}
