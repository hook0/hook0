# Specification: Advanced Authentication for Hook0

## Overview

This specification describes the implementation of an advanced authentication system for Hook0, supporting different authentication mechanisms (OAuth2, Bearer tokens, TLS certificates, etc.) configurable at the application level with the possibility of override at each subscription level. All secrets are stored encrypted within Hook0's database, ensuring complete control over sensitive authentication data.

## Business Context

Hook0 must be able to integrate with third-party endpoints requiring different types of authentication. The system must be flexible enough to handle varied use cases while remaining simple to use. All secrets are stored encrypted in Hook0's own database, eliminating dependencies on external secret management systems while maintaining security.

## 1. REST API

### 1.1 Authentication configuration integrated with application

#### Creating/Updating application with authentication

```http
PUT /api/v1/applications/{appId}
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "name": "My Application",
  "description": "Application description",
  "authentication": {
    "type": "oauth2|bearer|certificate|basic|custom",
    "config": {
      // Configuration specific to type
    }
  },
  // other application attributes...
}
```

#### OAuth2 Configuration

```json
{
  "type": "oauth2",
  "config": {
    "grant_type": "client_credentials|authorization_code|password",
    "client_id": "string",
    "client_secret": "string|env_reference|encrypted_value",
    "token_endpoint": "https://auth.example.com/oauth/token",
    "scopes": ["scope1", "scope2"],
    "token_refresh_threshold": 300, // seconds before expiration
    "custom_headers": {
      "X-Custom-Header": "value"
    }
  }
}
```

#### Bearer Token Configuration

```json
{
  "type": "bearer",
  "config": {
    "token": "string|env_reference|encrypted_value",
    "header_name": "Authorization", // Optional, default: Authorization
    "prefix": "Bearer" // Optional, default: Bearer
  }
}
```

#### TLS Certificate Configuration

```json
{
  "type": "certificate",
  "config": {
    "client_cert": "env_reference|encrypted_value",
    "client_key": "env_reference|encrypted_value",
    "ca_cert": "env_reference|encrypted_value", // Optional
    "verify_hostname": true,
    "mtls": true
  }
}
```

#### Basic Auth Configuration

```json
{
  "type": "basic",
  "config": {
    "username": "string",
    "password": "string|env_reference|encrypted_value"
  }
}
```

### 1.3 Secret Storage Approaches

Hook0 supports two approaches for storing sensitive authentication data:

#### Environment Reference (`env_reference`)

Environment references allow you to store sensitive values as environment variables on the Hook0 server. This approach is useful for configuration management and keeping secrets out of the database entirely.

**Format**: `env://VARIABLE_NAME`

**Examples**:
```json
{
  "type": "oauth2",
  "config": {
    "client_id": "my_client_id",
    "client_secret": "env://OAUTH_CLIENT_SECRET",
    "token_endpoint": "https://auth.example.com/oauth/token"
  }
}
```

```json
{
  "type": "bearer",
  "config": {
    "token": "env://API_TOKEN"
  }
}
```

**Benefits**:
- Secrets never stored in database
- Easy configuration management with deployment tools
- Supports standard environment variable practices

**Requirements**:
- Environment variables must be set on Hook0 server instances
- Variables are resolved at runtime when authentication is needed

#### Encrypted Value (`encrypted_value`)

Encrypted values are stored directly in Hook0's database using AES-256-GCM encryption with database-specific encryption keys. This approach provides complete control over secret storage without external dependencies.

**Format**: Direct string value (automatically encrypted by Hook0)

**Examples**:
```json
{
  "type": "oauth2", 
  "config": {
    "client_id": "my_client_id",
    "client_secret": "abc123secretkey456def",
    "token_endpoint": "https://auth.example.com/oauth/token"
  }
}
```

```json
{
  "type": "certificate",
  "config": {
    "client_cert": "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----",
    "client_key": "-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----"
  }
}
```

**Benefits**:
- No external dependencies on secret management systems
- Automatic encryption/decryption handled by Hook0
- Complete control over sensitive data lifecycle
- Works in all deployment scenarios

**Security Features**:
- AES-256-GCM encryption with authenticated encryption
- Unique encryption keys per database instance
- Secrets never stored in plaintext
- Automatic key rotation support

#### Usage Guidelines

