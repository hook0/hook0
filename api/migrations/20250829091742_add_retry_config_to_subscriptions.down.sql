-- Remove retry configuration columns from subscription table
ALTER TABLE webhook.subscription
DROP CONSTRAINT IF EXISTS subscription_retry_config_is_object;

ALTER TABLE webhook.subscription
DROP COLUMN retry_config;

-- Remove retry configuration columns from application table
ALTER TABLE event.application
DROP CONSTRAINT IF EXISTS application_retry_config_is_object;

ALTER TABLE event.application
DROP COLUMN retry_config;