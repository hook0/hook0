use chrono::{DateTime, Utc};
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::errors::*;

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct ApplicationSecret {
    application__id: Uuid,
    name: Option<String>,
    token: Uuid,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct QS {
    application_id: Uuid,
}

/// List application secrets
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    qs: Query<QS>,
) -> Result<Json<Vec<ApplicationSecret>>, UnexpectedError> {
    let application_secrets = query_as!(
        ApplicationSecret,
        "
            SELECT application__id, name, token, created_at, deleted_at
            FROM event.application_secret
            WHERE application__id = $1
        ",
        &qs.application_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| UnexpectedError::InternalServerError)?;

    Ok(Json(application_secrets))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationSecretPost {
    application_id: Uuid,
    name: Option<String>,
}

/// Create a new application secret
#[api_v2_operation]
pub async fn add(
    state: Data<crate::State>,
    body: Json<ApplicationSecretPost>,
) -> Result<CreatedJson<ApplicationSecret>, CreateError> {
    let application_secret = query_as!(
        ApplicationSecret,
        "
            INSERT INTO event.application_secret (application__id, name)
            VALUES ($1, $2)
            RETURNING application__id, name, token, created_at, deleted_at
        ",
        &body.application_id,
        body.name,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| CreateError::InternalServerError)?;

    Ok(CreatedJson(application_secret))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationSecretPut {
    name: Option<String>,
}

/// Edit an application secret
#[api_v2_operation]
pub async fn edit(
    state: Data<crate::State>,
    application_secret_token: Path<Uuid>,
    body: Json<ApplicationSecretPut>,
) -> Result<Json<ApplicationSecret>, EditError> {
    let application_secret = query_as!(
        ApplicationSecret,
        "
            UPDATE event.application_secret
            SET name = $1
            WHERE token = $2
            RETURNING application__id, name, token, created_at, deleted_at
        ",
        body.name,
        application_secret_token.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| EditError::InternalServerError)?;

    match application_secret {
        Some(a) => Ok(Json(a)),
        None => Err(EditError::NotFound),
    }
}

/// Destroy an application secret
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    application_secret_token: Path<Uuid>,
    qs: Query<QS>,
) -> Result<NoContent, ShowError> {
    let application_secret = query_as!(
        ApplicationSecret,
        "
            SELECT application__id, name, token, created_at, deleted_at
            FROM event.application_secret
            WHERE application__id = $1 AND token = $2
        ",
        &qs.application_id,
        &application_secret_token.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match application_secret {
        Some(a) => {
            query!(
                "
                    UPDATE event.application_secret
                    SET deleted_at = statement_timestamp()
                    WHERE application__id = $1 AND token = $2
                ",
                &a.application__id,
                &a.token
            )
            .execute(&state.db)
            .await
            .map_err(|_| ShowError::InternalServerError)?;
            Ok(NoContent)
        }
        None => Err(ShowError::NotFound),
    }
}
