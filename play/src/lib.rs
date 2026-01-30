pub mod api;
pub mod audit;
pub mod limits;
pub mod rate_limiter;
pub mod relay;
pub mod sanitize;
pub mod storage;
pub mod webhook;

use dashmap::DashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::info;

pub use limits::ServerLimits;
use rate_limiter::{InvalidTokenTracker, RateLimiter};
use storage::WebhookStorage;

/// Application state shared across all handlers
pub struct AppState {
    /// Webhook storage
    pub storage: WebhookStorage,
    /// Active WebSocket connections by token
    pub connections: DashMap<String, mpsc::Sender<String>>,
    /// Base URL for generating webhook URLs
    pub base_url: String,
    /// Server limits configuration
    pub limits: ServerLimits,
    /// Current connection count (for enforcing max_total_connections)
    pub connection_count: AtomicUsize,
    /// Connections per IP (for enforcing max_connections_per_ip)
    pub connections_per_ip: DashMap<String, usize>,
    /// Per-IP rate limiter for webhook endpoints
    pub rate_limiter_ip: RateLimiter,
    /// Per-token rate limiter for webhook endpoints
    pub rate_limiter_token: RateLimiter,
    /// Global rate limiter for webhook endpoints
    pub rate_limiter_global: RateLimiter,
    /// Invalid token attempt tracker
    pub invalid_token_tracker: InvalidTokenTracker,
}

impl AppState {
    /// Create a new application state with default limits
    pub fn new(base_url: String) -> Self {
        Self::with_limits(base_url, ServerLimits::default())
    }

    /// Create a new application state with custom limits
    pub fn with_limits(base_url: String, limits: ServerLimits) -> Self {
        let mut storage = WebhookStorage::with_limits(limits.max_webhooks_per_token);

        // Enable encryption if configured
        if limits.enable_encryption {
            let key = limits
                .encryption_key
                .as_ref()
                .and_then(|k| <[u8; 32]>::try_from(k.as_slice()).ok())
                .unwrap_or_else(|| {
                    let key = storage::StorageEncryption::generate_key();
                    info!("Generated encryption key (store securely for persistence)");
                    key
                });
            storage.enable_encryption(&key);
        }

        let rate_limiter_ip =
            RateLimiter::new(Duration::from_secs(1), limits.webhook_rate_limit_per_ip);
        let rate_limiter_token =
            RateLimiter::new(Duration::from_secs(1), limits.webhook_rate_limit_per_token);
        let rate_limiter_global =
            RateLimiter::new(Duration::from_secs(1), limits.webhook_rate_limit_global);
        let invalid_token_tracker = InvalidTokenTracker::new(
            Duration::from_secs(60),
            limits.max_invalid_token_attempts,
            limits.invalid_token_block_duration,
        );

        Self {
            storage,
            connections: DashMap::new(),
            base_url,
            limits,
            connection_count: AtomicUsize::new(0),
            connections_per_ip: DashMap::new(),
            rate_limiter_ip,
            rate_limiter_token,
            rate_limiter_global,
            invalid_token_tracker,
        }
    }

    /// Check if a new connection can be accepted from the given IP
    pub fn can_accept_connection(&self, ip: &str) -> Result<(), limits::LimitError> {
        use std::sync::atomic::Ordering;

        // Check total connections
        let current = self.connection_count.load(Ordering::SeqCst);
        if current >= self.limits.max_total_connections {
            return Err(limits::LimitError::TooManyConnections);
        }

        // Check per-IP connections
        let ip_count = self.connections_per_ip.get(ip).map(|v| *v).unwrap_or(0);
        if ip_count >= self.limits.max_connections_per_ip {
            return Err(limits::LimitError::TooManyConnectionsPerIp {
                ip: ip.to_string(),
                max: self.limits.max_connections_per_ip,
            });
        }

        Ok(())
    }

