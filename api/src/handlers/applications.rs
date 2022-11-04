use log::error;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventApplicationCreated, EventApplicationRemoved, EventApplicationUpdated, Hook0ClientEvent,
};
use crate::iam::{get_owner_organization, AuthProof, Role};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Application {
    application_id: Uuid,
    organization_id: Uuid,
    name: String,
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
    auth: AuthProof,
    body: Json<ApplicationPost>,
) -> Result<CreatedJson<Application>, Hook0Problem> {
    if auth
        .can_access_organization(&body.organization_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
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
            organization_id: get_owner_organization(&state.db, &application.application_id)
                .await
                .unwrap_or(Uuid::nil()),
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
    auth: AuthProof,
    application_id: Path<Uuid>,
) -> Result<Json<Application>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &application_id, &Role::Viewer)
        .await
        .is_none()
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
        Some(a) => Ok(Json(a)),
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
    auth: AuthProof,
    qs: Query<Qs>,
) -> Result<Json<Vec<Application>>, Hook0Problem> {
    if auth
        .can_access_organization(&qs.organization_id, &Role::Viewer)
        .await
        .is_none()
    {
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
    auth: AuthProof,
    application_id: Path<Uuid>,
    body: Json<ApplicationPost>,
) -> Result<Json<Application>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &application_id, &Role::Editor)
        .await
        .is_none()
        && auth
            .can_access_organization(&body.organization_id, &Role::Editor)
            .await
            .is_none()
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
    auth: AuthProof,
    application_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &application_id, &Role::Editor)
        .await
        .is_none()
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
