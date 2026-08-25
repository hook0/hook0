use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::Utc;
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{EventEventTypeCreated, EventEventTypeRemoved, Hook0ClientEvent};
use crate::iam::{Action, authorize_for_application, get_owner_organization};
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
    description = "Registers a new event type for an application. Event types follow the pattern 'service.resource.verb' (e.g., 'order.payment.completed'). Subscriptions can filter which event types trigger webhooks.",
    operation_id = "eventTypes.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp", "sdk")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<EventTypePost>,
) -> Result<CreatedJson<EventType>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeCreate {
            application_id: &body.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    state
        .quotas
        .enforce_event_types_per_application(&mut tx, &body.application_id)
        .await?;

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
    description = "Retrieves all active event types for an application. Event types follow the pattern 'service.resource.verb'. Use application_id query parameter to filter by application.",
    operation_id = "eventTypes.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp", "sdk")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<EventType>>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeList {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    let event_types = query_as!(
            EventType,
            "
                SELECT service__name AS service_name, resource_type__name AS resource_type_name, verb__name AS verb_name, event_type__name AS event_type_name
                FROM event.event_type
                WHERE application__id = $1 AND deactivated_at IS NULL
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
    description = "Retrieves details of a specific event type by its name (e.g., 'order.payment.completed'). Returns the service, resource type, and verb components.",
    operation_id = "eventTypes.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp", "sdk")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_type_name: Path<String>,
    qs: Query<Qs>,
) -> Result<Json<EventType>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeGet {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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
    description = "Deactivates an event type, preventing it from being used for new events. Existing events using this type remain unaffected. Use this to clean up unused event types.",
    operation_id = "eventTypes.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Events Management", "mcp", "sdk")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    event_type_name: Path<String>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::EventTypeDelete {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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

#[cfg(test)]
mod quota_race_tests {
    use crate::google_ads::test_support::{
        issue_user_token, seed_membership, seed_org, seed_user, test_state,
    };
    use crate::quotas::{QuotaLimits, QuotaValue, Quotas};
    use actix_web::{App, test, web};
    use futures_util::future::join_all;
    use sqlx::PgPool;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    const CONCURRENT_CALLERS: usize = 8;
    const EVENT_TYPES_ALLOWED: QuotaValue = 1;

    /// Same contract as the application limit, on the other side of the
    /// application boundary: the count the check reads has to stay true until
    /// the write lands, or callers arriving together all clear a limit only one
    /// of them should have passed.
    #[sqlx::test]
    async fn concurrent_creations_cannot_take_an_application_past_its_event_type_limit(
        pool: PgPool,
    ) {
        // Wide enough that the calls below are genuinely in flight together;
        // on a sequential pool the limit would hold for want of a connection.
        let options = (*pool.connect_options()).clone();
        let pool = PgPoolOptions::new()
            .max_connections(CONCURRENT_CALLERS as u32 + 2)
            .connect_with(options)
            .await
            .expect("open a wider pool on the test database");

        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let application = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, 'race')",
        )
        .bind(application)
        .bind(org)
        .execute(&pool)
        .await
        .expect("seed the application the event types hang off");

        let mut state = test_state(pool.clone(), private_key.clone(), None).await;
        state.quotas = Quotas::new(
            true,
            QuotaLimits {
                global_event_types_per_application_limit: EVENT_TYPES_ALLOWED,
                global_applications_per_organization_limit: QuotaValue::MAX,
                global_members_per_organization_limit: QuotaValue::MAX,
                global_events_per_day_limit: QuotaValue::MAX,
                global_days_of_events_retention_limit: QuotaValue::MAX,
                global_subscriptions_per_application_limit: QuotaValue::MAX,
            },
        );

        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };

        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1").service(
                    web::scope("/event_types")
                        .wrap(biscuit_auth)
                        .route("", web::post().to(super::create)),
                ),
            ),
        )
        .await;

        let app_ref = &app;
        let calls = (0..CONCURRENT_CALLERS).map(|i| {
            let request = test::TestRequest::post()
                .uri("/api/v1/event_types")
                .insert_header(("Authorization", format!("Bearer {user_token}")))
                .set_json(serde_json::json!({
                    "application_id": application,
                    "service": "race",
                    "resource_type": format!("r{i}"),
                    "verb": "created",
                }))
                .to_request();
            test::call_service(app_ref, request)
        });

        let accepted = join_all(calls)
            .await
            .iter()
            .filter(|response| response.status().is_success())
            .count();

        let stored: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM event.event_type WHERE application__id = $1")
                .bind(application)
                .fetch_one(&pool)
                .await
                .expect("count the event types the application ended up with");

        assert_eq!(
            stored,
            i64::from(EVENT_TYPES_ALLOWED),
            "the application is left holding more event types than its plan allows"
        );
        assert_eq!(
            accepted, EVENT_TYPES_ALLOWED as usize,
            "more callers were told their event type was created than the plan allows"
        );
    }
}