    /// Register a new connection
    pub fn register_connection(&self, ip: &str) {
        use std::sync::atomic::Ordering;
        self.connection_count.fetch_add(1, Ordering::SeqCst);
        *self.connections_per_ip.entry(ip.to_string()).or_insert(0) += 1;
    }

    /// Unregister a connection
    pub fn unregister_connection(&self, ip: &str) {
        use std::sync::atomic::Ordering;
        self.connection_count.fetch_sub(1, Ordering::SeqCst);
        if let Some(mut count) = self.connections_per_ip.get_mut(ip) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

/// Create the application with all routes
pub fn create_app(state: Arc<AppState>) -> axum::Router {
    use axum::routing::{any, get};

    axum::Router::new()
        // Health check
        .route("/health", get(health_check))
        // WebSocket endpoint for CLI connections
        .route("/ws", get(api::websocket_handler))
        // Webhook receiver endpoints (with and without trailing slash)
        .route("/in/{token}", any(webhook::webhook_receiver))
        .route("/in/{token}/", any(webhook::webhook_receiver))
        .route(
            "/in/{token}/{*path}",
            any(webhook::webhook_receiver_with_path),
        )
        // Inspection API
        .route("/api/tokens/{token}", get(api::get_session))
        .route(
            "/api/tokens/{token}/webhooks",
            get(api::get_webhooks).delete(api::delete_all_webhooks),
        )
        .route(
            "/api/tokens/{token}/webhooks/{webhook_id}",
            get(api::get_webhook).delete(api::delete_webhook),
        )
        // View URL (redirects to inspection or shows basic info)
        .route("/view/{token}", get(view_token))
        .with_state(state)
}

/// Start background cleanup tasks
pub fn start_background_tasks(state: Arc<AppState>) {
    let state_clone = state.clone();

    // Webhook TTL cleanup (runs every 5 minutes)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            let removed = state_clone
                .storage
                .cleanup_expired(state_clone.limits.webhook_ttl);
            if removed > 0 {
                info!("TTL cleanup: removed {} expired webhooks", removed);
            }
        }
    });

    let state_clone = state.clone();

    // Session timeout check (runs every minute)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            // Check session timeouts
            let timed_out = state_clone
                .storage
                .find_timed_out_sessions(state_clone.limits.session_timeout);
            for token in &timed_out {
                audit::log_audit(
                    audit::AuditEvent::SessionTimedOut,
                    Some(token),
                    None,
                    "background_check",
                );
                state_clone.connections.remove(token);
                state_clone.storage.set_disconnected(token);
                info!("Session timeout: disconnected {}", token);
            }

            // Check idle timeouts
            let idle = state_clone
                .storage
                .find_idle_sessions(state_clone.limits.idle_timeout);
            for token in &idle {
                audit::log_audit(
                    audit::AuditEvent::SessionIdleTimeout,
                    Some(token),
                    None,
                    "background_check",
                );
                state_clone.connections.remove(token);
                state_clone.storage.set_disconnected(token);
                info!("Idle timeout: disconnected {}", token);
            }
        }
    });

    let state_clone = state.clone();

    // Rate limiter cleanup (runs every 5 minutes)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            state_clone.rate_limiter_ip.cleanup();
            state_clone.rate_limiter_token.cleanup();
            state_clone.rate_limiter_global.cleanup();
            state_clone.invalid_token_tracker.cleanup();
        }
    });
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// View token - returns basic token info and webhook list URL
async fn view_token(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> impl axum::response::IntoResponse {
    use axum::http::StatusCode;
    use axum::Json;

    if !relay::is_valid_token(&token) {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "invalid_token",
                "message": "Token not found or invalid format"
            })),
        );
    }

    let session = state.storage.get_or_create_session(&token);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "session": session,
            "webhook_url": format!("{}/in/{}/", state.base_url, token),
            "api_url": format!("{}/api/tokens/{}/webhooks", state.base_url, token)
        })),
    )
}
