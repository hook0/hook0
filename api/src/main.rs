use actix_web::{middleware::Logger, App, HttpResponse, HttpServer, Responder};
use clap::{ArgSettings::HideEnvValues, Clap};
use log::{info, trace};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    Apiv2Schema, OpenApiExt,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Clone, Clap)]
#[clap(author, about, version)]
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
    trace!("Starting Hook0 API");

    // Create a DB connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.max_db_connections)
        .connect(&config.database_url)
        .await?;
    info!(
        "Started a pool of maximum {} DB connections",
        &config.max_db_connections
    );

    // Initialize state
    let initial_state = State { db: pool };

    // Run web server
    HttpServer::new(move || {
        App::new()
            .data(initial_state.clone())
            .wrap(Logger::default())
            .wrap_api()
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/test").route(web::post().to(test)))
            .with_json_spec_at("/api/spec")
            .build()
    })
    .bind(&format!("{}:{}", config.ip, config.port))?
    .run()
    .await
    .map_err(|e| e.into())
}

#[api_v2_operation]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Deserialize, Serialize, Apiv2Schema)]
struct Test {
    pub str: String,
    pub str_option: Option<String>,
    pub date: chrono::DateTime<chrono::Utc>,
    pub uuid: uuid::Uuid,
    pub num: u16,
    pub bool: bool,
}

#[api_v2_operation]
async fn test(test: Json<Test>) -> Result<Json<Test>, ()> {
    Ok(test)
}
