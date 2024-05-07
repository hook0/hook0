use ::hook0_client::Hook0Client;
use actix::Arbiter;
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::middleware::{Compat, Logger, NormalizePath};
use actix_web::{http, middleware, App, HttpServer};
use biscuit_auth::{KeyPair, PrivateKey};
use clap::builder::{BoolValueParser, TypedValueParser};
use clap::{crate_name, ArgGroup, Parser};
use lettre::Address;
use log::{debug, info, trace, warn};
use paperclip::actix::{web, OpenApiExt};
use reqwest::Url;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

mod extractor_user_ip;
mod handlers;
mod hook0_client;
mod iam;
mod mailer;
mod materialized_views;
mod middleware_biscuit;
mod middleware_get_user_ip;
mod old_events_cleanup;
mod openapi;
mod problems;
mod quotas;
mod rate_limiting;
mod validators;

#[cfg(feature = "migrate-users-from-keycloak")]
mod keycloak_api;

const APP_TITLE: &str = "Hook0 API";
const WEBAPP_INDEX_FILE: &str = "index.html";

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version, name = APP_TITLE)]
#[clap(group(
    ArgGroup::new("client")
        .multiple(true)
        .requires_all(&["hook0_client_api_url", "hook0_client_application_id", "hook0_client_token"]),
))]
struct Config {
    /// IP address on which to start the HTTP server
    #[clap(long, env, default_value = "127.0.0.1")]
    ip: String,

    /// Port on which to start the HTTP server
    #[clap(long, env, default_value = "8080")]
    port: String,

    /// A comma-separated list of trusted IP addresses that are allowed to set "X-Forwarded-For" and "Forwarded" headers
    #[clap(long, env = "CC_REVERSE_PROXY_IPS", use_value_delimiter = true)]
    reverse_proxy_ips: Vec<String>,

    /// Optional Sentry DSN for error reporting
    #[clap(long, env)]
    sentry_dsn: Option<String>,

    /// Optional sample rate for tracing transactions with Sentry (between 0.0 and 1.0)
    #[clap(long, env)]
    sentry_traces_sample_rate: Option<f32>,

    /// Database URL (with credentials)
    #[clap(long, env, hide_env_values = true)]
    database_url: String,

    /// Maximum number of connections to database
    #[clap(long, env, default_value = "5")]
    max_db_connections: u32,

    /// Path to the directory containing the web app to serve
    #[clap(long, env, default_value = "../frontend/dist/")]
    webapp_path: String,

    /// Set to true to disable serving the web app and only serve the API
    #[clap(long, env)]
    disable_serving_webapp: bool,

    /// Key for the health check endpoint; if not specified, endpoint is disabled; if empty, endpoint is public
    #[clap(long, env, hide_env_values = true)]
    health_check_key: Option<String>,

    /// Keycloak RS256 public key (with GPG delimiters)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env)]
    keycloak_oidc_public_key: String,

    /// Biscuit's private key, used for authentication
    #[clap(long, env, value_parser = parse_biscuit_private_key)]
    biscuit_private_key: Option<PrivateKey>,

    /// Disable automatic database migration
    #[clap(long = "no-auto-db-migration", env = "NO_AUTO_DB_MIGRATION", value_parser = BoolValueParser::new().map(|v| !v))]
    auto_db_migration: bool,

    /// A global admin API key that have almost all rights. Better left undefined, USE AT YOUR OWN RISKS!
    #[clap(long, env, hide_env_values = true)]
    master_api_key: Option<Uuid>,

    /// URL of a Keycloak instance (example: https://my.keycloak.net/auth)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env)]
    keycloak_url: Url,

    /// Keycloak realm
    #[clap(long, env)]
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_realm: String,

    /// OIDC client ID (the confidential client for Hook0 API)
    #[clap(long, env)]
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_id: String,

