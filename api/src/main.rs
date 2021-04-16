mod errors;
mod handlers;

use actix_files::{Files, NamedFile};
use actix_web::web::Data;
use actix_web::{middleware::Logger, App, HttpRequest, HttpResponse, HttpServer};
use clap::{crate_description, crate_name, crate_version, ArgSettings::HideEnvValues, Clap};
use log::{info, trace};
use paperclip::{
    actix::{api_v2_operation, web, OpenApiExt},
    v2::models::{DefaultApiRaw, Info},
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

const APP_TITLE: &str = "Hook0 API";
const WEBAPP_INDEX_FILE: &str = "index.html";

#[derive(Debug, Clone, Clap)]
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
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
    webapp_path: String,
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

    // Initialize state
    let initial_state = State {
        db: pool,
        webapp_path: config.webapp_path.clone(),
    };

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

        App::new()
            .data(initial_state.clone())
            .wrap(Logger::default())
            .wrap_api_with_spec(spec)
            .with_json_spec_at("/api/v1/swagger.json")
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/applications")
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::applications::list))
                                    .route(web::post().to(handlers::applications::add)),
                            )
                            .service(
                                web::resource("/{application_id}")
                                    .route(web::get().to(handlers::applications::show))
                                    .route(web::put().to(handlers::applications::edit))
                                    .route(web::delete().to(handlers::applications::destroy)),
                            ),
                    )
                    .service(
                        web::scope("/event_types")
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::event_types::list))
                                    .route(web::post().to(handlers::event_types::add)),
                            )
                            .service(
                                web::resource("/{event_type_name}")
                                    .route(web::get().to(handlers::event_types::show))
                                    .route(web::delete().to(handlers::event_types::destroy)),
                            ),
                    )
                    .service(
                        web::scope("/application_secrets")
                            .service(
                                web::resource("")
                                    .route(web::get().to(handlers::application_secrets::list))
                                    .route(web::post().to(handlers::application_secrets::add)),
                            )
                            .service(
                                web::resource("/{application_secret_token}")
                                    .route(web::put().to(handlers::application_secrets::edit))
                                    .route(
                                        web::delete().to(handlers::application_secrets::destroy),
                                    ),
                            ),
                    )
                    .service(
                        web::scope("/events")
                            .service(web::resource("").route(web::get().to(handlers::events::list)))
                            .service(
                                web::resource("/{event_id}")
                                    .route(web::get().to(handlers::events::show)),
                            ),
                    )
                    .service(
                        web::resource("/event").route(web::post().to(handlers::events::ingest)),
                    ),
                // TODO:
                // events
                // -----
                // subscriptions
                // request_attempts
            )
            .default_service(
                Files::new("/", webapp_path.as_str())
                    .index_file(WEBAPP_INDEX_FILE)
                    .default_handler(
                        web::resource("{path:.+}").route(web::get().to(default_handler)),
                    ),
            )
            .build()
    })
    .bind(&format!("{}:{}", config.ip, config.port))?
    .run()
    .await
    .map_err(|e| e.into())
}

#[api_v2_operation]
async fn default_handler(
    req: HttpRequest,
    state: Data<crate::State>,
) -> actix_web::Result<HttpResponse> {
    NamedFile::open(format!("{}/{}", &state.webapp_path, WEBAPP_INDEX_FILE))?.into_response(&req)
}