- **Use `env_reference`** for:
  - Shared secrets across multiple applications
  - Configuration managed by deployment tools
  - Secrets that need external rotation policies

- **Use `encrypted_value`** for:
  - Application-specific secrets
  - TLS certificates and private keys
  - Self-contained deployment scenarios
  - When external secret management is not available

#### Retrieving application with authentication

```http
GET /api/v1/applications/{appId}
Authorization: Bearer {api_key}

Response:
{
  "id": "uuid",
  "name": "My Application",
  "description": "Application description",
  "authentication": {
    "type": "oauth2",
    "config": {
      "grant_type": "client_credentials",
      "client_id": "string",
      "token_endpoint": "https://auth.example.com/oauth/token",
      "scopes": ["scope1", "scope2"]
    }
  },
  "created_at": "2025-01-29T10:00:00Z",
  "updated_at": "2025-01-29T10:00:00Z"
}
```

### 1.2 Subscription level authentication

```http
PUT /api/v1/subscriptions/{subscriptionId}
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "endpoint_url": "https://webhook.example.com",
  "authentication": {
    "type": "oauth2|bearer|certificate|basic|custom",
    "config": {
      // Specific configuration
    }
  },
  // other subscription attributes...
}
```

## 2. Database Modeling

### 2.1 PostgreSQL Schema

