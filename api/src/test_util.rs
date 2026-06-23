//! Test-only helpers shared across the API test suite.
//!
//! Nothing here is a mock of our own code: the fake Google Ads endpoint is a
//! real in-process socket server, and every seed helper writes to a real
//! Postgres (the test DB provisioned per `#[sqlx::test]`). The point is to
//! exercise the genuine code paths end to end while substituting only the
//! external Google Ads HTTP boundary, which we cannot (and must not) call from
//! the test suite.
#![cfg(test)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lettre::Address;
use sqlx::PgPool;
use url::Url;
use uuid::Uuid;

/// One captured HTTP request that hit the fake Google Ads endpoint.
#[derive(Clone, Debug)]
pub(crate) struct CapturedRequest {
    pub path: String,
    pub body: String,
}

/// A minimal in-process HTTP server standing in for the Google Ads REST API.
///
/// It is a real socket (no mocking library): it records every request it
/// receives and replies with a canned status + body, so tests can assert on
/// the exact outbound request our client builds. Dropping it stops the server.
pub(crate) struct FakeGoogleAds {
    pub base_url: String,
    requests: Arc<Mutex<Vec<CapturedRequest>>>,
    stop: Arc<AtomicBool>,
}

impl FakeGoogleAds {
    /// Start a fake server that answers every request with `status` and
    /// `response_body`.
    pub fn start(status: u16, response_body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake google ads server");
        let addr = listener.local_addr().expect("fake google ads local addr");
        listener
            .set_nonblocking(true)
            .expect("fake google ads non-blocking");

        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let requests_thread = Arc::clone(&requests);
        let stop_thread = Arc::clone(&stop);

        std::thread::spawn(move || {
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // The accepted socket may inherit the listener's
                        // non-blocking mode; force blocking so the request is
                        // read in full before we reply.
                        let _ = stream.set_nonblocking(false);
                        if let Some((path, body)) = read_http_request(&mut stream) {
                            requests_thread
                                .lock()
                                .expect("requests lock")
                                .push(CapturedRequest { path, body });
                        }
                        let response = format!(
                            "HTTP/1.1 {status} STATUS\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            response_body.len(),
                            response_body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if stop_thread.load(Ordering::Relaxed) {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            requests,
            stop,
        }
    }

    /// Snapshot of the requests captured so far.
    pub fn requests(&self) -> Vec<CapturedRequest> {
        self.requests.lock().expect("requests lock").clone()
    }

    /// Wait until at least `n` requests have been captured, or `timeout`
    /// elapses, then return the captured requests. Used for the detached
    /// fire-and-forget upload triggered by the handler.
    pub async fn wait_for(&self, n: usize, timeout: Duration) -> Vec<CapturedRequest> {
        let steps = (timeout.as_millis() / 50).max(1);
        for _ in 0..steps {
            if self.requests.lock().expect("requests lock").len() >= n {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        self.requests()
    }
}

impl Drop for FakeGoogleAds {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Read one HTTP/1.1 request (request line + headers + Content-Length body)
/// from a blocking stream. Returns `(path, body)`.
fn read_http_request(stream: &mut TcpStream) -> Option<(String, String)> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut header_end: Option<usize> = None;
    let mut content_length: usize = 0;

    loop {
        if let Some(he) = header_end
            && buf.len() >= he + content_length
        {
            break;
        }
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(_) => break,
        }
        if header_end.is_none()
            && let Some(pos) = find_subslice(&buf, b"\r\n\r\n")
        {
            header_end = Some(pos + 4);
            let headers = String::from_utf8_lossy(&buf[..pos]);
            for line in headers.split("\r\n") {
                let lower = line.to_ascii_lowercase();
                if let Some(rest) = lower.strip_prefix("content-length:") {
                    content_length = rest.trim().parse().unwrap_or(0);
                }
            }
        }
    }

    let he = header_end?;
    let request_line = String::from_utf8_lossy(&buf[..he.saturating_sub(4)]);
    let path = request_line
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or_default()
        .to_string();
    let end = (he + content_length).min(buf.len());
    let body = String::from_utf8_lossy(&buf[he..end]).to_string();
    Some((path, body))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ---------------------------------------------------------------------------
// DB seeding (real Postgres, runtime-checked queries so they compile offline)
// ---------------------------------------------------------------------------

/// Insert a verified user and return its id.
pub(crate) async fn seed_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
            INSERT INTO iam."user" (user__id, email, password, first_name, last_name, email_verified_at)
            VALUES ($1, $2, 'unused-hash', 'Test', 'User', statement_timestamp())
        "#,
    )
    .bind(user_id)
    .bind(format!("e2e-{user_id}@example.com"))
    .execute(pool)
    .await
    .expect("seed user");
    user_id
}

/// Insert an organization owned by `created_by` and return its id.
pub(crate) async fn seed_org(pool: &PgPool, created_by: Uuid) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO iam.organization (organization__id, name, created_by) VALUES ($1, 'E2E org', $2)",
    )
    .bind(org_id)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("seed org");
    org_id
}

/// Grant `user` the given role on `org`.
pub(crate) async fn seed_membership(pool: &PgPool, user: Uuid, org: Uuid, role: &str) {
    sqlx::query(
        "INSERT INTO iam.user__organization (user__id, organization__id, role) VALUES ($1, $2, $3)",
    )
    .bind(user)
    .bind(org)
    .bind(role)
    .execute(pool)
    .await
    .expect("seed membership");
}

/// Insert a signup attribution row. When `signup_uploaded` is true the
/// `signup_uploaded_at` timestamp is set (simulating a verified signup whose
/// signup conversion has already been uploaded).
pub(crate) async fn seed_attribution(
    pool: &PgPool,
    user: Uuid,
    org: Uuid,
    gclid: &str,
    signup_uploaded: bool,
) {
    sqlx::query(
        r#"
            INSERT INTO iam.signup_attribution (user__id, organization__id, gclid, signup_uploaded_at)
            VALUES ($1, $2, $3, CASE WHEN $4 THEN statement_timestamp() ELSE NULL END)
        "#,
    )
    .bind(user)
    .bind(org)
    .bind(gclid)
    .bind(signup_uploaded)
    .execute(pool)
    .await
    .expect("seed attribution");
}

/// Current `(gclid, activation_uploaded_at IS NOT NULL)` for an org's
/// attribution row.
pub(crate) async fn attribution_state(pool: &PgPool, org: Uuid) -> (Option<String>, bool) {
    let row: (Option<String>, bool) = sqlx::query_as(
        "SELECT gclid, (activation_uploaded_at IS NOT NULL) FROM iam.signup_attribution WHERE organization__id = $1",
    )
    .bind(org)
    .fetch_one(pool)
    .await
    .expect("read attribution state");
    row
}

/// Mint a user access token for `user` (carrying `role` on `org`) AND persist
/// it in `iam.token`, exactly like `do_login` does, so the auth middleware
/// (which verifies the token's revocation id exists and is unexpired) accepts
/// it. Returns the serialized biscuit for the `Authorization: Bearer` header.
pub(crate) async fn issue_user_token(
    pool: &PgPool,
    private_key: &biscuit_auth::PrivateKey,
    user: Uuid,
    org: Uuid,
    role: &str,
) -> String {
    let token_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let token = crate::iam::create_user_access_token(
        private_key,
        token_id,
        session_id,
        user,
        "e2e@example.com",
        "E2E",
        "User",
        vec![(org, role.to_string())],
    )
    .expect("mint user access token");

    sqlx::query(
        r#"
            INSERT INTO iam.token (token__id, type, revocation_id, expired_at, user__id, session_id)
            VALUES ($1, 'user_access', $2, $3, $4, $5)
        "#,
    )
    .bind(token_id)
    .bind(&token.revocation_id)
    .bind(token.expired_at)
    .bind(user)
    .bind(session_id)
    .execute(pool)
    .await
    .expect("persist token");

    token.serialized_biscuit
}

// ---------------------------------------------------------------------------
// Test State builder
// ---------------------------------------------------------------------------

/// Build a `State` suitable for handler tests: real DB pool + real Google Ads
/// client (pointed at the fake server by the caller), everything else inert
/// (no Pulsar, no object storage, no Hook0 self-eventing, quotas disabled, a
/// dummy SMTP transport that is never used by the tested paths).
pub(crate) async fn test_state(
    pool: PgPool,
    biscuit_private_key: biscuit_auth::PrivateKey,
    google_ads: Option<Arc<crate::google_ads::GoogleAdsClient>>,
) -> crate::State {
    let url = Url::parse("http://localhost").expect("localhost url");
    let support = Address::from_str("support@hook0.com").expect("support address");

    let smtp = crate::mailer::MailerSmtpConfig {
        smtp_connection_url: "smtp://127.0.0.1:2".to_string(),
        smtp_timeout: Duration::from_millis(200),
        sender_name: "Hook0 Test".to_string(),
        sender_address: Address::from_str("noreply@hook0.com").expect("sender address"),
    };
    let mailer = crate::mailer::Mailer::new(
        smtp,
        url.clone(),
        url.clone(),
        url.clone(),
        url.clone(),
        url.clone(),
        support.clone(),
        "Hook0 Test".to_string(),
        "test".to_string(),
        "test".to_string(),
    )
    .await
    .expect("build test mailer");

    let quota_limits = crate::quotas::QuotaLimits {
        global_members_per_organization_limit: i32::MAX,
        global_applications_per_organization_limit: i32::MAX,
        global_events_per_day_limit: i32::MAX,
        global_days_of_events_retention_limit: i32::MAX,
        global_subscriptions_per_application_limit: i32::MAX,
        global_event_types_per_application_limit: i32::MAX,
    };

    crate::State {
        db: pool,
        pulsar: None,
        object_storage: None,
        biscuit_private_key,
        mailer,
        app_url: url.clone(),
        #[cfg(feature = "migrate-users-from-keycloak")]
        enable_keycloak_migration: false,
        #[cfg(feature = "migrate-users-from-keycloak")]
        keycloak_url: url.clone(),
        #[cfg(feature = "migrate-users-from-keycloak")]
        keycloak_realm: "test".to_string(),
        #[cfg(feature = "migrate-users-from-keycloak")]
        keycloak_client_id: "test".to_string(),
        #[cfg(feature = "migrate-users-from-keycloak")]
        keycloak_client_secret: "test".to_string(),
        application_secret_compatibility: true,
        registration_disabled: false,
        password_minimum_length: 12,
        auto_db_migration: false,
        hook0_client: None,
        quotas: crate::quotas::Quotas::new(false, quota_limits),
        health_check_key: None,
        health_check_timeout: Duration::from_secs(5),
        max_authorization_time: Duration::from_secs(10),
        debug_authorizer: false,
        enable_quota_enforcement: false,
        matomo_url: None,
        matomo_site_id: None,
        formbricks_api_host: "https://app.formbricks.com".to_string(),
        formbricks_environment_id: None,
        quota_notification_events_per_day_threshold: 80,
        enable_quota_based_email_notifications: false,
        support_email_address: support,
        cloudflare_turnstile_site_key: None,
        cloudflare_turnstile_secret_key: None,
        google_ads,
    }
}
