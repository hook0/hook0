#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::authentication::{
        config::{
            AuthenticationConfigRequest, AuthenticationType, BasicAuthConfig,
            BearerTokenConfig, OAuth2Config, OAuth2GrantType,
        },
        encryption::SecretEncryption,
        service::AuthenticationService,
    };
    use sqlx::{PgPool, postgres::PgPoolOptions};
    use std::env;
    use uuid::Uuid;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path, body_string_contains};
    
    /// Create a test database pool
    async fn create_test_pool() -> PgPool {
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://hook0:hook0@localhost/hook0_test".to_string());
        
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool")
    }
    
    /// Run migrations for testing
    async fn run_migrations(pool: &PgPool) {
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }
    
    /// Create test application and user
    async fn create_test_app(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
        // Create organization
        let org_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO webhook.organization (organization__id, name)
            VALUES ($1, $2)
            "#,
            org_id,
            "Test Org"
        )
        .execute(pool)
        .await
        .unwrap();
        
        // Create user
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO webhook.user (user__id, email, first_name, last_name)
            VALUES ($1, $2, $3, $4)
            "#,
            user_id,
            "test@example.com",
            "Test",
            "User"
        )
        .execute(pool)
        .await
        .unwrap();
        
        // Create application
        let app_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO webhook.application (application__id, organization__id, name)
            VALUES ($1, $2, $3)
            "#,
            app_id,
            org_id,
            "Test App"
        )
        .execute(pool)
        .await
        .unwrap();
        
        (app_id, user_id, org_id)
    }
    
    #[tokio::test]
    async fn test_secret_encryption() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        let encryption = SecretEncryption::new(pool.clone()).unwrap();
        
        let plaintext = "my-secret-password";
        let (ciphertext, nonce) = encryption.encrypt(plaintext).unwrap();
        
        // Verify encryption produces different output
        assert_ne!(plaintext, ciphertext);
        
        // Verify decryption works
        let decrypted = encryption.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(plaintext, decrypted);
    }
    
    #[tokio::test]
    async fn test_oauth2_token_acquisition() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, user_id, _) = create_test_app(&pool).await;
        
        // Start mock OAuth server
        let mock_server = MockServer::start().await;
        
        // Setup OAuth token endpoint mock
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=client_credentials"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token",
                "token_type": "Bearer",
                "expires_in": 3600,
                "scope": "read write"
            })))
            .mount(&mock_server)
            .await;
        
        // Create OAuth2 configuration
        let oauth_config = OAuth2Config {
            grant_type: OAuth2GrantType::ClientCredentials,
            client_id: "test-client-id".to_string(),
            client_secret: "test-client-secret".to_string(),
            token_endpoint: format!("{}/oauth/token", mock_server.uri()),
            scopes: Some(vec!["read".to_string(), "write".to_string()]),
            token_refresh_threshold: 300,
            custom_headers: None,
        };
        
        let auth_request = AuthenticationConfigRequest {
            auth_type: AuthenticationType::OAuth2,
            config: serde_json::to_value(oauth_config).unwrap(),
        };
        
        // Create authentication service
        let auth_service = AuthenticationService::new(pool.clone()).unwrap();
        
        // Save authentication configuration
        let config_id = auth_service
            .save_authentication_config(app_id, None, auth_request, user_id)
            .await
            .unwrap();
        
        assert_ne!(config_id, Uuid::nil());
        
        // Get provider and verify it works
        let provider = auth_service
            .get_provider(app_id, None)
            .await
            .unwrap()
            .expect("Provider should exist");
        
        // Create a mock request
        let client = reqwest::Client::new();
        let mut request = client
            .get("https://api.example.com/test")
            .build()
            .unwrap();
        
        // Apply authentication
        provider.authenticate(&mut request).await.unwrap();
        
        // Verify Authorization header was added
        let auth_header = request.headers().get("Authorization").unwrap();
        assert_eq!(auth_header.to_str().unwrap(), "Bearer test-access-token");
    }
    
    #[tokio::test]
    async fn test_bearer_token_authentication() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        env::set_var("TEST_BEARER_TOKEN", "my-bearer-token");
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, user_id, _) = create_test_app(&pool).await;
        
        // Create Bearer configuration
        let bearer_config = BearerTokenConfig {
            token: "env://TEST_BEARER_TOKEN".to_string(),
            header_name: "Authorization".to_string(),
            prefix: "Bearer".to_string(),
        };
        
        let auth_request = AuthenticationConfigRequest {
            auth_type: AuthenticationType::Bearer,
            config: serde_json::to_value(bearer_config).unwrap(),
        };
        
        // Create authentication service
        let auth_service = AuthenticationService::new(pool.clone()).unwrap();
        
        // Save authentication configuration
        auth_service
            .save_authentication_config(app_id, None, auth_request, user_id)
            .await
            .unwrap();
        
        // Get provider
        let provider = auth_service
            .get_provider(app_id, None)
            .await
            .unwrap()
            .expect("Provider should exist");
        
        // Create a mock request
        let client = reqwest::Client::new();
        let mut request = client
            .get("https://api.example.com/test")
            .build()
            .unwrap();
        
        // Apply authentication
        provider.authenticate(&mut request).await.unwrap();
        
        // Verify Authorization header was added
        let auth_header = request.headers().get("Authorization").unwrap();
        assert_eq!(auth_header.to_str().unwrap(), "Bearer my-bearer-token");
    }
    
    #[tokio::test]
    async fn test_basic_authentication() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, user_id, _) = create_test_app(&pool).await;
        
        // Create Basic auth configuration
        let basic_config = BasicAuthConfig {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };
        
        let auth_request = AuthenticationConfigRequest {
            auth_type: AuthenticationType::Basic,
            config: serde_json::to_value(basic_config).unwrap(),
        };
        
        // Create authentication service
        let auth_service = AuthenticationService::new(pool.clone()).unwrap();
        
        // Save authentication configuration
        auth_service
            .save_authentication_config(app_id, None, auth_request, user_id)
            .await
            .unwrap();
        
        // Get provider
        let provider = auth_service
            .get_provider(app_id, None)
            .await
            .unwrap()
            .expect("Provider should exist");
        
        // Create a mock request
        let client = reqwest::Client::new();
        let mut request = client
            .get("https://api.example.com/test")
            .build()
            .unwrap();
        
        // Apply authentication
        provider.authenticate(&mut request).await.unwrap();
        
        // Verify Authorization header was added with Basic auth
        let auth_header = request.headers().get("Authorization").unwrap();
        let auth_str = auth_header.to_str().unwrap();
        assert!(auth_str.starts_with("Basic "));
        
        // Decode and verify credentials
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
        let encoded = &auth_str[6..];
        let decoded = String::from_utf8(BASE64.decode(encoded).unwrap()).unwrap();
        assert_eq!(decoded, "testuser:testpass");
    }
    
    #[tokio::test]
    async fn test_subscription_override() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, user_id, _) = create_test_app(&pool).await;
        
        // Create subscription
        let sub_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO webhook.subscription (
                subscription__id,
                application__id,
                description,
                is_enabled
            ) VALUES ($1, $2, $3, $4)
            "#,
            sub_id,
            app_id,
            "Test Subscription",
            true
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Create application-level Bearer configuration
        let app_bearer_config = BearerTokenConfig {
            token: "app-token".to_string(),
            header_name: "Authorization".to_string(),
            prefix: "Bearer".to_string(),
        };
        
        let app_auth_request = AuthenticationConfigRequest {
            auth_type: AuthenticationType::Bearer,
            config: serde_json::to_value(app_bearer_config).unwrap(),
        };
        
        // Create subscription-level Basic configuration
        let sub_basic_config = BasicAuthConfig {
            username: "subuser".to_string(),
            password: "subpass".to_string(),
        };
        
        let sub_auth_request = AuthenticationConfigRequest {
            auth_type: AuthenticationType::Basic,
            config: serde_json::to_value(sub_basic_config).unwrap(),
        };
        
        // Create authentication service
        let auth_service = AuthenticationService::new(pool.clone()).unwrap();
        
        // Save application authentication
        auth_service
            .save_authentication_config(app_id, None, app_auth_request, user_id)
            .await
            .unwrap();
        
        // Save subscription authentication override
        auth_service
            .save_authentication_config(app_id, Some(sub_id), sub_auth_request, user_id)
            .await
            .unwrap();
        
        // Get provider for subscription (should use Basic auth)
        let sub_provider = auth_service
            .get_provider(app_id, Some(sub_id))
            .await
            .unwrap()
            .expect("Subscription provider should exist");
        
        assert_eq!(sub_provider.get_type(), AuthenticationType::Basic);
        
        // Get provider for application (should use Bearer)
        let app_provider = auth_service
            .get_provider(app_id, None)
            .await
            .unwrap()
            .expect("Application provider should exist");
        
        assert_eq!(app_provider.get_type(), AuthenticationType::Bearer);
    }
    
    #[tokio::test]
    async fn test_encrypted_secret_storage() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, _, _) = create_test_app(&pool).await;
        
        let encryption = SecretEncryption::new(pool.clone()).unwrap();
        
        // Store a secret
        let secret_id = encryption
            .store_encrypted_secret(
                &app_id,
                "api-key",
                "super-secret-key",
                Some(serde_json::json!({"rotation": "monthly"})),
            )
            .await
            .unwrap();
        
        assert_ne!(secret_id, Uuid::nil());
        
        // Retrieve and verify the secret
        let retrieved = encryption
            .get_encrypted_secret(&app_id, "api-key")
            .await
            .unwrap();
        
        assert_eq!(retrieved, "super-secret-key");
        
        // Rotate the secret
        encryption
            .rotate_secret(&app_id, "api-key", "new-secret-key")
            .await
            .unwrap();
        
        // Verify rotated secret
        let rotated = encryption
            .get_encrypted_secret(&app_id, "api-key")
            .await
            .unwrap();
        
        assert_eq!(rotated, "new-secret-key");
        
        // Delete the secret
        encryption
            .delete_secret(&app_id, "api-key")
            .await
            .unwrap();
        
        // Verify deletion
        let result = encryption
            .get_encrypted_secret(&app_id, "api-key")
            .await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_authentication_audit_logging() {
        env::set_var("HOOK0_ENCRYPTION_KEY", SecretEncryption::generate_master_key());
        
        let pool = create_test_pool().await;
        run_migrations(&pool).await;
        let (app_id, _, _) = create_test_app(&pool).await;
        
        let auth_service = AuthenticationService::new(pool.clone()).unwrap();
        
        // Create subscription
        let sub_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO webhook.subscription (
                subscription__id,
                application__id,
                description,
                is_enabled
            ) VALUES ($1, $2, $3, $4)
            "#,
            sub_id,
            app_id,
            "Test Subscription",
            true
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Log successful authentication
        auth_service
            .log_authentication(
                Some(sub_id),
                None,
                AuthenticationType::OAuth2,
                true,
                None,
                Some(serde_json::json!({"client_id": "test-client"})),
            )
            .await
            .unwrap();
        
        // Log failed authentication
        auth_service
            .log_authentication(
                Some(sub_id),
                None,
                AuthenticationType::OAuth2,
                false,
                Some("Invalid credentials".to_string()),
                None,
            )
            .await
            .unwrap();
        
        // Verify audit logs were created
        let logs = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM auth.authentication_audit_log
            WHERE subscription__id = $1
            "#,
            sub_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(logs.count.unwrap(), 2);
        
        // Verify success/failure counts
        let success_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM auth.authentication_audit_log
            WHERE subscription__id = $1 AND is_success = true
            "#,
            sub_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(success_count.count.unwrap(), 1);
    }
}