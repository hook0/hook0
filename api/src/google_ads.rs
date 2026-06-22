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

#[cfg(test)]
mod tests {
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
}
