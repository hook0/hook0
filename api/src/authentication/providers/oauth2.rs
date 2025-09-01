use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::authentication::{
    config::{AuthenticationType, OAuth2Config, OAuth2GrantType, OAuthTokenCache},
    encryption::SecretEncryption,
};

use super::AuthenticationProvider;

/// OAuth2 token response
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    scope: Option<String>,
}

/// OAuth2 authentication provider
#[allow(dead_code)]
pub struct OAuth2Provider {
    config: OAuth2Config,
    config_id: Uuid,
    application_id: Uuid,
    token_cache: Arc<RwLock<Option<CachedToken>>>,
    http_client: Client,
    db_pool: PgPool,
    encryption: Arc<SecretEncryption>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedToken {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: DateTime<Utc>,
    scopes: Vec<String>,
}

#[allow(dead_code)]
impl OAuth2Provider {
    /// Create a new OAuth2 provider
    pub async fn new(
        config: OAuth2Config,
        config_id: Uuid,
        application_id: Uuid,
        db_pool: PgPool,
        encryption: Arc<SecretEncryption>,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let mut provider = Self {
            config,
            config_id,
            application_id,
            token_cache: Arc::new(RwLock::new(None)),
            http_client,
            db_pool,
            encryption,
        };

        // Try to load cached token from database
        provider.load_cached_token().await?;

        Ok(provider)
    }

