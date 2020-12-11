use log::error;
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
pub struct EventType {
    service__name: String,
    resource_type__name: String,
    verb__name: String,
    // status
    event_type__name: String,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
#[allow(non_snake_case)]
pub struct QS {
    application_id: Uuid,
}

/// List event types
#[api_v2_operation]
pub async fn list(
    state: Data<crate::State>,
    qs: Query<QS>,
) -> Result<Json<Vec<EventType>>, UnexpectedError> {
    let event_types = query_as!(
        EventType,
        "
            SELECT service__name, resource_type__name, verb__name, event_type__name
            FROM event.event_type
            WHERE application__id = $1
            ORDER BY event_type__name ASC
        ",
        &qs.application_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| UnexpectedError::InternalServerError)?;

    Ok(Json(event_types))
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct EventTypePost {
    application_id: Uuid,
    service: String,
    resource_type: String,
    verb: String,
}

/// Create a new event type
#[api_v2_operation]
pub async fn add(
    state: Data<crate::State>,
    body: Json<EventTypePost>,
) -> Result<CreatedJson<EventType>, CreateError> {
    let mut tx = state.db.begin().await.map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

    query!(
        "
        INSERT INTO event.service (application__id, service__name)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
    ",
        &body.application_id,
        &body.service,
    )
    .execute(&mut tx)
    .await
    .map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

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
    .execute(&mut tx)
    .await
    .map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

    query!(
        "
        INSERT INTO event.verb (application__id, verb__name)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
    ",
        &body.application_id,
        &body.verb,
    )
    .execute(&mut tx)
    .await
    .map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

    let event_type = query_as!(
        EventType,
        "
            INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name, status)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (event_type__name) DO UPDATE SET status = EXCLUDED.status
            RETURNING service__name, resource_type__name, verb__name, event_type__name
        ",
        &body.application_id,
        &body.service,
        &body.resource_type,
        &body.verb,
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(), // TODO: handle properly when lib_fst is setup
    )
    .fetch_one(&mut tx)
    .await
    .map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

    tx.commit().await.map_err(|e| {
        error!("{}", &e);
        CreateError::InternalServerError
    })?;

    Ok(CreatedJson(event_type))
}

/// Get an event type
#[api_v2_operation]
pub async fn show(
    state: Data<crate::State>,
    event_type_name: Path<String>,
    qs: Query<QS>,
) -> Result<Json<EventType>, ShowError> {
    let event_type = query_as!(
        EventType,
        "
            SELECT service__name, resource_type__name, verb__name, event_type__name
            FROM event.event_type
            WHERE application__id = $1 AND event_type__name = $2
        ",
        &qs.application_id,
        &event_type_name.into_inner(),
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match event_type {
        Some(a) => Ok(Json(a)),
        None => Err(ShowError::NotFound),
    }
}

/// Destroy an event type
#[api_v2_operation]
pub async fn destroy(
    state: Data<crate::State>,
    event_type_name: Path<String>,
    qs: Query<QS>,
) -> Result<NoContent, ShowError> {
    let application_id = qs.application_id;
    let event_type = query_as!(
        EventType,
        "
            SELECT service__name, resource_type__name, verb__name, event_type__name
            FROM event.event_type
            WHERE application__id = $1 AND event_type__name = $2
        ",
        &application_id,
        &event_type_name.into_inner(),
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| ShowError::InternalServerError)?;

    match event_type {
        Some(a) => {
            query!(
                "
                    DELETE FROM event.event_type
                    WHERE application__id = $1 AND event_type__name = $2
                ",
                &application_id,
                a.event_type__name,
            )
            .execute(&state.db)
            .await
            .map_err(|_| ShowError::InternalServerError)?;
            Ok(NoContent)
        }
        None => Err(ShowError::NotFound),
    }
}
