//! Construction of the Hook0 HTTP application: OpenAPI spec, middlewares and
//! route table.
//!
//! Kept apart from the server bootstrap so that the exact same application —
//! and therefore the exact same OpenAPI spec — can be built without binding a
//! socket or reaching a database.

use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Compat, NormalizePath};
use actix_web::{App, Error, http, middleware};
use ipnetwork::IpNetwork;
use paperclip::actix::{OpenApiExt, web};
use tracing_actix_web::TracingLogger;
use url::Url;
use uuid::Uuid;

use crate::rate_limiting::Hook0RateLimiters;
use crate::{
    State, WEBAPP_INDEX_FILE, handlers, middleware_biscuit, middleware_get_user_ip, openapi,
    openapi_postprocess, problems, quotas,
};

/// Everything [`build_app`] needs to produce one application instance.
///
/// `HttpServer` calls the application factory once per worker thread, so this
/// is moved into the factory closure and borrowed on each call.
#[derive(Clone)]
pub struct AppFactoryConfig {
    pub state: State,
    pub app_url: Url,
    pub reverse_proxy_cidrs: Vec<IpNetwork>,
    pub behind_cloudflare: bool,
    pub cors_allowed_origins: Vec<String>,
    pub master_api_key: Option<Uuid>,
    #[cfg(feature = "application-secret-compatibility")]
    pub enable_application_secret_compatibility: bool,
    pub enable_security_headers: bool,
    pub enable_hsts_header: bool,
    pub rate_limiters: Hook0RateLimiters,
    pub disable_serving_webapp: bool,
    pub webapp_path: String,
}

