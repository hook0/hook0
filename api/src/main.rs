use ::hook0_client::Hook0Client;
use actix::Arbiter;
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::middleware::{Compat, Logger, NormalizePath};
use actix_web::{http, App, HttpServer};
use actix_web_middleware_keycloak_auth::{AlwaysPassPolicy, DecodingKey, KeycloakAuth};
use clap::builder::{BoolValueParser, TypedValueParser};
use clap::{crate_name, ArgGroup, Parser};
use log::{debug, info, trace, warn};
use paperclip::actix::{web, OpenApiExt};
use reqwest::Url;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;
use uuid::Uuid;

mod extractor_user_ip;
mod handlers;
mod hook0_client;
mod iam;
mod keycloak_api;
mod middleware_application_secret;
mod middleware_get_user_ip;
mod openapi;
mod problems;
mod rate_limiting;
mod validators;

const APP_TITLE: &str = "Hook0 API";
const WEBAPP_INDEX_FILE: &str = "index.html";

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version, name = APP_TITLE)]
#[clap(group(
    ArgGroup::new("client")
        .multiple(true)
        .requires_all(&["hook0_client_api_url", "hook0_client_application_id", "hook0_client_application_secret"]),
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

    /// Optional sample rate for profiling transactions with Sentry (between 0.0 and 1.0)
    #[clap(long, env)]
    sentry_profiles_sample_rate: Option<f32>,

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

    /// Keycloak RS256 public key (with GPG delimiters)
    #[clap(long, env)]
    keycloak_oidc_public_key: String,

    /// Disable automatic database migration
    #[clap(long = "no-auto-db-migration", env = "NO_AUTO_DB_MIGRATION", value_parser = BoolValueParser::new().map(|v| !v))]
    auto_db_migration: bool,

    /// URL of a Keycloak instance (example: https://my.keycloak.net/auth)
    #[clap(long, env)]
    keycloak_url: Url,

    /// Keycloak realm
    #[clap(long, env)]
    keycloak_realm: String,

    /// OIDC client ID (the confidential client for Hook0 API)
    #[clap(long, env)]
    keycloak_client_id: String,

    /// OIDC client ID (the public client for Hook0 frontend)
    #[clap(long, env)]
    keycloak_front_client_id: String,

    /// OIDC client secret (the confidential client for Hook0 API)
    #[clap(long, env, hide_env_values = true)]
    keycloak_client_secret: String,

    /// Set to true to disable registration endpoint
    #[clap(long, env)]
    disable_registration: bool,

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

    /// Secret of a Hook0 application that will receive events from this Hook0 instance
    #[clap(long, env, group = "client")]
    hook0_client_application_secret: Option<Uuid>,

    /// Number of allowed retries when upserting event types to the linked Hook0 application fails
    #[clap(long, env, default_value = "10")]
    hook0_client_upserts_retries: u16,
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
    keycloak_url: Url,
    keycloak_realm: String,
    keycloak_client_id: String,
    keycloak_client_secret: String,
    keycloak_front_client_id: String,
    disable_registration: bool,
    auto_db_migration: bool,
    hook0_client: Option<Hook0Client>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    // Initialize app logger as well as Sentry integration
    // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
    let _sentry = sentry_integration::init(
        crate_name!(),
        &config.sentry_dsn,
        &config.sentry_traces_sample_rate,
        &config.sentry_profiles_sample_rate,
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
        config.hook0_client_application_secret,
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

    // Initialize state
    let initial_state = State {
        db: pool,
        keycloak_url: config.keycloak_url,
        keycloak_realm: config.keycloak_realm,
        keycloak_client_id: config.keycloak_client_id,
        keycloak_client_secret: config.keycloak_client_secret,
        keycloak_front_client_id: config.keycloak_front_client_id,
        disable_registration: config.disable_registration,
        auto_db_migration: config.auto_db_migration,
        hook0_client,
    };
    let keycloak_oidc_public_key = config.keycloak_oidc_public_key;
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

        // Prepare auth middleware
        let pk = DecodingKey::from_rsa_pem(keycloak_oidc_public_key.as_bytes()).unwrap();

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

        let jwt_auth = KeycloakAuth {
            detailed_responses: false,
            keycloak_oid_public_key: pk,
            required_roles: vec![],
            passthrough_policy: AlwaysPassPolicy,
        };

        let secret_auth = middleware_application_secret::ApplicationSecretAuth {
            db: initial_state.db.clone(),
        };

        let mut app = App::new()
            .app_data(web::Data::new(initial_state.clone()))
            .app_data(web::JsonConfig::default().error_handler(|e, _req| {
                let problem =
                    problems::Hook0Problem::JsonPayload(problems::JsonPayloadProblem::from(e));
                actix_web::error::Error::from(problem)
            }))
            .wrap(get_user_ip)
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
                    // no auth
                    .service(
                        web::scope("/instance").service(
                            web::resource("").route(web::get().to(handlers::instance::get)),
                        ),
                    )
                    .service(
                        web::scope("/errors").service(
                            web::resource("").route(web::get().to(handlers::errors::list)),
                        ),
                    )
                    .service(
                        web::scope("/payload_content_types").service(
                            web::resource("")
                                .route(web::get().to(handlers::events::payload_content_types)),
                        ),
                    )
                    .service(web::scope("/register").service(
                        web::resource("").route(web::post().to(handlers::registrations::register)),
                    ))
                    // with authentication
                    .service(
                        web::scope("/organizations")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
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
                                                web::delete().to(handlers::organizations::delete),
                                            ),
                                    )
                                    .service(
                                        web::resource("/invite")
                                            .route(web::put().to(handlers::organizations::invite))
                                            .route(
                                                web::delete().to(handlers::organizations::revoke),
                                            ),
                                    ),
                            ),
                    )
                    .service(
                        web::scope("/applications")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
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
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
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
                        web::scope("/application_secrets")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::application_secrets::list))
                                    .route(web::post().to(handlers::application_secrets::create)),
                            )
                            .service(
                                web::resource("/{application_secret_token}")
                                    .route(web::put().to(handlers::application_secrets::update))
                                    .route(web::delete().to(handlers::application_secrets::delete)),
                            ),
                    )
                    .service(
                        web::scope("/events")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
                            .service(web::resource("").route(web::get().to(handlers::events::list)))
                            .service(
                                web::resource("/{event_id}")
                                    .route(web::get().to(handlers::events::get)),
                            ),
                    )
                    .service(
                        web::scope("/event")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("").route(web::post().to(handlers::events::ingest)),
                            ),
                    )
                    .service(
                        web::scope("/subscriptions")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::subscriptions::list))
                                    .route(web::post().to(handlers::subscriptions::add)),
                            )
                            .service(
                                web::resource("/{subscription_id}")
                                    .route(web::get().to(handlers::subscriptions::get))
                                    .route(web::put().to(handlers::subscriptions::update))
                                    .route(web::delete().to(handlers::subscriptions::delete)),
                            ),
                    )
                    .service(
                        web::scope("/request_attempts")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth.clone())) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::request_attempts::list)),
                            ),
                    )
                    .service(
                        web::scope("/responses")
                            .wrap(Compat::new(rate_limiters.token()))
                            .wrap(secret_auth) // Middleware order is counter intuitive: this is executed second
                            .wrap(Compat::new(jwt_auth)) // Middleware order is counter intuitive: this is executed first
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
}
