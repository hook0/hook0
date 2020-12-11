use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::errors::*;

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct Application {
    application__id: Uuid,
    name: String,
}

/// List applications
#[api_v2_operation]
pub async fn list(state: Data<crate::State>) -> Result<Json<Vec<Application>>, UnexpectedError> {
    let applications = query_as!(
        Application,
        "SELECT application__id, name FROM event.application",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| UnexpectedError::InternalServerError)?;

    Ok(Json(applications))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationPost {
    name: String,
}

/// Create a new application
#[api_v2_operation]
pub async fn add(
    state: Data<crate::State>,
    body: Json<ApplicationPost>,
) -> Result<CreatedJson<Application>, CreateError> {
    let application = query_as!(
        Application,
        "INSERT INTO event.application (name) VALUES ($1) RETURNING application__id, name",
        body.name,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| CreateError::InternalServerError)?;

    Ok(CreatedJson(application))
}

/// Get an application
#[api_v2_operation]
pub async fn show(
    state: Data<crate::State>,
    application_id: Path<Uuid>,
) -> Result<Json<Application>, ShowError> {
    let application = query_as!(
        Application,
        "SELECT application__id, name FROM event.application WHERE application__id = $1",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match application {
        Some(a) => Ok(Json(a)),
        None => Err(ShowError::NotFound),
    }
}

/// Edit an application
#[api_v2_operation]
pub async fn edit(
    state: Data<crate::State>,
    application_id: Path<Uuid>,
    body: Json<ApplicationPost>,
) -> Result<Json<Application>, EditError> {
    let application = query_as!(
        Application,
        "UPDATE event.application SET name = $1 WHERE application__id = $2 RETURNING application__id, name",
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
}

/// Destroy an application
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    application_id: Path<Uuid>,
) -> Result<NoContent, ShowError> {
    let application = query_as!(
        Application,
        "SELECT application__id, name FROM event.application WHERE application__id = $1",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match application {
        Some(a) => {
            query!(
                "DELETE FROM event.application WHERE application__id = $1",
                a.application__id
            )
            .execute(&state.db)
            .await
            .map_err(|_| ShowError::InternalServerError)?;
            Ok(NoContent)
        }
        None => Err(ShowError::NotFound),
    }
}
