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
            app_url: Url::parse("https://app.example.test/").expect("parse app url"),
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
}
