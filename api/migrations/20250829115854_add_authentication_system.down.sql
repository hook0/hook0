-- Remove authentication columns from subscription table
ALTER TABLE webhook.subscription 
    DROP COLUMN IF EXISTS authentication;

-- Remove authentication columns from application table
ALTER TABLE webhook.application 
    DROP COLUMN IF EXISTS default_authentication_config,
    DROP COLUMN IF EXISTS default_authentication_type;

-- Drop triggers
DROP TRIGGER IF EXISTS update_authentication_config_updated_at ON auth.authentication_config;
DROP TRIGGER IF EXISTS update_encrypted_secret_updated_at ON auth.encrypted_secret;

-- Drop function
DROP FUNCTION IF EXISTS auth.update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS auth.idx_auth_config_app;
DROP INDEX IF EXISTS auth.idx_auth_config_sub;
DROP INDEX IF EXISTS auth.idx_auth_config_active;
DROP INDEX IF EXISTS auth.idx_oauth_cache_expires;
DROP INDEX IF EXISTS auth.idx_oauth_cache_config;
DROP INDEX IF EXISTS auth.idx_auth_audit_created;
DROP INDEX IF EXISTS auth.idx_auth_audit_subscription;
DROP INDEX IF EXISTS auth.idx_encrypted_secret_app_name;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS auth.authentication_audit_log;
DROP TABLE IF EXISTS auth.oauth_token_cache;
DROP TABLE IF EXISTS auth.encrypted_secret;
DROP TABLE IF EXISTS auth.authentication_config;
DROP TABLE IF EXISTS auth.secret_provider_type;
DROP TABLE IF EXISTS auth.authentication_type;

-- Drop schema
DROP SCHEMA IF EXISTS auth;