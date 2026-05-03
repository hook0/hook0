//! Hook0 acquisition service: server-side Google Ads conversion uploader.
//!
//! Receives signup events from the Hook0 API (or any service authorized via
//! a shared bearer token) and forwards them to Google Ads as Enhanced
//! Conversions for Leads — no client-side trackers, no cookies, no consent
//! banner needed on app.hook0.com.

pub mod google_ads;

use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::google_ads::{GoogleAdsClient, GoogleAdsError};

/// Shared application state injected into every handler.
pub struct AppState {
    pub google_ads: GoogleAdsClient,
    pub auth_token: String,
}

impl AppState {
    pub fn new(google_ads: GoogleAdsClient, auth_token: String) -> Self {
        Self {
            google_ads,
            auth_token,
        }
    }
}

/// Build the axum router used by both the production server and tests.
pub fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/conversion/signup", post(post_signup_conversion))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "hook0-acquisition",
    })
}

#[derive(Debug, Deserialize)]
pub struct SignupConversionRequest {
    pub gclid: String,
    pub email: String,
    /// Optional ISO-8601 timestamp; defaults to "now" if absent.
    pub conversion_date_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct SignupConversionResponse {
    pub status: &'static str,
    pub partial_failure: Option<serde_json::Value>,
}

async fn post_signup_conversion(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<SignupConversionRequest>,
) -> Response {
    if let Some(resp) = check_auth(&headers, &state.auth_token) {
        return resp;
    }

    if payload.gclid.trim().is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "missing_gclid",
            "gclid is required",
        );
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_email",
            "email is required and must contain @",
        );
    }

    let conversion_dt = payload.conversion_date_time.unwrap_or_else(Utc::now);

    info!(target: "acquisition", gclid_prefix = %&payload.gclid.chars().take(8).collect::<String>(), "uploading signup conversion");

    match state
        .google_ads
        .upload_click_conversion(&payload.gclid, &payload.email, conversion_dt)
        .await
    {
        Ok(resp) => {
            if resp.partial_failure_error.is_some() {
                warn!(target: "acquisition", "Google Ads partial failure: {:?}", resp.partial_failure_error);
            }
            Json(SignupConversionResponse {
                status: "uploaded",
                partial_failure: resp.partial_failure_error,
            })
            .into_response()
        }
        Err(e) => {
            error!(target: "acquisition", error = %e, "Google Ads upload failed");
            match e {
                GoogleAdsError::OAuth { .. } => error_response(
                    StatusCode::BAD_GATEWAY,
                    "oauth_failed",
                    "Google OAuth refresh failed",
                ),
                GoogleAdsError::Api { status: 400, .. } => error_response(
                    StatusCode::BAD_REQUEST,
                    "ads_api_rejected",
                    "Google Ads API rejected the conversion",
                ),
                _ => error_response(
                    StatusCode::BAD_GATEWAY,
                    "ads_api_error",
                    "Google Ads API call failed",
                ),
            }
        }
    }
}

type Response = axum::response::Response;

/// Returns `None` when the request is authorized, otherwise the response to
/// send back to the client. Returning `Option<Response>` avoids the
/// `Result<(), Response>` shape that triggers `clippy::result_large_err`.
fn check_auth(headers: &HeaderMap, expected: &str) -> Option<Response> {
    let auth = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(v) => v,
        None => {
            return Some(error_response(
                StatusCode::UNAUTHORIZED,
                "missing_auth",
                "Authorization header is required",
            ));
        }
    };
    let token = match auth.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            return Some(error_response(
                StatusCode::UNAUTHORIZED,
                "invalid_auth_scheme",
                "Authorization must use Bearer scheme",
            ));
        }
    };
    if !constant_time_eq(token.as_bytes(), expected.as_bytes()) {
        return Some(error_response(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Authorization token is invalid",
        ));
    }
    None
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

fn error_response(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (status, Json(ErrorBody { code, message })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }
}