    /// OIDC client secret (the confidential client for Hook0 API)
    #[cfg(feature = "migrate-users-from-keycloak")]
    #[clap(long, env, hide_env_values = true)]
    keycloak_client_secret: String,

    /// Set to true to disable registration endpoint
    #[clap(long, env)]
    disable_registration: bool,

    /// Minimum length of user passwords. This is checked when a user registers.
    #[clap(long, env, default_value = "12")]
    password_minimum_length: u8,

    /// Set to true to disable every API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting: bool,

    /// Set to true to disable global API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_global: bool,

    /// Global quota of API calls before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "2000")]
    api_rate_limiting_global_burst_size: u32,

    /// Duration (in millisecond) after which one global API call is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "1")]
    api_rate_limiting_global_replenish_period_in_ms: u64,

    /// Set to true to disable per-IP API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_ip: bool,

    /// Quota of API calls per IP before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "200")]
    api_rate_limiting_ip_burst_size: u32,

    /// Duration (in millisecond) after which one API call per IP is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "10")]
    api_rate_limiting_ip_replenish_period_in_ms: u64,

    /// Set to true to disable per-token API rate limiting
    #[clap(long, env)]
    disable_api_rate_limiting_token: bool,

    /// Quota of API calls per token before rate limiting blocks incomming requests (must be ≥ 1)
    #[clap(long, env, default_value = "20")]
    api_rate_limiting_token_burst_size: u32,

    /// Duration (in millisecond) after which one API call per token is restored in the quota (must be ≥ 1)
    #[clap(long, env, default_value = "100")]
    api_rate_limiting_token_replenish_period_in_ms: u64,

    /// Comma-separated allowed origins for CORS
    #[clap(long, env, use_value_delimiter = true)]
    cors_allowed_origins: Vec<String>,

    /// Base API URL of a Hook0 instance that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_api_url: Option<Url>,

    /// UUID of a Hook0 application that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_application_id: Option<Uuid>,

    /// Authentifcation token valid for a Hook0 application that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_token: Option<String>,

    /// Number of allowed retries when upserting event types to the linked Hook0 application fails
    #[clap(long, env, default_value = "10")]
    hook0_client_upserts_retries: u16,

    /// Set to true to apply quotas limits (default is not to)
    #[clap(long, env)]
    enable_quota_enforcement: bool,

    /// Default limit of members per organization (can be overriden by a plan)
    #[clap(long, env, default_value = "1")]
    quota_global_members_per_organization_limit: quotas::QuotaValue,

    /// Default limit of applications per organization (can be overriden by a plan)
    #[clap(long, env, default_value = "1")]
    quota_global_applications_per_organization_limit: quotas::QuotaValue,

    /// Default limit of events per day (can be overriden by a plan)
    #[clap(long, env, default_value = "100")]
    quota_global_events_per_day_limit: quotas::QuotaValue,

    /// Default limit of day of event's retention (can be overriden by a plan)
    #[clap(long, env, default_value = "7")]
    quota_global_days_of_events_retention_limit: quotas::QuotaValue,

    /// Duration (in second) to wait between materialized views refreshes
    #[clap(long, env, default_value = "60")]
    materialized_views_refresh_period_in_s: u64,

    /// Duration (in second) to wait between old events cleanups
    #[clap(long, env, default_value = "3600")]
    old_events_cleanup_period_in_s: u64,

    /// Duration (in day) to wait before actually deleting events that are passed retention period
    #[clap(long, env, default_value = "30")]
    old_events_cleanup_grace_period_in_day: u16,

    /// If true, old events will be reported and cleaned up; if false (default), they will only be reported
    #[clap(long, env, default_value = "false")]
    old_events_cleanup_report_and_delete: bool,

    /// If true, the secured HTTP headers will be enabled
    #[clap(long, env, default_value = "true")]
    enable_security_headers: bool,

    /// If true, the HSTS header will be enabled
    #[clap(long, env, default_value = "false")]
    enable_hsts_header: bool,