```sql
-- Create auth schema to segment authentication tables
CREATE SCHEMA IF NOT EXISTS auth;

-- Reference table for authentication types (replaces enum)
CREATE TABLE auth.authentication_type (
    authentication_type__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL CHECK (name IN ('oauth2', 'bearer', 'certificate', 'basic', 'custom')),
    description text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW(),
    
    CONSTRAINT authentication_type_name_key UNIQUE (name)
);

COMMENT ON TABLE auth.authentication_type IS 'Reference table for authentication types';
COMMENT ON COLUMN auth.authentication_type.authentication_type__id IS 'Primary key for authentication type';
COMMENT ON COLUMN auth.authentication_type.name IS 'Unique name of the authentication type';
COMMENT ON COLUMN auth.authentication_type.description IS 'Human readable description of the authentication type';
COMMENT ON COLUMN auth.authentication_type.is_active IS 'Whether this authentication type is currently supported';
COMMENT ON COLUMN auth.authentication_type.created_at IS 'Timestamp when the record was created';
COMMENT ON COLUMN auth.authentication_type.updated_at IS 'Timestamp when the record was last updated';

-- Reference table for secret provider types (replaces enum)
CREATE TABLE auth.secret_provider_type (
    secret_provider_type__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL CHECK (name IN ('env', 'encrypted')),
    description text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW(),
    
    CONSTRAINT secret_provider_type_name_key UNIQUE (name)
);

COMMENT ON TABLE auth.secret_provider_type IS 'Reference table for secret provider types';
COMMENT ON COLUMN auth.secret_provider_type.secret_provider_type__id IS 'Primary key for secret provider type';
COMMENT ON COLUMN auth.secret_provider_type.name IS 'Unique name of the secret provider type';
COMMENT ON COLUMN auth.secret_provider_type.description IS 'Human readable description of the secret provider';
COMMENT ON COLUMN auth.secret_provider_type.is_active IS 'Whether this secret provider is currently supported';
COMMENT ON COLUMN auth.secret_provider_type.created_at IS 'Timestamp when the record was created';
COMMENT ON COLUMN auth.secret_provider_type.updated_at IS 'Timestamp when the record was last updated';

-- Authentication configuration table
CREATE TABLE auth.authentication_config (
    authentication_config__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application__id UUID NOT NULL,
    subscription__id UUID,
    authentication_type__id UUID NOT NULL,
    config jsonb NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    updated_at timestamptz NOT NULL DEFAULT NOW(),
    created_by__user__id UUID,
    
    CONSTRAINT authentication_config_application_fk FOREIGN KEY (application__id) REFERENCES application(application__id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT authentication_config_subscription_fk FOREIGN KEY (subscription__id) REFERENCES subscription(subscription__id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT authentication_config_auth_type_fk FOREIGN KEY (authentication_type__id) REFERENCES auth.authentication_type(authentication_type__id) ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT authentication_config_created_by_fk FOREIGN KEY (created_by__user__id) REFERENCES user_account(user__id) ON DELETE SET NULL ON UPDATE CASCADE,
    
    -- Only one config per application (when subscription_id is NULL)
    CONSTRAINT authentication_config_app_unique UNIQUE (application__id) WHERE subscription__id IS NULL,
    -- Only one config per subscription (when subscription_id is NOT NULL)
    CONSTRAINT authentication_config_sub_unique UNIQUE (subscription__id) WHERE subscription__id IS NOT NULL
);

COMMENT ON TABLE auth.authentication_config IS 'Authentication configurations for applications and subscriptions';
COMMENT ON COLUMN auth.authentication_config.authentication_config__id IS 'Primary key for authentication configuration';
COMMENT ON COLUMN auth.authentication_config.application__id IS 'Foreign key to application table';
COMMENT ON COLUMN auth.authentication_config.subscription__id IS 'Foreign key to subscription table for overrides';
COMMENT ON COLUMN auth.authentication_config.authentication_type__id IS 'Foreign key to authentication type';
COMMENT ON COLUMN auth.authentication_config.config IS 'JSON configuration specific to the authentication type';
COMMENT ON COLUMN auth.authentication_config.is_active IS 'Whether this authentication configuration is active';
COMMENT ON COLUMN auth.authentication_config.created_at IS 'Timestamp when the record was created';
COMMENT ON COLUMN auth.authentication_config.updated_at IS 'Timestamp when the record was last updated';
COMMENT ON COLUMN auth.authentication_config.created_by__user__id IS 'User who created this configuration';

-- OAuth2 token cache table
CREATE TABLE auth.oauth_token_cache (
    oauth_token_cache__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    authentication_config__id UUID NOT NULL,
    access_token text NOT NULL,
    refresh_token text,
    expires_at timestamptz NOT NULL,
    scope_list text[],
    created_at timestamptz NOT NULL DEFAULT NOW(),
    
    CONSTRAINT oauth_token_cache_auth_config_fk FOREIGN KEY (authentication_config__id) REFERENCES auth.authentication_config(authentication_config__id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT oauth_token_cache_auth_config_unique UNIQUE (authentication_config__id)
);

COMMENT ON TABLE auth.oauth_token_cache IS 'Cache for OAuth2 access and refresh tokens';
COMMENT ON COLUMN auth.oauth_token_cache.oauth_token_cache__id IS 'Primary key for OAuth token cache';
COMMENT ON COLUMN auth.oauth_token_cache.authentication_config__id IS 'Foreign key to authentication configuration';
COMMENT ON COLUMN auth.oauth_token_cache.access_token IS 'OAuth2 access token';
COMMENT ON COLUMN auth.oauth_token_cache.refresh_token IS 'OAuth2 refresh token if available';
COMMENT ON COLUMN auth.oauth_token_cache.expires_at IS 'Token expiration timestamp';
COMMENT ON COLUMN auth.oauth_token_cache.scope_list IS 'List of OAuth2 scopes granted with this token';
COMMENT ON COLUMN auth.oauth_token_cache.created_at IS 'Timestamp when the record was created';

-- Authentication audit log table
CREATE TABLE auth.authentication_audit_log (
    authentication_audit_log__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription__id UUID,
    request_attempt__id UUID,
    authentication_type__id UUID NOT NULL,
    is_success boolean NOT NULL,
    error_message text,
    metadata jsonb,
    created_at timestamptz NOT NULL DEFAULT NOW(),
    
    CONSTRAINT auth_audit_log_subscription_fk FOREIGN KEY (subscription__id) REFERENCES subscription(subscription__id) ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT auth_audit_log_request_attempt_fk FOREIGN KEY (request_attempt__id) REFERENCES request_attempt(request_attempt__id) ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT auth_audit_log_auth_type_fk FOREIGN KEY (authentication_type__id) REFERENCES auth.authentication_type(authentication_type__id) ON DELETE RESTRICT ON UPDATE CASCADE
);

COMMENT ON TABLE auth.authentication_audit_log IS 'Audit log for authentication attempts';
COMMENT ON COLUMN auth.authentication_audit_log.authentication_audit_log__id IS 'Primary key for authentication audit log';
COMMENT ON COLUMN auth.authentication_audit_log.subscription__id IS 'Foreign key to subscription if applicable';
COMMENT ON COLUMN auth.authentication_audit_log.request_attempt__id IS 'Foreign key to request attempt';
COMMENT ON COLUMN auth.authentication_audit_log.authentication_type__id IS 'Foreign key to authentication type';
COMMENT ON COLUMN auth.authentication_audit_log.is_success IS 'Whether the authentication attempt was successful';
COMMENT ON COLUMN auth.authentication_audit_log.error_message IS 'Error message if authentication failed';
COMMENT ON COLUMN auth.authentication_audit_log.metadata IS 'Additional metadata about the authentication attempt';
COMMENT ON COLUMN auth.authentication_audit_log.created_at IS 'Timestamp when the audit record was created';

-- Performance indexes
CREATE INDEX authentication_config_application_idx ON auth.authentication_config(application__id);
CREATE INDEX authentication_config_subscription_idx ON auth.authentication_config(subscription__id) WHERE subscription__id IS NOT NULL;
CREATE INDEX oauth_token_cache_expires_idx ON auth.oauth_token_cache(expires_at);
CREATE INDEX authentication_audit_log_created_idx ON auth.authentication_audit_log(created_at);
CREATE INDEX authentication_audit_log_subscription_idx ON auth.authentication_audit_log(subscription__id) WHERE subscription__id IS NOT NULL;

-- Insert reference data
INSERT INTO auth.authentication_type (name, description) VALUES
    ('oauth2', 'OAuth 2.0 authentication flow'),
    ('bearer', 'Bearer token authentication'),
    ('certificate', 'TLS certificate-based authentication'),
    ('basic', 'HTTP Basic authentication'),
    ('custom', 'Custom authentication mechanism');

INSERT INTO auth.secret_provider_type (name, description) VALUES
    ('env', 'Environment variable secret provider'),
    ('encrypted', 'Internal encrypted secret storage');
```

