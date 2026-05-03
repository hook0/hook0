use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use hook0_acquisition::google_ads::{GoogleAdsClient, GoogleAdsConfig};
use hook0_acquisition::{create_app, AppState};

#[derive(Parser, Debug)]
#[command(name = "hook0-acquisition")]
#[command(about = "Hook0 server-side conversion uploader for Google Ads Enhanced Conversions")]
struct Args {
    /// Host to bind to
    #[arg(long, env = "ACQUISITION_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Port to bind to
    #[arg(short, long, env = "ACQUISITION_PORT", default_value = "3040")]
    port: u16,

    /// Shared bearer token required on every conversion upload request
    #[arg(long, env = "ACQUISITION_API_TOKEN")]
    api_token: String,

    #[arg(long, env = "GOOGLE_ADS_DEVELOPER_TOKEN")]
    developer_token: String,

    #[arg(long, env = "GOOGLE_ADS_CUSTOMER_ID")]
    customer_id: String,

    #[arg(long, env = "GOOGLE_ADS_LOGIN_CUSTOMER_ID")]
    login_customer_id: Option<String>,

    /// Numeric ID of the "Inscription" conversion action (e.g. 7576442588)
    #[arg(long, env = "GOOGLE_ADS_CONVERSION_ACTION_ID")]
    conversion_action_id: String,

    #[arg(long, env = "GOOGLE_ADS_OAUTH_CLIENT_ID")]
    oauth_client_id: String,

    #[arg(long, env = "GOOGLE_ADS_OAUTH_CLIENT_SECRET")]
    oauth_client_secret: String,

    #[arg(long, env = "GOOGLE_ADS_OAUTH_REFRESH_TOKEN")]
    oauth_refresh_token: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hook0_acquisition=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let google_ads_config = GoogleAdsConfig {
        developer_token: args.developer_token,
        customer_id: args.customer_id,
        login_customer_id: args.login_customer_id,
        conversion_action_id: args.conversion_action_id,
        oauth_client_id: args.oauth_client_id,
        oauth_client_secret: args.oauth_client_secret,
        oauth_refresh_token: args.oauth_refresh_token,
    };
    let google_ads = GoogleAdsClient::new(google_ads_config)
        .expect("Failed to construct Google Ads HTTP client");

    let state = Arc::new(AppState::new(google_ads, args.api_token));

    let app = create_app(state).layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Invalid host/port combination");
    info!("Starting hook0-acquisition on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
