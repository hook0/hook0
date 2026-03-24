use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use hook0_play::{create_app, start_background_tasks, AppState};

#[derive(Parser, Debug)]
#[command(name = "hook0-play")]
#[command(about = "Hook0 Webhook Relay Server - webhook inspection and local tunneling")]
struct Args {
    /// Host to bind to
    #[arg(long, env = "HOOKS_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Port to bind to
    #[arg(short, long, env = "HOOKS_PORT", default_value = "3030")]
    port: u16,

    /// Base URL for generating webhook URLs (e.g., https://play.hook0.com)
    #[arg(long, env = "HOOKS_BASE_URL")]
    base_url: Option<String>,

    /// Enable encrypted storage for webhook bodies
    #[arg(long, env = "HOOKS_ENABLE_ENCRYPTION")]
    enable_encryption: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hook0_play=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Determine base URL
    let base_url = args.base_url.unwrap_or_else(|| {
        if args.host == "0.0.0.0" || args.host == "127.0.0.1" || args.host == "localhost" {
            format!("http://localhost:{}", args.port)
        } else {
            format!("http://{}:{}", args.host, args.port)
        }
    });

    info!("Starting Hook0 Play server");
    info!("Base URL: {}", base_url);

    // Build server limits from args
    let mut limits = hook0_play::ServerLimits::default();
    if args.enable_encryption {
        limits.enable_encryption = true;
        info!("Encrypted storage: enabled");
    }

    // Create application state
    let state = Arc::new(AppState::with_limits(base_url.clone(), limits));

    // Start background cleanup tasks (TTL, session timeouts, rate limiter cleanup)
    start_background_tasks(state.clone());

    // Build the router with middleware
    let app = create_app(state).layer(TraceLayer::new_for_http()).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    // Parse address
    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Invalid host/port combination");

    info!("Listening on {}", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);
    info!("Webhook URL format: {}/in/<token>/", base_url);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server error");
}
