use anyhow::{anyhow, Result};
use reqwest::{Client, Request};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    config::{
        AuthenticationConfig, AuthenticationConfigRequest, AuthenticationType,
        BasicAuthConfig, BearerTokenConfig, CertificateConfig, OAuth2Config,
    },
    encryption::SecretEncryption,
    providers::{
        basic::BasicAuthProvider,
        bearer::BearerTokenProvider,
        certificate::{CertificateProvider, create_client_with_certificates},
        oauth2::OAuth2Provider,
        AuthenticationProvider,
    },
};

/// Main authentication service
pub struct AuthenticationService {
    db_pool: PgPool,
    encryption: Arc<SecretEncryption>,
    providers: Arc<RwLock<HashMap<Uuid, Arc<Box<dyn AuthenticationProvider>>>>>,
    http_clients: Arc<RwLock<HashMap<Uuid, Client>>>,
}

impl AuthenticationService {
    /// Create a new authentication service
    pub fn new(db_pool: PgPool) -> Result<Self> {
        let encryption = Arc::new(SecretEncryption::new(db_pool.clone())?);
        
        Ok(Self {
            db_pool,
            encryption,
            providers: Arc::new(RwLock::new(HashMap::new())),
            http_clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get or create an authentication provider for a subscription
    pub async fn get_provider(
        &self,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
    ) -> Result<Option<Arc<Box<dyn AuthenticationProvider>>>> {
        // First check if we have a cached provider
        if let Some(sub_id) = subscription_id {
            let providers = self.providers.read().await;
            if let Some(provider) = providers.get(&sub_id) {
                return Ok(Some(provider.clone()));
            }
        }
        
        // Load configuration from database
        let config = self.load_authentication_config(application_id, subscription_id).await?;
        
        if let Some(config) = config {
            let provider = self.create_provider(config, application_id).await?;
            
            // Cache the provider
            if let Some(sub_id) = subscription_id {
                let mut providers = self.providers.write().await;
                providers.insert(sub_id, provider.clone());
            }
            
            Ok(Some(provider))
        } else {
            Ok(None)
        }
    }
    
    /// Load authentication configuration from database
    async fn load_authentication_config(
        &self,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
    ) -> Result<Option<AuthenticationConfig>> {
        // First try to load subscription-specific config
        if let Some(sub_id) = subscription_id {
            let config = sqlx::query_as!(
                AuthenticationConfig,
                r#"
                SELECT 
                    ac.authentication_config__id,
                    ac.application__id,
                    ac.subscription__id,
                    ac.authentication_type__id,
                    ac.config,
                    ac.is_active,
                    ac.created_at,
                    ac.updated_at,
                    ac.created_by
                FROM auth.authentication_config ac
                WHERE ac.subscription__id = $1 AND ac.is_active = true
                "#,
                sub_id
            )
            .fetch_optional(&self.db_pool)
            .await?;
            
            if config.is_some() {
                return Ok(config);
            }
        }
        
        // Fall back to application default
        let config = sqlx::query_as!(
            AuthenticationConfig,
            r#"
            SELECT 
                ac.authentication_config__id,
                ac.application__id,
                ac.subscription__id,
                ac.authentication_type__id,
                ac.config,
                ac.is_active,
                ac.created_at,
                ac.updated_at,
                ac.created_by
            FROM auth.authentication_config ac
            WHERE ac.application__id = $1 
                AND ac.subscription__id IS NULL 
                AND ac.is_active = true
            "#,
            application_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        Ok(config)
    }
    
    /// Create a provider from configuration
    async fn create_provider(
        &self,
        config: AuthenticationConfig,
        application_id: Uuid,
    ) -> Result<Arc<Box<dyn AuthenticationProvider>>> {
        // Get authentication type name
        let auth_type = sqlx::query!(
            r#"
            SELECT name 
            FROM auth.authentication_type 
            WHERE authentication_type__id = $1
            "#,
            config.authentication_type__id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let auth_type = AuthenticationType::from_str(&auth_type.name)
            .ok_or_else(|| anyhow!("Unknown authentication type: {}", auth_type.name))?;
        
        let provider: Box<dyn AuthenticationProvider> = match auth_type {
            AuthenticationType::OAuth2 => {
                let oauth_config: OAuth2Config = serde_json::from_value(config.config)?;
                Box::new(
                    OAuth2Provider::new(
                        oauth_config,
                        config.authentication_config__id,
                        application_id,
                        self.db_pool.clone(),
                        self.encryption.clone(),
                    )
                    .await?,
                )
            }
            AuthenticationType::Bearer => {
                let bearer_config: BearerTokenConfig = serde_json::from_value(config.config)?;
                Box::new(BearerTokenProvider::new(
                    bearer_config,
                    application_id,
                    self.encryption.clone(),
                ))
            }
            AuthenticationType::Basic => {
                let basic_config: BasicAuthConfig = serde_json::from_value(config.config)?;
                Box::new(BasicAuthProvider::new(
                    basic_config,
                    application_id,
                    self.encryption.clone(),
                ))
            }
            AuthenticationType::Certificate => {
                let cert_config: CertificateConfig = serde_json::from_value(config.config)?;
                let provider = CertificateProvider::new(
                    cert_config,
                    application_id,
                    self.encryption.clone(),
                )
                .await?;
                
                // Create and cache a special HTTP client for certificate auth
                let client = create_client_with_certificates(&provider)?;
                let mut clients = self.http_clients.write().await;
                clients.insert(config.authentication_config__id, client);
                
                Box::new(provider)
            }
            AuthenticationType::Custom => {
                return Err(anyhow!("Custom authentication not yet implemented"));
            }
        };
        
        Ok(Arc::new(provider))
    }
    
    /// Apply authentication to an HTTP request
    pub async fn authenticate_request(
        &self,
        request: &mut Request,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
    ) -> Result<()> {
        if let Some(provider) = self.get_provider(application_id, subscription_id).await? {
            provider.authenticate(request).await?;
        }
        
        Ok(())
    }
    
    /// Get HTTP client for a specific configuration (needed for certificate auth)
    pub async fn get_http_client(
        &self,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
    ) -> Result<Option<Client>> {
        // Check if we have a special client (for certificate auth)
        if let Some(config) = self.load_authentication_config(application_id, subscription_id).await? {
            let clients = self.http_clients.read().await;
            if let Some(client) = clients.get(&config.authentication_config__id) {
                return Ok(Some(client.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Save or update authentication configuration
    pub async fn save_authentication_config(
        &self,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
        request: AuthenticationConfigRequest,
        created_by: Uuid,
    ) -> Result<Uuid> {
        // Get authentication type ID
        let auth_type = sqlx::query!(
            r#"
            SELECT authentication_type__id 
            FROM auth.authentication_type 
            WHERE name = $1
            "#,
            request.auth_type.as_str()
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Validate configuration based on type
        match request.auth_type {
            AuthenticationType::OAuth2 => {
                let _: OAuth2Config = serde_json::from_value(request.config.clone())?;
            }
            AuthenticationType::Bearer => {
                let _: BearerTokenConfig = serde_json::from_value(request.config.clone())?;
            }
            AuthenticationType::Basic => {
                let _: BasicAuthConfig = serde_json::from_value(request.config.clone())?;
            }
            AuthenticationType::Certificate => {
                let _: CertificateConfig = serde_json::from_value(request.config.clone())?;
            }
            AuthenticationType::Custom => {
                // Custom validation if needed
            }
        }
        
        // Insert or update configuration
        let result = sqlx::query!(
            r#"
            INSERT INTO auth.authentication_config (
                application__id,
                subscription__id,
                authentication_type__id,
                config,
                created_by
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (application__id) WHERE subscription__id IS NULL
            DO UPDATE SET
                authentication_type__id = EXCLUDED.authentication_type__id,
                config = EXCLUDED.config,
                updated_at = NOW()
            ON CONFLICT (subscription__id) WHERE subscription__id IS NOT NULL
            DO UPDATE SET
                authentication_type__id = EXCLUDED.authentication_type__id,
                config = EXCLUDED.config,
                updated_at = NOW()
            RETURNING authentication_config__id
            "#,
            application_id,
            subscription_id,
            auth_type.authentication_type__id,
            request.config,
            created_by
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Clear cached provider if updating
        if let Some(sub_id) = subscription_id {
            let mut providers = self.providers.write().await;
            providers.remove(&sub_id);
        }
        
        Ok(result.authentication_config__id)
    }
    
    /// Delete authentication configuration
    pub async fn delete_authentication_config(
        &self,
        application_id: Uuid,
        subscription_id: Option<Uuid>,
    ) -> Result<()> {
        if let Some(sub_id) = subscription_id {
            sqlx::query!(
                r#"
                DELETE FROM auth.authentication_config
                WHERE subscription__id = $1
                "#,
                sub_id
            )
            .execute(&self.db_pool)
            .await?;
            
            // Clear cached provider
            let mut providers = self.providers.write().await;
            providers.remove(&sub_id);
        } else {
            sqlx::query!(
                r#"
                DELETE FROM auth.authentication_config
                WHERE application__id = $1 AND subscription__id IS NULL
                "#,
                application_id
            )
            .execute(&self.db_pool)
            .await?;
        }
        
        Ok(())
    }
    
    /// Log authentication attempt
    pub async fn log_authentication(
        &self,
        subscription_id: Option<Uuid>,
        request_attempt_id: Option<Uuid>,
        auth_type: AuthenticationType,
        is_success: bool,
        error_message: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO auth.authentication_audit_log (
                subscription__id,
                request_attempt__id,
                authentication_type,
                is_success,
                error_message,
                metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            subscription_id,
            request_attempt_id,
            auth_type.as_str(),
            is_success,
            error_message,
            metadata
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
}