use actix_web::web::ReqData;
use biscuit_auth::Biscuit;
use chrono::{DateTime, Utc};
use paperclip::actix::web::{Data, Json, Path, Query};
use paperclip::actix::{Apiv2Schema, CreatedJson, NoContent, api_v2_operation};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use tracing::error;
use uuid::Uuid;
use validator::Validate;

use crate::hook0_client::{
    EventApplicationSecretCreated, EventApplicationSecretRemoved, EventApplicationSecretUpdated,
    Hook0ClientEvent,
};
use crate::iam::{Action, authorize_for_application, get_owner_organization};
use crate::openapi::OaBiscuit;
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
    description = "Generates a new API token for an application.",
    operation_id = "applicationSecrets.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<ApplicationSecretPost>,
) -> Result<CreatedJson<ApplicationSecret>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationSecretCreate {
            application_id: &body.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventApplicationSecretCreated {
            organization_id: get_owner_organization(&state.db, &body.application_id)
                .await
                .unwrap_or(Uuid::nil()),
            application_id: body.application_id,
            name: application_secret.name.to_owned(),
            created_at: application_secret.created_at.to_owned(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    // Activation conversion (Google Ads): the first API key an organization
    // creates is its activation signal. Fired at most once per organization
    // (atomic claim). Strictly fire-and-forget — it must NEVER block or fail
    // application secret creation, so every error here is logged, not returned.
    // Only attributed to the organization created at signup (which carries the
    // gclid); activation in a different org simply finds no attribution row.
    if let Some(client) = state.google_ads.as_ref().cloned()
        && client.has_activation_conversion()
        && let Some(organization_id) = get_owner_organization(&state.db, &body.application_id).await
    {
        match crate::google_ads::claim_activation_gclid(&state.db, &organization_id).await {
            Ok(Some(gclid)) => {
                crate::google_ads::spawn_upload(
                    client,
                    gclid,
                    crate::google_ads::ConversionKind::Activation,
                );
                // Both conversions are now uploaded for this org → minimise.
                crate::google_ads::clear_gclid_if_fully_uploaded_by_org(
                    &state.db,
                    &organization_id,
                )
                .await;
            }
            Ok(None) => {}
            Err(e) => error!("activation conversion claim failed: {e}"),
        }
    }

    Ok(CreatedJson(application_secret))
}

#[api_v2_operation(
    summary = "List application secrets",
    description = "Retrieves all active API tokens for a given application.",
    operation_id = "applicationSecrets.read",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<ApplicationSecret>>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationSecretList {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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
    description = "Updates the name of an existing API token.",
    operation_id = "applicationSecrets.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_secret_token: Path<Uuid>,
    body: Json<ApplicationSecretPost>,
) -> Result<Json<ApplicationSecret>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationSecretEdit {
            application_id: &body.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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
        Some(a) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationSecretUpdated {
                    organization_id: get_owner_organization(&state.db, &body.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: body.application_id,
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
    summary = "Delete an application secret",
    description = "Marks an API token as revoked, preventing any further use. This operation is irreversible.",
    operation_id = "applicationSecrets.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_secret_token: Path<Uuid>,
    qs: Query<Qs>,
) -> Result<NoContent, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationSecretDelete {
            application_id: &qs.application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

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

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationSecretRemoved {
                    organization_id: get_owner_organization(&state.db, &qs.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: qs.application_id,
                    name: a.name.to_owned(),
                    token: a.token,
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
mod activation_conversion_tests {
    use crate::google_ads::test_support::{
        FakeGoogleAds, attribution_state, issue_user_token, seed_attribution, seed_membership,
        seed_org, seed_user, test_state,
    };
    use actix_web::{App, test, web};
    use sqlx::PgPool;
    use std::time::Duration;

    /// Full HTTP path: a gclid-attributed organization creating its first API
    /// key fires the activation conversion (with the activation conversion
    /// action) and minimises the gclid. Drives the real handlers + biscuit auth
    /// against a real Postgres, with only the Google Ads endpoint faked.
    #[sqlx::test]
    async fn creating_first_api_key_fires_activation_conversion(pool: PgPool) {
        let fake = FakeGoogleAds::start(200, "{}");
        let google_ads =
            crate::google_ads::test_client_with_base_url(fake.base_url.clone(), Some("777"));

        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let state = test_state(pool.clone(), private_key.clone(), Some(google_ads)).await;

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        // Signup already uploaded; activation pending.
        seed_attribution(&pool, user, org, "gclid-handler", true).await;

        // Mint + persist a user access token (the same path login uses) so the
        // auth middleware, which checks the token exists in iam.token, accepts it.
        let token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
            db: pool.clone(),
            biscuit_private_key: private_key.clone(),
            master_api_key: None,
            enable_application_secret_compatibility: true,
        };

        let app = test::init_service(
            App::new().app_data(web::Data::new(state)).service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/applications")
                            .wrap(biscuit_auth.clone())
                            .route("", web::post().to(crate::handlers::applications::create)),
                    )
                    .service(
                        web::scope("/application_secrets")
                            .wrap(biscuit_auth.clone())
                            .route("", web::post().to(super::create)),
                    ),
            ),
        )
        .await;

        // 1) Create an application via the real handler.
        let create_app = test::TestRequest::post()
            .uri("/api/v1/applications")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({"organization_id": org, "name": "e2e-app"}))
            .to_request();
        let resp = test::call_service(&app, create_app).await;
        assert!(
            resp.status().is_success(),
            "application creation failed: {}",
            resp.status()
        );
        let app_body: serde_json::Value = test::read_body_json(resp).await;
        let application_id = app_body["application_id"]
            .as_str()
            .expect("application_id in response")
            .to_string();

        // Creating the application must NOT fire the activation conversion.
        assert!(
            fake.requests().is_empty(),
            "activation must only fire on the first API key, not on app creation"
        );

        // 2) Create the first API key → fires the activation conversion.
        let create_key = test::TestRequest::post()
            .uri("/api/v1/application_secrets")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({"application_id": application_id, "name": "e2e-key"}))
            .to_request();
        let resp = test::call_service(&app, create_key).await;
        assert!(
            resp.status().is_success(),
            "API key creation failed: {}",
            resp.status()
        );
        let key_body: serde_json::Value = test::read_body_json(resp).await;
        assert!(key_body["token"].is_string(), "API key token returned");

        // The upload is fire-and-forget: wait for it to reach the fake endpoint.
        let reqs = fake.wait_for(1, Duration::from_secs(5)).await;
        assert_eq!(reqs.len(), 1, "exactly one activation upload");
        assert_eq!(reqs[0].path, "/customers/1234567890:uploadClickConversions");
        let body: serde_json::Value = serde_json::from_str(&reqs[0].body).expect("json body");
        assert_eq!(body["conversions"][0]["gclid"], "gclid-handler");
        assert_eq!(
            body["conversions"][0]["conversionAction"],
            "customers/1234567890/conversionActions/777"
        );

        // The attribution lifecycle ran synchronously in the handler:
        // activation claimed, and (both conversions done) the gclid minimised.
        let (gclid, activation_uploaded) = attribution_state(&pool, org).await;
        assert!(activation_uploaded, "activation_uploaded_at set");
        assert_eq!(gclid, None, "gclid minimised after both conversions");
    }
}
