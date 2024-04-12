use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::Utc;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{api_v2_operation, Apiv2Schema, CreatedJson, NoContent};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{EventEventTypeCreated, EventEventTypeRemoved, Hook0ClientEvent};
use crate::iam::{authorize_for_application, get_owner_organization, Action};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct EventType {
    service_name: String,
    resource_type_name: String,
    verb_name: String,
    // status
    event_type_name: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    application_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct EventTypePost {
    application_id: Uuid,
    #[validate(non_control_character, length(min = 1, max = 50))]
    service: String,
    #[validate(non_control_character, length(min = 1, max = 50))]
    resource_type: String,
    #[validate(non_control_character, length(min = 1, max = 50))]
    verb: String,
}

#[api_v2_operation(
    summary = "Create a new event type",
    description = "",
    operation_id = "eventTypes.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<EventTypePost>,
) -> Result<CreatedJson<EventType>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeCreate {
            application_id: &body.application_id,
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

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    query!(
        "
            INSERT INTO event.service (application__id, service__name)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
        ",
        &body.application_id,
        &body.service,
    )
    .execute(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    query!(
        "
            INSERT INTO event.resource_type (application__id, service__name, resource_type__name)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
        ",
        &body.application_id,
        &body.service,
        &body.resource_type,
    )
    .execute(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    query!(
        "
            INSERT INTO event.verb (application__id, verb__name)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
        ",
        &body.application_id,
        &body.verb,
    )
    .execute(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    let event_type = query_as!(
            EventType,
            "
                INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name)
                VALUES ($1, $2, $3, $4)
                RETURNING service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
            ",
            &body.application_id,
            &body.service,
            &body.resource_type,
            &body.verb
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Hook0Problem::from)?;

    tx.commit().await.map_err(Hook0Problem::from)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventEventTypeCreated {
            organization_id: get_owner_organization(&state.db, &body.application_id)
                .await
                .unwrap_or(Uuid::nil()),
            application_id: body.application_id,
            service_name: event_type.service_name.to_owned(),
            resource_type_name: event_type.resource_type_name.to_owned(),
            verb_name: event_type.verb_name.to_owned(),
            event_type_name: event_type.event_type_name.to_owned(),
            created_at: Utc::now(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(event_type))
}

#[api_v2_operation(
    summary = "List event types",
    description = "",
    operation_id = "eventTypes.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<EventType>>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeList {
            application_id: &qs.application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let event_types = query_as!(
            EventType,
            "
                SELECT service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
                FROM event.event_type
                WHERE application__id = $1
                ORDER BY event_type__name ASC
            ",
            &qs.application_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    Ok(Json(event_types))
}

#[api_v2_operation(
    summary = "Get an event type by its name",
    description = "",
    operation_id = "eventTypes.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_type_name: Path<String>,
    qs: Query<Qs>,
) -> Result<Json<EventType>, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeGet {
            application_id: &qs.application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let event_type = query_as!(
            EventType,
            "
                SELECT service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
                FROM event.event_type
                WHERE application__id = $1 AND event_type__name = $2
            ",
            &qs.application_id,
            &event_type_name.into_inner(),
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    match event_type {
        Some(a) => Ok(Json(a)),
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "Delete an event type",
    description = "",
    operation_id = "eventTypes.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_type_name: Path<String>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    if authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeDelete {
            application_id: &qs.application_id,
        },
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    let application_id = qs.application_id;
    let event_type = query_as!(
            EventType,
            "
                SELECT service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
                FROM event.event_type
                WHERE application__id = $1 AND event_type__name = $2
            ",
            &application_id,
            &event_type_name.into_inner(),
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    match event_type {
        Some(a) => {
            query!(
                "
                    DELETE FROM event.event_type
                    WHERE application__id = $1 AND event_type__name = $2
                ",
                &application_id,
                &a.event_type_name,
            )
            .execute(&state.db)
            .await
            .map_err(Hook0Problem::from)?;

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventEventTypeRemoved {
                    organization_id: get_owner_organization(&state.db, &qs.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: qs.application_id,
                    event_type_name: a.event_type_name,
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
