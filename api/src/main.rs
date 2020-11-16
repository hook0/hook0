use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use clap::{ArgSettings::HideEnvValues, Clap};

#[derive(Debug, Clone, Clap)]
#[clap(author, about, version)]
struct Config {
    /// IP address on which to start the HTTP server
    #[clap(long = "ip", env = "IP", default_value = "127.0.0.1")]
    ip: String,

    /// Port on which to start the HTTP server
    #[clap(long = "port", env = "PORT", default_value = "8080")]
    port: String,

    /// Database URL (with credentials)
    #[clap(long, env, setting = HideEnvValues)]
    database_url: String,
}

/// The app state
#[derive(Clone)]
pub struct State {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Config::parse();

    // Initialize state
    let initial_state = State {};

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
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
