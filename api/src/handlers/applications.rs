use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson, NoContent};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventApplicationCreated, EventApplicationRemoved, EventApplicationUpdated, Hook0ClientEvent,
};
use crate::iam::{authorize, authorize_for_application, get_owner_organization, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::quotas::{Quota, QuotaValue};

#[derive(Debug, Serialize, Apiv2Schema)]
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
    #[validate(non_control_character, length(min = 1, max = 50))]
    name: String,
}

#[api_v2_operation(
    summary = "Create a new application",
    description = "An application emit events that are consumed by customers through webhooks",
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
    description = "An application emit events that are consumed by customers through webhooks",
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

            Ok(Json(ApplicationInfo {
                application_id: a.application_id,
                organization_id: a.organization_id,
                name: a.name,
                quotas,
            }))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "List applications",
    description = "",
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
    if authorize(&biscuit, Some(qs.organization_id), Action::ApplicationList).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let applications = query_as!(
            Application,
            "SELECT application__id AS application_id, organization__id AS organization_id, name FROM event.application WHERE organization__id = $1",
            &qs.organization_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    Ok(Json(applications))
}

#[api_v2_operation(
    summary = "Edit an application",
    description = "Change the name of an application",
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
    description = "Delete an application, further events won't be sent, active webhook subscriptions will also be deleted.",
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
        ",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match application {
        Some(a) => {
            query!(
                "DELETE FROM event.application WHERE application__id = $1",
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