    /// Sender email address
    #[clap(long, env)]
    email_sender_address: Address,

    /// Sender name
    #[clap(long, env, default_value = "Hook0")]
    email_sender_name: String,

    /// Connection URL to SMTP server; for example: `smtp://localhost:1025`, `smtps://user:password@provider.com:465` (SMTP over TLS) or `smtp://user:password@provider.com:465?tls=required` (SMTP with STARTTLS)
    #[clap(long, env, hide_env_values = true)]
    smtp_connection_url: String,

    /// Duration (in second) to use as timeout when sending emails to the SMTP server
    #[clap(long, env, default_value = "5")]
    smtp_timeout_in_s: u64,

    /// Frontend application URL (used for building links in emails)
    #[clap(long, env)]
    app_url: String,
}

fn parse_biscuit_private_key(input: &str) -> Result<PrivateKey, String> {
    PrivateKey::from_bytes_hex(input)
        .map_err(|e| format!("Value of BISCUIT_PRIVATE_KEY is invalid ({e}). Re-run this app without the environment variable set to get a randomly generated key."))
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
    biscuit_private_key: PrivateKey,
    mailer: mailer::Mailer,
    app_url: String,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_url: Url,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_realm: String,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_id: String,
    #[cfg(feature = "migrate-users-from-keycloak")]
    keycloak_client_secret: String,
    registration_disabled: bool,
    password_minimum_length: u8,
    auto_db_migration: bool,
    hook0_client: Option<Hook0Client>,
    quotas: quotas::Quotas,
    health_check_key: Option<String>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    if let Some(biscuit_private_key) = config.biscuit_private_key {
        // Initialize app logger as well as Sentry integration
        // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
        let _sentry = sentry_integration::init(
            crate_name!(),
            &config.sentry_dsn,
            &config.sentry_traces_sample_rate,
        );

        trace!("Starting {}", APP_TITLE);

        // Prepare trusted reverse proxies IPs
        let reverse_proxy_ips = config
            .reverse_proxy_ips
            .iter()
            .map(|str| str.trim().to_owned())
            .collect::<Vec<_>>();
        if reverse_proxy_ips.is_empty() {
            warn!("No trusted reverse proxy IPs were set; if this is a production instance this is a problem");
        } else {
            debug!(
                "The following IPs will be considered as trusted reverse proxies: {}",
                &reverse_proxy_ips.join(", ")
            );
        }

        // Prepare rate limiting configuration
        let rate_limiters = rate_limiting::Hook0RateLimiters::new(
            config.disable_api_rate_limiting,
            config.disable_api_rate_limiting_global,
            config.api_rate_limiting_global_burst_size,
            config.api_rate_limiting_global_replenish_period_in_ms,
            config.disable_api_rate_limiting_ip,
            config.api_rate_limiting_ip_burst_size,
            config.api_rate_limiting_ip_replenish_period_in_ms,
            config.disable_api_rate_limiting_token,
            config.api_rate_limiting_token_burst_size,
            config.api_rate_limiting_token_replenish_period_in_ms,
        );

        // Create a DB connection pool
        let pool = PgPoolOptions::new()
            .max_connections(config.max_db_connections)
            .connect_with(
                PgConnectOptions::from_str(&config.database_url)?.application_name(crate_name!()),
            )
            .await?;
        info!(
            "Started a pool of maximum {} DB connections",
            &config.max_db_connections
        );

        // Run migrations
        if config.auto_db_migration {
            info!("Checking/running DB migrations");
            sqlx::migrate!("./migrations").run(&pool).await?;
        }

        // Initialize Hook0 client
        let hook0_client = hook0_client::initialize(
            config.hook0_client_api_url.clone(),
            config.hook0_client_application_id,
            config.hook0_client_token,
        );
        if let Some(client) = &hook0_client {
            let upsert_client = client.clone();
            let upserts_retries = config.hook0_client_upserts_retries;
            Arbiter::current().spawn(async move {
                trace!("Starting Hook0 client upsert task");
                hook0_client::upsert_event_types(
                    &upsert_client,
                    hook0_client::EVENT_TYPES,
                    upserts_retries,
                )
                .await;
            });
        }

        // Initialize quotas manager
        let quotas = quotas::Quotas::new(
            config.enable_quota_enforcement,
            config.quota_global_members_per_organization_limit,
            config.quota_global_applications_per_organization_limit,
            config.quota_global_events_per_day_limit,
            config.quota_global_days_of_events_retention_limit,
        );
        if config.enable_quota_enforcement {
            info!("Quota enforcement is enabled");
        } else {
            info!("Quota enforcement is disabled");
        }

        // Prepare master API key
        let master_api_key = config.master_api_key;
        if master_api_key.is_some() {
            warn!("The master API key is defined in the current configuration; THIS MAY BE A SECURITY ISSUE IN PRODUCTION");
        }

        // Spawn task to refresh materialized views
        let refresh_db = pool.clone();
        actix_web::rt::spawn(async move {
            materialized_views::periodically_refresh_materialized_views(
                &refresh_db,
                Duration::from_secs(config.materialized_views_refresh_period_in_s),
            )
            .await;
        });

        // Spawn task to clean up old events
        let cleanup_db = pool.clone();
        actix_web::rt::spawn(async move {
            old_events_cleanup::periodically_clean_up_old_events(
                &cleanup_db,
                Duration::from_secs(config.old_events_cleanup_period_in_s),
                config.quota_global_days_of_events_retention_limit,
                config.old_events_cleanup_grace_period_in_day,
                config.old_events_cleanup_report_and_delete,
            )
            .await;
        });

        // Create Mailer
        let mailer = mailer::Mailer::new(
            &config.smtp_connection_url,
            Duration::from_secs(config.smtp_timeout_in_s),
            config.email_sender_name,
            config.email_sender_address,
        )
        .await
        .expect("Could not initialize mailer; check SMTP configuration");

        // Initialize state
        let initial_state = State {
            db: pool,
            app_url: config.app_url,
            biscuit_private_key,
            mailer,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_url: config.keycloak_url,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_realm: config.keycloak_realm,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_id: config.keycloak_client_id,
            #[cfg(feature = "migrate-users-from-keycloak")]
            keycloak_client_secret: config.keycloak_client_secret,
            registration_disabled: config.disable_registration,
            password_minimum_length: config.password_minimum_length,
            auto_db_migration: config.auto_db_migration,
            hook0_client,
            quotas,
            health_check_key: config.health_check_key,
        };
        let hook0_client_api_url = config.hook0_client_api_url;

        // Run web server
        let webapp_path = config.webapp_path.clone();
        HttpServer::new(move || {
            // Compute default OpenAPI spec
            let spec = openapi::default_spec(&hook0_client_api_url);

            // Prepare user IP extraction middleware
            let get_user_ip = middleware_get_user_ip::GetUserIp {
                reverse_proxy_ips: reverse_proxy_ips.clone(),
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
                db: initial_state.db.clone(),
                biscuit_private_key: initial_state.biscuit_private_key.clone(),
                master_api_key,
            };

        let security_headers = middleware::DefaultHeaders::new()
            .add(("X-Content-Type-Options", "nosniff"))
            .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
            .add(("X-XSS-Protection", "1; mode=block"))
            .add(("Referrer-Policy", "SAMEORIGIN"));

        let hsts_header = middleware::DefaultHeaders::new()
            .add(("Strict-Transport-Security", "max-age=157680000"));

        let security_headers_condition =
            middleware::Condition::new(config.enable_security_headers, security_headers);

        let hsts_header_condition =
            middleware::Condition::new(config.enable_hsts_header, hsts_header);
            let mut app = App::new()
                .app_data(web::Data::new(initial_state.clone()))
                .app_data(web::JsonConfig::default().error_handler(|e, _req| {
                    let problem =
                        problems::Hook0Problem::JsonPayload(problems::JsonPayloadProblem::from(e));
                    actix_web::error::Error::from(problem)
                }))
                .wrap(get_user_ip)
                .wrap(hsts_header_condition)
                .wrap(security_headers_condition)
                .wrap(cors)
                .wrap(Logger::default())
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
                                    web::resource("/login")
                                        .route(web::post().to(handlers::auth::login)),
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
                                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                        .route(web::post().to(handlers::auth::reset_password)),
                                )
                                .service(
                                    web::resource("/password")
                                        .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                        .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first
                                        .route(web::post().to(handlers::auth::change_password)),
                                ),
                        )
                        // no auth
                        .service(web::scope("/instance").service(
                            web::resource("").route(web::get().to(handlers::instance::get)),
                        ))
                        .service(web::scope("/health").service(
                            web::resource("").route(web::get().to(handlers::instance::health)),
                        ))
                        .service(web::scope("/errors").service(
                            web::resource("").route(web::get().to(handlers::errors::list)),
                        ))
                        .service(
                            web::scope("/payload_content_types").service(
                                web::resource("")
                                    .route(web::get().to(handlers::events::payload_content_types)),
                            ),
                        )
                        .service(
                            web::scope("/register").service(
                                web::resource("")
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
                                                .route(
                                                    web::delete()
                                                        .to(handlers::organizations::delete),
                                                ),
                                        )
                                        .service(
                                            web::resource("/invite")
                                                .route(
                                                    web::put().to(handlers::organizations::invite),
                                                )
                                                .route(
                                                    web::delete()
                                                        .to(handlers::organizations::revoke),
                                                ),
                                        ),
                                ),
                        )
                        .service(
                            web::scope("/applications")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
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
                                        .route(
                                            web::post().to(handlers::application_secrets::create),
                                        ),
                                )
                                .service(
                                    web::resource("/{application_secret_token}")
                                        .route(web::put().to(handlers::application_secrets::edit))
                                        .route(
                                            web::delete().to(handlers::application_secrets::delete),
                                        ),
                                ),
                            #[cfg(not(feature = "application-secret-compatibility"))]
                            web::scope(""),
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
                                        .route(web::put().to(handlers::service_token::edit))
                                        .route(web::delete().to(handlers::service_token::delete)),
                                ),
                        )
                        .service(
                            web::scope("/events")
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("").route(web::get().to(handlers::events::list)),
                                )
                                .service(
                                    web::resource("/{event_id}")
                                        .route(web::get().to(handlers::events::get)),
                                ),
                        )
                        .service(
                            web::scope("/event")
                                .wrap(Compat::new(rate_limiters.token()))
                                .wrap(Compat::new(rate_limiters.token())) // Middleware order is counter intuitive: this is executed second
                                .wrap(biscuit_auth.clone()) // Middleware order is counter intuitive: this is executed first/ Middleware order is counter intuitive: this is executed first
                                .service(
                                    web::resource("")
                                        .route(web::post().to(handlers::events::ingest)),
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
                    Files::new("/", webapp_path.as_str())
                        .index_file(WEBAPP_INDEX_FILE)
                        .default_handler(
                            NamedFile::open(format!("{}/{}", &webapp_path, WEBAPP_INDEX_FILE))
                                .expect("Cannot open SPA main file"),
                        ),
                );
            }
            app.build()
        })
        .bind(&format!("{}:{}", config.ip, config.port))?
        .run()
        .await
        .map_err(|e| e.into())
    } else {
        let keypair = KeyPair::new();
        println!("The BISCUIT_PRIVATE_KEY environnement variable is required for authentication and authorization. You can use this randomly generated value: {}", keypair.private().to_bytes_hex());
        std::process::exit(1);
    }
}
