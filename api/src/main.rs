use actix_files::{Files, NamedFile};
use actix_web::{middleware::Logger, App, HttpServer};
use actix_web_middleware_keycloak_auth::{AlwaysPassPolicy, DecodingKey, KeycloakAuth};
use clap::{crate_description, crate_name, crate_version, ArgSettings::HideEnvValues, Parser};
use log::{info, trace};
use paperclip::{
    actix::{web, OpenApiExt},
    v2::models::{DefaultApiRaw, Info},
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

mod handlers;
mod iam;
mod middleware_application_secret;
mod problems;

const APP_TITLE: &str = "Hook0 API";
const WEBAPP_INDEX_FILE: &str = "index.html";

#[derive(Debug, Clone, Parser)]
#[clap(author, about, version, name = APP_TITLE)]
struct Config {
    /// IP address on which to start the HTTP server
    #[clap(long, env, default_value = "127.0.0.1")]
    ip: String,

    /// Port on which to start the HTTP server
    #[clap(long, env, default_value = "8080")]
    port: String,

    /// Optional Sentry DSN for error reporting
    #[clap(long, env)]
    sentry_dsn: Option<String>,

    /// Database URL (with credentials)
    #[clap(long, env, setting = HideEnvValues)]
    database_url: String,

    /// Maximum number of connections to database
    #[clap(long, env, default_value = "5")]
    max_db_connections: u32,

    /// Path to the directory containing the web app to serve
    #[clap(long, env, default_value = "../frontend/dist/")]
    webapp_path: String,

    /// Keycloak RS256 public key (with GPG delimiters)
    #[clap(long, env)]
    keycloak_oidc_public_key: String,

    /// Disable automatic database migration
    #[clap(long = "no-auto-db-migration", env = "NO_AUTO_DB_MIGRATION", parse(from_flag = std::ops::Not::not))]
    auto_db_migration: bool,
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    // Initialize app logger as well as Sentry integration
    // Return value *must* be kept in a variable or else it will be dropped and Sentry integration won't work
    let _sentry = sentry_integration::init(crate_name!(), &config.sentry_dsn);

    trace!("Starting {}", APP_TITLE);

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

    // Initialize state
    let initial_state = State { db: pool };
    let keycloak_oidc_public_key = config.keycloak_oidc_public_key;

    // Run web server
    let webapp_path = config.webapp_path.clone();
    HttpServer::new(move || {
        // Compute default OpenAPI spec
        let spec = DefaultApiRaw {
            info: Info {
                title: APP_TITLE.to_owned(),
                description: match crate_description!() {
                    "" => None,
                    d => Some(d.to_owned()),
                },
                version: crate_version!().to_owned(),
                ..Default::default()
            },
            ..Default::default()
        };

        // Prepare auth middleware
        let pk = Box::new(keycloak_oidc_public_key.clone());
        let pk: &'static String = Box::leak(pk);
        let pk = DecodingKey::from_rsa_pem(pk.as_bytes()).unwrap();

        let jwt_auth = KeycloakAuth {
            detailed_responses: false,
            keycloak_oid_public_key: pk.clone(),
            required_roles: vec![],
            passthrough_policy: AlwaysPassPolicy,
        };

        let secret_auth = middleware_application_secret::ApplicationSecretAuth {
            db: initial_state.db.clone(),
        };

        App::new()
            .app_data(web::Data::new(initial_state.clone()))
            .wrap(Logger::default())
            .wrap_api_with_spec(spec)
            .with_json_spec_at("/api/v1/swagger.json")
            .service(
                web::scope("/api/v1")
                    // no auth
                    .service(
                        web::scope("/errors").service(
                            web::resource("").route(web::get().to(handlers::errors::list)),
                        ),
                    )
                    // with authentication
                    .service(
                        web::scope("/organizations")
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::organizations::list)),
                            ),
                    )
                    .service(
                        web::scope("/applications")
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
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
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
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
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
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
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(web::resource("").route(web::get().to(handlers::events::list)))
                            .service(
                                web::resource("/{event_id}")
                                    .route(web::get().to(handlers::events::get)),
                            ),
                    )
                    .service(
                        web::scope("/event")
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("").route(web::post().to(handlers::events::ingest)),
                            ),
                    )
                    .service(
                        web::scope("/subscriptions")
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::subscriptions::list))
                                    .route(web::post().to(handlers::subscriptions::add)),
                            )
                            .service(
                                web::resource("/{subscription_id}")
                                    .route(web::put().to(handlers::subscriptions::update))
                                    .route(web::delete().to(handlers::subscriptions::delete)),
                            ),
                    )
                    .service(
                        web::scope("/request_attempts")
                            .wrap(secret_auth.clone()) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::request_attempts::list)),
                            ),
                    )
                    .service(
                        web::scope("/responses")
                            .wrap(secret_auth) // Middleware order is counter intuitive: this is executed second
                            .wrap(jwt_auth.clone()) // Middleware order is counter intuitive: this is executed first
                            .service(
                                web::resource("/{response_id}")
                                    .route(web::get().to(handlers::responses::get)),
                            ),
                    ),
            )
            .default_service(
                Files::new("/", webapp_path.as_str())
                    .index_file(WEBAPP_INDEX_FILE)
                    .default_handler(
                        NamedFile::open(format!("{}/{}", &webapp_path, WEBAPP_INDEX_FILE))
                            .expect("Cannot open PWA page"),
                    ),
            )
            .build()
    })
    .bind(&format!("{}:{}", config.ip, config.port))?
    .run()
    .await
    .map_err(|e| e.into())
}
