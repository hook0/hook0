use paperclip::actix::Apiv2Schema;
use serde::Serialize;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Apiv2Schema)]
enum OnboardingStepStatus {
    #[serde(rename(serialize = "todo"))]
    ToDo,
    #[serde(rename(serialize = "done"))]
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

#[derive(Debug, Clone, Serialize, Apiv2Schema)]
pub struct OnboardingStep {
    code: String,
    description: String,
    status: OnboardingStepStatus,
}

fn mk_orgnization_onboarding(
    create_app: OnboardingStepStatus,
    create_event_type: OnboardingStepStatus,
    create_sub: OnboardingStepStatus,
    create_app_secret: OnboardingStepStatus,
    send_event: OnboardingStepStatus,
) -> Vec<OnboardingStep> {
    vec![
        OnboardingStep {
            code: "create-application".to_owned(),
            description: "Create an application".to_owned(),
            status: create_app,
        },
        OnboardingStep {
            code: "create-event-type".to_owned(),
            description: "Create an event type".to_owned(),
            status: create_event_type,
        },
        OnboardingStep {
            code: "create-subscription".to_owned(),
            description: "Create a subscription".to_owned(),
            status: create_sub,
        },
        OnboardingStep {
            code: "create-application-secret".to_owned(),
            description: "Create an application secret".to_owned(),
            status: create_app_secret,
        },
        OnboardingStep {
            code: "send-event".to_owned(),
            description: "Send an event".to_owned(),
            status: send_event,
        },
    ]
}

pub fn new_organization_onboarding() -> Vec<OnboardingStep> {
    mk_orgnization_onboarding(
        OnboardingStepStatus::ToDo,
        OnboardingStepStatus::ToDo,
        OnboardingStepStatus::ToDo,
        OnboardingStepStatus::ToDo,
        OnboardingStepStatus::ToDo,
    )
}

pub async fn organization_onboarding(
    db: &PgPool,
    organization_id: &Uuid,
) -> Result<Vec<OnboardingStep>, sqlx::Error> {
    struct AppId {
        application_id: Uuid,
    }

    let apps = query_as!(
        AppId,
        "
            SELECT application__id AS application_id
            FROM event.application
            WHERE organization__id = $1
        ",
        organization_id,
    )
    .fetch_all(db)
    .await?
    .iter()
    .map(|app_id| app_id.application_id)
    .collect::<Vec<_>>();

    let (create_app, create_event_type, create_sub, create_app_secret, send_event) = if apps
        .is_empty()
    {
        (
            OnboardingStepStatus::ToDo,
            OnboardingStepStatus::ToDo,
            OnboardingStepStatus::ToDo,
            OnboardingStepStatus::ToDo,
            OnboardingStepStatus::ToDo,
        )
    } else {
        struct ExistenceResuls {
            event_type: bool,
            sub: bool,
            app_secret: bool,
            event: bool,
        }

        let existance_results = query_as!(
                ExistenceResuls,
                r#"
                    SELECT
                        EXISTS(SELECT 1 FROM event.event_type WHERE application__id = ANY($1)) AS "event_type!",
                        EXISTS(SELECT 1 FROM webhook.subscription WHERE application__id = ANY($1)) AS "sub!",
                        EXISTS(SELECT 1 FROM event.application_secret WHERE application__id = ANY($1)) AS "app_secret!",
                        EXISTS(SELECT 1 FROM event.event WHERE application__id = ANY($1)) AS "event!"
                "#,
                &apps,
            )
            .fetch_one(db)
            .await?;

        (
            OnboardingStepStatus::Done,
            existance_results.event_type.into(),
            existance_results.sub.into(),
            existance_results.app_secret.into(),
            existance_results.event.into(),
        )
    };

    Ok(mk_orgnization_onboarding(
        create_app,
        create_event_type,
        create_sub,
        create_app_secret,
        send_event,
    ))
}
