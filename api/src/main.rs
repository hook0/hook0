mod errors;
mod handlers;

use actix_web::{middleware::Logger, App, HttpResponse, HttpServer, Responder};
use clap::{crate_description, crate_name, crate_version, ArgSettings::HideEnvValues, Clap};
use log::{info, trace};
use paperclip::{
    actix::{api_v2_operation, web, OpenApiExt},
    v2::models::{DefaultApiRaw, Info},
};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

const APP_TITLE: &str = "Hook0 API";

#[derive(Debug, Clone, Clap)]
#[clap(author, about, version, name = APP_TITLE)]
struct Config {
    /// IP address on which to start the HTTP server
    #[clap(long, env, default_value = "127.0.0.1")]
    ip: String,

    /// Port on which to start the HTTP server
    #[clap(long, env, default_value = "8080")]
    port: String,

    /// Database URL (with credentials)
    #[clap(long, env, setting = HideEnvValues)]
    database_url: String,

    /// Maximum number of connections to database
    #[clap(long, env, default_value = "5")]
    max_db_connections: u32,
}

/// The app state
#[derive(Debug, Clone)]
pub struct State {
    db: PgPool,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = Config::parse();
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
    let initial_state = State { db: pool };

    // Run web server
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
            .service(web::resource("/").route(web::get().to(hello_world)))
            .service(
                web::scope("/v1").service(
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
                    // TODO:
                    // event_types
                    // application_secrets
                    // events
                    // -----
                    // subscriptions
                    // request_attempts
                ),
            )
            .with_json_spec_at("/api/spec")
            .build()
    })
    .bind(&format!("{}:{}", config.ip, config.port))?
    .run()
    .await
    .map_err(|e| e.into())
}

#[api_v2_operation]
async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
