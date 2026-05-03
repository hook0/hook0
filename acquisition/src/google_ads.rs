//! Google Ads API client for Enhanced Conversions for Leads.
//!
//! Uploads click conversions with hashed user identifiers (email) so Google
//! can match them server-side to ad clicks. No client-side trackers required.
//!
//! Reference:
//! - <https://developers.google.com/google-ads/api/docs/conversions/upload-clicks>
//! - <https://developers.google.com/google-ads/api/docs/conversions/enhance-conversions-leads>

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, info};

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

/// Configuration required to authenticate against the Google Ads API.
#[derive(Debug, Clone)]
pub struct GoogleAdsConfig {
    pub developer_token: String,
    pub customer_id: String,
    pub login_customer_id: Option<String>,
    pub conversion_action_id: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_refresh_token: String,
}

impl GoogleAdsConfig {
    /// Strip dashes (123-456-7890 -> 1234567890) for use in URL paths.
    fn normalized_customer_id(&self) -> String {
        self.customer_id.replace('-', "")
    }

    fn normalized_login_customer_id(&self) -> Option<String> {
        self.login_customer_id
            .as_ref()
            .map(|id| id.replace('-', ""))
    }

    fn conversion_action_resource(&self) -> String {
        format!(
            "customers/{}/conversionActions/{}",
            self.normalized_customer_id(),
            self.conversion_action_id
        )
    }
}

/// SHA-256 hash of a normalized email address (lowercase + trim).
///
/// Google requires emails to be hashed before upload for Enhanced Conversions.
pub fn hash_email(email: &str) -> String {
    let normalized = email.trim().to_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    hex::encode(digest)
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

/// Client wrapping the Google Ads API. Caches the OAuth access token until it expires.
pub struct GoogleAdsClient {
    http: reqwest::Client,
    config: GoogleAdsConfig,
    cached_token: Mutex<Option<CachedToken>>,
    base_url: String,
    oauth_url: String,
}

impl GoogleAdsClient {
    pub fn new(config: GoogleAdsConfig) -> Result<Self, GoogleAdsError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;
        Ok(Self {
            http,
            config,
            cached_token: Mutex::new(None),
            base_url: GOOGLE_ADS_BASE_URL.to_string(),
            oauth_url: GOOGLE_OAUTH_TOKEN_URL.to_string(),
        })
    }

    /// Override the API endpoints (used by integration tests with a mock server).
    #[doc(hidden)]
    pub fn with_endpoints(mut self, base_url: String, oauth_url: String) -> Self {
        self.base_url = base_url;
        self.oauth_url = oauth_url;
        self
    }

    async fn get_access_token(&self) -> Result<String, GoogleAdsError> {
        let mut guard = self.cached_token.lock().await;
        if let Some(cached) = guard.as_ref() {
            if cached.is_fresh() {
                debug!("Using cached Google OAuth access token");
                return Ok(cached.value.clone());
            }
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

    /// Upload a single click conversion with Enhanced Conversions for Leads.
    pub async fn upload_click_conversion(
        &self,
        gclid: &str,
        email: &str,
        conversion_date_time: DateTime<Utc>,
    ) -> Result<UploadResponse, GoogleAdsError> {
        let access_token = self.get_access_token().await?;
        let headers = self.build_headers(&access_token)?;

        let url = format!(
            "{}/customers/{}:uploadClickConversions",
            self.base_url,
            self.config.normalized_customer_id()
        );

        // Format: "YYYY-MM-DD HH:MM:SS+00:00" (Google's expected format)
        let formatted_dt = conversion_date_time
            .format("%Y-%m-%d %H:%M:%S%:z")
            .to_string();

        let body = serde_json::json!({
            "conversions": [{
                "gclid": gclid,
                "conversionAction": self.config.conversion_action_resource(),
                "conversionDateTime": formatted_dt,
                "userIdentifiers": [{
                    "hashedEmail": hash_email(email),
                    "userIdentifierSource": "FIRST_PARTY"
                }]
            }],
            "partialFailure": true,
            "validateOnly": false
        });

        debug!(target: "acquisition::google_ads", url = %url, "uploading click conversion");
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

        let parsed: UploadResponse =
            serde_json::from_str(&response_body).map_err(|e| GoogleAdsError::Api {
                status: status.as_u16(),
                body: format!("invalid JSON: {e}: {response_body}"),
            })?;
        Ok(parsed)
    }
}

/// Subset of Google Ads response we surface to the caller.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    #[serde(default)]
    pub partial_failure_error: Option<serde_json::Value>,
    #[serde(default)]
    pub results: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_hash_normalizes_then_sha256() {
        // sha256("user@example.com") = b4c9a289...
        let h = hash_email("  USER@example.com  ");
        assert_eq!(
            h,
            "b4c9a289323b21a01c3e940f150eb9b8c542587f1abfd8f0e1cc1ffc5e475514"
        );
    }

    #[test]
    fn customer_id_is_normalized() {
        let cfg = GoogleAdsConfig {
            developer_token: "t".into(),
            customer_id: "123-456-7890".into(),
            login_customer_id: Some("987-654-3210".into()),
            conversion_action_id: "42".into(),
            oauth_client_id: "c".into(),
            oauth_client_secret: "s".into(),
            oauth_refresh_token: "r".into(),
        };
        assert_eq!(cfg.normalized_customer_id(), "1234567890");
        assert_eq!(
            cfg.normalized_login_customer_id().as_deref(),
            Some("9876543210")
        );
        assert_eq!(
            cfg.conversion_action_resource(),
            "customers/1234567890/conversionActions/42"
        );
    }
}