/// Build the whole Hook0 application: OpenAPI spec, middlewares, API routes and
/// (unless disabled) the web app static files.
///
/// Nothing is borrowed from `config` past this call (`use<>`), so the returned
/// application is `'static` and can be handed to `HttpServer` or to a test
/// service.
pub fn build_app(
    config: &AppFactoryConfig,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody + use<>>,
        Error = Error,
        InitError = (),
    > + use<>,
> {
    // Compute default OpenAPI spec and apply post-processing
    let mut spec = openapi::default_spec(&config.app_url);
    openapi_postprocess::enrich_openapi_spec(&mut spec);

    // Prepare user IP extraction middleware
    let get_user_ip = middleware_get_user_ip::GetUserIp {
        reverse_proxy_cidrs: config.reverse_proxy_cidrs.clone(),
        behind_cloudflare: config.behind_cloudflare,
    };

    // Prepare CORS configuration
    let cors = {
        let mut c = Cors::default()
            .allowed_headers([
                http::header::ACCEPT,
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .max_age(3600);

        for origin in &config.cors_allowed_origins {
            c = c.allowed_origin(origin);
        }

        c
    };

    // Prepare auth middleware
    let biscuit_auth = middleware_biscuit::BiscuitAuth {
        db: config.state.db.clone(),
        biscuit_private_key: config.state.biscuit_private_key.clone(),
        master_api_key: config.master_api_key,
        #[cfg(feature = "application-secret-compatibility")]
        enable_application_secret_compatibility: config.enable_application_secret_compatibility,
    };

    let security_headers = middleware::DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
        .add(("X-XSS-Protection", "1; mode=block"))
        .add(("Referrer-Policy", "SAMEORIGIN"))
        .add(("X-Frame-Options", "DENY"));

    let hsts_header =
        middleware::DefaultHeaders::new().add(("Strict-Transport-Security", "max-age=63072000"));

    let security_headers_condition =
        middleware::Condition::new(config.enable_security_headers, security_headers);

    let hsts_header_condition = middleware::Condition::new(config.enable_hsts_header, hsts_header);

    let rate_limiters = &config.rate_limiters;

    let mut app = App::new()
        .app_data(web::Data::new(config.state.clone()))
        .app_data(web::JsonConfig::default().error_handler(|e, _req| {
            let problem =
                problems::Hook0Problem::JsonPayload(problems::JsonPayloadProblem::from(e));
            actix_web::error::Error::from(problem)
        }))
        .wrap(get_user_ip)
        .wrap(hsts_header_condition)
        .wrap(security_headers_condition)
        .wrap(cors)
        .wrap(TracingLogger::default())
        .wrap(NormalizePath::trim())
        .wrap(sentry_actix::Sentry::new())
        .wrap_api_with_spec(spec)
        .with_json_spec_v3_at("/api/v1/swagger.json")
        .service(
            web::scope("/api/v1")
                .wrap(Compat::new(rate_limiters.ip()))
                .wrap(Compat::new(rate_limiters.global()))
                .service(
                    web::scope("/auth")
                        .service(
                            web::resource("/verify-email")
                                .route(web::post().to(handlers::auth::verify_email)),
                        )
                        .service(
                            web::resource("/resend-verification-email")
                                .route(web::post().to(handlers::auth::resend_verification_email)),
                        )
                        .service(
                            web::resource("/login").route(web::post().to(handlers::auth::login)),
                        )
                        .service(
                            web::resource("/refresh")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .route(web::post().to(handlers::auth::refresh)),
                        )
                        .service(
                            web::resource("/logout")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .route(web::post().to(handlers::auth::logout)),
                        )
                        .service(
                            web::resource("/begin-reset-password")
                                .route(web::post().to(handlers::auth::begin_reset_password)),
                        )
                        .service(
                            web::resource("/reset-password")
                                .route(web::post().to(handlers::auth::reset_password)),
                        )
                        .service(
                            web::resource("/password")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                .route(web::post().to(handlers::auth::change_password)),
                        ),
                )
                // no auth: authenticated by the signed token carried by the
                // unsubscribe link of a reactivation email
                .service(web::scope("/email-preferences").service(
                    web::resource("/unsubscribe-reactivation").route(
                        web::post().to(handlers::email_preferences::unsubscribe_reactivation),
                    ),
                ))
                // no auth
                .service(
                    web::scope("/instance")
                        .service(web::resource("").route(web::get().to(handlers::instance::get))),
                )
                .service(
                    web::scope("/quotas")
                        .service(web::resource("").route(web::get().to(quotas::get))),
                )
                .service(web::scope("/environment_variables").service(
                    web::resource("").route(web::get().to(handlers::environment_variables::get)),
                ))
                .service({
                    let srv = web::scope("/health").service(
                        web::resource("").route(web::get().to(handlers::instance::health)),
                    );

                    #[cfg(feature = "profiling")]
                    {
                        srv.service(
                            web::resource("/profiling/heap")
                                .route(web::get().to(handlers::instance::pprof_heap)),
                        )
                        .service(
                            web::resource("/profiling/cpu")
                                .route(web::get().to(handlers::instance::pprof_cpu)),
                        )
                    }

                    #[cfg(not(feature = "profiling"))]
                    srv
                })
                .service(
                    web::scope("/errors")
                        .service(web::resource("").route(web::get().to(handlers::errors::list))),
                )
                .service(web::scope("/payload_content_types").service(
                    web::resource("").route(web::get().to(handlers::events::payload_content_types)),
                ))
                .service(web::scope("/register").service(
                    web::resource("").route(web::post().to(handlers::registrations::register)),
                ))
                // with authentication
                .service(
                    web::scope("/organizations")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::organizations::list))
                                .route(web::post().to(handlers::organizations::create)),
                        )
                        .service(
                            web::scope("/{organization_id}")
                                .service(
                                    web::resource("")
                                        .route(web::get().to(handlers::organizations::get))
                                        .route(web::put().to(handlers::organizations::edit))
                                        .route(web::delete().to(handlers::organizations::delete)),
                                )
                                .service(
                                    web::resource("/invite")
                                        .route(web::post().to(handlers::organizations::invite))
                                        .route(web::delete().to(handlers::organizations::revoke))
                                        .route(web::put().to(handlers::organizations::edit_role)),
                                ),
                        ),
                )
                .service(
                    web::scope("/applications")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::applications::list))
                                .route(web::post().to(handlers::applications::create)),
                        )
                        .service(
                            web::resource("/{application_id}")
                                .route(web::get().to(handlers::applications::get))
                                .route(web::put().to(handlers::applications::edit))
                                .route(web::delete().to(handlers::applications::delete)),
                        ),
                )
                .service(
                    web::scope("/event_types")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::event_types::list))
                                .route(web::post().to(handlers::event_types::create)),
                        )
                        .service(
                            web::resource("/{event_type_name}")
                                .route(web::get().to(handlers::event_types::get))
                                .route(web::delete().to(handlers::event_types::delete)),
                        ),
                )
                .service(
                    #[cfg(feature = "application-secret-compatibility")]
                    web::scope("/application_secrets")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::application_secrets::list))
                                .route(web::post().to(handlers::application_secrets::create)),
                        )
                        .service(
                            web::resource("/{application_secret_token}")
                                .route(web::put().to(handlers::application_secrets::edit))
                                .route(web::delete().to(handlers::application_secrets::delete)),
                        ),
                    #[cfg(not(feature = "application-secret-compatibility"))]
                    web::resource("/"),
                )
                .service(
                    web::scope("/service_token")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::service_token::list))
                                .route(web::post().to(handlers::service_token::create)),
                        )
                        .service(
                            web::resource("/{service_token_id}")
                                .route(web::get().to(handlers::service_token::get))
                                .route(web::put().to(handlers::service_token::update))
                                .route(web::delete().to(handlers::service_token::delete)),
                        ),
                )
                .service(
                    web::scope("/events")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(web::resource("").route(web::get().to(handlers::events::list)))
                        .service(
                            web::resource("/{event_id}")
                                .route(web::get().to(handlers::events::get)),
                        )
                        .service(
                            web::resource("/{event_id}/replay")
                                .route(web::post().to(handlers::events::replay)),
                        ),
                )
                .service(
                    web::scope("/event")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(web::resource("").route(web::post().to(handlers::events::ingest))),
                )
                .service(
                    web::scope("/events_per_day")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("/application")
                                .route(web::get().to(handlers::events_per_day::application)),
                        )
                        .service(
                            web::resource("/organization")
                                .route(web::get().to(handlers::events_per_day::organization)),
                        ),
                )
                .service(
                    web::scope("/subscriptions")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::subscriptions::list))
                                .route(web::post().to(handlers::subscriptions::create)),
                        )
                        .service(
                            web::resource("/{subscription_id}")
                                .route(web::get().to(handlers::subscriptions::get))
                                .route(web::put().to(handlers::subscriptions::edit))
                                .route(web::delete().to(handlers::subscriptions::delete)),
                        ),
                )
                .service(
                    web::scope("/request_attempts")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("")
                                .route(web::get().to(handlers::request_attempts::list)),
                        )
                        .service(
                            web::resource("/{request_attempt_id}")
                                .route(web::get().to(handlers::request_attempts::get)),
                        ),
                )
                .service(
                    web::scope("/responses")
                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                        .service(
                            web::resource("/{response_id}")
                                .route(web::get().to(handlers::responses::get)),
                        ),
                ),
        );

    if !config.disable_serving_webapp {
        app = app.default_service(
            Files::new("/", config.webapp_path.as_str())
                .index_file(WEBAPP_INDEX_FILE)
                .default_handler(
                    NamedFile::open(format!("{}/{}", config.webapp_path, WEBAPP_INDEX_FILE))
                        .expect("Cannot open SPA main file"),
                ),
        );
    }
    app.build()
}

