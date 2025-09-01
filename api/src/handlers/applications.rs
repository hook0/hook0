use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::handlers::subscriptions::RetryConfig;
use crate::hook0_client::{
    EventApplicationCreated, EventApplicationRemoved, EventApplicationUpdated, Hook0ClientEvent,
};
use crate::iam::{Action, authorize, authorize_for_application, get_owner_organization};
use crate::onboarding::{ApplicationOnboardingSteps, get_application_onboarding_steps};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::quotas::{Quota, QuotaValue};

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Application {
    application_id: Uuid,
    organization_id: Uuid,
    name: String,
    retry_config: RetryConfig,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationInfo {
    application_id: Uuid,
    organization_id: Uuid,
    name: String,
    retry_config: RetryConfig,
    quotas: ApplicationQuotas,
    consumption: ApplicationConsumption,
    onboarding_steps: ApplicationOnboardingSteps,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationConsumption {
    events_per_day: Option<i32>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationQuotas {
    events_per_day_limit: QuotaValue,
    days_of_events_retention_limit: QuotaValue,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    organization_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ApplicationPost {
    organization_id: Uuid,
    #[validate(non_control_character, length(min = 2, max = 50))]
    name: String,
    #[validate(nested)]
    retry_config: Option<RetryConfig>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ApplicationPut {
    #[validate(non_control_character, length(min = 2, max = 50))]
    name: Option<String>,
    #[validate(nested)]
    retry_config: Option<RetryConfig>,
}

#[api_v2_operation(
    summary = "Create a new application",
    description = "Registers a new application within an organization. An application emits events that customers can subscribe to using webhooks.",
    operation_id = "applications.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<ApplicationPost>,
) -> Result<CreatedJson<Application>, Hook0Problem> {
    if authorize(
        &biscuit,
        Some(body.organization_id),
        Action::ApplicationCreate,
        state.max_authorization_time_in_ms,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let quota_limit = state
        .quotas
        .get_limit_for_organization(
            &state.db,
            Quota::ApplicationsPerOrganization,
            &body.organization_id,
        )
        .await?;
    struct QueryResult {
        val: i64,
    }
    let quota_current = query_as!(
        QueryResult,
        r#"
            SELECT COUNT(application__id) AS "val!"
            FROM event.application
            WHERE organization__id = $1
            AND deleted_at IS NULL
        "#,
        &body.organization_id,
    )
    .fetch_one(&state.db)
    .await?;
    if quota_current.val >= quota_limit as i64 {
        return Err(Hook0Problem::TooManyApplicationsPerOrganization(
            quota_limit,
        ));
    }

    let retry_config = match body.retry_config.as_ref() {
        Some(rc) => {
            serde_json::to_value(rc.clone()).expect("could not serialize retry config into JSON")
        }
        None => serde_json::to_value(RetryConfig::default())
            .expect("could not serialize default retry config into JSON"),
    };

    // First, we need to use a raw query because of the retry_config JSON
    #[allow(non_snake_case)]
    struct RawApplication {
        application__id: Uuid,
        organization__id: Uuid,
        name: String,
        retry_config: Value,
    }

    let raw_app = query_as!(
        RawApplication,
        "
                INSERT INTO event.application (organization__id, name, retry_config) 
                VALUES ($1, $2, $3)
                RETURNING application__id, organization__id, name, retry_config
            ",
        body.organization_id,
        body.name,
        retry_config,
    )
    .fetch_one(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    let application = Application {
        application_id: raw_app.application__id,
        organization_id: raw_app.organization__id,
        name: raw_app.name,
        retry_config: serde_json::from_value(raw_app.retry_config)
            .unwrap_or_else(|_| RetryConfig::default()),
    };

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventApplicationCreated {
            organization_id: body.organization_id,
            application_id: application.application_id,
            name: application.name.to_owned(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(application))
}

#[api_v2_operation(
    summary = "Get an application by its ID",
    description = "Retrieves details about a specific application, including quotas and consumption statistics.",
    operation_id = "applications.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
) -> Result<Json<ApplicationInfo>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationGet {
            application_id: &application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let application_id = application_id.into_inner();

    #[allow(non_snake_case)]
    struct RawApplication {
        application__id: Uuid,
        organization__id: Uuid,
        name: String,
        retry_config: Value,
    }

    let raw_app = query_as!(
        RawApplication,
        "
            SELECT application__id, organization__id, name, retry_config
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        &application_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    let application = raw_app.map(|a| Application {
        application_id: a.application__id,
        organization_id: a.organization__id,
        name: a.name,
        retry_config: serde_json::from_value(a.retry_config)
            .unwrap_or_else(|_| RetryConfig::default()),
    });

    match application {
        Some(a) => {
            let quotas = ApplicationQuotas {
                events_per_day_limit: state
                    .quotas
                    .get_limit_for_application(&state.db, Quota::EventsPerDay, &application_id)
                    .await?,
                days_of_events_retention_limit: state
                    .quotas
                    .get_limit_for_application(
                        &state.db,
                        Quota::DaysOfEventsRetention,
                        &application_id,
                    )
                    .await?,
            };

            let consumption = query_as!(
                ApplicationConsumption,
                "
                    SELECT COALESCE(amount, 0) as events_per_day
                    FROM event.events_per_day
                    WHERE application__id = $1
                    AND date = CURRENT_DATE
                ",
                &application_id,
            )
            .fetch_optional(&state.db)
            .await
            .map_err(Hook0Problem::from)?
            .unwrap_or(ApplicationConsumption {
                events_per_day: Some(0),
            });

            let onboarding_steps =
                get_application_onboarding_steps(&state.db, &application_id).await?;

            Ok(Json(ApplicationInfo {
                application_id: a.application_id,
                organization_id: a.organization_id,
                name: a.name,
                retry_config: a.retry_config,
                quotas,
                consumption,
                onboarding_steps,
            }))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "List applications",
    description = "Retrieves all applications.",
    operation_id = "applications.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Application>>, Hook0Problem> {
    if authorize(
        &biscuit,
        Some(qs.organization_id),
        Action::ApplicationList,
        state.max_authorization_time_in_ms,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    #[allow(non_snake_case)]
    struct RawApplication {
        application__id: Uuid,
        organization__id: Uuid,
        name: String,
        retry_config: Value,
    }

    let raw_apps = query_as!(
            RawApplication,
            "SELECT application__id, organization__id, name, retry_config FROM event.application WHERE organization__id = $1 AND deleted_at IS NULL",
            &qs.organization_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    let applications: Vec<Application> = raw_apps
        .into_iter()
        .map(|a| Application {
            application_id: a.application__id,
            organization_id: a.organization__id,
            name: a.name,
            retry_config: serde_json::from_value(a.retry_config)
                .unwrap_or_else(|_| RetryConfig::default()),
        })
        .collect();

    Ok(Json(applications))
}

#[api_v2_operation(
    summary = "Edit an application",
    description = "Updates the name of an existing application.",
    operation_id = "applications.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
    body: Json<ApplicationPut>,
) -> Result<Json<Application>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationEdit {
            application_id: &application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let application_id = application_id.into_inner();

    // Build update query dynamically based on what fields are provided
    let mut query_parts = Vec::new();
    let mut params = Vec::new();
    let mut param_counter = 1;

    if let Some(name) = &body.name {
        query_parts.push(format!("name = ${}", param_counter));
        params.push((name.clone(), param_counter));
        param_counter += 1;
    }

    if let Some(retry_config) = &body.retry_config {
        let config_json =
            serde_json::to_value(retry_config).expect("could not serialize retry config into JSON");
        query_parts.push(format!("retry_config = ${}", param_counter));
        params.push((config_json.to_string(), param_counter));
        param_counter += 1;
    }

    if query_parts.is_empty() {
        // No fields to update
        return Err(Hook0Problem::Validation(validator::ValidationErrors::new()));
    }

    #[allow(non_snake_case)]
    struct RawApplication {
        application__id: Uuid,
        organization__id: Uuid,
        name: String,
        retry_config: Value,
    }

    // For simplicity, we'll fetch the current state and update it
    let raw_app = if body.name.is_some() && body.retry_config.is_some() {
        query_as!(
            RawApplication,
            "
                UPDATE event.application
                SET name = $1, retry_config = $2
                WHERE application__id = $3
                AND deleted_at IS NULL
                RETURNING application__id, organization__id, name, retry_config
            ",
            body.name.as_ref().unwrap(),
            serde_json::to_value(body.retry_config.as_ref().unwrap()).unwrap(),
            application_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    } else if body.name.is_some() {
        query_as!(
            RawApplication,
            "
                UPDATE event.application
                SET name = $1
                WHERE application__id = $2
                AND deleted_at IS NULL
                RETURNING application__id, organization__id, name, retry_config
            ",
            body.name.as_ref().unwrap(),
            application_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    } else if body.retry_config.is_some() {
        query_as!(
            RawApplication,
            "
                UPDATE event.application
                SET retry_config = $1
                WHERE application__id = $2
                AND deleted_at IS NULL
                RETURNING application__id, organization__id, name, retry_config
            ",
            serde_json::to_value(body.retry_config.as_ref().unwrap()).unwrap(),
            application_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?
    } else {
        None
    };

    let application = raw_app.map(|a| Application {
        application_id: a.application__id,
        organization_id: a.organization__id,
        name: a.name,
        retry_config: serde_json::from_value(a.retry_config)
            .unwrap_or_else(|_| RetryConfig::default()),
    });

    match application {
        Some(a) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationUpdated {
                    organization_id: get_owner_organization(&state.db, &a.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: a.application_id,
                    name: a.name.to_owned(),
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(Json(a))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Delete an application",
    description = "Marks an application as deleted. No more events will be emitted, and all active webhook subscriptions will be removed.",
    operation_id = "applications.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationDelete {
            application_id: &application_id,
        },
        state.max_authorization_time_in_ms,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    #[allow(non_snake_case)]
    struct RawApplication {
        application__id: Uuid,
        organization__id: Uuid,
        name: String,
        retry_config: Value,
    }

    let raw_application = query_as!(
        RawApplication,
        "
            SELECT application__id, organization__id, name, retry_config
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match raw_application {
        Some(raw) => {
            let a = Application {
                application_id: raw.application__id,
                organization_id: raw.organization__id,
                name: raw.name.clone(),
                retry_config: serde_json::from_value(raw.retry_config).expect("could not deserialize retry config from JSON"),
            };
            
            query!(
                "UPDATE event.application SET deleted_at = NOW() WHERE application__id = $1",
                a.application_id
            )
            .execute(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationRemoved {
                    organization_id: get_owner_organization(&state.db, &a.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: a.application_id,
                    name: a.name.to_owned(),
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(NoContent)
        }
        None => Err(Hook0Problem::NotFound),
    }
}
