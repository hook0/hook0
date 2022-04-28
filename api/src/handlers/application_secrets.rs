use chrono::{DateTime, Utc};
use paperclip::actix::{
    api_v2_operation,
    web::{Data, Json, Path, Query},
    Apiv2Schema, CreatedJson, NoContent,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{AuthProof, Role};
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
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

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ApplicationSecretPost {
    application_id: Uuid,
    #[validate(non_control_character, length(max = 50))]
    name: Option<String>,
}

#[api_v2_operation(
    summary = "Create a new application secret",
    description = "",
    operation_id = "applicationSecrets.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn create(
    state: Data<crate::State>,
    auth: AuthProof,
    body: Json<ApplicationSecretPost>,
) -> Result<CreatedJson<ApplicationSecret>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &body.application_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

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
    .map_err(Hook0Problem::from)?;

    Ok(CreatedJson(application_secret))
}

#[api_v2_operation(
    summary = "List application secrets",
    description = "",
    operation_id = "applicationSecrets.read",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn list(
    state: Data<crate::State>,
    auth: AuthProof,
    qs: Query<Qs>,
) -> Result<Json<Vec<ApplicationSecret>>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &qs.application_id, &Role::Viewer)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let application_secrets = query_as!(
        ApplicationSecret,
        "
            SELECT name, token, created_at, deleted_at
            FROM event.application_secret
            WHERE deleted_at IS NULL AND application__id = $1
            ORDER BY created_at ASC
        ",
        &qs.application_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(application_secrets))
}

#[api_v2_operation(
    summary = "Update an application secret",
    description = "",
    operation_id = "applicationSecrets.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn update(
    state: Data<crate::State>,
    auth: AuthProof,
    application_secret_token: Path<Uuid>,
    body: Json<ApplicationSecretPost>,
) -> Result<Json<ApplicationSecret>, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &body.application_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

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
    .map_err(Hook0Problem::from)?;

    match application_secret {
        Some(a) => Ok(Json(a)),
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Delete an application secret",
    description = "",
    operation_id = "applicationSecrets.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    auth: AuthProof,
    application_secret_token: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    if auth
        .can_access_application(&state.db, &qs.application_id, &Role::Editor)
        .await
        .is_none()
    {
        return Err(Hook0Problem::Forbidden);
    }

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
    .map_err(Hook0Problem::from)?;

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
            .map_err(Hook0Problem::from)?;
            Ok(NoContent)
        }
        None => Err(Hook0Problem::NotFound),
    }
}
