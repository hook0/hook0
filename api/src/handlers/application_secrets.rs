use actix_web_middleware_keycloak_auth::UnstructuredClaims;
use chrono::{DateTime, Utc};
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query, ReqData},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::errors::*;
use crate::iam::{can_access_application, Role};

#[derive(Debug, Serialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct ApplicationSecret {
    pub name: Option<String>,
    pub token: Uuid,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
}

/// List application secrets
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    qs: Query<Qs>,
) -> Result<Json<Vec<ApplicationSecret>>, UnexpectedError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Viewer,
    )
    .await
    {
        let application_secrets = query_as!(
            ApplicationSecret,
            "
                SELECT name, token, created_at, deleted_at
                FROM event.application_secret
                WHERE application__id = $1
                ORDER BY created_at ASC
            ",
            &qs.application_id,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|_| UnexpectedError::InternalServerError)?;

        Ok(Json(application_secrets))
    } else {
        Err(UnexpectedError::Forbidden)
    }
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
    unstructured_claims: ReqData<UnstructuredClaims>,
    body: Json<ApplicationSecretPost>,
) -> Result<CreatedJson<ApplicationSecret>, CreateError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        let application_secret = query_as!(
            ApplicationSecret,
            "
                INSERT INTO event.application_secret (application__id, name)
                VALUES ($1, $2)
                RETURNING name, token, created_at, deleted_at
            ",
            &body.application_id,
            body.name,
        )
        .fetch_one(&state.db)
        .await
        .map_err(|_| CreateError::InternalServerError)?;

        Ok(CreatedJson(application_secret))
    } else {
        Err(CreateError::Forbidden)
    }
}

/// Edit an application secret
#[api_v2_operation]
pub async fn edit(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    application_secret_token: Path<Uuid>,
    body: Json<ApplicationSecretPost>,
) -> Result<Json<ApplicationSecret>, EditError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &body.application_id,
        &Role::Editor,
    )
    .await
    {
        let application_secret = query_as!(
            ApplicationSecret,
            "
                UPDATE event.application_secret
                SET name = $1
                WHERE application__id = $2 AND token = $3
                RETURNING name, token, created_at, deleted_at
            ",
            body.name,
            &body.application_id,
            &application_secret_token.into_inner()
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|_| EditError::InternalServerError)?;

        match application_secret {
            Some(a) => Ok(Json(a)),
            None => Err(EditError::NotFound),
        }
    } else {
        Err(EditError::Forbidden)
    }
}

/// Destroy an application secret
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    unstructured_claims: ReqData<UnstructuredClaims>,
    application_secret_token: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, ShowError> {
    if can_access_application(
        &state.db,
        &unstructured_claims,
        &qs.application_id,
        &Role::Editor,
    )
    .await
    {
        let application_id = qs.application_id;
        let application_secret = query_as!(
            ApplicationSecret,
            "
                SELECT name, token, created_at, deleted_at
                FROM event.application_secret
                WHERE application__id = $1 AND token = $2
            ",
            &application_id,
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
                    &application_id,
                    &a.token
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
