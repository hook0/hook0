use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson, NoContent};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::iam::{authorize, Action, RootToken};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::{
    hook0_client::{
        EventServiceTokenCreated, EventServiceTokenRemoved, EventServiceTokenUpdated,
        Hook0ClientEvent,
    },
    iam::create_service_access_token,
};

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ServiceToken {
    pub token_id: Uuid,
    pub name: String,
    pub biscuit: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    organization_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ServiceTokenPost {
    organization_id: Uuid,
    #[validate(non_control_character, length(max = 50))]
    name: String,
}

#[api_v2_operation(
    summary = "Create a new service token",
    description = "",
    operation_id = "serviceToken.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Organization Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<ServiceTokenPost>,
) -> Result<CreatedJson<ServiceToken>, Hook0Problem> {
    let organization_id = body.organization_id;

    if authorize(&biscuit, Some(organization_id), Action::ServiceTokenCreate).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let token_id = Uuid::new_v4();

    match create_service_access_token(&state.biscuit_private_key, token_id, organization_id) {
        Ok(RootToken {
            serialized_biscuit,
            revocation_id,
            ..
        }) => {
            let service_token = query_as!(
                ServiceToken,
                r#"
                    INSERT INTO iam.token (type, revocation_id, organization__id, name, biscuit)
                    VALUES ('service_access', $1, $2, $3, $4)
                    RETURNING token__id AS token_id, name AS "name!", biscuit AS "biscuit!", created_at
                "#,
                revocation_id,
                Some(organization_id),
                Some(&body.name),
                Some(serialized_biscuit),
            )
            .fetch_one(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventServiceTokenCreated {
                    token_id: service_token.token_id,
                    organization_id,
                    name: service_token.name.to_owned(),
                    created_at: service_token.created_at.to_owned(),
                }
                .into();
                if let Err(e) = hook0_client
                    .send_event(&hook0_client_event.mk_hook0_event())
                    .await
                {
                    error!("Hook0ClientError: {e}");
                };
            }

            Ok(CreatedJson(service_token))
        }
        Err(e) => {
            error!("Could not create a Biscuit (service access token): {e}");
            Err(Hook0Problem::InternalServerError)
        }
    }
}

#[api_v2_operation(
    summary = "List service tokens",
    description = "",
    operation_id = "serviceToken.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Organization Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<ServiceToken>>, Hook0Problem> {
    let organization_id = qs.organization_id;

    if authorize(&biscuit, Some(organization_id), Action::ServiceTokenList).is_err() {
        return Err(Hook0Problem::Forbidden);
    }

    let service_tokens = query_as!(
        ServiceToken,
        r#"
            SELECT token__id AS token_id, name AS "name!", biscuit AS "biscuit!", created_at
            FROM iam.token
            WHERE organization__id = $1
                AND type = 'service_access'
                AND (expired_at IS NULL OR expired_at > statement_timestamp())
            ORDER BY created_at ASC
        "#,
        &organization_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    Ok(Json(service_tokens))
}

#[api_v2_operation(
    summary = "Edit a service token",
    description = "",
    operation_id = "serviceToken.edit",
    consumes = "application/json",
    produces = "application/json",
    tags("Organization Management")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    token_id: Path<Uuid>,
    body: Json<ServiceTokenPost>,
) -> Result<Json<ServiceToken>, Hook0Problem> {
    let organization_id = body.organization_id;
    let token_id = token_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::ServiceTokenEdit {
            service_token_id: &token_id,
        },
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let service_token = query_as!(
        ServiceToken,
        r#"
            UPDATE iam.token
            SET name = $1
            WHERE token__id = $2
                AND type = 'service_access'
                AND organization__id = $3
                AND (expired_at IS NULL OR expired_at > statement_timestamp())
            RETURNING token__id AS token_id, name AS "name!", biscuit AS "biscuit!", created_at
        "#,
        body.name,
        &token_id,
        &organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match service_token {
        Some(a) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventServiceTokenUpdated {
                    token_id,
                    organization_id: body.organization_id,
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
    summary = "Delete a service token",
    description = "",
    operation_id = "serviceToken.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Organization Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    token_id: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    let organization_id = qs.organization_id;
    let token_id = token_id.into_inner();

    if authorize(
        &biscuit,
        Some(organization_id),
        Action::ServiceTokenDelete {
            service_token_id: &token_id,
        },
    )
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let service_token = query_as!(
        ServiceToken,
        r#"
            SELECT token__id AS token_id, name AS "name!", biscuit AS "biscuit!", created_at
            FROM iam.token
            WHERE token__id = $1
                AND type = 'service_access'
                AND organization__id = $2
                AND (expired_at IS NULL OR expired_at > statement_timestamp())
        "#,
        &token_id,
        &qs.organization_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match service_token {
        Some(st) => {
            query!(
                "
                    UPDATE iam.token
                    SET expired_at = statement_timestamp()
                    WHERE token__id = $1
                        AND type = 'service_access'
                        AND organization__id = $2
                ",
                &token_id,
                &qs.organization_id,
            )
            .execute(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventServiceTokenRemoved {
                    token_id,
                    organization_id,
                    name: st.name.to_owned(),
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
