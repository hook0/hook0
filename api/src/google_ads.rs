//! Server-side Google Ads conversion uploader.
//!
//! Uploads click conversions (`uploadClickConversions`) using only the
//! `gclid` — no user identifiers, no email hash. RGPD posture: the gclid
//! is a pseudonymous identifier already issued by Google during the ad
//! click; sending it back does not transmit any first-party PII from
//! Hook0 users to a third party.
//!
//! The handler is fire-and-forget: failures are logged but never block
//! the user-facing signup response.
//!
//! Reference:
//! - <https://developers.google.com/google-ads/api/docs/conversions/upload-clicks>
//!
//! This module also owns the `iam.signup_attribution` gclid lifecycle shared by
//! the registration / email-verification flow (signup conversion) and the
//! application-secret creation flow (activation conversion): the gclid is
//! retained until BOTH conversions have been uploaded, then cleared (data
//! minimisation). The 30-day cleanup in `handlers::registrations` is the safety
//! net for rows that never reach that state. See
//! `documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md`.

use chrono::{DateTime, Utc};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const GOOGLE_OAUTH_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_ADS_BASE_URL: &str = "https://googleads.googleapis.com/v23";
const ACCESS_TOKEN_LIFETIME_BUFFER: Duration = Duration::from_secs(60);

#[derive(Debug, Error)]
pub enum GoogleAdsError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OAuth refresh failed (HTTP {status}): {body}")]
    OAuth { status: u16, body: String },
    #[error("Google Ads API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },
    #[error("invalid header value: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
}

/// Configuration required to talk to the Google Ads API.
///
/// `Debug` is intentionally NOT derived: this struct holds the OAuth client
/// secret and refresh token; printing it would leak credentials into logs.
#[derive(Clone)]
pub struct GoogleAdsConfig {
    pub developer_token: String,
    pub customer_id: String,
    pub login_customer_id: Option<String>,
    /// Numeric ID of the "signup" conversion action (required).
    pub signup_conversion_action_id: String,
    /// Numeric ID of the "activation" conversion action (optional). When
    /// `None`, activation uploads are skipped and only signup is tracked.
    pub activation_conversion_action_id: Option<String>,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_refresh_token: String,
}

/// Which conversion action a click conversion targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionKind {
    /// User verified their email after signing up.
    Signup,
    /// Organization created its first API key (first real product use).
    Activation,
}

impl GoogleAdsConfig {
    fn normalized_customer_id(&self) -> String {
        self.customer_id.replace('-', "")
    }

    fn normalized_login_customer_id(&self) -> Option<String> {
        self.login_customer_id
            .as_ref()
            .map(|id| id.replace('-', ""))
    }

    /// Build the `customers/{cid}/conversionActions/{id}` resource for a given
    /// conversion kind. Returns `None` for `Activation` when no activation
    /// conversion action is configured.
    fn conversion_action_resource(&self, kind: ConversionKind) -> Option<String> {
        let conversion_action_id = match kind {
            ConversionKind::Signup => self.signup_conversion_action_id.clone(),
            ConversionKind::Activation => self.activation_conversion_action_id.clone()?,
        };
        Some(format!(
            "customers/{}/conversionActions/{}",
            self.normalized_customer_id(),
            conversion_action_id
        ))
    }
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug)]
struct CachedToken {
    value: String,
    fetched_at: Instant,
    lifetime: Duration,
}

impl CachedToken {
    fn is_fresh(&self) -> bool {
        match self.lifetime.checked_sub(ACCESS_TOKEN_LIFETIME_BUFFER) {
            Some(safe_lifetime) => self.fetched_at.elapsed() < safe_lifetime,
            None => false,
        }
    }
}

/// Google Ads HTTP client with cached OAuth access token.
pub struct GoogleAdsClient {
    http: reqwest::Client,
    config: GoogleAdsConfig,
    cached_token: Mutex<Option<CachedToken>>,
    base_url: String,
    oauth_url: String,
}