    /// Load cached token from database
    async fn load_cached_token(&mut self) -> Result<()> {
        let cached = sqlx::query_as::<_, OAuthTokenCache>(
            r#"
            SELECT 
                oauth_token_cache__id,
                authentication_config__id,
                access_token,
                refresh_token,
                expires_at,
                scopes,
                created_at
            FROM auth.oauth_token_cache
            WHERE authentication_config__id = $1
            "#,
        )
        .bind(self.config_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(cache_entry) = cached {
            let token = CachedToken {
                access_token: cache_entry.access_token,
                refresh_token: cache_entry.refresh_token,
                expires_at: cache_entry.expires_at,
                scopes: cache_entry.scopes.unwrap_or_default(),
            };

            *self.token_cache.write().await = Some(token);
        }

        Ok(())
    }

    /// Save token to cache and database
    async fn save_token(&self, token_response: &TokenResponse) -> Result<()> {
        let expires_at = if let Some(expires_in) = token_response.expires_in {
            Utc::now() + Duration::seconds(expires_in)
        } else {
            // Default to 1 hour if not specified
            Utc::now() + Duration::hours(1)
        };

        let scopes: Vec<String> = token_response
            .scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let cached_token = CachedToken {
            access_token: token_response.access_token.clone(),
            refresh_token: token_response.refresh_token.clone(),
            expires_at,
            scopes: scopes.clone(),
        };

        // Update in-memory cache
        *self.token_cache.write().await = Some(cached_token);

        // Save to database
        sqlx::query(
            r#"
            INSERT INTO auth.oauth_token_cache (
                authentication_config__id,
                access_token,
                refresh_token,
                expires_at,
                scopes
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (authentication_config__id)
            DO UPDATE SET
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                expires_at = EXCLUDED.expires_at,
                scopes = EXCLUDED.scopes,
                created_at = NOW()
            "#,
        )
        .bind(self.config_id)
        .bind(&token_response.access_token)
        .bind(&token_response.refresh_token)
        .bind(expires_at)
        .bind(&scopes)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Acquire a new access token
    async fn acquire_token(&self) -> Result<TokenResponse> {
        // Resolve client secret
        let client_secret = self
            .encryption
            .resolve_secret(&self.config.client_secret, &self.application_id)
            .await?;

        let mut params = vec![
            ("client_id", self.config.client_id.clone()),
            ("client_secret", client_secret),
        ];

        match self.config.grant_type {
            OAuth2GrantType::ClientCredentials => {
                params.push(("grant_type", "client_credentials".to_string()));
            }
            OAuth2GrantType::Password => {
                return Err(anyhow!("Password grant type not yet implemented"));
            }
            OAuth2GrantType::AuthorizationCode => {
                return Err(anyhow!("Authorization code grant type not yet implemented"));
            }
        }

        // Add scopes if specified
        if let Some(scopes) = &self.config.scopes {
            params.push(("scope", scopes.join(" ")));
        }

        let mut request = self
            .http_client
            .post(&self.config.token_endpoint)
            .form(&params);

        // Add custom headers if specified
        if let Some(headers) = &self.config.custom_headers {
            if let Some(headers_obj) = headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Failed to acquire OAuth2 token: {} - {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response.json().await?;
        self.save_token(&token_response).await?;

        Ok(token_response)
    }

    /// Refresh the access token using refresh token
    async fn refresh_token(&self) -> Result<TokenResponse> {
        let cache = self.token_cache.read().await;
        let refresh_token = cache
            .as_ref()
            .and_then(|t| t.refresh_token.clone())
            .ok_or_else(|| anyhow!("No refresh token available"))?;
        drop(cache);

        // Resolve client secret
        let client_secret = self
            .encryption
            .resolve_secret(&self.config.client_secret, &self.application_id)
            .await?;

        let params = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh_token),
            ("client_id", self.config.client_id.clone()),
            ("client_secret", client_secret),
        ];

        let mut request = self
            .http_client
            .post(&self.config.token_endpoint)
            .form(&params);

        // Add custom headers if specified
        if let Some(headers) = &self.config.custom_headers {
            if let Some(headers_obj) = headers.as_object() {
                for (key, value) in headers_obj {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            // If refresh fails, try to acquire a new token
            return self.acquire_token().await;
        }

        let token_response: TokenResponse = response.json().await?;
        self.save_token(&token_response).await?;

        Ok(token_response)
    }

    /// Get a valid access token (refresh if needed)
    async fn get_valid_token(&self) -> Result<String> {
        let cache = self.token_cache.read().await;

        if let Some(cached_token) = cache.as_ref() {
            let refresh_threshold = Duration::seconds(self.config.token_refresh_threshold as i64);
            let should_refresh = cached_token.expires_at - Utc::now() < refresh_threshold;

            if !should_refresh {
                return Ok(cached_token.access_token.clone());
            }
        }

        drop(cache);

        // Need to refresh or acquire new token
        if self
            .token_cache
            .read()
            .await
            .as_ref()
            .and_then(|t| t.refresh_token.as_ref())
            .is_some()
        {
            // Try to refresh first
            match self.refresh_token().await {
                Ok(token_response) => Ok(token_response.access_token),
                Err(_) => {
                    // If refresh fails, acquire new token
                    self.acquire_token().await.map(|t| t.access_token)
                }
            }
        } else {
            // No refresh token, acquire new token
            self.acquire_token().await.map(|t| t.access_token)
        }
    }
}

#[async_trait]
impl AuthenticationProvider for OAuth2Provider {
    async fn authenticate(&self, request: &mut Request) -> Result<()> {
        let token = self.get_valid_token().await?;

        // Add Authorization header
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .map_err(|e| anyhow!("Invalid token: {}", e))?,
        );

        Ok(())
    }

    async fn refresh_if_needed(&self) -> Result<()> {
        let cache = self.token_cache.read().await;

        if let Some(cached_token) = cache.as_ref() {
            let refresh_threshold = Duration::seconds(self.config.token_refresh_threshold as i64);
            let should_refresh = cached_token.expires_at - Utc::now() < refresh_threshold;

            if should_refresh {
                drop(cache);
                self.get_valid_token().await?;
            }
        } else {
            drop(cache);
            self.get_valid_token().await?;
        }

        Ok(())
    }

    fn get_type(&self) -> AuthenticationType {
        AuthenticationType::OAuth2
    }

    fn needs_refresh(&self) -> bool {
        // Try to get a read lock without blocking
        if let Ok(cache) = self.token_cache.try_read() {
            if let Some(cached_token) = cache.as_ref() {
                let refresh_threshold =
                    Duration::seconds(self.config.token_refresh_threshold as i64);
                cached_token.expires_at - Utc::now() < refresh_threshold
            } else {
                true
            }
        } else {
            // If we can't get the lock, assume we need refresh to be safe
            true
        }
    }
}
