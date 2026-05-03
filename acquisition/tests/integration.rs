//! Black-box integration tests for hook0-acquisition.
//!
//! Spins up two real HTTP servers:
//! - the acquisition service itself, bound to an ephemeral port
//! - a mock Google Ads + OAuth endpoint, used as the upstream
//!
//! The acquisition client is configured with `with_endpoints` to talk to
//! the mock instead of the real Google API. No mocking framework is used.

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use hook0_acquisition::{
    create_app,
    google_ads::{GoogleAdsClient, GoogleAdsConfig},
    AppState,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const API_TOKEN: &str = "test-shared-secret-1234567890";

#[derive(Default)]
struct UpstreamLog {
    last_oauth_call: Option<()>,
    last_conversion_body: Option<Value>,
    next_conversion_response: Option<(StatusCode, Value)>,
}

type SharedLog = Arc<Mutex<UpstreamLog>>;

async fn mock_oauth(State(log): State<SharedLog>) -> impl IntoResponse {
    log.lock().await.last_oauth_call = Some(());
    Json(json!({
        "access_token": "mock-access-token-abc",
        "expires_in": 3600,
        "token_type": "Bearer"
    }))
}

async fn mock_upload(State(log): State<SharedLog>, Json(body): Json<Value>) -> impl IntoResponse {
    let mut guard = log.lock().await;
    guard.last_conversion_body = Some(body);
    if let Some((status, payload)) = guard.next_conversion_response.take() {
        return (status, Json(payload)).into_response();
    }
    Json(json!({
        "results": [{
            "gclid": "mock-gclid",
            "conversionAction": "customers/1234567890/conversionActions/42",
            "conversionDateTime": "2026-05-03 09:00:00+00:00"
        }]
    }))
    .into_response()
}

async fn spawn_mock_upstream() -> (String, SharedLog) {
    let log: SharedLog = Arc::new(Mutex::new(UpstreamLog::default()));
    // Customer ID is fixed (1234567890) in the GoogleAdsConfig used by tests,
    // so we hard-code the path here rather than fight axum's "one parameter per
    // segment" rule (the `{id}:operation` shape is not a valid axum route).
    let app = Router::new()
        .route("/token", post(mock_oauth))
        .route(
            "/v23/customers/1234567890:uploadClickConversions",
            post(mock_upload),
        )
        .with_state(log.clone());

    let port = portpicker::pick_unused_port().expect("no free port");
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind mock");
    let url = format!("http://{addr}");
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    (url, log)
}

async fn spawn_acquisition(upstream_url: &str) -> String {
    let cfg = GoogleAdsConfig {
        developer_token: "mock-developer-token".into(),
        customer_id: "1234567890".into(),
        login_customer_id: None,
        conversion_action_id: "42".into(),
        oauth_client_id: "mock-client-id".into(),
        oauth_client_secret: "mock-client-secret".into(),
        oauth_refresh_token: "mock-refresh-token".into(),
    };
    let client = GoogleAdsClient::new(cfg).expect("client").with_endpoints(
        format!("{upstream_url}/v23"),
        format!("{upstream_url}/token"),
    );

    let state = Arc::new(AppState::new(client, API_TOKEN.to_string()));
    let app = create_app(state);

    let port = portpicker::pick_unused_port().expect("no free port");
    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind acquisition");
    let url = format!("http://{addr}");
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;
    url
}

#[tokio::test]
async fn health_returns_ok() {
    let (upstream, _log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;
    let resp = reqwest::get(format!("{url}/health")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "hook0-acquisition");
}

#[tokio::test]
async fn rejects_request_without_authorization_header() {
    let (upstream, _log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;
    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .json(&json!({"gclid": "x", "email": "a@b.c"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn rejects_request_with_wrong_token() {
    let (upstream, _log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;
    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth("wrong-token")
        .json(&json!({"gclid": "x", "email": "a@b.c"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn rejects_invalid_email() {
    let (upstream, _log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;
    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth(API_TOKEN)
        .json(&json!({"gclid": "x", "email": "not-an-email"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn rejects_empty_gclid() {
    let (upstream, _log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;
    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth(API_TOKEN)
        .json(&json!({"gclid": "", "email": "user@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn happy_path_uploads_hashed_email_and_gclid() {
    let (upstream, log) = spawn_mock_upstream().await;
    let url = spawn_acquisition(&upstream).await;

    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth(API_TOKEN)
        .json(&json!({
            "gclid": "real-gclid-abc123",
            "email": "  USER@example.com  "
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "unexpected status");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "uploaded");

    // Verify the upstream was called with the right shape
    let log = log.lock().await;
    assert!(log.last_oauth_call.is_some(), "oauth was not called");
    let upload = log
        .last_conversion_body
        .as_ref()
        .expect("upload body missing");
    let conversion = &upload["conversions"][0];
    assert_eq!(conversion["gclid"], "real-gclid-abc123");
    assert_eq!(
        conversion["conversionAction"],
        "customers/1234567890/conversionActions/42"
    );
    let hashed = conversion["userIdentifiers"][0]["hashedEmail"]
        .as_str()
        .expect("hashedEmail missing");
    // sha256("user@example.com") — case+whitespace normalized
    assert_eq!(
        hashed,
        "b4c9a289323b21a01c3e940f150eb9b8c542587f1abfd8f0e1cc1ffc5e475514"
    );
    assert!(upload["partialFailure"].as_bool().unwrap_or(false));
}

#[tokio::test]
async fn surfaces_partial_failure_from_upstream() {
    let (upstream, log) = spawn_mock_upstream().await;
    {
        let mut guard = log.lock().await;
        guard.next_conversion_response = Some((
            StatusCode::OK,
            json!({
                "partialFailureError": {
                    "code": 3,
                    "message": "Invalid gclid"
                },
                "results": []
            }),
        ));
    }
    let url = spawn_acquisition(&upstream).await;

    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth(API_TOKEN)
        .json(&json!({"gclid": "bad-gclid", "email": "user@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "uploaded");
    assert_eq!(body["partial_failure"]["code"], 3);
}

#[tokio::test]
async fn surfaces_upstream_500_as_bad_gateway() {
    let (upstream, log) = spawn_mock_upstream().await;
    {
        let mut guard = log.lock().await;
        guard.next_conversion_response = Some((
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"error": {"code": 500, "message": "boom"}}),
        ));
    }
    let url = spawn_acquisition(&upstream).await;

    let resp = reqwest::Client::new()
        .post(format!("{url}/conversion/signup"))
        .bearer_auth(API_TOKEN)
        .json(&json!({"gclid": "g", "email": "user@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 502);
}