/// Inert dependencies that let the application be built — and its OpenAPI spec
/// read — without a listening socket, a reachable database or an SMTP server.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    use actix_web::test;
    use biscuit_auth::KeyPair;
    use lettre::Address;
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::time::Duration;

    use crate::mailer;

    /// Public URL of the Hook0 instance the served document describes. Only the
    /// factory's `app_url` reaches the document, as the `servers` entry every
    /// consumer of the snapshot resolves relative paths against — hence the real
    /// one rather than a test hostname.
    pub(crate) const APP_URL: &str = "https://app.hook0.com/";

    /// A state whose database pool is lazy (it is never dialled because no
    /// handler runs) and whose mailer points at a closed local port.
    pub(crate) async fn inert_state() -> State {
        let db = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(PgConnectOptions::new());

        let mailer = mailer::Mailer::new(
            mailer::MailerSmtpConfig {
                smtp_connection_url: "smtp://127.0.0.1:1".to_owned(),
                smtp_timeout: Duration::from_secs(1),
                sender_name: "Hook0".to_owned(),
                sender_address: Address::from_str("hook0@example.test")
                    .expect("parse sender address"),
            },
            Url::parse("https://example.test/logo.png").expect("parse logo url"),
            Url::parse("https://example.test/").expect("parse website url"),
            Url::parse("https://app.example.test/").expect("parse app url"),
            Url::parse("https://documentation.example.test/").expect("parse doc url"),
            Url::parse("https://example.test/privacy-policy").expect("parse privacy policy url"),
            Address::from_str("support@example.test").expect("parse support address"),
            "Example".to_owned(),
            "1 street, city, country".to_owned(),
            "RCS Example 000 000 000".to_owned(),
        )
        .await
        .expect("build mailer");

        State {
            db,
            pulsar: None,
            object_storage: None,
            biscuit_private_key: KeyPair::new().private(),
            mailer,
            app_url: Url::parse("https://app.example.test/").expect("parse app url"),
            #[cfg(feature = "migrate-users-from-keycloak")]
            enable_keycloak_migration: false,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_url: Url::parse("https://keycloak.example.test/auth")
                .expect("parse keycloak url"),
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_realm: "hook0".to_owned(),
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_id: "hook0".to_owned(),
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_secret: "secret".to_owned(),
            application_secret_compatibility: false,
            registration_disabled: false,
            password_minimum_length: 12,
            auto_db_migration: false,
            hook0_client: None,
            quotas: quotas::Quotas::new(
                false,
                quotas::QuotaLimits {
                    global_members_per_organization_limit: 1,
                    global_applications_per_organization_limit: 1,
                    global_events_per_day_limit: 100,
                    global_days_of_events_retention_limit: 7,
                    global_subscriptions_per_application_limit: 10,
                    global_event_types_per_application_limit: 10,
                },
            ),
            health_check_key: None,
            health_check_timeout: Duration::from_secs(5),
            max_authorization_time: Duration::from_millis(30),
            debug_authorizer: false,
            enable_quota_enforcement: false,
            matomo_url: None,
            matomo_site_id: None,
            formbricks_api_host: "https://app.formbricks.com".to_owned(),
            formbricks_environment_id: None,
            quota_notification_events_per_day_threshold: 80,
            enable_quota_based_email_notifications: false,
            support_email_address: Address::from_str("support@example.test")
                .expect("parse support address"),
            cloudflare_turnstile_site_key: None,
            cloudflare_turnstile_secret_key: None,
            google_ads: None,
            signup_attribution_retention_in_days: 30,
        }
    }

    /// The application factory configuration used by tests: same route table as
    /// production, no static files to serve and no rate limiting.
    pub(crate) async fn inert_app_factory_config() -> AppFactoryConfig {
        AppFactoryConfig {
            state: inert_state().await,
            app_url: Url::parse(APP_URL).expect("parse app url"),
            reverse_proxy_cidrs: vec![],
            behind_cloudflare: false,
            cors_allowed_origins: vec![],
            master_api_key: None,
            #[cfg(feature = "application-secret-compatibility")]
            enable_application_secret_compatibility: false,
            enable_security_headers: true,
            enable_hsts_header: false,
            rate_limiters: Hook0RateLimiters::new(
                true, true, 2000, 1, true, 200, 10, true, 20, 100,
            ),
            disable_serving_webapp: true,
            webapp_path: String::new(),
        }
    }

    /// Build the application and read back the OpenAPI document it serves.
    /// This goes through the real route table, so the returned spec is the one
    /// clients get in production.
    pub(crate) async fn openapi_spec() -> serde_json::Value {
        let config = inert_app_factory_config().await;
        let app = test::init_service(build_app(&config)).await;
        let request = test::TestRequest::get()
            .uri("/api/v1/swagger.json")
            // The user IP middleware rejects requests without a peer address,
            // which a real connection always carries.
            .peer_addr(SocketAddr::from(([127, 0, 0, 1], 5678)))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert!(
            response.status().is_success(),
            "the OpenAPI document is served, got status {}",
            response.status()
        );
        let body = test::read_body(response).await;
        serde_json::from_slice(&body).expect("the served OpenAPI document is valid JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::openapi_spec;

    use serde_json::Value;
    use std::collections::BTreeSet;

    /// The document generated clients and the MCP server are built from.
    /// Committed so neither has to reach a running instance.
    const SNAPSHOT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.snapshot.json");

    /// Tags that keep an operation in the snapshot. `public` is the surface
    /// generated clients expose to their users; `mcp` is what the MCP server
    /// turns into tools, and it covers a couple of operations the SDKs do not.
    /// Everything else is the control plane the dashboard drives, and it stays
    /// out: nobody generates a client for it.
    const RETAINED_TAGS: [&str; 2] = ["public", "mcp"];

    /// Keys of a path item that describe an operation rather than the path.
    const HTTP_METHODS: [&str; 8] = [
        "get", "put", "post", "delete", "options", "head", "patch", "trace",
    ];

    /// Prefix of a reference to a schema of the document.
    const SCHEMA_REFERENCE_PREFIX: &str = "#/components/schemas/";

    /// The types the dashboard is built against, generated from the served
    /// document by `openapi-typescript` and indexed by operation identifier.
    const FRONTEND_TYPES_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../frontend/src/types.ts");

    /// Ceiling on how much of that file is read, so a runaway one cannot be
    /// pulled into memory whole.
    const FRONTEND_TYPES_MAX_BYTES: u64 = 4 * 1024 * 1024;

    /// Header of the block the generated file declares its operations under.
    const FRONTEND_OPERATIONS_BLOCK: &str = "export interface operations {";

    /// How the generated file points at one of those operations.
    const FRONTEND_OPERATION_REFERENCE: &str = "operations['";

    /// What to run to regenerate the frontend types. Unlike the snapshot, this
    /// reads the document over HTTP, so it needs an instance to read it from.
    const FRONTEND_TYPES_UPDATE_COMMAND: &str =
        "cd frontend && npm run generate:types  # against a running API";

    /// Set to any value to rewrite the snapshot instead of failing on a difference.
    const SNAPSHOT_UPDATE_VAR: &str = "UPDATE_OPENAPI_SNAPSHOT";

    /// What to run to adopt a deliberate change of the API surface.
    const SNAPSHOT_UPDATE_COMMAND: &str =
        "UPDATE_OPENAPI_SNAPSHOT=1 cargo test -p hook0-api openapi_snapshot";

    /// How many differences a failure report lists before it stops.
    const MAX_REPORTED_DIFFERENCES: usize = 40;

    /// How much of a differing value a failure report prints.
    const MAX_RENDERED_VALUE_CHARS: usize = 120;

    /// How to read the difference list.
    const REPORT_LEGEND: &str =
        "(`-` in the snapshot only, `+` in the served document only, `~` snapshot -> served)";

    /// The application can be built and its OpenAPI document read back without
    /// a listening socket or a reachable database, which is what makes the
    /// generated spec inspectable from tests.
    #[actix_web::test]
    async fn openapi_document_is_produced_by_the_built_application() {
        let spec = openapi_spec().await;

        let paths = spec
            .get("paths")
            .and_then(|paths| paths.as_object())
            .expect("the OpenAPI document exposes a paths object");

        assert!(
            !paths.is_empty(),
            "the OpenAPI document describes at least one path"
        );

        println!("OpenAPI document served with {} paths", paths.len());
    }

    /// Everything downstream — generated SDKs, the MCP tool definitions, the
    /// frontend types — is built from the committed snapshot rather than from a
    /// running instance. Letting the two drift apart would ship clients for an
    /// API that no longer exists, so a change of the served document only lands
    /// once someone has adopted it into the snapshot.
    ///
    /// The snapshot describes the default feature set, the one the API ships
    /// with; turning a feature on adds routes and the report then names them.
    #[actix_web::test]
    async fn openapi_snapshot_matches_the_served_document() {
        let served = client_surface(&openapi_spec().await);

        if std::env::var_os(SNAPSHOT_UPDATE_VAR).is_some() {
            let mut rendered =
                serde_json::to_string_pretty(&served).expect("the served document serializes");
            rendered.push('\n');
            std::fs::write(SNAPSHOT_PATH, rendered).expect("the snapshot is writable");
            println!("Wrote {SNAPSHOT_PATH}");
            return;
        }

        let committed = std::fs::read(SNAPSHOT_PATH).unwrap_or_else(|err| {
            panic!(
                "the OpenAPI snapshot cannot be read ({err}).\n\
                 Generate it with:\n    {SNAPSHOT_UPDATE_COMMAND}\n\
                 Snapshot: {SNAPSHOT_PATH}"
            )
        });
        let committed: Value =
            serde_json::from_slice(&committed).expect("the committed snapshot is valid JSON");

        if committed == served {
            return;
        }

        panic!(
            "the served OpenAPI document no longer matches the committed snapshot.\n\
             {}\n\
             Adopt the change by running:\n    {SNAPSHOT_UPDATE_COMMAND}\n\
             Snapshot: {SNAPSHOT_PATH}",
            report(&committed, &served)
        );
    }

    /// A generated SDK exposing `auth.login` or `organizations.invite` would be
    /// handing out the dashboard's own control plane as if it were product.
    /// Checked against the file rather than against the filter that wrote it, so
    /// the two would have to be wrong in the same way to let something through.
    #[actix_web::test]
    async fn the_snapshot_leaves_the_private_control_plane_out() {
        let served = openapi_spec().await;
        let committed: Value = serde_json::from_slice(
            &std::fs::read(SNAPSHOT_PATH).expect("the snapshot is readable"),
        )
        .expect("the committed snapshot is valid JSON");

        let private = operation_ids(&served, |operation| !is_retained(operation));
        let snapshotted = operation_ids(&committed, |_| true);

        assert!(
            !private.is_empty(),
            "the served document tags every operation, so this proves nothing"
        );
        let leaked: Vec<_> = snapshotted.intersection(&private).collect();
        assert!(
            leaked.is_empty(),
            "the snapshot exposes operations no client should be generated for: {leaked:?}"
        );

        println!(
            "{} operations in the snapshot, {} kept out of it",
            snapshotted.len(),
            private.len()
        );
    }

    /// Narrowing the document down also drops the components the operations
    /// left behind used to reach. Dropping one too many yields a document that
    /// still parses but that no generator can resolve, so the snapshot has to
    /// carry exactly the components it points at — no dangling reference, and
    /// no schema nothing points at either.
    #[actix_web::test]
    async fn the_snapshot_resolves_every_reference_it_makes() {
        let committed = std::fs::read(SNAPSHOT_PATH).expect("the snapshot is readable");
        let committed: Value =
            serde_json::from_slice(&committed).expect("the committed snapshot is valid JSON");

        let declared: BTreeSet<String> = committed["components"]["schemas"]
            .as_object()
            .expect("the snapshot declares schemas")
            .keys()
            .cloned()
            .collect();
        let referenced = referenced_schemas(&committed);

        assert_eq!(
            referenced.difference(&declared).collect::<Vec<_>>(),
            Vec::<&String>::new(),
            "the snapshot points at schemas it does not carry"
        );
        assert_eq!(
            declared.difference(&referenced).collect::<Vec<_>>(),
            Vec::<&String>::new(),
            "the snapshot carries schemas nothing points at"
        );

        let schemes: BTreeSet<String> = committed["components"]["securitySchemes"]
            .as_object()
            .expect("the snapshot declares security schemes")
            .keys()
            .cloned()
            .collect();
        assert_eq!(
            required_security_schemes(&committed["paths"])
                .difference(&schemes)
                .collect::<Vec<_>>(),
            Vec::<&String>::new(),
            "the snapshot requires security schemes it does not carry"
        );
    }
    /// The dashboard is typed against a file generated from the served document
    /// and indexed by operation identifier, and nothing ties the two together
    /// afterwards: rename an operation and the file still compiles, against
    /// identifiers the API no longer serves — which only shows up once a request
    /// is actually made.
    ///
    /// Checked against the served document rather than the snapshot, because the
    /// snapshot is narrowed to what clients are generated from while the
    /// dashboard also drives the private control plane: signing in, registering,
    /// administering an organization.
    /// Operations the API serves that the generated frontend types predate.
    ///
    /// They are named rather than tolerated as a count, so a third one cannot
    /// slip in unnoticed, and the list empties itself the day the types are
    /// regenerated against a running API. Regenerating them today would also
    /// bring the closed enumeration of problem identifiers, which several
    /// dashboard call sites do not satisfy — that is a change of its own.
    const FRONTEND_TYPES_STALE_SINCE: [&str; 2] = [
        "auth.resend_verification_email",
        "emailPreferences.unsubscribeReactivation",
    ];

    #[actix_web::test]
    async fn the_frontend_types_are_indexed_by_the_operations_the_api_serves() {
        let served = operation_ids(&openapi_spec().await, |_| true);
        let frontend = frontend_operation_ids(&read_frontend_types());

        let unserved: BTreeSet<_> = frontend.difference(&served).cloned().collect();
        let missing: BTreeSet<_> = served
            .difference(&frontend)
            .filter(|id| !FRONTEND_TYPES_STALE_SINCE.contains(&id.as_str()))
            .cloned()
            .collect();

        if unserved.is_empty() && missing.is_empty() {
            println!("{} operations shared with the frontend types", served.len());
            return;
        }

        let mut report = String::new();
        if !unserved.is_empty() {
            report.push_str(&format!(
                "Indexed by the frontend types, not served by the API:\n    {}\n",
                names(&unserved)
            ));
        }
        if !missing.is_empty() {
            report.push_str(&format!(
                "Served by the API, absent from the frontend types:\n    {}\n",
                names(&missing)
            ));
        }
        panic!(
            "the generated frontend types no longer describe the served API.\n\
             {report}\
             Regenerate them with:\n    {FRONTEND_TYPES_UPDATE_COMMAND}\n\
             File: {FRONTEND_TYPES_PATH}"
        );
    }

    /// Reads the generated frontend types, refusing a file past
    /// [`FRONTEND_TYPES_MAX_BYTES`] rather than growing to hold it.
    fn read_frontend_types() -> String {
        use std::io::Read;

        let file = std::fs::File::open(FRONTEND_TYPES_PATH).unwrap_or_else(|err| {
            panic!(
                "the generated frontend types cannot be read ({err}).\n\
                 Generate them with:\n    {FRONTEND_TYPES_UPDATE_COMMAND}\n\
                 File: {FRONTEND_TYPES_PATH}"
            )
        });

        let mut source = String::new();
        let read = file
            .take(FRONTEND_TYPES_MAX_BYTES + 1)
            .read_to_string(&mut source)
            .expect("the generated frontend types are valid UTF-8");
        assert!(
            read as u64 <= FRONTEND_TYPES_MAX_BYTES,
            "the generated frontend types are larger than the {FRONTEND_TYPES_MAX_BYTES} bytes \
             this reads.\nFile: {FRONTEND_TYPES_PATH}"
        );

        source
    }

    /// Identifiers the generated frontend types name, taken from both places the
    /// generator writes them: the keys of its `operations` block, and the
    /// references its path table resolves against them. Reading both means an
    /// identifier edited on one side alone is reported rather than covered for by
    /// the other.
    fn frontend_operation_ids(source: &str) -> BTreeSet<String> {
        let mut ids = declared_frontend_operations(source);
        ids.extend(referenced_frontend_operations(source));
        ids
    }

    /// Keys of the `operations` block, which the generator lists one per line at
    /// the block's own indentation, quoted unless the formatter could drop the
    /// quotes.
    fn declared_frontend_operations(source: &str) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        let Some((_, block)) = source.split_once(FRONTEND_OPERATIONS_BLOCK) else {
            return ids;
        };

        for line in block.lines() {
            // The block closes on the first line to return to column zero.
            if line == "}" {
                break;
            }
            let Some(entry) = line.strip_prefix("  ") else {
                continue;
            };
            // Anything indented further belongs to an operation, not to the block.
            if entry.starts_with(' ') {
                continue;
            }
            if let Some(key) = entry.strip_suffix(": {") {
                ids.insert(key.trim_matches('\'').to_owned());
            }
        }

        ids
    }

    /// Identifiers the file resolves a path against, as `operations['…']`.
    fn referenced_frontend_operations(source: &str) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();

        for tail in source.split(FRONTEND_OPERATION_REFERENCE).skip(1) {
            // An identifier is written on one line, so a reference the closing
            // quote of which is further down is not one.
            if let Some((id, _)) = tail.split_once('\'')
                && !id.is_empty()
                && !id.contains('\n')
            {
                ids.insert(id.to_owned());
            }
        }

        ids
    }

    /// Names, as many as a failure message carries.
    fn names(ids: &BTreeSet<String>) -> String {
        let listed: Vec<&str> = ids
            .iter()
            .take(MAX_REPORTED_DIFFERENCES)
            .map(String::as_str)
            .collect();

        let mut rendered = listed.join(", ");
        if ids.len() > listed.len() {
            rendered.push_str(&format!(", … {} more", ids.len() - listed.len()));
        }
        rendered
    }

    /// Identifiers of the operations of a document that a predicate keeps.
    fn operation_ids(document: &Value, keep: impl Fn(&Value) -> bool) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        for item in document["paths"]
            .as_object()
            .into_iter()
            .flat_map(|paths| paths.values())
        {
            for (method, operation) in item.as_object().into_iter().flatten() {
                if HTTP_METHODS.contains(&method.as_str())
                    && keep(operation)
                    && let Some(id) = operation["operationId"].as_str()
                {
                    ids.insert(id.to_owned());
                }
            }
        }
        ids
    }

    /// Narrows the served document down to what clients are generated from: the
    /// operations carrying one of [`RETAINED_TAGS`], the paths that still hold
    /// one, and the components those reach.
    ///
    /// The control plane the dashboard drives — signing in, registering,
    /// administering an organization — is not something anyone should find in a
    /// generated SDK, so it never enters the snapshot in the first place.
    fn client_surface(served: &Value) -> Value {
        let mut surface = served.clone();

        let paths = surface["paths"]
            .as_object_mut()
            .expect("the served document exposes a paths object");
        for item in paths.values_mut() {
            let Some(item) = item.as_object_mut() else {
                continue;
            };
            item.retain(|key, operation| {
                !HTTP_METHODS.contains(&key.as_str()) || is_retained(operation)
            });
        }
        paths.retain(|_, item| {
            item.as_object()
                .is_some_and(|item| item.keys().any(|key| HTTP_METHODS.contains(&key.as_str())))
        });

        let reachable = reachable_schemas(&surface["paths"], &surface["components"]["schemas"]);
        let required = required_security_schemes(&surface["paths"]);

        if let Some(components) = surface["components"].as_object_mut() {
            if let Some(schemas) = components["schemas"].as_object_mut() {
                schemas.retain(|name, _| reachable.contains(name));
            }
            if let Some(schemes) = components["securitySchemes"].as_object_mut() {
                schemes.retain(|name, _| required.contains(name));
            }
        }

        surface
    }

    /// Whether an operation is one clients are generated from.
    fn is_retained(operation: &Value) -> bool {
        operation["tags"].as_array().is_some_and(|tags| {
            tags.iter()
                .filter_map(Value::as_str)
                .any(|tag| RETAINED_TAGS.contains(&tag))
        })
    }

    /// Names of the schemas the retained operations reach, references followed
    /// until the set stops growing — a schema pointing at itself, directly or
    /// through others, therefore terminates.
    fn reachable_schemas(paths: &Value, schemas: &Value) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        let mut pending: Vec<String> = referenced_schemas(paths).into_iter().collect();

        while let Some(name) = pending.pop() {
            if !reachable.insert(name.clone()) {
                continue;
            }
            pending.extend(referenced_schemas(&schemas[&name]));
        }

        reachable
    }

    /// Names of the schemas a fragment of the document references directly.
    fn referenced_schemas(fragment: &Value) -> BTreeSet<String> {
        let mut referenced = BTreeSet::new();
        collect_schema_references(fragment, &mut referenced);
        referenced
    }

    fn collect_schema_references(fragment: &Value, out: &mut BTreeSet<String>) {
        match fragment {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if key == "$ref"
                        && let Some(name) = value
                            .as_str()
                            .and_then(|reference| reference.strip_prefix(SCHEMA_REFERENCE_PREFIX))
                    {
                        out.insert(name.to_owned());
                    }
                    collect_schema_references(value, out);
                }
            }
            Value::Array(entries) => {
                for entry in entries {
                    collect_schema_references(entry, out);
                }
            }
            _ => {}
        }
    }

    /// Names of the security schemes the retained operations require.
    fn required_security_schemes(paths: &Value) -> BTreeSet<String> {
        let mut required = BTreeSet::new();
        for item in paths
            .as_object()
            .into_iter()
            .flat_map(|paths| paths.values())
        {
            for operation in item.as_object().into_iter().flat_map(|item| item.values()) {
                for entry in operation["security"]
                    .as_array()
                    .into_iter()
                    .flat_map(|entries| entries.iter())
                {
                    required.extend(
                        entry
                            .as_object()
                            .into_iter()
                            .flat_map(|entry| entry.keys().cloned()),
                    );
                }
            }
        }
        required
    }

    /// Renders the differences as a bounded list of JSON pointers, since neither
    /// document fits in a failure message: `-` is in the snapshot only, `+` in
    /// the served document only, `~` is a value the two disagree on.
    fn report(committed: &Value, served: &Value) -> String {
        let mut differences = Vec::new();
        collect(String::new(), committed, served, &mut differences);

        let truncated = differences.len() > MAX_REPORTED_DIFFERENCES;
        differences.truncate(MAX_REPORTED_DIFFERENCES);

        let mut report = format!("{REPORT_LEGEND}\n{}", differences.join("\n"));
        if truncated {
            report.push_str(&format!(
                "\n… list stops at {MAX_REPORTED_DIFFERENCES} differences"
            ));
        }
        report
    }

    /// Walks both documents together, naming every disagreement by its JSON
    /// pointer. Stops once the report is full, so a wholesale rewrite of the
    /// document costs a bounded walk rather than the whole tree.
    fn collect(pointer: String, committed: &Value, served: &Value, out: &mut Vec<String>) {
        if out.len() > MAX_REPORTED_DIFFERENCES {
            return;
        }

        match (committed, served) {
            (Value::Object(committed), Value::Object(served)) => {
                for (key, value) in committed {
                    match served.get(key) {
                        Some(counterpart) => {
                            collect(child(&pointer, key), value, counterpart, out);
                        }
                        None => out.push(format!("- {} (snapshot only)", child(&pointer, key))),
                    }
                    if out.len() > MAX_REPORTED_DIFFERENCES {
                        return;
                    }
                }
                for key in served.keys().filter(|key| !committed.contains_key(*key)) {
                    out.push(format!("+ {} (served only)", child(&pointer, key)));
                    if out.len() > MAX_REPORTED_DIFFERENCES {
                        return;
                    }
                }
            }
            (Value::Array(committed), Value::Array(served)) => {
                for (index, (value, counterpart)) in committed.iter().zip(served.iter()).enumerate()
                {
                    collect(child(&pointer, &index.to_string()), value, counterpart, out);
                    if out.len() > MAX_REPORTED_DIFFERENCES {
                        return;
                    }
                }
                // Entries past the shorter of the two have no counterpart to
                // compare against, so they are named with the value they hold —
                // a tag or a security requirement is usually the whole news.
                let common = committed.len().min(served.len());
                for (index, value) in committed.iter().enumerate().skip(common) {
                    out.push(format!(
                        "- {} = {} (snapshot only)",
                        child(&pointer, &index.to_string()),
                        render(value)
                    ));
                    if out.len() > MAX_REPORTED_DIFFERENCES {
                        return;
                    }
                }
                for (index, value) in served.iter().enumerate().skip(common) {
                    out.push(format!(
                        "+ {} = {} (served only)",
                        child(&pointer, &index.to_string()),
                        render(value)
                    ));
                    if out.len() > MAX_REPORTED_DIFFERENCES {
                        return;
                    }
                }
            }
            _ => {
                if committed != served {
                    out.push(format!(
                        "~ {pointer}: {} -> {}",
                        render(committed),
                        render(served)
                    ));
                }
            }
        }
    }

    /// Appends one segment to a JSON pointer, escaped as RFC 6901 asks, so the
    /// reported pointer resolves as-is against the document.
    fn child(pointer: &str, segment: &str) -> String {
        format!(
            "{pointer}/{}",
            segment.replace('~', "~0").replace('/', "~1")
        )
    }

    /// A value, short enough to read in a failure message and cut on a character
    /// boundary.
    fn render(value: &Value) -> String {
        let rendered = value.to_string();
        match rendered.char_indices().nth(MAX_RENDERED_VALUE_CHARS) {
            Some((cut, _)) => format!("{}…", &rendered[..cut]),
            None => rendered,
        }
    }
}