### 2.2 Entity Relationship Diagram

```mermaid
erDiagram
    application ||--o{ auth_authentication_config : "has authentication"
    subscription ||--o| auth_authentication_config : "may override authentication"
    auth_authentication_type ||--o{ auth_authentication_config : "defines type"
    auth_authentication_config ||--o| auth_oauth_token_cache : "may have cached token"
    auth_authentication_type ||--o{ auth_authentication_audit_log : "logs attempts"
    subscription ||--o{ auth_authentication_audit_log : "has audit entries"
    request_attempt ||--o{ auth_authentication_audit_log : "logs auth attempts"
    user_account ||--o{ auth_authentication_config : "created by"
    auth_secret_provider_type ||--o{ auth_authentication_config : "reference only"

    application {
        uuid application__id PK
        text name
        text description
        timestamptz created_at
        timestamptz updated_at
    }

    subscription {
        uuid subscription__id PK
        uuid application__id FK
        text endpoint_url
        timestamptz created_at
        timestamptz updated_at
    }

    auth_authentication_type {
        uuid authentication_type__id PK
        text name UK "oauth2, bearer, certificate, basic, custom"
        text description
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
    }

    auth_secret_provider_type {
        uuid secret_provider_type__id PK
        text name UK "env, encrypted"
        text description
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
    }

    auth_authentication_config {
        uuid authentication_config__id PK
        uuid application__id FK
        uuid subscription__id FK "nullable for overrides"
        uuid authentication_type__id FK
        jsonb config "authentication configuration"
        boolean is_active
        timestamptz created_at
        timestamptz updated_at
        uuid created_by__user__id FK
    }

    auth_oauth_token_cache {
        uuid oauth_token_cache__id PK
        uuid authentication_config__id FK
        text access_token
        text refresh_token "nullable"
        timestamptz expires_at
        text[] scope_list
        timestamptz created_at
    }

    auth_authentication_audit_log {
        uuid authentication_audit_log__id PK
        uuid subscription__id FK "nullable"
        uuid request_attempt__id FK "nullable"
        uuid authentication_type__id FK
        boolean is_success
        text error_message "nullable"
        jsonb metadata
        timestamptz created_at
    }

    user_account {
        uuid user__id PK
        text email
        text name
        timestamptz created_at
    }

    request_attempt {
        uuid request_attempt__id PK
        uuid subscription__id FK
        timestamptz created_at
        text status
    }
```

## 3. Architecture and Components

### 3.1 Authentication Service

