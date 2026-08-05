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
//! the registration / email-verification flow (signup conversion) and the two
//! background jobs that upload the first-event conversion and the
//! first-webhook-delivered conversion (the deepest funnel step: an org's first
//! successful webhook delivery). The gclid is retained until every ENABLED
//! conversion has been uploaded, then cleared (data minimisation): signup always
//! counts; the first-event and first-webhook-delivered conversions each gate
//! gclid nullification only when they are configured. The retention-window
//! cleanup in `handlers::registrations` (see
//! `SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS`) is the safety net for rows that never
//! reach that state. See
//! `documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md`.

use chrono::{DateTime, Utc};
use clap::crate_name;
use opentelemetry::metrics::Counter;
use opentelemetry::{KeyValue, global};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Background uploader for the "first event sent" conversion (periodic scan;
/// never on the event-ingestion hot path).
pub mod first_event_conversion;

/// Background uploader for the "first webhook delivered" conversion (periodic
/// scan; never on the webhook-delivery hot path).
pub mod first_webhook_delivered_conversion;

// The counter is built once on first use and stays bound to the global meter
// provider that exists at that moment. `opentelemetry::init()` sets that
// provider during startup, before any conversion upload can happen, so this is
// safe. A caller running before `init()` would bind to the no-op provider.
static CONVERSIONS_UPLOADED: LazyLock<Counter<u64>> = LazyLock::new(|| {
    global::meter(crate_name!())
        .u64_counter("conversions.uploaded")
        .with_description("Google Ads click conversions uploaded, by kind and terminal outcome")
        .build()
});

/// Count one terminal conversion-upload outcome. `kind` is the conversion kind
/// (`signup` / `first_event` / `first_webhook_delivered`); `outcome` is `success`,
/// `partial_failure` (Google rejected the operation, e.g. unknown gclid) or
/// `failed` (transport/API error after exhausting retries).
pub(crate) fn report_conversion_uploaded(kind: &'static str, outcome: &'static str) {
    CONVERSIONS_UPLOADED.add(
        1,
        &[
            KeyValue::new("kind", kind),
            KeyValue::new("outcome", outcome),
        ],
    );
}

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
    /// Numeric ID of the "first event sent" conversion action (optional). When
    /// `None`, the first-event conversion (and its background job) is disabled.
    pub first_event_conversion_action_id: Option<String>,
    /// Numeric ID of the "first webhook delivered" conversion action (optional).
    /// When `None`, the first-webhook-delivered conversion (and its background
    /// job) is disabled. This is the north-star activation signal: the org's
    /// first successful webhook delivery (a request attempt with `succeeded_at`).
    pub first_webhook_delivered_conversion_action_id: Option<String>,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_refresh_token: String,
}

/// Which conversion action a click conversion targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionKind {
    /// User verified their email after signing up.
    Signup,
    /// Organization ingested its first event (the mid-funnel activation signal:
    /// the account is doing real work, not merely created).
    FirstEvent,
    /// Organization delivered its first webhook successfully (its first request
    /// attempt with `succeeded_at` set) — the north-star activation signal,
    /// deeper in the funnel than a merely-ingested event.
    FirstWebhookDelivered,
}

impl ConversionKind {
    /// Stable label used for the `conversions.uploaded` metric dimension.
    fn metric_label(self) -> &'static str {
        match self {
            ConversionKind::Signup => "signup",
            ConversionKind::FirstEvent => "first_event",
            ConversionKind::FirstWebhookDelivered => "first_webhook_delivered",
        }
    }
}

