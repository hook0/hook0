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
    EventApplicationCreated, EventApplicationRemoved, EventApplicationSecretCreated,
    EventApplicationUpdated, Hook0ClientEvent,
};
use crate::iam::{
    Action, authorize_for_application, authorize_for_organization, get_owner_organization,
};
use crate::onboarding::{ApplicationOnboardingSteps, get_application_onboarding_steps};
use crate::openapi::OaBiscuit;
use crate::opentelemetry::report_cancelled_request_attempts;
use crate::problems::Hook0Problem;
use crate::quotas::{Quota, QuotaValue};

/// Name given to the application secret that is automatically provisioned when
/// an application is created, so the application is immediately usable without a
/// separate manual step.
const DEFAULT_APPLICATION_SECRET_NAME: &str = "Default";

/// A Hook0 application.
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Application {
    /// Unique identifier of the application.
    application_id: Uuid,
    /// UUID of the organization this application belongs to.
    organization_id: Uuid,
    /// Name of the application. Length: 2-50 characters.
    name: String,
}

/// Detailed information about a Hook0 application.
#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationInfo {
    /// Unique identifier of the application.
    application_id: Uuid,
    /// UUID of the organization this application belongs to.
    organization_id: Uuid,
    /// Name of the application. Length: 2-50 characters.
    name: String,
    /// Quota limits for this application.
    quotas: ApplicationQuotas,
    /// Current consumption metrics for this application.
    consumption: ApplicationConsumption,
    /// Onboarding completion status for this application.
    onboarding_steps: ApplicationOnboardingSteps,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct ApplicationConsumption {
    events_per_day: Option<i32>,
}

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct ApplicationQuotas {
    events_per_day_limit: QuotaValue,
    days_of_events_retention_limit: QuotaValue,
}

#[derive(Debug, Serialize, Deserialize, Apiv2Schema)]
pub struct Qs {
    organization_id: Uuid,
}

/// Request body to create a new application.
#[derive(Debug, Serialize, Deserialize, Apiv2Schema, Validate)]
pub struct ApplicationPost {
    /// UUID of the organization this application belongs to.
    organization_id: Uuid,
    /// Name of the application. Length: 2-50 characters.
    #[validate(non_control_character, length(min = 2, max = 50))]
    name: String,
}