```rust
// Main service structure
pub struct AuthenticationService {
    db_pool: PgPool,
    http_client: HttpClient,
    token_cache: Arc<RwLock<TokenCache>>,
    secret_resolver: SecretResolver,
}

// Trait for different providers
pub trait AuthenticationProvider: Send + Sync {
    async fn authenticate(&self, request: &mut Request) -> Result<(), AuthError>;
    async fn refresh_if_needed(&self) -> Result<(), AuthError>;
    fn get_type(&self) -> AuthType;
}

// Specific implementations
pub struct OAuth2Provider {
    config: OAuth2Config,
    token_cache: Arc<RwLock<TokenCache>>,
}

pub struct BearerTokenProvider {
    config: BearerConfig,
}

pub struct CertificateProvider {
    config: CertificateConfig,
}
```

### 3.2 Secret Management Integration

```rust
pub struct SecretResolver {
    encryption_key: Vec<u8>,
}

impl SecretResolver {
    pub async fn resolve_secret(&self, reference: &str) -> Result<Secret, SecretError> {
        match self.parse_reference(reference)? {
            SecretReference::Environment { key } => {
                // Resolve from environment variable
                std::env::var(&key)
                    .map_err(|_| SecretError::NotFound(key))
            }
            SecretReference::Encrypted { value } => {
                // Decrypt stored value
                self.decrypt_value(&value).await
            }
            SecretReference::PlainText { value } => {
                // Direct value (for non-sensitive config)
                Ok(value)
            }
        }
    }
    
    pub async fn encrypt_secret(&self, value: &str) -> Result<String, SecretError> {
        // Encrypt sensitive values for database storage
        self.encrypt_value(value).await
    }
}

#[derive(Debug)]
pub enum SecretReference {
    Environment { key: String },
    Encrypted { value: String },
    PlainText { value: String },
}
```

## 4. Tests