impl std::fmt::Debug for GoogleAdsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleAdsClient")
            .field("customer_id", &self.config.customer_id)
            .field(
                "signup_conversion_action_id",
                &self.config.signup_conversion_action_id,
            )
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

impl GoogleAdsClient {
    pub fn new(config: GoogleAdsConfig) -> Result<Arc<Self>, GoogleAdsError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Arc::new(Self {
            http,
            config,
            cached_token: Mutex::new(None),
            base_url: GOOGLE_ADS_BASE_URL.to_string(),
            oauth_url: GOOGLE_OAUTH_TOKEN_URL.to_string(),
        }))
    }

    async fn get_access_token(&self) -> Result<String, GoogleAdsError> {
        let mut guard = self.cached_token.lock().await;
        if let Some(cached) = guard.as_ref()
            && cached.is_fresh()
        {
            debug!("Using cached Google OAuth access token");
            return Ok(cached.value.clone());
        }

        info!("Refreshing Google OAuth access token");
        let resp = self
            .http
            .post(&self.oauth_url)
            .form(&[
                ("client_id", self.config.oauth_client_id.as_str()),
                ("client_secret", self.config.oauth_client_secret.as_str()),
                ("refresh_token", self.config.oauth_refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(GoogleAdsError::OAuth {
                status: status.as_u16(),
                body,
            });
        }
        let parsed: OAuthTokenResponse =
            serde_json::from_str(&body).map_err(|e| GoogleAdsError::OAuth {
                status: status.as_u16(),
                body: format!("invalid JSON: {e}: {body}"),
            })?;

        *guard = Some(CachedToken {
            value: parsed.access_token.clone(),
            fetched_at: Instant::now(),
            lifetime: Duration::from_secs(parsed.expires_in),
        });
        Ok(parsed.access_token)
    }

    fn build_headers(&self, access_token: &str) -> Result<HeaderMap, GoogleAdsError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {access_token}"))?,
        );
        headers.insert(
            "developer-token",
            HeaderValue::from_str(&self.config.developer_token)?,
        );
        if let Some(login_id) = self.config.normalized_login_customer_id() {
            headers.insert("login-customer-id", HeaderValue::from_str(&login_id)?);
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// Returns `true` if an activation conversion action is configured.
    pub fn has_activation_conversion(&self) -> bool {
        self.config.activation_conversion_action_id.is_some()
    }

    /// Upload a click conversion using only the gclid (no PII).
    ///
    /// For [`ConversionKind::Activation`] when no activation conversion action
    /// is configured, this is a silent no-op (returns `Ok(())`).
    pub async fn upload_click_conversion(
        &self,
        gclid: &str,
        kind: ConversionKind,
        conversion_date_time: DateTime<Utc>,
    ) -> Result<(), GoogleAdsError> {
        let Some(conversion_action) = self.config.conversion_action_resource(kind) else {
            debug!(
                target: "api::google_ads",
                ?kind,
                "conversion action not configured; skipping upload"
            );
            return Ok(());
        };

        let access_token = self.get_access_token().await?;
        let headers = self.build_headers(&access_token)?;

        let url = format!(
            "{}/customers/{}:uploadClickConversions",
            self.base_url,
            self.config.normalized_customer_id()
        );

        let formatted_dt = conversion_date_time
            .format("%Y-%m-%d %H:%M:%S%:z")
            .to_string();

        let body = serde_json::json!({
            "conversions": [{
                "gclid": gclid,
                "conversionAction": conversion_action,
                "conversionDateTime": formatted_dt,
            }],
            "partialFailure": true,
            "validateOnly": false
        });

        debug!(target: "api::google_ads", url = %url, ?kind, "uploading click conversion");
        let resp = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let status = resp.status();
        let response_body = resp.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(GoogleAdsError::Api {
                status: status.as_u16(),
                body: response_body,
            });
        }

        // partialFailure: true means HTTP 200 may still contain per-operation
        // errors. Surface them via warn but treat them as non-fatal — Google
        // already has the conversion or it was rejected for a non-retryable
        // reason (e.g. unknown gclid).
        if response_body.contains("partialFailureError") {
            warn!(target: "api::google_ads", body = %response_body, "Google Ads partial failure");
        }
        Ok(())
    }
}

