use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::Utc;
use log::error;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, query_scalar};
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{EventEventTypeCreated, EventEventTypeRemoved, Hook0ClientEvent};
use crate::iam::{Action, authorize_for_application, get_owner_organization};
use crate::openapi::OaBiscuit;
use crate::problems::Hook0Problem;
use crate::query_builder::QueryBuilder;
use crate::quotas::Quota;

#[derive(Debug, Serialize, Apiv2Schema, sqlx::FromRow)]
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
    #[serde(default, rename = "service.name")]
    service_name: Option<String>,
    #[serde(default, rename = "resource.name")]
    resource_name: Option<String>,
    #[serde(default, rename = "verb.name")]
    verb_name: Option<String>,
    #[serde(default, rename = "application.name")]
    application_name: Option<String>,
    #[serde(default)]
    name: Option<String>, // event_type__name
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
    description = "Defines a new event type for an application. Event types help categorize and structure emitted events, making them easier to manage and subscribe to.",
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let quota_limit = state
        .quotas
        .get_limit_for_application(
            &state.db,
            Quota::EventTypesPerApplication,
            &body.application_id,
        )
        .await?;

    let quota_current = query_scalar!(
        r#"
            SELECT COUNT(*) AS "val!"
            FROM event.event_type
            WHERE application__id = $1
                AND deactivated_at IS NULL
        "#,
        &body.application_id,
    )
    .fetch_one(&state.db)
    .await?;

    if quota_current >= i64::from(quota_limit) {
        return Err(Hook0Problem::TooManyEventTypesPerApplication(quota_limit));
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
                ON CONFLICT (application__id, event_type__name) DO UPDATE SET deactivated_at = NULL
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
    description = "Retrieves all active event types registered for a given application. Each event type is uniquely identified by its service, resource type, and verb.",
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
    )
    .await
    .is_err()
    {
        return Err(Hook0Problem::Forbidden);
    }

    // Build dynamic WHERE conditions using QueryBuilder
    let mut query_builder = QueryBuilder::new(
        "et.application__id = $1 AND et.deactivated_at IS NULL".to_string(),
        2, // Next parameter index after application_id
    );

    // Add all filters
    query_builder.add_string_filter("et.service__name", qs.service_name.clone());
    query_builder.add_string_filter("et.resource_type__name", qs.resource_name.clone());
    query_builder.add_string_filter("et.verb__name", qs.verb_name.clone());
    query_builder.add_string_filter("et.event_type__name", qs.name.clone());

    // Add application.name filter (requires JOIN)
    let needs_app_join = qs.application_name.is_some();
    if needs_app_join {
        query_builder.add_string_filter("app.name", qs.application_name.clone());
    }

    let where_clause = query_builder.build_where_clause();
    let sql = if needs_app_join {
        format!(
            "
                SELECT et.service__name AS service_name, et.resource_type__name AS resource_type_name, et.verb__name AS verb_name, et.event_type__name AS event_type_name
                FROM event.event_type AS et
                INNER JOIN event.application AS app ON app.application__id = et.application__id
                WHERE {}
                ORDER BY et.event_type__name ASC
            ",
            where_clause
        )
    } else {
        format!(
            "
                SELECT service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
                FROM event.event_type AS et
                WHERE {}
                ORDER BY event_type__name ASC
            ",
            where_clause
        )
    };

    let query = query_as::<_, EventType>(&sql).bind(qs.application_id);
    let query = query_builder.bind_params(query);

    let event_types = query
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    Ok(Json(event_types))
}

#[api_v2_operation(
    summary = "Get an event type by its name",
    description = "Retrieves details of a specific event type if it exists within the given application. Event types define the structure of emitted events.",
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
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
                WHERE application__id = $1 AND event_type__name = $2 AND deactivated_at IS NULL
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
    description = "Marks an event type as deactivated, preventing it from being used for new event emissions. Existing events using this type remain unaffected.",
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
        state.max_authorization_time_in_ms,
        state.debug_authorizer,
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
                WHERE application__id = $1 AND event_type__name = $2 AND deactivated_at IS NULL
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
                    UPDATE event.event_type
                    SET deactivated_at = statement_timestamp()
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