### 4.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hook0_test_utils::*;
    
    #[tokio::test]
    async fn test_oauth2_token_acquisition() {
        let db_pool = setup_test_db().await;
        let auth_service = AuthenticationService::new(db_pool);
        
        // Create test OAuth2 configuration
        let config = create_test_oauth2_config(&auth_service).await;
        
        // Test token acquisition
        let token = auth_service.acquire_oauth2_token(config.authentication_config__id).await.unwrap();
        
        assert!(!token.access_token.is_empty());
        assert!(token.expires_at > Utc::now());
    }
    
    #[tokio::test]
    async fn test_oauth2_token_refresh() {
        let db_pool = setup_test_db().await;
        let auth_service = AuthenticationService::new(db_pool);
        
        // Test automatic token refresh
        let config = create_test_oauth2_config(&auth_service).await;
        let old_token = auth_service.acquire_oauth2_token(config.authentication_config__id).await.unwrap();
        
        // Simulate token expiration
        set_token_expired(&auth_service, config.authentication_config__id).await;
        
        // Refresh should be automatic
        let new_token = auth_service.get_valid_token(config.authentication_config__id).await.unwrap();
        assert_ne!(old_token.access_token, new_token.access_token);
    }
    
    #[tokio::test]
    async fn test_bearer_token_injection() {
        let db_pool = setup_test_db().await;
        let auth_service = AuthenticationService::new(db_pool);
        
        // Test bearer token injection
        let config = create_test_bearer_config(&auth_service).await;
        let mut request = create_test_request();
        
        auth_service.authenticate_request(&mut request, config.authentication_config__id).await.unwrap();
        
        assert_eq!(request.headers().get("Authorization").unwrap(), "Bearer test_token");
    }
    
    #[tokio::test]
    async fn test_certificate_authentication() {
        let db_pool = setup_test_db().await;
        let auth_service = AuthenticationService::new(db_pool);
        
        // Test certificate authentication
        let config = create_test_certificate_config(&auth_service).await;
        let mut request = create_test_request();
        
        auth_service.authenticate_request(&mut request, config.authentication_config__id).await.unwrap();
        
        // Verify client certificate is attached
        assert!(request.has_client_certificate());
    }
    
    #[tokio::test]
    async fn test_subscription_authentication_override() {
        let db_pool = setup_test_db().await;
        let auth_service = AuthenticationService::new(db_pool);
        
        // Test subscription level override
        let app_config = create_test_oauth2_config(&auth_service).await;
        let subscription = create_test_subscription().await;
        let sub_config = create_test_bearer_override_config(&auth_service, subscription.subscription__id).await;
        
        // Subscription should use its override, not the application config
        let resolved_config = auth_service.resolve_authentication_config(
            app_config.application__id,
            Some(subscription.subscription__id)
        ).await.unwrap();
        
        assert_eq!(resolved_config.authentication_config__id, sub_config.authentication_config__id);
    }
    
    #[tokio::test]
    async fn test_secret_resolution() {
        let auth_service = AuthenticationService::new_with_test_secrets().await;
        
        // Test environment variable resolution
        std::env::set_var("TEST_SECRET", "secret_value");
        let secret = auth_service.resolve_secret("env://TEST_SECRET").await.unwrap();
        assert_eq!(secret, "secret_value");
        
        // Test encrypted value resolution
        let encrypted_secret = auth_service.encrypt_secret("sensitive_value").await.unwrap();
        let resolved_secret = auth_service.resolve_secret(&format!("encrypted://{}", encrypted_secret)).await.unwrap();
        assert_eq!(resolved_secret, "sensitive_value");
    }
}
```

### 4.2 Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_oauth2_flow() {
    let test_env = setup_integration_test_env().await;
    
    // 1. Create application with OAuth2 authentication
    let application = test_env.create_application_with_oauth2_auth().await;
    
    // 2. Create subscription
    let subscription = test_env.create_subscription(application.application__id).await;
    
    // 3. Send event
    let event = test_env.create_test_event(application.application__id).await;
    test_env.dispatch_event(event).await;
    
    // 4. Verify webhook is sent with correct OAuth2 token
    let webhook_request = test_env.capture_webhook_request().await;
    assert!(webhook_request.headers().contains_key("Authorization"));
    
    let auth_header = webhook_request.headers().get("Authorization").unwrap();
    assert!(auth_header.starts_with("Bearer "));
    
    // 5. Verify automatic refresh after expiration
    test_env.advance_time_past_token_expiry().await;
    
    let second_event = test_env.create_test_event(application.application__id).await;
    test_env.dispatch_event(second_event).await;
    
    let second_webhook_request = test_env.capture_webhook_request().await;
    let second_auth_header = second_webhook_request.headers().get("Authorization").unwrap();
    
    // Token should be different (automatic refresh)
    assert_ne!(auth_header, second_auth_header);
}

#[tokio::test]
async fn test_authentication_fallback_on_failure() {
    let test_env = setup_integration_test_env().await;
    
    // Create application with OAuth2 but with invalid credentials
    let application = test_env.create_application_with_invalid_oauth2_auth().await;
    let subscription = test_env.create_subscription(application.application__id).await;
    
    // Send event
    let event = test_env.create_test_event(application.application__id).await;
    let result = test_env.dispatch_event(event).await;
    
    // Send should fail with authentication error
    assert!(result.is_err());
    
    // Verify error is logged in audit
    let audit_entries = test_env.get_authentication_audit_log(subscription.subscription__id).await;
    assert!(!audit_entries.is_empty());
    assert!(!audit_entries[0].is_success);
}

#[tokio::test]
async fn test_concurrent_oauth2_token_refresh() {
    let test_env = setup_integration_test_env().await;
    let application = test_env.create_application_with_oauth2_auth().await;
    
    // Simulate multiple simultaneous requests requiring refresh
    test_env.set_token_near_expiry(application.application__id).await;
    
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let test_env = test_env.clone();
            let app_id = application.application__id;
            tokio::spawn(async move {
                let event = test_env.create_test_event(app_id).await;
                test_env.dispatch_event(event).await
            })
        })
        .collect();
    
    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;
    
    // All should succeed
    for result in results {
        assert!(result.unwrap().is_ok());
    }
    
    // There should be only one refresh token in the database
    let token_cache = test_env.get_oauth_token_cache(application.application__id).await;
    assert_eq!(token_cache.len(), 1);
}
```

### 4.3 Performance Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_oauth2_with_cache(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let test_env = rt.block_on(setup_performance_test_env());
    let application = rt.block_on(test_env.create_application_with_oauth2_auth());
    
    // Warm up cache
    rt.block_on(async {
        test_env.authenticate_request(application.application__id).await.unwrap();
    });
    
    c.bench_function("oauth2_with_cache", |b| {
        b.to_async(&rt).iter(|| async {
            let result = test_env.authenticate_request(black_box(application.application__id)).await;
            black_box(result).unwrap()
        })
    });
}