/// Returns true if the error is worth retrying. 4xx (except 429) are treated
/// as permanent (bad request, unauthorized, forbidden) — retrying won't help.
/// 429 (rate limit), 5xx and transport errors are retryable.
fn is_retryable(err: &GoogleAdsError) -> bool {
    match err {
        GoogleAdsError::Api { status, .. } => *status >= 500 || *status == 429,
        GoogleAdsError::OAuth { status, .. } => *status >= 500 || *status == 429,
        GoogleAdsError::Http(_) => true,
        GoogleAdsError::Header(_) => false,
    }
}

/// Delays inserted between attempts. Total: 4 attempts, 3 inter-attempt
/// delays of 30s, 2min, 10min.
const RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(30),
    Duration::from_secs(120),
    Duration::from_secs(600),
];

/// Spawn a fire-and-forget task that uploads the conversion. Errors are
/// logged (and reported to Sentry on final failure) but never propagated.
/// Returns immediately. Retries up to 3 times with exponential backoff
/// (30s / 2min / 10min) on retryable errors (5xx, 429, network).
pub fn spawn_upload(client: Arc<GoogleAdsClient>, gclid: String, kind: ConversionKind) {
    tokio::spawn(async move {
        let started = Instant::now();
        let gclid_prefix: String = gclid.chars().take(8).collect();
        let max_attempts = RETRY_DELAYS.len() + 1;

        for attempt in 1..=max_attempts {
            match client
                .upload_click_conversion(&gclid, kind, Utc::now())
                .await
            {
                Ok(()) => {
                    debug!(
                        target: "api::google_ads",
                        gclid = %gclid,
                        attempt = attempt,
                        "click conversion uploaded (full gclid)"
                    );
                    info!(
                        target: "api::google_ads",
                        gclid_prefix = %gclid_prefix,
                        conversion = ?kind,
                        attempt = attempt,
                        duration_ms = started.elapsed().as_millis() as u64,
                        "click conversion uploaded"
                    );
                    return;
                }
                Err(e) => {
                    if !is_retryable(&e) {
                        // error! emits a Sentry event via sentry-tracing
                        // layer (configured by hook0-sentry-integration).
                        // Non-retryable errors usually indicate a config
                        // issue (4xx) that needs manual review.
                        error!(
                            target: "api::google_ads",
                            gclid_prefix = %gclid_prefix,
                            conversion = ?kind,
                            attempt = attempt,
                            error = %e,
                            "click conversion upload failed (non-retryable)"
                        );
                        debug!(
                            target: "api::google_ads",
                            gclid = %gclid,
                            error = %e,
                            "click conversion upload failed (full gclid)"
                        );
                        return;
                    }

                    if attempt == max_attempts {
                        // error! emits a Sentry event via sentry-tracing
                        // layer. A lost conversion after exhausted retries
                        // is operationally significant.
                        error!(
                            target: "api::google_ads",
                            gclid_prefix = %gclid_prefix,
                            conversion = ?kind,
                            attempts = attempt,
                            error = %e,
                            "click conversion upload abandoned after retries"
                        );
                        debug!(
                            target: "api::google_ads",
                            gclid = %gclid,
                            error = %e,
                            "click conversion upload abandoned (full gclid)"
                        );
                        return;
                    }

                    let delay = RETRY_DELAYS[attempt - 1];
                    warn!(
                        target: "api::google_ads",
                        gclid_prefix = %gclid_prefix,
                        attempt = attempt,
                        next_retry_in_ms = delay.as_millis() as u64,
                        error = %e,
                        "click conversion upload failed, will retry"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// gclid attribution lifecycle (table `iam.signup_attribution`)
// ---------------------------------------------------------------------------

/// Maximum gclid length accepted, mirroring the `signup_attribution_gclid_length`
/// DB CHECK. Real Google gclids are ~50-100 chars; anything longer is treated as
/// invalid and dropped — this bounds untrusted input and avoids failing the
/// INSERT on the length CHECK.
pub const MAX_GCLID_LEN: usize = 256;

/// Normalize a raw gclid from the registration payload: trim surrounding
/// whitespace, then drop it if empty or longer than [`MAX_GCLID_LEN`] characters.
/// Returns the value to store, or `None` when there is nothing valid to keep.
pub fn normalize_gclid(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty() && s.chars().count() <= MAX_GCLID_LEN)
        .map(str::to_string)
}

/// Atomically claim the activation conversion for an organization.
///
/// The `UPDATE ... RETURNING` makes this fire **at most once** per organization
/// even under concurrent first-API-key creations: only the statement that flips
/// `activation_uploaded_at` from NULL wins and returns the gclid. Returns `None`
/// when there is nothing to upload (no attribution row for the org, gclid
/// already cleared, or activation already claimed).
pub async fn claim_activation_gclid(
    db: &PgPool,
    organization_id: &Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET activation_uploaded_at = statement_timestamp()
            WHERE organization__id = $1
              AND activation_uploaded_at IS NULL
              AND gclid IS NOT NULL
            RETURNING gclid
        ",
        organization_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(row.and_then(|r| r.gclid))
}

/// Clear the gclid (data minimisation) once BOTH conversions are uploaded, for
/// the attribution row of `user_id`. Best-effort: errors are logged, never
/// surfaced (the conversion has already been queued).
pub async fn clear_gclid_if_fully_uploaded_by_user(db: &PgPool, user_id: &Uuid) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE user__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND activation_uploaded_at IS NOT NULL
        ",
        user_id,
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        warn!(
            target: "api::signup_attribution",
            error = %e,
            "failed to clear minimised gclid (by user)"
        );
    }
}

/// Same as [`clear_gclid_if_fully_uploaded_by_user`], keyed by organization.
pub async fn clear_gclid_if_fully_uploaded_by_org(db: &PgPool, organization_id: &Uuid) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE organization__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND activation_uploaded_at IS NOT NULL
        ",
        organization_id,
    )
    .execute(db)
    .await;

    if let Err(e) = result {
        warn!(
            target: "api::signup_attribution",
            error = %e,
            "failed to clear minimised gclid (by org)"
        );
    }
}

/// Test-only helpers shared by this module's tests and the handler integration
/// test in `main`. Nothing here mocks our own code: the fake Google Ads
/// endpoint is a real in-process socket server, and every seed helper writes to
/// a real Postgres (the test DB provisioned per `#[sqlx::test]`). Only the
/// external Google Ads HTTP boundary is substituted.
#[cfg(test)]
pub(crate) mod test_support {
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
    /// the exact outbound request our client builds. Dropping it stops it.
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
                            // non-blocking mode; force blocking so the request
                            // is read in full before we reply.
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

    // -----------------------------------------------------------------------
    // DB seeding (real Postgres, runtime-checked queries so they compile offline)
    // -----------------------------------------------------------------------

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
    /// (which verifies the token's revocation id exists and is unexpired)
    /// accepts it. Returns the serialized biscuit for the `Authorization` header.
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

    /// Build a `State` suitable for handler tests: real DB pool + real Google
    /// Ads client (pointed at the fake server by the caller), everything else
    /// inert (no Pulsar, no object storage, no Hook0 self-eventing, quotas
    /// disabled, a dummy SMTP transport never used by the tested paths).
    pub(crate) async fn test_state(
        pool: PgPool,
        biscuit_private_key: biscuit_auth::PrivateKey,
        google_ads: Option<Arc<super::GoogleAdsClient>>,
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
}

/// Build a client whose Google Ads base URL is overridden (to point at a local
/// fake server) and whose OAuth token is pre-seeded (so no token endpoint is
/// hit). Lives at module level so both the unit tests here and the handler
/// integration test can construct one despite the private fields.
#[cfg(test)]
pub(crate) fn test_client_with_base_url(
    base_url: String,
    activation_conversion_action_id: Option<&str>,
) -> Arc<GoogleAdsClient> {
    let config = GoogleAdsConfig {
        developer_token: "t".into(),
        customer_id: "123-456-7890".into(),
        login_customer_id: Some("987-654-3210".into()),
        signup_conversion_action_id: "42".into(),
        activation_conversion_action_id: activation_conversion_action_id.map(str::to_string),
        oauth_client_id: "c".into(),
        oauth_client_secret: "s".into(),
        oauth_refresh_token: "r".into(),
    };
    Arc::new(GoogleAdsClient {
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("test http client"),
        config,
        cached_token: Mutex::new(Some(CachedToken {
            value: "test-access-token".to_string(),
            fetched_at: Instant::now(),
            lifetime: Duration::from_secs(3600),
        })),
        base_url,
        oauth_url: "http://127.0.0.1:9/unused".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::test_support::{
        FakeGoogleAds, attribution_state, seed_attribution, seed_org, seed_user,
    };
    use super::*;
    use proptest::prelude::*;

    fn test_config(activation: Option<&str>) -> GoogleAdsConfig {
        GoogleAdsConfig {
            developer_token: "t".into(),
            customer_id: "123-456-7890".into(),
            login_customer_id: Some("987-654-3210".into()),
            signup_conversion_action_id: "42".into(),
            activation_conversion_action_id: activation.map(|s| s.to_string()),
            oauth_client_id: "c".into(),
            oauth_client_secret: "s".into(),
            oauth_refresh_token: "r".into(),
        }
    }

    #[test]
    fn customer_id_is_normalized() {
        let cfg = test_config(None);
        assert_eq!(cfg.normalized_customer_id(), "1234567890");
        assert_eq!(
            cfg.normalized_login_customer_id().as_deref(),
            Some("9876543210")
        );
    }

    #[test]
    fn signup_conversion_resource_is_built() {
        let cfg = test_config(None);
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::Signup)
                .as_deref(),
            Some("customers/1234567890/conversionActions/42")
        );
    }

    #[test]
    fn activation_conversion_resource_requires_configuration() {
        // Not configured → None (upload becomes a no-op).
        let cfg = test_config(None);
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::Activation),
            None
        );

        // Configured → resolves to its own conversion action id.
        let cfg = test_config(Some("99"));
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::Activation)
                .as_deref(),
            Some("customers/1234567890/conversionActions/99")
        );
        // Signup is unaffected by the activation id.
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::Signup)
                .as_deref(),
            Some("customers/1234567890/conversionActions/42")
        );
    }

    #[test]
    fn is_retryable_classifies_errors_correctly() {
        // 5xx and 429 from the Ads API are retryable
        assert!(is_retryable(&GoogleAdsError::Api {
            status: 500,
            body: "".into()
        }));
        assert!(is_retryable(&GoogleAdsError::Api {
            status: 503,
            body: "".into()
        }));
        assert!(is_retryable(&GoogleAdsError::Api {
            status: 429,
            body: "".into()
        }));

        // 4xx (other than 429) are permanent — bad request, auth, forbidden
        assert!(!is_retryable(&GoogleAdsError::Api {
            status: 400,
            body: "".into()
        }));
        assert!(!is_retryable(&GoogleAdsError::Api {
            status: 401,
            body: "".into()
        }));
        assert!(!is_retryable(&GoogleAdsError::Api {
            status: 403,
            body: "".into()
        }));

        // OAuth refresh: same logic — 5xx/429 retryable, 4xx permanent
        assert!(is_retryable(&GoogleAdsError::OAuth {
            status: 503,
            body: "".into()
        }));
        assert!(is_retryable(&GoogleAdsError::OAuth {
            status: 429,
            body: "".into()
        }));
        assert!(!is_retryable(&GoogleAdsError::OAuth {
            status: 401,
            body: "".into()
        }));
        assert!(!is_retryable(&GoogleAdsError::OAuth {
            status: 400,
            body: "".into()
        }));

        // Header errors are programming bugs, never retryable. Note: we cannot
        // construct an InvalidHeaderValue easily from a unit test (no public
        // constructor), and reqwest::Error has no public constructor either,
        // so transport-level variants are intentionally not asserted here.
    }

    #[test]
    fn normalize_drops_absent_empty_and_whitespace() {
        assert_eq!(normalize_gclid(None), None);
        assert_eq!(normalize_gclid(Some("")), None);
        assert_eq!(normalize_gclid(Some("   ")), None);
        assert_eq!(normalize_gclid(Some("\t\n ")), None);
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_gclid(Some("  Cj0KCQ...  ")),
            Some("Cj0KCQ...".to_string())
        );
    }

    #[test]
    fn normalize_drops_overlong_keeps_at_limit() {
        let too_long = "a".repeat(MAX_GCLID_LEN + 1);
        assert_eq!(normalize_gclid(Some(&too_long)), None);

        let at_limit = "a".repeat(MAX_GCLID_LEN);
        assert_eq!(normalize_gclid(Some(&at_limit)), Some(at_limit));
    }

    proptest! {
        // Output invariant: the stored gclid is always None, or a non-empty,
        // trimmed string within the DB length bound. Guarantees we never INSERT
        // a value the `signup_attribution_gclid_length` CHECK would reject.
        #[test]
        fn normalized_output_is_bounded_and_trimmed(raw in ".*") {
            if let Some(s) = normalize_gclid(Some(&raw)) {
                prop_assert!(!s.is_empty());
                prop_assert!(s.chars().count() <= MAX_GCLID_LEN);
                prop_assert_eq!(s.trim(), s.as_str());
            }
        }

        // Idempotence: normalizing an already-normalized value changes nothing.
        #[test]
        fn normalize_is_idempotent(raw in ".*") {
            let once = normalize_gclid(Some(&raw));
            let twice = normalize_gclid(once.as_deref());
            prop_assert_eq!(once, twice);
        }
    }

    // ----- Upload boundary: the real outbound request, against a local socket -----

    #[actix_web::test]
    async fn activation_upload_targets_activation_conversion_action() {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"));

        client
            .upload_click_conversion("gclid-activation", ConversionKind::Activation, Utc::now())
            .await
            .expect("activation upload should succeed");

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 1, "exactly one upload request");
        assert_eq!(reqs[0].path, "/customers/1234567890:uploadClickConversions");

        let body: serde_json::Value = serde_json::from_str(&reqs[0].body).expect("json body");
        assert_eq!(body["partialFailure"], serde_json::json!(true));
        assert_eq!(body["conversions"][0]["gclid"], "gclid-activation");
        assert_eq!(
            body["conversions"][0]["conversionAction"],
            "customers/1234567890/conversionActions/777"
        );
    }

    #[actix_web::test]
    async fn signup_and_activation_use_distinct_conversion_actions() {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"));

        client
            .upload_click_conversion("g1", ConversionKind::Signup, Utc::now())
            .await
            .expect("signup upload");
        client
            .upload_click_conversion("g2", ConversionKind::Activation, Utc::now())
            .await
            .expect("activation upload");

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 2);
        let action_of = |i: usize| -> String {
            let v: serde_json::Value = serde_json::from_str(&reqs[i].body).expect("json");
            v["conversions"][0]["conversionAction"]
                .as_str()
                .expect("conversionAction")
                .to_string()
        };
        assert_eq!(action_of(0), "customers/1234567890/conversionActions/42");
        assert_eq!(action_of(1), "customers/1234567890/conversionActions/777");
    }

    #[actix_web::test]
    async fn upload_is_noop_when_activation_action_unconfigured() {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), None);

        client
            .upload_click_conversion("g", ConversionKind::Activation, Utc::now())
            .await
            .expect("noop upload returns Ok");

        assert!(
            fake.requests().is_empty(),
            "no request is sent when the activation action is not configured"
        );
    }

    #[actix_web::test]
    async fn partial_failure_is_non_fatal() {
        let fake = FakeGoogleAds::start(
            200,
            r#"{"partialFailureError":{"code":3,"message":"gclid invalid"}}"#,
        );
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"));

        // A 200 carrying a per-operation partialFailureError (e.g. unknown
        // gclid) is treated as Ok — the conversion is not worth retrying.
        client
            .upload_click_conversion("bad-gclid", ConversionKind::Activation, Utc::now())
            .await
            .expect("partial failure is non-fatal");
    }

    #[actix_web::test]
    async fn api_4xx_is_a_non_retryable_error() {
        let fake = FakeGoogleAds::start(400, r#"{"error":"bad request"}"#);
        let client = test_client_with_base_url(fake.base_url.clone(), Some("777"));

        let err = client
            .upload_click_conversion("g", ConversionKind::Activation, Utc::now())
            .await
            .expect_err("4xx must surface as an error");
        assert!(!is_retryable(&err), "a 4xx upload error is permanent");
    }

    // ----- gclid attribution lifecycle, against a real Postgres -----

    #[sqlx::test]
    async fn claim_activation_fires_at_most_once(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        seed_attribution(&pool, user, org, "gclid-claim", true).await;

        let first = claim_activation_gclid(&pool, &org).await.expect("claim ok");
        assert_eq!(first.as_deref(), Some("gclid-claim"));

        let second = claim_activation_gclid(&pool, &org)
            .await
            .expect("second claim ok");
        assert_eq!(second, None, "activation is claimed only once per org");

        let (_, activation_uploaded) = attribution_state(&pool, org).await;
        assert!(
            activation_uploaded,
            "activation_uploaded_at is set after claim"
        );
    }

    #[sqlx::test]
    async fn claim_returns_none_without_attribution_row(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // No attribution row: an org with no gclid never fires an activation.
        let claimed = claim_activation_gclid(&pool, &org).await.expect("claim ok");
        assert_eq!(claimed, None);
    }

    #[sqlx::test]
    async fn gclid_cleared_only_after_both_conversions(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup already uploaded, activation still pending.
        seed_attribution(&pool, user, org, "gclid-clear", true).await;

        // Activation not yet claimed → clearing is a no-op.
        clear_gclid_if_fully_uploaded_by_org(&pool, &org).await;
        let (gclid, _) = attribution_state(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-clear"),
            "gclid kept until both conversions are uploaded"
        );

        // Claim activation, then clearing nulls the gclid (data minimisation).
        claim_activation_gclid(&pool, &org).await.expect("claim");
        clear_gclid_if_fully_uploaded_by_org(&pool, &org).await;
        let (gclid, activation_uploaded) = attribution_state(&pool, org).await;
        assert_eq!(gclid, None, "gclid nulled once both conversions uploaded");
        assert!(activation_uploaded);
    }

    #[sqlx::test]
    async fn gclid_not_cleared_while_signup_pending(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup NOT uploaded yet.
        seed_attribution(&pool, user, org, "gclid-keep", false).await;

        claim_activation_gclid(&pool, &org).await.expect("claim");
        clear_gclid_if_fully_uploaded_by_org(&pool, &org).await;
        let (gclid, _) = attribution_state(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-keep"),
            "signup still pending → gclid retained"
        );
    }
}
