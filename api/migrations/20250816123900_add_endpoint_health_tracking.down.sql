-- Rollback endpoint health tracking

-- Drop operational webhook config table
DROP TABLE IF EXISTS webhook.operational_webhook_config;

-- Drop endpoint health notification table  
DROP TABLE IF EXISTS webhook.endpoint_health_notification;