fn bench_oauth2_without_cache(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let test_env = rt.block_on(setup_performance_test_env());
    let application = rt.block_on(test_env.create_application_with_oauth2_auth());
    
    c.bench_function("oauth2_without_cache", |b| {
        b.to_async(&rt).iter(|| async {
            // Clear cache before each measurement
            test_env.clear_token_cache(application.application__id).await;
            let result = test_env.authenticate_request(black_box(application.application__id)).await;
            black_box(result).unwrap()
        })
    });
}

fn bench_bearer_token(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let test_env = rt.block_on(setup_performance_test_env());
    let application = rt.block_on(test_env.create_application_with_bearer_auth());
    
    c.bench_function("bearer_token", |b| {
        b.to_async(&rt).iter(|| async {
            let result = test_env.authenticate_request(black_box(application.application__id)).await;
            black_box(result).unwrap()
        })
    });
}

fn bench_concurrent_authentication(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let test_env = rt.block_on(setup_performance_test_env());
    let application = rt.block_on(test_env.create_application_with_oauth2_auth());
    
    c.bench_function("concurrent_auth_10", |b| {
        b.to_async(&rt).iter(|| async {
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let test_env = test_env.clone();
                    let app_id = application.application__id;
                    tokio::spawn(async move {
                        test_env.authenticate_request(app_id).await
                    })
                })
                .collect();
                
            let results = futures::future::join_all(handles).await;
            for result in results {
                black_box(result.unwrap()).unwrap();
            }
        })
    });
}

criterion_group!(
    auth_benches,
    bench_oauth2_with_cache,
    bench_oauth2_without_cache,
    bench_bearer_token,
    bench_concurrent_authentication
);
criterion_main!(auth_benches);
```

## 5. SLO (Service Level Objectives)

### 5.1 Availability

- **Objective**: 99.9% availability of authentication service
- **Measurement**: Percentage of webhooks sent successfully with authentication

### 5.2 Performance

- **Authentication latency P50**: < 50ms
- **Authentication latency P95**: < 200ms
- **Authentication latency P99**: < 500ms
- **OAuth2 token acquisition time**: < 1s
- **Secret resolution time**: < 100ms

### 5.3 Reliability

- **Authentication success rate**: > 99.5%
- **Token refresh success rate**: > 99.9%
- **Recovery time after failure**: < 30s

## 6. Metrics and Observability

### 6.1 Prometheus Metrics

```prometheus
# Counters
hook0_auth_requests_total{type="oauth2|bearer|certificate|basic", status="success|failure"}
hook0_auth_token_refreshes_total{status="success|failure"}
hook0_auth_secret_resolutions_total{provider="env|encrypted", status="success|failure"}

# Histograms
hook0_auth_duration_seconds{type="oauth2|bearer|certificate|basic"}
hook0_auth_token_acquisition_duration_seconds
hook0_auth_secret_resolution_duration_seconds{provider="env|encrypted"}

# Gauges
hook0_auth_cached_tokens_count
hook0_auth_token_expiry_seconds{config_id="..."}
```

### 6.2 Structured Logs

```json
{
  "timestamp": "2025-01-29T10:00:00Z",
  "level": "INFO",
  "component": "authentication",
  "subscription__id": "uuid",
  "auth_type": "oauth2",
  "operation": "token_refresh",
  "duration_ms": 150,
  "is_success": true,
  "metadata": {
    "token_endpoint": "https://auth.example.com/token",
    "scope_list": ["read", "write"]
  }
}
```

### 6.3 OpenTelemetry Traces

```yaml
spans:
  - name: "webhook.send"
    children:
      - name: "auth.resolve"
        attributes:
          auth.type: "oauth2"
          auth.authentication_config__id: "uuid"
        children:
          - name: "auth.token.get"
            children:
              - name: "auth.token.cache.lookup"
              - name: "auth.token.acquire" # If cache miss
                children:
                  - name: "auth.oauth2.request"
                  - name: "auth.token.cache.store"