/// Outcome of a single click-conversion upload, so callers can tell a clean
/// success from a per-operation rejection (`partialFailureError`) and never
/// count the latter as a success.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadOutcome {
    /// No conversion action configured for this kind; nothing was sent.
    Skipped,
    /// HTTP 2xx with no per-operation error.
    Success,
    /// HTTP 2xx but the response carried a `partialFailureError` (e.g. an
    /// unknown or expired gclid). Terminal: retrying will not help.
    PartialFailure,
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
    /// conversion kind. Returns `None` for an optional conversion kind whose
    /// conversion action is not configured.
    fn conversion_action_resource(&self, kind: ConversionKind) -> Option<String> {
        let conversion_action_id = match kind {
            ConversionKind::Signup => self.signup_conversion_action_id.clone(),
            ConversionKind::FirstEvent => self.first_event_conversion_action_id.clone()?,
            ConversionKind::FirstWebhookDelivered => {
                self.first_webhook_delivered_conversion_action_id.clone()?
            }
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
            // Never follow redirects: neither the Google Ads REST endpoint nor
            // the OAuth token endpoint legitimately redirects, and a 307/308
            // would replay this OAuth-bearing POST body to an attacker-chosen
            // `Location`.
            .redirect(reqwest::redirect::Policy::none())
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

    /// Returns `true` if a first-event conversion action is configured.
    pub fn has_first_event_conversion(&self) -> bool {
        self.config.first_event_conversion_action_id.is_some()
    }

    /// Returns `true` if a first-webhook-delivered conversion action is configured.
    pub fn has_first_webhook_delivered_conversion(&self) -> bool {
        self.config
            .first_webhook_delivered_conversion_action_id
            .is_some()
    }

    /// Upload a click conversion using only the gclid (no PII).
    ///
    /// When no conversion action is configured for `kind`, this is a silent
    /// no-op returning [`UploadOutcome::Skipped`].
    pub async fn upload_click_conversion(
        &self,
        gclid: &str,
        kind: ConversionKind,
        conversion_date_time: DateTime<Utc>,
    ) -> Result<UploadOutcome, GoogleAdsError> {
        let Some(conversion_action) = self.config.conversion_action_resource(kind) else {
            debug!(
                target: "api::google_ads",
                ?kind,
                "conversion action not configured; skipping upload"
            );
            return Ok(UploadOutcome::Skipped);
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
        // reason (e.g. unknown gclid). Reported as its own outcome so callers
        // never count it as a success.
        if response_body.contains("partialFailureError") {
            warn!(target: "api::google_ads", body = %response_body, "Google Ads partial failure");
            return Ok(UploadOutcome::PartialFailure);
        }
        Ok(UploadOutcome::Success)
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
        // Capture the conversion timestamp ONCE, before the retry loop. Google
        // Ads deduplicates a re-uploaded conversion by (gclid, conversionAction,
        // conversionDateTime); recomputing the time on each retry would defeat
        // that dedup and let a retried-then-actually-succeeded upload be
        // counted twice.
        let conversion_date_time = Utc::now();
        upload_with_retries(&client, &gclid, kind, &RETRY_DELAYS, conversion_date_time).await;
    });
}

/// Upload a conversion, retrying retryable failures with the given inter-attempt
/// `delays` (so `delays.len() + 1` attempts total). Factored out of
/// [`spawn_upload`] so tests can drive the real retry loop with short delays.
///
/// `conversion_date_time` is captured once by the caller and reused for every
/// attempt — see [`spawn_upload`]. Records the `conversions.uploaded` metric
/// exactly once, with the terminal outcome (`success` / `partial_failure` /
/// `failed`); a skipped no-op is not counted.
async fn upload_with_retries(
    client: &GoogleAdsClient,
    gclid: &str,
    kind: ConversionKind,
    delays: &[Duration],
    conversion_date_time: DateTime<Utc>,
) {
    let started = Instant::now();
    let gclid_prefix: String = gclid.chars().take(8).collect();
    let max_attempts = delays.len() + 1;

    for attempt in 1..=max_attempts {
        match client
            .upload_click_conversion(gclid, kind, conversion_date_time)
            .await
        {
            Ok(UploadOutcome::Skipped) => return,
            Ok(outcome) => {
                let metric_outcome = match outcome {
                    UploadOutcome::PartialFailure => "partial_failure",
                    _ => "success",
                };
                report_conversion_uploaded(kind.metric_label(), metric_outcome);
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
                    outcome = metric_outcome,
                    attempt = attempt,
                    duration_ms = started.elapsed().as_millis() as u64,
                    "click conversion uploaded"
                );
                return;
            }
            Err(e) => {
                if !is_retryable(&e) {
                    report_conversion_uploaded(kind.metric_label(), "failed");
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
                    report_conversion_uploaded(kind.metric_label(), "failed");
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

                let delay = delays[attempt - 1];
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

/// Clear the gclid (data minimisation) once every ENABLED conversion has been
/// uploaded, for the attribution row of `user_id`. Signup is always required;
/// the first-event conversion is required only when
/// `first_event_tracking_enabled` is `true`, and the first-webhook-delivered
/// conversion only when `first_webhook_delivered_tracking_enabled` is `true`
/// (otherwise there is no such conversion to wait for, and holding the gclid
/// past the earlier stages would needlessly weaken data minimisation).
///
/// This user-keyed variant carries the same first-event / first-webhook-delivered
/// gating as [`clear_gclid_if_fully_uploaded_by_org`]. It is invoked from the
/// email-verification step, but the first event (and even the first webhook
/// delivery) can precede email verification — nothing forces a user to verify
/// before using the API — so a later funnel stage may already be uploaded here.
/// Gating on every enabled conversion, first-webhook-delivered included, ensures
/// the gclid is never cleared while any enabled conversion is still pending,
/// which would otherwise exclude the org from the periodic first-webhook-delivered
/// scan (`WHERE gclid IS NOT NULL`) and permanently lose that conversion.
///
/// Best-effort: errors are logged, never surfaced (the conversion has already
/// been queued).
pub async fn clear_gclid_if_fully_uploaded_by_user(
    db: &PgPool,
    user_id: &Uuid,
    first_event_tracking_enabled: bool,
    first_webhook_delivered_tracking_enabled: bool,
) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE user__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND (first_event_uploaded_at IS NOT NULL OR NOT $2)
              AND (first_webhook_delivered_uploaded_at IS NOT NULL OR NOT $3)
        ",
        user_id,
        first_event_tracking_enabled,
        first_webhook_delivered_tracking_enabled,
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
///
/// This organization-keyed variant additionally waits on the first-webhook-
/// delivered conversion: when `first_webhook_delivered_tracking_enabled` is
/// `true`, the gclid is kept until that conversion has been uploaded too.
/// The webhook-delivery signal is the deepest funnel step (it only fires after
/// an event has been ingested AND delivered), so clearing on signup + first-event
/// alone would drop the gclid before the first-webhook-delivered job could ever
/// upload it.
pub async fn clear_gclid_if_fully_uploaded_by_org(
    db: &PgPool,
    organization_id: &Uuid,
    first_event_tracking_enabled: bool,
    first_webhook_delivered_tracking_enabled: bool,
) {
    let result = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET gclid = NULL
            WHERE organization__id = $1
              AND gclid IS NOT NULL
              AND signup_uploaded_at IS NOT NULL
              AND (first_event_uploaded_at IS NOT NULL OR NOT $2)
              AND (first_webhook_delivered_uploaded_at IS NOT NULL OR NOT $3)
        ",
        organization_id,
        first_event_tracking_enabled,
        first_webhook_delivered_tracking_enabled,
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

/// Atomically mark the first-event conversion as uploaded for an organization,
/// but only if not already marked. Returns `true` when this call is the one
/// that flipped `first_event_uploaded_at` from NULL.
///
/// This is the claim-on-success counterpart used by the background job AFTER a
/// confirmed upload: keeping the flag NULL until the upload is confirmed lets a
/// crashed pass auto-recover (the org is picked up again on the next scan).
/// Idempotent: a second call returns `false`.
pub async fn mark_first_event_uploaded(
    db: &PgPool,
    organization_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET first_event_uploaded_at = statement_timestamp()
            WHERE organization__id = $1
              AND first_event_uploaded_at IS NULL
            RETURNING user__id
        ",
        organization_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(row.is_some())
}

/// Atomically mark the first-webhook-delivered conversion as uploaded for an
/// organization, but only if not already marked. Returns `true` when this call
/// is the one that flipped `first_webhook_delivered_uploaded_at` from NULL.
///
/// This is the claim-on-success counterpart used by the background job AFTER a
/// confirmed upload — same rationale as [`mark_first_event_uploaded`]: keeping
/// the flag NULL until the upload is confirmed lets a crashed pass auto-recover
/// (the org is picked up again on the next scan). Idempotent: a second call
/// returns `false`.
pub async fn mark_first_webhook_delivered_uploaded(
    db: &PgPool,
    organization_id: &Uuid,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        "
            UPDATE iam.signup_attribution
            SET first_webhook_delivered_uploaded_at = statement_timestamp()
            WHERE organization__id = $1
              AND first_webhook_delivered_uploaded_at IS NULL
            RETURNING user__id
        ",
        organization_id,
    )
    .fetch_optional(db)
    .await?;

    Ok(row.is_some())
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use sqlx::PgPool;
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

    /// Current gclid for an org's attribution row (`None` once minimised).
    pub(crate) async fn current_gclid(pool: &PgPool, org: Uuid) -> Option<String> {
        let row: (Option<String>,) =
            sqlx::query_as("SELECT gclid FROM iam.signup_attribution WHERE organization__id = $1")
                .bind(org)
                .fetch_one(pool)
                .await
                .expect("read attribution state");
        row.0
    }

    /// Whether the first-event conversion has been marked uploaded for an org.
    pub(crate) async fn first_event_uploaded(pool: &PgPool, org: Uuid) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT first_event_uploaded_at IS NOT NULL FROM iam.signup_attribution WHERE organization__id = $1",
        )
        .bind(org)
        .fetch_one(pool)
        .await
        .expect("read first-event state");
        row.0
    }

    /// Whether the first-webhook-delivered conversion has been marked uploaded
    /// for an org.
    pub(crate) async fn first_webhook_delivered_uploaded(pool: &PgPool, org: Uuid) -> bool {
        let row: (bool,) = sqlx::query_as(
            "SELECT first_webhook_delivered_uploaded_at IS NOT NULL FROM iam.signup_attribution WHERE organization__id = $1",
        )
        .bind(org)
        .fetch_one(pool)
        .await
        .expect("read first-webhook-delivered state");
        row.0
    }

    /// Give an org a real ingested event (application + event type + event) so
    /// the first-event `EXISTS` predicate matches it. Mirrors the minimal set of
    /// NOT NULL columns and the (application, event_type) foreign key. Returns
    /// `(application_id, event_id)` so callers can also seed a request attempt.
    pub(crate) async fn seed_event(pool: &PgPool, org: Uuid) -> (Uuid, Uuid) {
        let application_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO event.application (application__id, organization__id, name) VALUES ($1, $2, 'E2E app')",
        )
        .bind(application_id)
        .bind(org)
        .execute(pool)
        .await
        .expect("seed application");

        // An event_type references service / verb / resource_type, so those must
        // exist first. event_type__name is a generated column (service.resource.
        // verb), so it is not inserted; here it resolves to 'test.resource.created'.
        sqlx::query(
            "INSERT INTO event.service (application__id, service__name) VALUES ($1, 'test')",
        )
        .bind(application_id)
        .execute(pool)
        .await
        .expect("seed service");
        sqlx::query("INSERT INTO event.verb (application__id, verb__name) VALUES ($1, 'created')")
            .bind(application_id)
            .execute(pool)
            .await
            .expect("seed verb");
        sqlx::query(
            "INSERT INTO event.resource_type (application__id, service__name, resource_type__name) VALUES ($1, 'test', 'resource')",
        )
        .bind(application_id)
        .execute(pool)
        .await
        .expect("seed resource type");
        sqlx::query(
            r#"
                INSERT INTO event.event_type (application__id, service__name, resource_type__name, verb__name)
                VALUES ($1, 'test', 'resource', 'created')
            "#,
        )
        .bind(application_id)
        .execute(pool)
        .await
        .expect("seed event type");

        let event_id: Uuid = sqlx::query_scalar(
            r#"
                INSERT INTO event.event (application__id, event_type__name, payload_content_type, ip, occurred_at)
                VALUES ($1, 'test.resource.created', 'application/json', '127.0.0.1'::inet, statement_timestamp())
                RETURNING event__id
            "#,
        )
        .bind(application_id)
        .fetch_one(pool)
        .await
        .expect("seed event");

        (application_id, event_id)
    }

    /// Seed a webhook delivery attempt for `application_id` / `event_id`. When
    /// `succeeded` is `true` its `succeeded_at` is set, which is exactly what the
    /// first-webhook-delivered `EXISTS` predicate matches; otherwise it is left
    /// pending (`succeeded_at` NULL) to model a delivery that has not succeeded
    /// yet. Creates the required subscription so the request attempt's NOT NULL
    /// `subscription__id` foreign key is satisfied.
    pub(crate) async fn seed_request_attempt(
        pool: &PgPool,
        application_id: Uuid,
        event_id: Uuid,
        succeeded: bool,
    ) {
        let subscription_id = Uuid::new_v4();
        sqlx::query(
            r#"
                INSERT INTO webhook.subscription
                    (subscription__id, application__id, is_enabled, secret, metadata, labels, target__id, created_at, updated_at)
                VALUES ($1, $2, true, public.gen_random_uuid(), '{}'::jsonb, '{"e2e":"1"}'::jsonb, public.gen_random_uuid(), statement_timestamp(), statement_timestamp())
            "#,
        )
        .bind(subscription_id)
        .bind(application_id)
        .execute(pool)
        .await
        .expect("seed subscription");

        sqlx::query(
            r#"
                INSERT INTO webhook.request_attempt
                    (event__id, subscription__id, application__id, succeeded_at)
                VALUES ($1, $2, $3, CASE WHEN $4 THEN statement_timestamp() ELSE NULL END)
            "#,
        )
        .bind(event_id)
        .bind(subscription_id)
        .bind(application_id)
        .bind(succeeded)
        .execute(pool)
        .await
        .expect("seed request attempt");
    }
}

/// Build a client whose Google Ads base URL is overridden (to point at a local
/// fake server) and whose OAuth token is pre-seeded (so no token endpoint is
/// hit). Lives at module level so both the unit tests here and the handler
/// integration test can construct one despite the private fields.
#[cfg(test)]
pub(crate) fn test_client_with_base_url(
    base_url: String,
    first_event_conversion_action_id: Option<&str>,
    first_webhook_delivered_conversion_action_id: Option<&str>,
) -> Arc<GoogleAdsClient> {
    let config = GoogleAdsConfig {
        developer_token: "t".into(),
        customer_id: "123-456-7890".into(),
        login_customer_id: Some("987-654-3210".into()),
        signup_conversion_action_id: "42".into(),
        first_event_conversion_action_id: first_event_conversion_action_id.map(str::to_string),
        first_webhook_delivered_conversion_action_id: first_webhook_delivered_conversion_action_id
            .map(str::to_string),
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
        FakeGoogleAds, current_gclid, first_event_uploaded, seed_attribution, seed_org, seed_user,
    };
    use super::*;
    use proptest::prelude::*;

    fn test_config() -> GoogleAdsConfig {
        GoogleAdsConfig {
            developer_token: "t".into(),
            customer_id: "123-456-7890".into(),
            login_customer_id: Some("987-654-3210".into()),
            signup_conversion_action_id: "42".into(),
            first_event_conversion_action_id: None,
            first_webhook_delivered_conversion_action_id: None,
            oauth_client_id: "c".into(),
            oauth_client_secret: "s".into(),
            oauth_refresh_token: "r".into(),
        }
    }

    #[test]
    fn customer_id_is_normalized() {
        let cfg = test_config();
        assert_eq!(cfg.normalized_customer_id(), "1234567890");
        assert_eq!(
            cfg.normalized_login_customer_id().as_deref(),
            Some("9876543210")
        );
    }

    #[test]
    fn signup_conversion_resource_is_built() {
        let cfg = test_config();
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::Signup)
                .as_deref(),
            Some("customers/1234567890/conversionActions/42")
        );
    }

    #[test]
    fn first_event_conversion_resource_requires_configuration() {
        // Not configured → None (upload becomes a no-op).
        let cfg = test_config();
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::FirstEvent),
            None
        );

        // Configured → resolves to its own conversion action id.
        let mut cfg = test_config();
        cfg.first_event_conversion_action_id = Some("99".into());
        assert_eq!(
            cfg.conversion_action_resource(ConversionKind::FirstEvent)
                .as_deref(),
            Some("customers/1234567890/conversionActions/99")
        );
        // Signup is unaffected by the first-event id.
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
    async fn signup_upload_targets_signup_conversion_action() {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), None, None);

        client
            .upload_click_conversion("gclid-signup", ConversionKind::Signup, Utc::now())
            .await
            .expect("signup upload should succeed");

        let reqs = fake.requests();
        assert_eq!(reqs.len(), 1, "exactly one upload request");
        assert_eq!(reqs[0].path, "/customers/1234567890:uploadClickConversions");

        let body: serde_json::Value = serde_json::from_str(&reqs[0].body).expect("json body");
        assert_eq!(body["partialFailure"], serde_json::json!(true));
        assert_eq!(body["conversions"][0]["gclid"], "gclid-signup");
        assert_eq!(
            body["conversions"][0]["conversionAction"],
            "customers/1234567890/conversionActions/42"
        );
    }

    #[actix_web::test]
    async fn upload_is_noop_when_first_event_action_unconfigured() {
        let fake = FakeGoogleAds::start(200, "{}");
        let client = test_client_with_base_url(fake.base_url.clone(), None, None);

        let outcome = client
            .upload_click_conversion("g", ConversionKind::FirstEvent, Utc::now())
            .await
            .expect("noop upload returns Ok");

        assert_eq!(outcome, UploadOutcome::Skipped);
        assert!(
            fake.requests().is_empty(),
            "no request is sent when the first-event action is not configured"
        );
    }

    #[actix_web::test]
    async fn partial_failure_is_non_fatal() {
        let fake = FakeGoogleAds::start(
            200,
            r#"{"partialFailureError":{"code":3,"message":"gclid invalid"}}"#,
        );
        let client = test_client_with_base_url(fake.base_url.clone(), None, None);

        // A 200 carrying a per-operation partialFailureError (e.g. unknown
        // gclid) is reported as PartialFailure (not a success, not an error) —
        // the conversion is not worth retrying.
        let outcome = client
            .upload_click_conversion("bad-gclid", ConversionKind::Signup, Utc::now())
            .await
            .expect("partial failure is non-fatal");
        assert_eq!(outcome, UploadOutcome::PartialFailure);
    }

    #[actix_web::test]
    async fn api_4xx_is_a_non_retryable_error() {
        let fake = FakeGoogleAds::start(400, r#"{"error":"bad request"}"#);
        let client = test_client_with_base_url(fake.base_url.clone(), None, None);

        let err = client
            .upload_click_conversion("g", ConversionKind::Signup, Utc::now())
            .await
            .expect_err("4xx must surface as an error");
        assert!(!is_retryable(&err), "a 4xx upload error is permanent");
    }

    // ----- gclid attribution lifecycle, against a real Postgres -----

    #[sqlx::test]
    async fn gclid_cleared_after_signup_when_first_event_and_webhook_disabled(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup already uploaded. This instance tracks neither the first event
        // nor the first webhook delivery, so signup alone is "fully uploaded".
        seed_attribution(&pool, user, org, "gclid-clear", true).await;

        // No activation gate any more: clearing nulls the gclid as soon as
        // signup is uploaded and no later enabled conversion is pending.
        clear_gclid_if_fully_uploaded_by_org(&pool, &org, false, false).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid, None,
            "gclid nulled once signup is uploaded (no activation conversion needed)"
        );
    }

    #[sqlx::test]
    async fn gclid_retained_until_first_event_when_first_event_enabled(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup uploaded. First-event tracking is ON.
        seed_attribution(&pool, user, org, "gclid-2way", true).await;

        // With first-event tracking enabled, signup alone is NOT enough:
        // clearing now would purge the gclid before the first-event conversion
        // could ever be uploaded (the hazard this guards against).
        clear_gclid_if_fully_uploaded_by_org(&pool, &org, true, false).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-2way"),
            "gclid retained until the first-event conversion is uploaded too"
        );

        // Mark the first-event conversion uploaded → now both enabled conversions
        // (signup + first-event) are done, with no activation step in between.
        mark_first_event_uploaded(&pool, &org)
            .await
            .expect("mark first event");
        clear_gclid_if_fully_uploaded_by_org(&pool, &org, true, false).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid, None,
            "gclid nulled once signup + first-event uploaded (no activation needed)"
        );
    }

    #[sqlx::test]
    async fn gclid_retained_until_first_webhook_delivered_when_enabled(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup uploaded. First-webhook-delivered tracking is ON, first-event
        // tracking is OFF.
        seed_attribution(&pool, user, org, "gclid-webhook", true).await;

        // With first-webhook-delivered tracking enabled, signup alone is NOT
        // enough: clearing now would purge the gclid before the north-star
        // conversion could ever be uploaded (the hazard this guards against).
        clear_gclid_if_fully_uploaded_by_org(&pool, &org, false, true).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-webhook"),
            "gclid retained until the first-webhook-delivered conversion is uploaded too"
        );

        // Mark the first-webhook-delivered conversion uploaded → now every
        // enabled conversion is done.
        mark_first_webhook_delivered_uploaded(&pool, &org)
            .await
            .expect("mark first webhook delivered");
        clear_gclid_if_fully_uploaded_by_org(&pool, &org, false, true).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid, None,
            "gclid nulled once signup + first-webhook-delivered uploaded"
        );
    }

    #[sqlx::test]
    async fn user_keyed_gclid_retained_until_first_webhook_delivered_when_enabled(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Use-before-verify: an org can send its first event (and even receive
        // its first webhook) BEFORE the user verifies their email, since API
        // usage is not gated on email verification. Here signup is already
        // uploaded when the user-keyed clear runs at verification time.
        // First-webhook-delivered tracking is ON, first-event tracking OFF.
        seed_attribution(&pool, user, org, "gclid-user-webhook", true).await;

        // With first-webhook-delivered tracking enabled, signup alone is NOT
        // enough: clearing now would purge the gclid before the north-star
        // conversion could ever be uploaded, and the periodic scan
        // (`WHERE gclid IS NOT NULL`) would then exclude this org forever.
        clear_gclid_if_fully_uploaded_by_user(&pool, &user, false, true).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-user-webhook"),
            "user-keyed clear retains gclid until first-webhook-delivered is uploaded too"
        );

        // Mark the first-webhook-delivered conversion uploaded → now every
        // enabled conversion is done and the user-keyed clear nulls the gclid.
        mark_first_webhook_delivered_uploaded(&pool, &org)
            .await
            .expect("mark first webhook delivered");
        clear_gclid_if_fully_uploaded_by_user(&pool, &user, false, true).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid, None,
            "gclid nulled once signup + first-webhook-delivered uploaded"
        );
    }

    #[sqlx::test]
    async fn gclid_not_cleared_while_signup_pending(pool: PgPool) {
        let user = seed_user(&pool).await;
        let org = seed_org(&pool, user).await;
        // Signup NOT uploaded yet.
        seed_attribution(&pool, user, org, "gclid-keep", false).await;

        clear_gclid_if_fully_uploaded_by_org(&pool, &org, false, false).await;
        let gclid = current_gclid(&pool, org).await;
        assert_eq!(
            gclid.as_deref(),
            Some("gclid-keep"),
            "signup still pending → gclid retained"
        );
    }

    #[sqlx::test]
    async fn mark_first_event_uploaded_is_at_most_once(pool: PgPool) {
        // Property: however many times the mark is attempted for an org, exactly
        // one attempt claims it (returns true); every later attempt returns
        // false. Checked exhaustively over the small, meaningful call-count
        // domain (1..=6).
        for attempts in 1u32..=6 {
            let user = seed_user(&pool).await;
            let org = seed_org(&pool, user).await;
            seed_attribution(&pool, user, org, "gclid-mark", true).await;

            let mut claims = 0u32;
            for _ in 0..attempts {
                if mark_first_event_uploaded(&pool, &org)
                    .await
                    .expect("mark ok")
                {
                    claims += 1;
                }
            }

            assert_eq!(claims, 1, "exactly one claim across {attempts} attempts");
            assert!(first_event_uploaded(&pool, org).await);
        }
    }

    // ----- Retry loop: conversionDateTime is stable across attempts -----

    proptest! {
        // The (gclid, conversionAction, conversionDateTime) triple is Google
        // Ads' dedup key. If a retry recomputed the timestamp, a retried-then-
        // succeeded upload would be counted twice. Property: across every
        // attempt of a single upload, the conversionDateTime sent is identical.
        #![proptest_config(ProptestConfig::with_cases(12))]
        #[test]
        fn conversion_date_time_is_stable_across_retries(epoch_secs in 1_600_000_000i64..2_000_000_000i64) {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            runtime.block_on(async move {
                // 503 is retryable → all attempts run (delays are zero here).
                let fake = FakeGoogleAds::start(503, "{}");
                let client = test_client_with_base_url(fake.base_url.clone(), None, None);
                let captured = DateTime::from_timestamp(epoch_secs, 0).expect("valid timestamp");

                upload_with_retries(
                    &client,
                    "gclid-retry",
                    ConversionKind::Signup,
                    &[Duration::ZERO, Duration::ZERO, Duration::ZERO],
                    captured,
                )
                .await;

                let reqs = fake.wait_for(4, Duration::from_secs(5)).await;
                prop_assert_eq!(reqs.len(), 4, "one request per attempt");
                let date_times: Vec<String> = reqs
                    .iter()
                    .map(|r| {
                        let v: serde_json::Value =
                            serde_json::from_str(&r.body).expect("json body");
                        v["conversions"][0]["conversionDateTime"]
                            .as_str()
                            .expect("conversionDateTime")
                            .to_string()
                    })
                    .collect();
                for dt in &date_times {
                    prop_assert_eq!(dt, &date_times[0], "conversionDateTime stable across retries");
                }
                Ok(())
            })?;
        }
    }
}
