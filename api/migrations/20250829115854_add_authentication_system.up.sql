-- Create auth schema
CREATE SCHEMA IF NOT EXISTS auth;

-- Create authentication type reference table
CREATE TABLE auth.authentication_type (
    authentication_type__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE CHECK (name IN ('oauth2', 'bearer', 'certificate', 'basic', 'custom')),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default authentication types
INSERT INTO auth.authentication_type (name, description) VALUES
    ('oauth2', 'OAuth 2.0 authentication with automatic token refresh'),
    ('bearer', 'Static bearer token authentication'),
    ('certificate', 'TLS client certificate authentication'),
    ('basic', 'Basic HTTP authentication'),
    ('custom', 'Custom authentication implementation');

-- Create secret provider type reference table
CREATE TABLE auth.secret_provider_type (
    secret_provider_type__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE CHECK (name IN ('env', 'encrypted')),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default secret provider types
INSERT INTO auth.secret_provider_type (name, description) VALUES
    ('env', 'Environment variable reference'),
    ('encrypted', 'Encrypted value stored in database');

-- Create authentication configuration table
CREATE TABLE auth.authentication_config (
    authentication_config__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application__id UUID NOT NULL,
    subscription__id UUID,
    authentication_type__id UUID NOT NULL,
    config JSONB NOT NULL, -- Stores authentication-specific configuration
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    
    -- Foreign keys
    CONSTRAINT authentication_config_application_fk 
        FOREIGN KEY (application__id) 
        REFERENCES webhook.application(application__id) 
        ON DELETE CASCADE 
        ON UPDATE CASCADE,
    
    CONSTRAINT authentication_config_subscription_fk 
        FOREIGN KEY (subscription__id) 
        REFERENCES webhook.subscription(subscription__id) 
        ON DELETE CASCADE 
        ON UPDATE CASCADE,
    
    CONSTRAINT authentication_config_auth_type_fk 
        FOREIGN KEY (authentication_type__id) 
        REFERENCES auth.authentication_type(authentication_type__id) 
        ON DELETE RESTRICT 
        ON UPDATE CASCADE,
    
    CONSTRAINT authentication_config_created_by_fk 
        FOREIGN KEY (created_by) 
        REFERENCES webhook.user(user__id) 
        ON DELETE SET NULL 
        ON UPDATE CASCADE,
    
    -- Ensure only one default config per application
    CONSTRAINT authentication_config_app_unique 
        UNIQUE(application__id) WHERE subscription__id IS NULL,
    
    -- Ensure only one config per subscription
    CONSTRAINT authentication_config_sub_unique 
        UNIQUE(subscription__id) WHERE subscription__id IS NOT NULL
);

COMMENT ON TABLE auth.authentication_config IS 'Stores authentication configurations for applications and subscriptions';
COMMENT ON COLUMN auth.authentication_config.authentication_config__id IS 'Primary key of the authentication configuration';
COMMENT ON COLUMN auth.authentication_config.application__id IS 'Reference to the application this configuration belongs to';
COMMENT ON COLUMN auth.authentication_config.subscription__id IS 'Optional reference to a specific subscription (overrides application default)';
COMMENT ON COLUMN auth.authentication_config.authentication_type__id IS 'Type of authentication (oauth2, bearer, certificate, basic, custom)';
COMMENT ON COLUMN auth.authentication_config.config IS 'JSON configuration specific to the authentication type';
COMMENT ON COLUMN auth.authentication_config.is_active IS 'Whether this configuration is currently active';
COMMENT ON COLUMN auth.authentication_config.created_at IS 'Timestamp when the configuration was created';
COMMENT ON COLUMN auth.authentication_config.updated_at IS 'Timestamp when the configuration was last updated';
COMMENT ON COLUMN auth.authentication_config.created_by IS 'User who created this configuration';

-- Create encrypted secrets table
CREATE TABLE auth.encrypted_secret (
    encrypted_secret__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application__id UUID NOT NULL,
    name TEXT NOT NULL,
    encrypted_value TEXT NOT NULL, -- AES-256-GCM encrypted
    nonce TEXT NOT NULL, -- Encryption nonce
    metadata JSONB, -- Additional metadata (rotation policy, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    
    -- Foreign keys
    CONSTRAINT encrypted_secret_application_fk 
        FOREIGN KEY (application__id) 
        REFERENCES webhook.application(application__id) 
        ON DELETE CASCADE 
        ON UPDATE CASCADE,
    
    -- Unique name per application
    CONSTRAINT encrypted_secret_app_name_unique 
        UNIQUE(application__id, name)
);

COMMENT ON TABLE auth.encrypted_secret IS 'Stores encrypted secrets for authentication';
COMMENT ON COLUMN auth.encrypted_secret.encrypted_secret__id IS 'Primary key of the encrypted secret';
COMMENT ON COLUMN auth.encrypted_secret.application__id IS 'Application that owns this secret';
COMMENT ON COLUMN auth.encrypted_secret.name IS 'Secret name for reference';
COMMENT ON COLUMN auth.encrypted_secret.encrypted_value IS 'AES-256-GCM encrypted secret value';
COMMENT ON COLUMN auth.encrypted_secret.nonce IS 'Encryption nonce for this secret';
COMMENT ON COLUMN auth.encrypted_secret.metadata IS 'Additional metadata like rotation policy';
COMMENT ON COLUMN auth.encrypted_secret.rotated_at IS 'Last rotation timestamp';

-- Create OAuth token cache table
CREATE TABLE auth.oauth_token_cache (
    oauth_token_cache__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    authentication_config__id UUID NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    scopes TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT oauth_cache_config_fk 
        FOREIGN KEY (authentication_config__id) 
        REFERENCES auth.authentication_config(authentication_config__id) 
        ON DELETE CASCADE 
        ON UPDATE CASCADE,
    
    -- Only one cached token per config
    CONSTRAINT oauth_cache_config_unique 
        UNIQUE(authentication_config__id)
);

COMMENT ON TABLE auth.oauth_token_cache IS 'Caches OAuth2 tokens to avoid repeated token requests';
COMMENT ON COLUMN auth.oauth_token_cache.oauth_token_cache__id IS 'Primary key of the token cache entry';
COMMENT ON COLUMN auth.oauth_token_cache.authentication_config__id IS 'Configuration this token belongs to';
COMMENT ON COLUMN auth.oauth_token_cache.access_token IS 'OAuth2 access token';
COMMENT ON COLUMN auth.oauth_token_cache.refresh_token IS 'OAuth2 refresh token for obtaining new access tokens';
COMMENT ON COLUMN auth.oauth_token_cache.expires_at IS 'When the access token expires';
COMMENT ON COLUMN auth.oauth_token_cache.scopes IS 'OAuth2 scopes granted for this token';

-- Create authentication audit log table
CREATE TABLE auth.authentication_audit_log (
    authentication_audit_log__id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription__id UUID,
    request_attempt__id UUID,
    authentication_type TEXT NOT NULL,
    is_success BOOLEAN NOT NULL,
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT auth_audit_subscription_fk 
        FOREIGN KEY (subscription__id) 
        REFERENCES webhook.subscription(subscription__id) 
        ON DELETE SET NULL 
        ON UPDATE CASCADE,
    
    CONSTRAINT auth_audit_request_attempt_fk 
        FOREIGN KEY (request_attempt__id) 
        REFERENCES webhook.request_attempt(request_attempt__id) 
        ON DELETE CASCADE 
        ON UPDATE CASCADE
);

COMMENT ON TABLE auth.authentication_audit_log IS 'Audit log for all authentication attempts';
COMMENT ON COLUMN auth.authentication_audit_log.authentication_audit_log__id IS 'Primary key of the audit entry';
COMMENT ON COLUMN auth.authentication_audit_log.subscription__id IS 'Subscription that was authenticated';
COMMENT ON COLUMN auth.authentication_audit_log.request_attempt__id IS 'Request attempt that triggered authentication';
COMMENT ON COLUMN auth.authentication_audit_log.authentication_type IS 'Type of authentication used';
COMMENT ON COLUMN auth.authentication_audit_log.is_success IS 'Whether authentication succeeded';
COMMENT ON COLUMN auth.authentication_audit_log.error_message IS 'Error details if authentication failed';
COMMENT ON COLUMN auth.authentication_audit_log.metadata IS 'Additional context about the authentication attempt';

-- Create indexes for performance
CREATE INDEX idx_auth_config_app ON auth.authentication_config(application__id);
CREATE INDEX idx_auth_config_sub ON auth.authentication_config(subscription__id) WHERE subscription__id IS NOT NULL;
CREATE INDEX idx_auth_config_active ON auth.authentication_config(is_active) WHERE is_active = true;
CREATE INDEX idx_oauth_cache_expires ON auth.oauth_token_cache(expires_at);
CREATE INDEX idx_oauth_cache_config ON auth.oauth_token_cache(authentication_config__id);
CREATE INDEX idx_auth_audit_created ON auth.authentication_audit_log(created_at);
CREATE INDEX idx_auth_audit_subscription ON auth.authentication_audit_log(subscription__id);
CREATE INDEX idx_encrypted_secret_app_name ON auth.encrypted_secret(application__id, name);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION auth.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at
CREATE TRIGGER update_authentication_config_updated_at
    BEFORE UPDATE ON auth.authentication_config
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at_column();

CREATE TRIGGER update_encrypted_secret_updated_at
    BEFORE UPDATE ON auth.encrypted_secret
    FOR EACH ROW
    EXECUTE FUNCTION auth.update_updated_at_column();

-- Add authentication columns to application table
ALTER TABLE webhook.application 
    ADD COLUMN default_authentication_config JSONB,
    ADD COLUMN default_authentication_type TEXT;

COMMENT ON COLUMN webhook.application.default_authentication_config IS 'Default authentication configuration for subscriptions';
COMMENT ON COLUMN webhook.application.default_authentication_type IS 'Default authentication type for quick reference';

-- Add authentication column to subscription table
ALTER TABLE webhook.subscription 
    ADD COLUMN authentication JSONB;

COMMENT ON COLUMN webhook.subscription.authentication IS 'Override authentication configuration for this subscription';