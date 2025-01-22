use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use sqlx::{query_as, Acquire, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Apiv2Schema)]
pub enum OnboardingStepStatus {
    ToDo,
    Done,
}

impl From<bool> for OnboardingStepStatus {
    fn from(val: bool) -> Self {
        if val {
            Self::Done
        } else {
            Self::ToDo
        }
    }
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct OrganizationOnboardingSteps {
    pub application: OnboardingStepStatus,
    pub event_type: OnboardingStepStatus,
    pub subscription: OnboardingStepStatus,
    pub event: OnboardingStepStatus,
}

pub async fn get_organization_onboarding_steps<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    organization_id: &Uuid,
) -> Result<OrganizationOnboardingSteps, sqlx::Error> {
    let mut db = db.acquire().await?;

    query_as!(
        OrganizationOnboardingSteps,
        r#"
            WITH application_ids as (
                SELECT ARRAY_AGG(application__id) as applications_ids FROM event.application WHERE organization__id = $1 AND deleted_at IS NULL
            )
            SELECT
                COALESCE(CARDINALITY(application_ids.applications_ids), 0) >= 1 as "application!",
                EXISTS(SELECT 1 FROM event.event_type WHERE application__id = ANY(application_ids.applications_ids) AND deactivated_at IS NULL) AS "event_type!",
                EXISTS(SELECT 1 FROM webhook.subscription WHERE application__id = ANY(application_ids.applications_ids) AND deleted_at IS NULL) AS "subscription!",
                EXISTS(SELECT 1 FROM event.event WHERE application__id = ANY(application_ids.applications_ids)) AS "event!"
            FROM application_ids
        "#,
        organization_id
    )
    .fetch_one(&mut *db)
    .await
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationOnboardingSteps {
    pub event_type: OnboardingStepStatus,
    pub subscription: OnboardingStepStatus,
    pub event: OnboardingStepStatus,
}

// TODO: Demander à David si qlq chose à faire ici
pub async fn get_application_onboarding_steps<'a, A: Acquire<'a, Database = Postgres>>(
    db: A,
    application_id: &Uuid,
) -> Result<ApplicationOnboardingSteps, sqlx::Error> {
    let mut db = db.acquire().await?;

    query_as!(
        ApplicationOnboardingSteps,
        r#"
            SELECT
                EXISTS(SELECT 1 FROM event.event_type WHERE application__id = $1 AND deactivated_at IS NULL) AS "event_type!",
                EXISTS(SELECT 1 FROM webhook.subscription WHERE application__id = $1 AND deleted_at IS NULL) AS "subscription!",
                EXISTS(SELECT 1 FROM event.event WHERE application__id = $1) AS "event!"
        "#,
        application_id
    )
    .fetch_one(&mut *db)
    .await
}
