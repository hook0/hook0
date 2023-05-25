use sqlx::{query_as, Acquire, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quota {
    MembersPerOrganization,
    ApplicationsPerOrganization,
    EventsPerDay,
    DaysOfEventsRetention,
}

pub type QuotaValue = i32;

#[derive(Debug, Clone)]
struct QueryResult {
    val: Option<QuotaValue>,
}

#[derive(Debug, Clone)]
pub struct Quotas {
    enabled: bool,
    global_members_per_organization_limit: QuotaValue,
    global_applications_per_organization_limit: QuotaValue,
    global_events_per_day_limit: QuotaValue,
    global_days_of_events_retention_limit: QuotaValue,
}

impl Quotas {
    pub fn new(
        enabled: bool,
        global_members_per_organization_limit: QuotaValue,
        global_applications_per_organization_limit: QuotaValue,
        global_events_per_day_limit: QuotaValue,
        global_days_of_events_retention_limit: QuotaValue,
    ) -> Self {
        Self {
            enabled,
            global_members_per_organization_limit,
            global_applications_per_organization_limit,
            global_events_per_day_limit,
            global_days_of_events_retention_limit,
        }
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
            }?
            .and_then(|r| r.val);
            Ok(plan_value.unwrap_or(match quota {
                Quota::MembersPerOrganization => self.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.global_days_of_events_retention_limit,
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
            };
            let plan_value = match app_value {
                Some(r) => r.val,
                None => match quota {
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
                    }
                }?
                .and_then(|r| r.val),
            };
            Ok(plan_value.unwrap_or(match quota {
                Quota::MembersPerOrganization => self.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.global_days_of_events_retention_limit,
            }))
        } else {
            Ok(QuotaValue::MAX)
        }
    }
}
