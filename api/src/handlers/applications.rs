use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventApplicationCreated, EventApplicationRemoved, EventApplicationUpdated, Hook0ClientEvent,
};
use crate::iam::{Action, authorize, authorize_for_application, get_owner_organization};
use crate::onboarding::{ApplicationOnboardingSteps, get_application_onboarding_steps};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::query_builder::QueryBuilder;
use crate::quotas::{Quota, QuotaValue};

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
pub struct Application {
    application_id: Uuid,
    organization_id: Uuid,
    name: String,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationInfo {
    application_id: Uuid,
    organization_id: Uuid,
    name: String,
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
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ApplicationPost {
    organization_id: Uuid,
    #[validate(non_control_character, length(min = 2, max = 50))]
    name: String,
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
        state.debug_authorizer,
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

    let application = query_as!(
            Application,
            "
                INSERT INTO event.application (organization__id, name) VALUES ($1, $2)
                RETURNING application__id AS application_id, organization__id AS organization_id, name
            ",
            body.organization_id, body.name,
        )
        .fetch_one(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

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
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let application_id = application_id.into_inner();

    let application = query_as!(
        Application,
        "
            SELECT application__id AS application_id, organization__id AS organization_id, name
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        &application_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

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
        state.debug_authorizer,
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // Build dynamic WHERE conditions using QueryBuilder
    let mut query_builder = QueryBuilder::new(
        "organization__id = $1 AND deleted_at IS NULL".to_string(),
        2, // Next parameter index after organization_id
    );

    query_builder.add_string_filter("name", qs.name.clone());

    let where_clause = query_builder.build_where_clause();
    let sql = format!(
        "SELECT application__id AS application_id, organization__id AS organization_id, name FROM event.application WHERE {}",
        where_clause
    );

    let query = query_as::<_, Application>(&sql).bind(qs.organization_id);
    let query = query_builder.bind_params(query);

    let applications = query
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

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
    body: Json<ApplicationPost>,
) -> Result<Json<Application>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationEdit {
            application_id: &application_id,
        },
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let application = query_as!(
            Application,
            "
                UPDATE event.application
                SET name = $1 WHERE application__id = $2
                AND deleted_at IS NULL
                RETURNING application__id AS application_id, organization__id AS organization_id, name
            ",
            body.name,
            application_id.into_inner()
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

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
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let application = query_as!(
        Application,
        "
            SELECT application__id AS application_id, organization__id AS organization_id, name
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match application {
        Some(a) => {
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
