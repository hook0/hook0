-- Add endpoint health notification tracking
-- This table tracks which notifications have been sent to avoid duplicate emails

CREATE TABLE webhook.endpoint_health_notification (
    notification__id UUID NOT NULL DEFAULT gen_random_uuid(),
    subscription__id UUID NOT NULL,
    notification_type TEXT NOT NULL CHECK (notification_type IN ('warning_3_days', 'disabled_5_days', 'recovered')),
    sent_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    details JSONB,
    
    CONSTRAINT endpoint_health_notification_pkey PRIMARY KEY (notification__id)
);

-- Add foreign key constraint to subscription
ALTER TABLE webhook.endpoint_health_notification
ADD CONSTRAINT endpoint_health_notification_subscription__id_fkey
FOREIGN KEY (subscription__id)
REFERENCES webhook.subscription(subscription__id)
ON DELETE CASCADE;

-- Create indexes for efficient queries
CREATE INDEX notification_subscription_idx ON webhook.endpoint_health_notification(subscription__id, notification_type);
CREATE INDEX notification_sent_at_idx ON webhook.endpoint_health_notification(sent_at);

-- Note: We rely on application logic to prevent duplicate notifications within the same day
-- A unique index per subscription and notification type would be too restrictive

-- Add comment for documentation
COMMENT ON TABLE webhook.endpoint_health_notification IS 'Tracks notifications sent for endpoint health events';
COMMENT ON COLUMN webhook.endpoint_health_notification.notification_type IS 'Type of notification: warning_3_days, disabled_5_days, or recovered';
COMMENT ON COLUMN webhook.endpoint_health_notification.details IS 'Additional details about the notification (failure count, error types, etc.)';

-- Create table for operational webhooks configuration
CREATE TABLE webhook.operational_webhook_config (
    config__id UUID NOT NULL DEFAULT gen_random_uuid(),
    organization__id UUID NOT NULL,
    event_type TEXT NOT NULL CHECK (event_type IN ('endpoint.disabled', 'endpoint.warning', 'message.attempt.exhausted', 'endpoint.recovered')),
    target_url TEXT NOT NULL,
    headers JSONB DEFAULT '{}',
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    
    CONSTRAINT operational_webhook_config_pkey PRIMARY KEY (config__id),
    CONSTRAINT operational_webhook_config_unique UNIQUE (organization__id, event_type)
);

-- Add foreign key constraint to organization
ALTER TABLE webhook.operational_webhook_config
ADD CONSTRAINT operational_webhook_config_organization__id_fkey
FOREIGN KEY (organization__id)
REFERENCES iam.organization(organization__id)
ON DELETE CASCADE;

-- Create index for efficient queries
CREATE INDEX operational_webhook_config_org_idx ON webhook.operational_webhook_config(organization__id)
WHERE is_enabled = true;

COMMENT ON TABLE webhook.operational_webhook_config IS 'Configuration for operational webhooks (system events)';
COMMENT ON COLUMN webhook.operational_webhook_config.event_type IS 'Type of operational event to subscribe to';