```

## 7. Security

### 7.1 Secret Storage

- No plaintext secrets in database
- Use environment variables for non-sensitive configuration
- Use encrypted storage for sensitive values in Hook0's database
- AES-256-GCM encryption with database-specific keys
- Automatic rotation of secrets according to policy

### 7.2 Audit and Compliance

- Logging of all authentication operations
- Traceability of secret access
- Support for mTLS for sensitive connections
- GDPR compliance for personal data

### 7.3 Isolation

- Strict isolation between applications
- No token sharing between subscriptions
- Automatic cleanup of expired tokens

## 8. Implementation Details

### 8.1 Secret Encryption Implementation

Hook0 uses AES-256-GCM for encrypting sensitive values stored in the database:

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct SecretEncryption {
    cipher: Aes256Gcm,
}

impl SecretEncryption {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        let nonce = Nonce::from_slice(b"unique nonce"); // Generate random nonce
        let ciphertext = self.cipher.encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        // Combine nonce + ciphertext and base64 encode
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);
        Ok(base64::encode(encrypted_data))
    }

    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, EncryptionError> {
        let data = base64::decode(encrypted_data)
            .map_err(|_| EncryptionError::InvalidFormat)?;
        
        if data.len() < 12 {
            return Err(EncryptionError::InvalidFormat);
        }

        let (nonce, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::InvalidFormat)
    }
}
```

### 8.2 Database Migration Strategy

When migrating from external secret management systems to Hook0's encrypted storage:

1. **Preparation Phase**:
   - Export secrets from external systems
   - Validate encryption keys are properly configured
   - Create backup of current authentication configurations

2. **Migration Phase**:
   - For each secret reference (env://, kms://, vault://):
     - Resolve the actual secret value
     - Encrypt using Hook0's encryption system
     - Update database record with encrypted value
   - Verify each migrated authentication still works

3. **Cleanup Phase**:
   - Remove dependencies on external secret systems
   - Update documentation and deployment scripts
   - Monitor authentication success rates

### 8.3 Key Management

Hook0 manages encryption keys through:

- **Key Generation**: Cryptographically secure random 32-byte keys
- **Key Storage**: Environment variables or secure key files
- **Key Rotation**: Gradual migration to new keys with dual-key support
- **Key Backup**: Encrypted backups stored separately from application data

## 9. User Documentation

### 9.1 Quick Start Guides

- OAuth2 guide with examples
- TLS certificates guide
- Secret management guide
- Migration guide

### 9.2 API Reference

Complete OpenAPI documentation with examples for each endpoint

### 9.3 SDK Support

Support in official SDKs (Rust, Go, Node.js, Python)

## 10. Estimation and Planning

### 10.1 Estimated Effort

- **Backend (Rust)**: 15-20 days
  - Core authentication: 5 days
  - OAuth2 provider: 3 days
  - Certificate provider: 2 days
  - Secret management integration: 3 days
  - Tests and documentation: 5 days

- **API**: 5 days
  - REST endpoints: 2 days
  - Validation and security: 2 days
  - OpenAPI documentation: 1 day

- **Database**: 3 days
  - Schema and migrations: 1 day
  - Optimizations: 1 day
  - Tests: 1 day

- **Frontend**: 5-7 days
  - Configuration interface: 3 days
  - Secret management: 2 days
  - Tests: 2 days

**Total estimated**: 28-35 developer days

### 10.2 Dependencies

- Encryption libraries for database storage
- OAuth2 libraries (oauth2-rs)
- TLS libraries (rustls, native-tls)

### 10.3 Identified Risks

1. **OAuth2 complexity**: Different flows and providers
2. **Performance**: Impact on webhook latency
3. **Security**: Secure secret management
4. **Compatibility**: Support for multiple providers

## 11. Acceptance Criteria

- [ ] Support OAuth2 client_credentials
- [ ] Support static Bearer tokens
- [ ] Support mutual TLS certificates
- [ ] Encrypted secret storage functional
- [ ] Override at subscription level
- [ ] Token cache with automatic refresh
- [ ] Complete audit log
- [ ] Unit tests > 80% coverage
- [ ] Passing integration tests
- [ ] Complete documentation
- [ ] Exposed Prometheus metrics
- [ ] Performance < 50ms P50 for auth

## 12. Open Questions

1. Support for other OAuth2 flows (authorization_code, PKCE)?
2. Support for specific providers (Azure AD, Okta, Auth0)?
3. Need for rate limiting on auth endpoints?
4. Secret rotation policy?
5. Support for signed webhooks (HMAC, RSA)?