#[api_v2_operation(
    summary = "Create a new application",
    description = "Creates a new Hook0 application within an organization. An application is the container for event types, subscriptions, and events. Use this when setting up a new service that will emit or receive webhook events. Creation also provisions an application secret named 'Default', so there is no need to create a first API key afterwards: read it with applicationSecrets.read. That secret is scoped to this application alone, but within it grants full control (sending events, and managing event types, subscriptions and secrets), so it must be handled as a credential.",
    operation_id = "applications.create",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management", "mcp")
)]
pub async fn create(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    body: Json<ApplicationPost>,
) -> Result<CreatedJson<Application>, Hook0Problem> {
    authorize_for_organization(
        &biscuit,
        Some(body.organization_id),
        Action::ApplicationCreate,
        state.max_authorization_time,
        state.debug_authorizer,
    )?;

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    state
        .quotas
        .enforce_applications_per_organization(&mut tx, &body.organization_id)
        .await?;

    let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

    let application = query_as!(
            Application,
            "
                INSERT INTO event.application (organization__id, name) VALUES ($1, $2)
                RETURNING application__id AS application_id, organization__id AS organization_id, name
            ",
            body.organization_id, body.name,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Hook0Problem::from)?;

    // Provision a default application secret (API token) in the same
    // transaction so a freshly created application can send its first event
    // without a separate manual step. Reuses the standard secret generation:
    // the `token` column defaults to a random UUID. If this insert fails, the
    // whole transaction rolls back, so an application is never persisted without
    // its default secret. The token itself is deliberately not returned here:
    // callers read it back from the application secrets endpoint.
    struct ProvisionedSecret {
        created_at: DateTime<Utc>,
    }
    let provisioned_secret = query_as!(
        ProvisionedSecret,
        "
            INSERT INTO event.application_secret (application__id, name)
            VALUES ($1, $2)
            RETURNING created_at
        ",
        application.application_id,
        DEFAULT_APPLICATION_SECRET_NAME,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(Hook0Problem::from)?;

    tx.commit().await.map_err(Hook0Problem::from)?;

    if let Some(hook0_client) = state.hook0_client.as_ref() {
        let hook0_client_event: Hook0ClientEvent = EventApplicationCreated {
            organization_id: body.organization_id,
            application_id: application.application_id,
            name: application.name.to_owned(),
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };

        // The provisioned secret is a secret like any other: announce it on the
        // same feed as the ones created by hand, so subscribers of
        // `api.application_secret.created` see every secret that exists.
        let hook0_client_event: Hook0ClientEvent = EventApplicationSecretCreated {
            organization_id: body.organization_id,
            application_id: application.application_id,
            name: Some(DEFAULT_APPLICATION_SECRET_NAME.to_owned()),
            created_at: provisioned_secret.created_at,
        }
        .into();
        if let Err(e) = hook0_client
            .send_event(&hook0_client_event.mk_hook0_event())
            .await
        {
            error!("Hook0ClientError: {e}");
        };
    }

    Ok(CreatedJson(application))
}

#[api_v2_operation(
    summary = "Get an application by its ID",
    description = "Retrieves details about a specific application, including quotas, consumption statistics, and onboarding progress. Use this to check application health and usage limits.",
    operation_id = "applications.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management", "mcp")
)]
pub async fn get(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
) -> Result<Json<ApplicationInfo>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationGet {
            application_id: &application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    let application_id = application_id.into_inner();

    let application = query_as!(
        Application,
        "
            SELECT application__id AS application_id, organization__id AS organization_id, name
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        &application_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match application {
        Some(a) => {
            let quotas = ApplicationQuotas {
                events_per_day_limit: state
                    .quotas
                    .get_limit_for_application(&state.db, Quota::EventsPerDay, &application_id)
                    .await?,
                days_of_events_retention_limit: state
                    .quotas
                    .get_limit_for_application(
                        &state.db,
                        Quota::DaysOfEventsRetention,
                        &application_id,
                    )
                    .await?,
            };

            let consumption = query_as!(
                ApplicationConsumption,
                "
                    SELECT COALESCE(amount, 0) as events_per_day
                    FROM event.events_per_day
                    WHERE application__id = $1
                    AND date = CURRENT_DATE
                ",
                &application_id,
            )
            .fetch_optional(&state.db)
            .await
            .map_err(Hook0Problem::from)?
            .unwrap_or(ApplicationConsumption {
                events_per_day: Some(0),
            });

            let onboarding_steps =
                get_application_onboarding_steps(&state.db, &application_id).await?;

            Ok(Json(ApplicationInfo {
                application_id: a.application_id,
                organization_id: a.organization_id,
                name: a.name,
                quotas,
                consumption,
                onboarding_steps,
            }))
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[api_v2_operation(
    summary = "List applications",
    description = "Retrieves all applications within an organization. Each application contains event types, subscriptions, and events. Use organization_id query parameter to filter by organization.",
    operation_id = "applications.list",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management", "mcp")
)]
pub async fn list(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    qs: Query<Qs>,
) -> Result<Json<Vec<Application>>, Hook0Problem> {
    authorize_for_organization(
        &biscuit,
        Some(qs.organization_id),
        Action::ApplicationList,
        state.max_authorization_time,
        state.debug_authorizer,
    )?;

    let applications = query_as!(
            Application,
            "SELECT application__id AS application_id, organization__id AS organization_id, name FROM event.application WHERE organization__id = $1 AND deleted_at IS NULL",
            &qs.organization_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    Ok(Json(applications))
}

#[api_v2_operation(
    summary = "Edit an application",
    description = "Updates the name of an existing application. Use this to rename applications for better organization.",
    operation_id = "applications.update",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management", "mcp")
)]
pub async fn edit(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
    body: Json<ApplicationPost>,
) -> Result<Json<Application>, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationEdit {
            application_id: &application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    if let Err(e) = body.validate() {
        return Err(Hook0Problem::Validation(e));
    }

    let application = query_as!(
            Application,
            "
                UPDATE event.application
                SET name = $1 WHERE application__id = $2
                AND deleted_at IS NULL
                RETURNING application__id AS application_id, organization__id AS organization_id, name
            ",
            body.name,
            application_id.into_inner()
        )
        .fetch_optional(&state.db)
        .await
        .map_err(Hook0Problem::from)?;

    match application {
        Some(a) => {
            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationUpdated {
                    organization_id: get_owner_organization(&state.db, &a.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: a.application_id,
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
    summary = "Delete an application",
    description = "Permanently deletes an application. No more events will be emitted, and all active webhook subscriptions will be removed. All pending request attempts will be automatically marked as failed. This action is irreversible.",
    operation_id = "applications.delete",
    consumes = "application/json",
    produces = "application/json",
    tags("Applications Management", "mcp")
)]
pub async fn delete(
    state: Data<crate::State>,
    _: OaBiscuit,
    biscuit: ReqData<Biscuit>,
    application_id: Path<Uuid>,
) -> Result<NoContent, Hook0Problem> {
    authorize_for_application(
        &state.db,
        &biscuit,
        Action::ApplicationDelete {
            application_id: &application_id,
        },
        state.max_authorization_time,
        state.debug_authorizer,
    )
    .await?;

    let application = query_as!(
        Application,
        "
            SELECT application__id AS application_id, organization__id AS organization_id, name
            FROM event.application
            WHERE application__id = $1
            AND deleted_at IS NULL
        ",
        application_id.into_inner()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(Hook0Problem::from)?;

    match application {
        Some(a) => {
            let mut tx = state.db.begin().await.map_err(Hook0Problem::from)?;

            query!(
                "UPDATE event.application SET deleted_at = NOW() WHERE application__id = $1",
                a.application_id
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;

            // Mark pending request attempts as failed for all subscriptions of this application
            let cancelled_request_attempts_result = query!(
                "
                    UPDATE webhook.request_attempt AS ra
                    SET failed_at = statement_timestamp()
                    FROM webhook.subscription AS s
                    WHERE ra.subscription__id = s.subscription__id
                      AND s.application__id = $1
                      AND ra.failed_at IS NULL
                      AND ra.succeeded_at IS NULL
                ",
                &a.application_id
            )
            .execute(&mut *tx)
            .await
            .map_err(Hook0Problem::from)?;
            let cancelled_request_attempts = cancelled_request_attempts_result.rows_affected();

            tx.commit().await.map_err(Hook0Problem::from)?;

            if cancelled_request_attempts > 0 {
                report_cancelled_request_attempts(cancelled_request_attempts);
            }

            if let Some(hook0_client) = state.hook0_client.as_ref() {
                let hook0_client_event: Hook0ClientEvent = EventApplicationRemoved {
                    organization_id: get_owner_organization(&state.db, &a.application_id)
                        .await
                        .unwrap_or(Uuid::nil()),
                    application_id: a.application_id,
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

            Ok(NoContent)
        }
        None => Err(Hook0Problem::NotFound),
    }
}

#[cfg(test)]
mod default_secret_tests {
    use crate::google_ads::test_support::{
        issue_user_token, seed_membership, seed_org, seed_user, test_state,
    };
    use actix_web::{App, test, web};
    use sqlx::PgPool;
    use uuid::Uuid;

    /// Spin up the real application-creation and application-secrets endpoints,
    /// wrapped in the real biscuit auth middleware, over the test database.
    /// A macro rather than a function because the type of an initialized actix
    /// test service is not nameable here.
    macro_rules! init_api {
        ($pool:expr, $private_key:expr) => {{
            let state = test_state($pool.clone(), $private_key.clone(), None).await;
            let biscuit_auth = crate::middleware_biscuit::BiscuitAuth {
                db: $pool.clone(),
                biscuit_private_key: $private_key.clone(),
                master_api_key: None,
                // Pinned on rather than read from the runtime default: passing a
                // raw application secret as a Bearer token only works through
                // the compatibility path, so if `enable_application_secret_compatibility`
                // ever defaults to off, these tests keep passing while the
                // provisioned secret stops authenticating anything in production.
                enable_application_secret_compatibility: true,
            };

            test::init_service(
                App::new().app_data(web::Data::new(state)).service(
                    web::scope("/api/v1")
                        .service(
                            web::scope("/applications")
                                .wrap(biscuit_auth.clone())
                                .route("", web::post().to(super::create)),
                        )
                        .service(
                            web::scope("/application_secrets")
                                .wrap(biscuit_auth.clone())
                                .route(
                                    "",
                                    web::get().to(crate::handlers::application_secrets::list),
                                ),
                        ),
                ),
            )
            .await
        }};
    }

    /// Create an application through the real endpoint and return its id.
    /// A macro for the same reason as [`init_api`]: the initialized test service
    /// cannot be named in a helper signature.
    macro_rules! create_application {
        ($app:expr, $user_token:expr, $organization_id:expr, $name:expr) => {{
            let request = test::TestRequest::post()
                .uri("/api/v1/applications")
                .insert_header(("Authorization", format!("Bearer {}", $user_token)))
                .set_json(serde_json::json!({"organization_id": $organization_id, "name": $name}))
                .to_request();
            let resp = test::call_service(&$app, request).await;
            assert!(
                resp.status().is_success(),
                "application creation failed: {}",
                resp.status()
            );
            let body: serde_json::Value = test::read_body_json(resp).await;
            body["application_id"]
                .as_str()
                .and_then(|id| Uuid::parse_str(id).ok())
                .expect("application_id in response")
        }};
    }

    /// Creating an application must provision a default application secret in the
    /// same request, and that secret must be immediately usable to authenticate a
    /// real API call. Drives the real handlers + biscuit auth middleware against a
    /// real Postgres, with no mocking.
    #[sqlx::test]
    async fn creating_an_application_provisions_a_usable_default_secret(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let app = init_api!(pool, private_key);

        // 1) Create an application via the real handler.
        let create_app = test::TestRequest::post()
            .uri("/api/v1/applications")
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .set_json(serde_json::json!({"organization_id": org, "name": "default-secret-app"}))
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

        // 2) A default secret must exist right after creation, with no manual
        //    step. List the application's secrets with the user token.
        let list_secrets = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/application_secrets?application_id={application_id}"
            ))
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .to_request();
        let resp = test::call_service(&app, list_secrets).await;
        assert!(
            resp.status().is_success(),
            "listing secrets failed: {}",
            resp.status()
        );
        let secrets: serde_json::Value = test::read_body_json(resp).await;
        let secrets = secrets.as_array().expect("secrets array");
        assert_eq!(
            secrets.len(),
            1,
            "exactly one default secret is provisioned at creation"
        );
        assert_eq!(
            secrets[0]["name"], "Default",
            "the default secret is labelled Default"
        );
        let default_secret_token = secrets[0]["token"]
            .as_str()
            .expect("default secret token")
            .to_string();

        // 3) The default secret must authenticate a real API call: use it as a
        //    Bearer token (application secret compatibility) to list this
        //    application's secrets. A 2xx proves the token is valid and scoped to
        //    the application. This step is what depends on the compatibility
        //    setting pinned on in `init_api!` — with it off, a raw secret is not
        //    a Bearer token at all and this call is a 401.
        let authed_call = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/application_secrets?application_id={application_id}"
            ))
            .insert_header(("Authorization", format!("Bearer {default_secret_token}")))
            .to_request();
        let resp = test::call_service(&app, authed_call).await;
        assert!(
            resp.status().is_success(),
            "default secret failed to authenticate an API call: {}",
            resp.status()
        );
        let authed_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(
            authed_body.as_array().expect("secrets array").len(),
            1,
            "the authenticated call returns the application's secrets"
        );
    }

    /// An application and its default secret are created in one transaction: if
    /// the secret cannot be inserted, the application must not exist either — a
    /// half-created application would be invisible in the UI's "ready to send"
    /// path yet still consume the organization's application quota.
    ///
    /// The failure is forced at the database level (a CHECK constraint that
    /// rejects the provisioned name), so the real handler runs its real
    /// transaction and hits a real constraint violation.
    #[sqlx::test]
    async fn a_failed_secret_insert_leaves_no_application_behind(pool: PgPool) {
        sqlx::query(
            "
                ALTER TABLE event.application_secret
                ADD CONSTRAINT reject_provisioned_secret CHECK (name IS DISTINCT FROM 'Default')
            ",
        )
        .execute(&pool)
        .await
        .expect("make the provisioned secret impossible to insert");

        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let app = init_api!(pool, private_key);

        let create_app = test::TestRequest::post()
            .uri("/api/v1/applications")
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .set_json(serde_json::json!({"organization_id": org, "name": "rolled-back-app"}))
            .to_request();
        let resp = test::call_service(&app, create_app).await;
        assert!(
            !resp.status().is_success(),
            "creation must fail when the default secret cannot be inserted, got {}",
            resp.status()
        );

        let applications: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM event.application WHERE organization__id = $1",
        )
        .bind(org)
        .fetch_one(&pool)
        .await
        .expect("count applications");
        assert_eq!(
            applications, 0,
            "the application row must be rolled back with its secret"
        );
    }

    /// Every application now ships with a secret, so the blast radius of one
    /// leaking is exactly what confines it: the default secret of application A
    /// must be refused on application B (same organization) and on an
    /// application of another organization.
    ///
    /// Authenticating with the raw secret only exists behind the compatibility
    /// setting pinned on in `init_api!`; with it off there is no such blast
    /// radius to measure here, and this test would still pass.
    #[sqlx::test]
    async fn the_default_secret_is_confined_to_its_own_application(pool: PgPool) {
        let keypair = biscuit_auth::KeyPair::new();
        let private_key = keypair.private();

        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_membership(&pool, user, org, "editor").await;
        let user_token = issue_user_token(&pool, &private_key, user, org, "editor").await;

        let other_org = seed_org(&pool, user).await;
        seed_membership(&pool, user, other_org, "editor").await;
        let other_org_token =
            issue_user_token(&pool, &private_key, user, other_org, "editor").await;

        let app = init_api!(pool, private_key);

        let application_a = create_application!(app, user_token, org, "app-a");
        let application_b = create_application!(app, user_token, org, "app-b");
        let application_c = create_application!(app, other_org_token, other_org, "app-c");

        // Read A's provisioned secret through the API, as a user would.
        let list_secrets = test::TestRequest::get()
            .uri(&format!(
                "/api/v1/application_secrets?application_id={application_a}"
            ))
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .to_request();
        let resp = test::call_service(&app, list_secrets).await;
        assert!(resp.status().is_success(), "listing A's secrets failed");
        let secrets: serde_json::Value = test::read_body_json(resp).await;
        let secret_of_a = secrets.as_array().expect("secrets array")[0]["token"]
            .as_str()
            .expect("default secret token")
            .to_string();

        for (application_id, description) in [
            (
                application_b,
                "another application of the same organization",
            ),
            (application_c, "an application of another organization"),
        ] {
            let cross_application_call = test::TestRequest::get()
                .uri(&format!(
                    "/api/v1/application_secrets?application_id={application_id}"
                ))
                .insert_header(("Authorization", format!("Bearer {secret_of_a}")))
                .to_request();
            let resp = test::call_service(&app, cross_application_call).await;
            assert_eq!(
                resp.status(),
                actix_web::http::StatusCode::FORBIDDEN,
                "the default secret of application A must be refused on {description}"
            );
        }
    }
}
