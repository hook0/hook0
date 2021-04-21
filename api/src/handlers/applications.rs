use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query, ReqData},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::errors::*;
use crate::iam::{can_access_application, can_access_organization, Role};

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

/// List applications
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Application>>, UnexpectedError> {
    if can_access_organization(&unstructured_claims, &qs.organization_id, &Role::Viewer).await {
        let applications = query_as!(
            Application,
            "SELECT application__id AS application_id, organization__id AS organization_id, name FROM event.application WHERE organization__id = $1",
            &qs.organization_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(|_| UnexpectedError::InternalServerError)?;

        Ok(Json(applications))
    } else {
        Err(UnexpectedError::Forbidden)
    }
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationPost {
    organization_id: Uuid,
    name: String,
}

/// Create a new application
#[api_v2_operation]
pub async fn add(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    body: Json<ApplicationPost>,
) -> Result<CreatedJson<Application>, CreateError> {
    if can_access_organization(&unstructured_claims, &body.organization_id, &Role::Editor).await {
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
        .map_err(|_| CreateError::InternalServerError)?;

        Ok(CreatedJson(application))
    } else {
        Err(CreateError::Forbidden)
    }
}

/// Get an application
#[api_v2_operation]
pub async fn show(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    application_id: Path<Uuid>,
) -> Result<Json<Application>, ShowError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &application_id,
        &Role::Viewer,
    )
    .await
    {
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
        .map_err(|_| ShowError::InternalServerError)?;

        match application {
            Some(a) => Ok(Json(a)),
            None => Err(ShowError::NotFound),
        }
    } else {
        Err(ShowError::Forbidden)
    }
}

/// Edit an application
#[api_v2_operation]
pub async fn edit(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    application_id: Path<Uuid>,
    body: Json<ApplicationPost>,
) -> Result<Json<Application>, EditError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &application_id,
        &Role::Editor,
    )
    .await
        && can_access_organization(&unstructured_claims, &body.organization_id, &Role::Editor).await
    {
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
        .map_err(|_| EditError::InternalServerError)?;

        match application {
            Some(a) => Ok(Json(a)),
            None => Err(EditError::NotFound),
        }
    } else {
        Err(EditError::Forbidden)
    }
}

/// Destroy an application
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    application_id: Path<Uuid>,
) -> Result<NoContent, ShowError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &application_id,
        &Role::Editor,
    )
    .await
    {
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
        .map_err(|_| ShowError::InternalServerError)?;

        match application {
            Some(a) => {
                query!(
                    "DELETE FROM event.application WHERE application__id = $1",
                    a.application_id
                )
                .execute(&state.db)
                .await
                .map_err(|_| ShowError::InternalServerError)?;
                Ok(NoContent)
            }
            None => Err(ShowError::NotFound),
        }
    } else {
        Err(ShowError::Forbidden)
    }
}
