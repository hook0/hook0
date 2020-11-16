use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use clap::{ArgSettings::HideEnvValues, Clap};
use log::{info, trace};
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

    /// Maximum number of connection to database
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
            .route("/", web::get().to(index))
    })
    .bind(&format!("{}:{}", config.ip, config.port))?
    .run()
    .await
    .map_err(|e| e.into())
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
