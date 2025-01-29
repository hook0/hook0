use actix_web::web::Data;
use sqlx::{query_as, Acquire, Postgres};
use uuid::Uuid;

use paperclip::actix::web::Json;
use paperclip::actix::{api_v2_operation, Apiv2Schema};
use serde::{Deserialize, Serialize};

use crate::problems::Hook0Problem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quota {
    MembersPerOrganization,
    ApplicationsPerOrganization,
    EventsPerDay,
    DaysOfEventsRetention,
    SubscriptionsPerApplication,
    EventTypesPerApplication,
}

pub type QuotaValue = i32;

#[derive(Debug, Clone)]
struct QueryResult {
    val: Option<QuotaValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Apiv2Schema, Copy, PartialEq, Eq)]
pub struct Quotas {
    enabled: bool,
    global_members_per_organization_limit: QuotaValue,
    global_applications_per_organization_limit: QuotaValue,
    global_events_per_day_limit: QuotaValue,
    global_days_of_events_retention_limit: QuotaValue,
    global_subscriptions_per_application_limit: QuotaValue,
    global_event_types_per_application_limit: QuotaValue,
}

impl Quotas {
    pub fn new(
        enabled: bool,
        global_members_per_organization_limit: QuotaValue,
        global_applications_per_organization_limit: QuotaValue,
        global_events_per_day_limit: QuotaValue,
        global_days_of_events_retention_limit: QuotaValue,
        global_subscriptions_per_application_limit: QuotaValue,
        global_event_types_per_application_limit: QuotaValue,
    ) -> Self {
        Self {
            enabled,
            global_members_per_organization_limit,
            global_applications_per_organization_limit,
            global_events_per_day_limit,
            global_days_of_events_retention_limit,
            global_subscriptions_per_application_limit,
            global_event_types_per_application_limit,
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
                Quota::SubscriptionsPerApplication => Ok(None),
                Quota::EventTypesPerApplication => Ok(None),
            }?
            .and_then(|r| r.val);
            Ok(plan_value.unwrap_or(match quota {
                Quota::MembersPerOrganization => self.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.global_days_of_events_retention_limit,
                Quota::SubscriptionsPerApplication => {
                    self.global_subscriptions_per_application_limit
                }
                Quota::EventTypesPerApplication => self.global_event_types_per_application_limit,
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
                Quota::MembersPerOrganization => self.global_members_per_organization_limit,
                Quota::ApplicationsPerOrganization => {
                    self.global_applications_per_organization_limit
                }
                Quota::EventsPerDay => self.global_events_per_day_limit,
                Quota::DaysOfEventsRetention => self.global_days_of_events_retention_limit,
                Quota::SubscriptionsPerApplication => {
                    self.global_subscriptions_per_application_limit
                }
                Quota::EventTypesPerApplication => self.global_event_types_per_application_limit,
            }))
        } else {
            Ok(QuotaValue::MAX)
        }
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
pub async fn get(state: Data<crate::State>) -> Result<Json<Quotas>, Hook0Problem> {
    Ok(Json(state.quotas))
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web::{self, Json}, App};
    use crate::{problems::Hook0Problem, quotas::Quotas};

    #[derive(Clone)]
    struct MockState {
        quotas: Quotas,
    }

    async fn mock_get_quota(state: web::Data<MockState>) -> Result<Json<Quotas>, Hook0Problem> {
        Ok(Json(state.quotas))
    }

    const MOCK_STATE: MockState = MockState {
        quotas: Quotas {
            enabled: true,
            global_members_per_organization_limit: 10,
            global_applications_per_organization_limit: 10,
            global_events_per_day_limit: 10,
            global_days_of_events_retention_limit: 10,
            global_subscriptions_per_application_limit: 10,
            global_event_types_per_application_limit: 10,
        },
    };

    #[actix_web::test]
    async fn test_get_quota_successfull() {

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(MOCK_STATE.clone()))
                .route("/quotas", web::get().to(mock_get_quota)),
        )
        .await;

        let req = test::TestRequest::get().uri("/quotas").to_request();
        let resp: Quotas = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp, MOCK_STATE.quotas);
    }
}

