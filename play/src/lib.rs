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
use storage::{StorageBackend, WebhookStorageBackend};

/// Application state shared across all handlers
pub struct AppState {
    /// Webhook storage (in-memory or Redis, selected at startup)
    pub storage: StorageBackend,
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

    /// Create a new application state with custom limits and in-memory storage
    pub fn with_limits(base_url: String, limits: ServerLimits) -> Self {
        let storage = StorageBackend::InMemory(storage::InMemoryStorage::with_limits(
            limits.max_webhooks_per_token,
        ));
        Self::with_storage(base_url, limits, storage)
    }

    /// Create a new application state with an explicit storage backend
    pub fn with_storage(
        base_url: String,
        limits: ServerLimits,
        mut storage: StorageBackend,
    ) -> Self {
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

/// Build the Content-Security-Policy connect-src directive dynamically from the base URL.
///
/// In production (https), allows `wss://` connections to the same host.
/// In development (http), allows `ws://localhost:*` connections.
/// Always includes `'self'` for same-origin API calls.
fn build_csp(base_url: &str) -> String {
    let connect_src = url::Url::parse(base_url)
        .map(|parsed| {
            let scheme = parsed.scheme();
            let host = parsed.host_str().unwrap_or("localhost");
            let port = parsed.port();
            if scheme == "https" {
                match port {
                    Some(p) => format!("'self' wss://{}:{} ws://localhost:*", host, p),
                    None => format!("'self' wss://{} ws://localhost:*", host),
                }
            } else {
                match port {
                    Some(p) => format!("'self' ws://{}:{} ws://localhost:*", host, p),
                    None => "'self' ws://localhost:*".to_string(),
                }
            }
        })
        .unwrap_or_else(|_| "'self' ws://localhost:*".to_string());

    format!(
        "default-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src {}; img-src 'self' data:; font-src 'none'",
        connect_src,
    )
}

/// Read the web UI HTML content.
///
/// In release mode, the HTML is embedded in the binary via `include_str!`.
/// In debug mode, the file is read from disk so changes are picked up without recompilation.
fn read_index_html() -> Result<String, std::io::Error> {
    #[cfg(not(debug_assertions))]
    {
        static INDEX_HTML: &str = include_str!("../static/index.html");
        Ok(INDEX_HTML.to_string())
    }
    #[cfg(debug_assertions)]
    {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("static/index.html");
        std::fs::read_to_string(path)
    }
}

/// Serve the web UI HTML page at `GET /`.
///
/// Content negotiation: if the `Accept` header contains `application/json` but not `text/html`,
/// returns a JSON response with basic API info. Otherwise returns the full HTML page with
/// security headers (CSP, X-Content-Type-Options, X-Frame-Options, Referrer-Policy).
async fn serve_index(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Content negotiation: JSON-only clients get JSON
    if accept.contains("application/json") && !accept.contains("text/html") {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
            serde_json::json!({
                "name": "Hook0 Play",
                "description": "Free webhook tester — receive and inspect HTTP requests in real-time",
                "docs": format!("{}/api/tokens/{{token}}/webhooks", state.base_url),
                "ws": format!("{}/ws", state.base_url.replace("http://", "ws://").replace("https://", "wss://"))
            })
            .to_string(),
        )
            .into_response();
    }

    // Serve the HTML page
    let html = match read_index_html() {
        Ok(content) => content,
        Err(err) => {
            tracing::error!("Failed to read index.html: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                "Web UI not available".to_string(),
            )
                .into_response();
        }
    };

    let csp = build_csp(&state.base_url);

    let mut response = (StatusCode::OK, html).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/html; charset=utf-8"),
    );
    if let Ok(csp_value) = header::HeaderValue::from_str(&csp) {
        headers.insert("Content-Security-Policy", csp_value);
    }
    headers.insert(
        "X-Content-Type-Options",
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", header::HeaderValue::from_static("DENY"));
    headers.insert(
        "Referrer-Policy",
        header::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    response
}

/// Create the application with all routes
pub fn create_app(state: Arc<AppState>) -> axum::Router {
    use axum::routing::{any, get};

    axum::Router::new()
        // Web UI
        .route("/", get(serve_index))
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
                .cleanup_expired(state_clone.limits.webhook_ttl)
                .await;
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
                .find_timed_out_sessions(state_clone.limits.session_timeout)
                .await;
            for token in &timed_out {
                audit::log_audit(
                    audit::AuditEvent::SessionTimedOut,
                    Some(token),
                    None,
                    "background_check",
                );
                state_clone.connections.remove(token);
                state_clone.storage.set_disconnected(token).await;
                info!("Session timeout: disconnected {}", token);
            }

            // Check idle timeouts
            let idle = state_clone
                .storage
                .find_idle_sessions(state_clone.limits.idle_timeout)
                .await;
            for token in &idle {
                audit::log_audit(
                    audit::AuditEvent::SessionIdleTimeout,
                    Some(token),
                    None,
                    "background_check",
                );
                state_clone.connections.remove(token);
                state_clone.storage.set_disconnected(token).await;
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

    let session = state.storage.get_or_create_session(&token).await;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_csp_https_base_url_produces_wss() {
        let csp = build_csp("https://play.hook0.com");
        assert!(
            csp.contains("wss://play.hook0.com"),
            "CSP should contain wss:// for HTTPS base URL, got: {}",
            csp
        );
        assert!(csp.contains("'self'"));
        assert!(csp.contains("ws://localhost:*"));
    }

    #[test]
    fn test_build_csp_http_base_url_with_port_produces_ws_with_port() {
        let csp = build_csp("http://localhost:3000");
        assert!(
            csp.contains("ws://localhost:3000"),
            "CSP should contain ws://localhost:3000, got: {}",
            csp
        );
        assert!(csp.contains("'self'"));
    }

    #[test]
    fn test_build_csp_https_base_url_with_port_produces_wss_with_port() {
        let csp = build_csp("https://play.hook0.com:8443");
        assert!(
            csp.contains("wss://play.hook0.com:8443"),
            "CSP should contain wss://play.hook0.com:8443, got: {}",
            csp
        );
        assert!(csp.contains("ws://localhost:*"));
    }

    #[test]
    fn test_build_csp_invalid_url_falls_back_to_self() {
        let csp = build_csp("not a valid url");
        assert!(
            csp.contains("'self' ws://localhost:*"),
            "CSP should fall back to 'self' ws://localhost:* for invalid URL, got: {}",
            csp
        );
    }

    #[test]
    fn test_build_csp_http_no_port_falls_back_to_localhost() {
        let csp = build_csp("http://example.com");
        assert!(
            csp.contains("'self' ws://localhost:*"),
            "CSP for http://example.com (no port) should contain 'self' ws://localhost:*, got: {}",
            csp
        );
    }

    #[test]
    fn test_build_csp_contains_all_directives() {
        let csp = build_csp("https://play.hook0.com");
        assert!(csp.contains("default-src 'none'"));
        assert!(csp.contains("script-src 'self' 'unsafe-inline'"));
        assert!(csp.contains("style-src 'self' 'unsafe-inline'"));
        assert!(csp.contains("img-src 'self' data:"));
        assert!(csp.contains("font-src 'none'"));
    }
}
