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

use crate::client_options::{CLIENT_OPTIONS_HEADER, Hook0RootSpanBuilder};
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
                // Every published SDK sends this on every request, so leaving it out of the
                // allow-list makes a browser refuse the preflight and the request never leaves.
                // It is the constant the request handler reads the header with, so the two
                // cannot drift into naming it differently.
                CLIENT_OPTIONS_HEADER,
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
        .wrap(TracingLogger::<Hook0RootSpanBuilder>::new())
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
                                .wrap(Compat::new(rate_limiters.email()))
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
                                .wrap(Compat::new(rate_limiters.email()))
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
                // Registration mails a verification link to an address the
                // caller names, so it belongs to the same family as
                // begin-reset-password and resend-verification-email and is
                // bounded by the same per-IP limiter. Unlike those two it cannot
                // be aimed at one mailbox twice — the address is unique — so what
                // this bounds is a sweep across many addresses. Turnstile already
                // stands in front of it wherever it is configured; this holds on
                // the instances where it is not.
                .service(
                    web::scope("/register").service(
                        web::resource("")
                            .wrap(Compat::new(rate_limiters.email()))
                            .route(web::post().to(handlers::registrations::register)),
                    ),
                )
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
                                .route(web::put().to(handlers::service_token::edit))
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
                true, true, 2000, 1, true, 200, 10, true, 20, 100, true, 5, 60_000,
            ),
            disable_serving_webapp: true,
            webapp_path: String::new(),
        }
    }

    /// A browser can send what the SDKs send.
    ///
    /// Every published client puts `Hook0-Client-Options` on every request, and the header is not
    /// one the fetch specification safelists, so a browser asks permission for it before the
    /// request is made. A header missing from the allow-list makes that preflight fail, and the
    /// request the caller wrote never leaves the page. The SDK works from a server and stops
    /// working from a browser, with nothing in the API log to say why.
    ///
    /// The preflight is driven rather than the allow-list read, because what decides is what
    /// `actix-cors` does with the list rather than what the list contains.
    #[actix_web::test]
    async fn a_browser_may_send_the_header_every_sdk_sends() {
        let mut config = inert_app_factory_config().await;
        const ORIGIN: &str = "https://dashboard.hook0.test";
        config.cors_allowed_origins = vec![ORIGIN.to_owned()];
        let app = test::init_service(build_app(&config)).await;

        let response = test::call_service(
            &app,
            test::TestRequest::default()
                .method(actix_web::http::Method::OPTIONS)
                .uri("/api/v1/events/")
                .insert_header(("Origin", ORIGIN))
                .insert_header(("Access-Control-Request-Method", "POST"))
                .insert_header((
                    "Access-Control-Request-Headers",
                    CLIENT_OPTIONS_HEADER.as_str(),
                ))
                .peer_addr(SocketAddr::from(([127, 0, 0, 1], 5678)))
                .to_request(),
        )
        .await;

        assert!(
            response.status().is_success(),
            "the preflight for `{}` was answered {}, so a browser would refuse to send the \
             request; add the header to the allow-list in `build_app`",
            CLIENT_OPTIONS_HEADER.as_str(),
            response.status()
        );
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
    use std::collections::{BTreeMap, BTreeSet};

    /// The document generated clients and the MCP server are built from.
    /// Committed so neither has to reach a running instance.
    const SNAPSHOT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.snapshot.json");

    /// Tags that keep an operation in the snapshot. `sdk` is the surface
    /// generated clients expose to their users; `mcp` is what the MCP server
    /// turns into tools, and it covers a couple of operations the SDKs do not.
    /// Everything else is the control plane the dashboard drives, and it stays
    /// out: nobody generates a client for it.
    const RETAINED_TAGS: [&str; 2] = ["sdk", "mcp"];

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

    // -- Documentation JSON examples against the OpenAPI schemas --------------
    //
    // A `json` example in the documentation that depicts an API object rots the
    // same way an SDK snippet does: a field is renamed or invented, the reader
    // copies it, and the page describes an API that never existed. This holds
    // every such example to the schema the API actually serves (the committed
    // snapshot), the same source the SDKs and the frontend types are built from.
    //
    // The binding is explicit: an example states the schema it shows with a
    // marker on the line above its fence — `<!-- openapi: RequestAttempt -->` —
    // and is then checked field-by-field against it. That an example carries the
    // right marker is the one thing this cannot infer for a *renamed* block (it
    // no longer looks like anything); what it can, and does, catch is the block
    // that still reads unmistakably as a schema yet was left unbound, so nobody
    // adds an API example past the check by forgetting to tag it.

    /// The tree whose `json` examples are checked, relative to the crate.
    const DOCUMENTATION_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../documentation");

    /// Prefix a reported path is trimmed to, so a failure names
    /// `documentation/…` rather than an absolute path.
    const REPOSITORY_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

    /// Set to any value to insert the missing markers instead of reporting them.
    const DOC_MARKER_UPDATE_VAR: &str = "UPDATE_DOC_OPENAPI_MARKERS";

    /// What to run to bind every newly recognized example at once.
    const DOC_MARKER_UPDATE_COMMAND: &str =
        "UPDATE_DOC_OPENAPI_MARKERS=1 cargo test -p hook0-api doc_json_examples";

    /// How many of a schema's *unique* fields an unbound example must show at its
    /// top level before it is unmistakably that schema and has to be bound. Two
    /// keeps a coincidental single field (an event payload that happens to carry
    /// `amount`) from being mistaken for an API object.
    const SIGNATURE_FIELDS_REQUIRING_A_MARKER: usize = 2;

    /// Ceilings so a runaway tree cannot pull the process over: files walked,
    /// bytes read per file, and examples examined.
    const MAX_DOC_FILES: usize = 4096;
    const MAX_DOC_FILE_BYTES: u64 = 4 * 1024 * 1024;
    const MAX_DOC_EXAMPLES: usize = 8192;

    /// How deep a nested example or schema is followed, a backstop against a
    /// `$ref` cycle in the schemas or a pathological example.
    const MAX_JSON_DEPTH: usize = 24;

    /// The `components.schemas` object of the committed snapshot: the definitions
    /// downstream is generated from, and the authority an example is held to.
    fn snapshot_schemas() -> serde_json::Map<String, Value> {
        let committed: Value = serde_json::from_slice(
            &std::fs::read(SNAPSHOT_PATH).expect("the snapshot is readable"),
        )
        .expect("the committed snapshot is valid JSON");
        committed
            .get("components")
            .and_then(|components| components.get("schemas"))
            .and_then(Value::as_object)
            .cloned()
            .expect("the snapshot exposes components.schemas")
    }

    /// Follows a local `$ref` to the schema it names, leaving anything else as it
    /// is. One hop: the caller recurses, so a chain still resolves.
    fn resolve<'a>(schemas: &'a serde_json::Map<String, Value>, node: &'a Value) -> &'a Value {
        if let Some(Value::String(reference)) = node.get("$ref")
            && let Some(name) = reference.strip_prefix("#/components/schemas/")
            && let Some(target) = schemas.get(name)
        {
            return target;
        }
        node
    }

    /// Every property name a schema declares, at any depth, following `$ref` once
    /// per name so a cycle terminates.
    fn collect_property_names(
        schemas: &serde_json::Map<String, Value>,
        node: &Value,
        visited: &mut BTreeSet<String>,
        out: &mut BTreeSet<String>,
        depth: usize,
    ) {
        if depth == 0 {
            return;
        }
        if let Some(Value::String(reference)) = node.get("$ref")
            && let Some(name) = reference.strip_prefix("#/components/schemas/")
        {
            if !visited.insert(name.to_owned()) {
                return;
            }
            if let Some(target) = schemas.get(name) {
                collect_property_names(schemas, target, visited, out, depth - 1);
            }
            return;
        }
        if let Some(Value::Object(properties)) = node.get("properties") {
            for (name, child) in properties {
                out.insert(name.clone());
                collect_property_names(schemas, child, visited, out, depth - 1);
            }
        }
        if let Some(items) = node.get("items") {
            collect_property_names(schemas, items, visited, out, depth - 1);
        }
    }

    /// Field name → the one schema that declares it, kept only for names no other
    /// schema shares. These are what let an unbound example be recognized: a
    /// block carrying `request_attempt_id` can be nothing but a `RequestAttempt`.
    fn signature_fields(schemas: &serde_json::Map<String, Value>) -> BTreeMap<String, String> {
        let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (schema_name, schema) in schemas {
            let mut names = BTreeSet::new();
            let mut visited = BTreeSet::new();
            collect_property_names(schemas, schema, &mut visited, &mut names, MAX_JSON_DEPTH);
            for field in names {
                owners.entry(field).or_default().insert(schema_name.clone());
            }
        }
        owners
            .into_iter()
            .filter_map(|(field, schemas)| {
                let mut schemas = schemas.into_iter();
                match (schemas.next(), schemas.next()) {
                    (Some(only), None) => Some((field, only)),
                    _ => None,
                }
            })
            .collect()
    }

    /// The paths of an example that name a field its schema has no place for.
    /// An array is read as a collection of the object schema (or of its `items`
    /// when it declares them); an object whose schema declares no properties is
    /// free-form and left alone rather than judged.
    fn illegal_paths(
        schemas: &serde_json::Map<String, Value>,
        value: &Value,
        schema: &Value,
        base: &str,
        out: &mut Vec<String>,
        depth: usize,
    ) {
        if depth == 0 {
            return;
        }
        let schema = resolve(schemas, schema);
        match value {
            Value::Array(items) => {
                let element = match schema.get("items") {
                    Some(items_schema) => resolve(schemas, items_schema),
                    None => schema,
                };
                for (index, item) in items.iter().enumerate() {
                    illegal_paths(
                        schemas,
                        item,
                        element,
                        &format!("{base}[{index}]"),
                        out,
                        depth - 1,
                    );
                }
            }
            Value::Object(map) => {
                let properties = match schema.get("properties").and_then(Value::as_object) {
                    Some(properties) => properties,
                    None => return,
                };
                for (key, child_value) in map {
                    let path = if base.is_empty() {
                        key.clone()
                    } else {
                        format!("{base}.{key}")
                    };
                    match properties.get(key) {
                        None => out.push(path),
                        Some(child_schema) => {
                            if matches!(child_value, Value::Object(_) | Value::Array(_)) {
                                illegal_paths(
                                    schemas,
                                    child_value,
                                    child_schema,
                                    &path,
                                    out,
                                    depth - 1,
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// The object an example is about: itself, or the first element when it is a
    /// non-empty array of objects (a list endpoint's response).
    fn top_level_object(value: &Value) -> Option<&serde_json::Map<String, Value>> {
        match value {
            Value::Object(map) => Some(map),
            Value::Array(items) => items.first().and_then(Value::as_object),
            _ => None,
        }
    }

    /// One fenced `json` example: where its fence is, the schema it bound itself
    /// to (if any), and its body.
    struct DocExample {
        fence_line: usize,
        marker: Option<String>,
        body: String,
    }

    /// The schema a marker line names, e.g. `<!-- openapi: RequestAttempt -->`.
    fn parse_marker(line: &str) -> Option<String> {
        let inner = line.trim().strip_prefix("<!--")?.trim();
        let inner = inner.strip_prefix("openapi:")?.trim();
        let name = inner.strip_suffix("-->")?.trim();
        let named = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        named.then(|| name.to_owned())
    }

    /// The `json` examples of a page, each with the marker on the line above its
    /// fence when there is one.
    fn json_examples(source: &str) -> Vec<DocExample> {
        let lines: Vec<&str> = source.lines().collect();
        let mut examples = Vec::new();
        let mut last_nonblank: Option<usize> = None;
        let mut i = 0;
        while i < lines.len() {
            if lines[i].trim() == "```json" {
                let marker = last_nonblank.and_then(|index| parse_marker(lines[index]));
                let mut body = String::new();
                let mut j = i + 1;
                while j < lines.len() && lines[j].trim() != "```" {
                    body.push_str(lines[j]);
                    body.push('\n');
                    j += 1;
                }
                examples.push(DocExample {
                    fence_line: i,
                    marker,
                    body,
                });
                last_nonblank = (j < lines.len()).then_some(j);
                i = j + 1;
                continue;
            }
            if !lines[i].trim().is_empty() {
                last_nonblank = Some(i);
            }
            i += 1;
        }
        examples
    }

    /// The `.md`/`.mdx` files of the documentation tree, sorted, bounded, with
    /// the build output skipped.
    fn documentation_files() -> Vec<std::path::PathBuf> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            if out.len() >= MAX_DOC_FILES {
                return;
            }
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            let mut children: Vec<std::path::PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                .collect();
            children.sort();
            for path in children {
                if out.len() >= MAX_DOC_FILES {
                    return;
                }
                if path.is_dir() {
                    let skipped =
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| {
                                matches!(name, "node_modules" | "build" | ".docusaurus")
                            });
                    if !skipped {
                        walk(&path, out);
                    }
                } else if path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension == "md" || extension == "mdx")
                {
                    out.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(std::path::Path::new(DOCUMENTATION_DIR), &mut files);
        files
    }

    /// Reads a documentation page, refusing one past [`MAX_DOC_FILE_BYTES`]
    /// rather than growing to hold it.
    fn read_documentation_file(path: &std::path::Path) -> String {
        use std::io::Read;

        let file = std::fs::File::open(path)
            .unwrap_or_else(|err| panic!("the page {} cannot be read ({err})", path.display()));
        let mut source = String::new();
        let read = file
            .take(MAX_DOC_FILE_BYTES + 1)
            .read_to_string(&mut source)
            .unwrap_or_else(|err| panic!("the page {} is not UTF-8 ({err})", path.display()));
        assert!(
            read as u64 <= MAX_DOC_FILE_BYTES,
            "the page {} is larger than the {MAX_DOC_FILE_BYTES} bytes this reads",
            path.display()
        );
        source
    }

    /// Nothing downstream — SDKs, MCP tools, frontend types — reaches a `json`
    /// example, so a reader is the only thing that catches one describing a field
    /// the API does not have. This makes the check that reader: every bound
    /// example is held to its schema's fields, and every unbound block that still
    /// reads as a schema is reported until it is bound or corrected.
    ///
    /// Setting `UPDATE_DOC_OPENAPI_MARKERS` binds the recognized ones in place
    /// instead, the way `UPDATE_OPENAPI_SNAPSHOT` adopts a surface change.
    #[test]
    fn doc_json_examples_match_the_openapi_schemas() {
        let schemas = snapshot_schemas();
        let signatures = signature_fields(&schemas);
        let update = std::env::var_os(DOC_MARKER_UPDATE_VAR).is_some();

        let mut problems: Vec<String> = Vec::new();
        let mut examples_seen = 0usize;
        let mut bound_examples = 0usize;
        let mut markers_inserted = 0usize;

        for path in documentation_files() {
            let source = read_documentation_file(&path);
            let display = path
                .strip_prefix(REPOSITORY_ROOT)
                .unwrap_or(path.as_path())
                .display()
                .to_string();
            let mut insertions: Vec<(usize, String)> = Vec::new();

            for example in json_examples(&source) {
                examples_seen += 1;
                if examples_seen > MAX_DOC_EXAMPLES {
                    break;
                }
                let line = example.fence_line + 1;
                let value: Value = match serde_json::from_str(&example.body) {
                    Ok(value) => value,
                    Err(_) => continue,
                };

                match &example.marker {
                    Some(schema_name) => {
                        let Some(schema) = schemas.get(schema_name) else {
                            problems.push(format!(
                                "{display}:{line}  is bound to `{schema_name}`, which the API serves no schema of"
                            ));
                            continue;
                        };
                        bound_examples += 1;
                        let mut illegal = Vec::new();
                        illegal_paths(&schemas, &value, schema, "", &mut illegal, MAX_JSON_DEPTH);
                        if !illegal.is_empty() {
                            problems.push(format!(
                                "{display}:{line}  [{schema_name}] shows fields the schema has no place for: {}",
                                illegal.join(", ")
                            ));
                        }
                    }
                    None => {
                        let Some(top) = top_level_object(&value) else {
                            continue;
                        };
                        let mut hits: BTreeMap<&str, usize> = BTreeMap::new();
                        for key in top.keys() {
                            if let Some(schema_name) = signatures.get(key.as_str()) {
                                *hits.entry(schema_name.as_str()).or_default() += 1;
                            }
                        }
                        let Some((schema_name, count)) = hits
                            .iter()
                            .max_by_key(|(_, count)| **count)
                            .map(|(name, count)| (*name, *count))
                        else {
                            continue;
                        };
                        if count < SIGNATURE_FIELDS_REQUIRING_A_MARKER {
                            continue;
                        }
                        let candidates: Vec<&str> = hits
                            .iter()
                            .filter(|(_, other)| **other == count)
                            .map(|(name, _)| *name)
                            .collect();
                        if candidates.len() > 1 {
                            problems.push(format!(
                                "{display}:{line}  reads as one of {candidates:?}; bind it with a `<!-- openapi: … -->` marker or correct its fields"
                            ));
                            continue;
                        }
                        if update {
                            insertions.push((
                                example.fence_line,
                                format!("<!-- openapi: {schema_name} -->"),
                            ));
                            markers_inserted += 1;
                        } else {
                            problems.push(format!(
                                "{display}:{line}  reads as `{schema_name}` ({count} of its unique fields present) but is not bound to it; add `<!-- openapi: {schema_name} -->` above the fence, or correct its fields"
                            ));
                        }
                    }
                }
            }

            if update && !insertions.is_empty() {
                let mut lines: Vec<String> = source.lines().map(str::to_owned).collect();
                insertions.sort_by_key(|insertion| std::cmp::Reverse(insertion.0));
                for (fence_line, marker) in insertions {
                    let indent: String = lines[fence_line]
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect();
                    lines.insert(fence_line, format!("{indent}{marker}"));
                }
                let mut rebuilt = lines.join("\n");
                if source.ends_with('\n') {
                    rebuilt.push('\n');
                }
                std::fs::write(&path, rebuilt).expect("the documentation page is writable");
            }
        }

        if update {
            println!("Inserted {markers_inserted} openapi marker(s)");
            return;
        }

        assert!(
            problems.is_empty(),
            "documentation json examples drifted from the API schemas ({} issue(s)):\n{}\n\n\
             An example that shows an API object binds itself to its schema with a marker on the \
             line above its fence, e.g. `<!-- openapi: RequestAttempt -->`, and is then held to \
             that schema's fields. Bind the recognized ones with:\n    {DOC_MARKER_UPDATE_COMMAND}\n\
             Schemas: {SNAPSHOT_PATH}",
            problems.len(),
            problems
                .iter()
                .take(MAX_REPORTED_DIFFERENCES)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        );

        println!("{bound_examples} bound json example(s) validated against the OpenAPI schemas");
    }
}
