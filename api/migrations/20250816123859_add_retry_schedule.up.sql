-- Add retry schedule system for webhook delivery management
-- This migration adds support for configurable retry policies similar to Svix/Stripe

-- Create retry schedule table to store different retry policies
CREATE TABLE webhook.retry_schedule (
    retry_schedule__id UUID NOT NULL DEFAULT gen_random_uuid(),
    organization__id UUID NOT NULL,
    name TEXT NOT NULL CHECK (length(name) >= 1),
    strategy TEXT NOT NULL CHECK (strategy IN ('exponential', 'linear', 'custom')),
    intervals INTEGER[] NOT NULL CHECK (array_length(intervals, 1) > 0),
    max_attempts INTEGER NOT NULL DEFAULT 8 CHECK (max_attempts > 0 AND max_attempts <= 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    
    CONSTRAINT retry_schedule_pkey PRIMARY KEY (retry_schedule__id),
    CONSTRAINT retry_schedule_org_name_unique UNIQUE (organization__id, name)
);

-- Add foreign key constraint to iam.organization
ALTER TABLE webhook.retry_schedule 
ADD CONSTRAINT retry_schedule_organization__id_fkey
FOREIGN KEY (organization__id) 
REFERENCES iam.organization(organization__id)
ON DELETE CASCADE;

-- Add retry schedule reference to subscription table
ALTER TABLE webhook.subscription 
ADD COLUMN retry_schedule__id UUID REFERENCES webhook.retry_schedule(retry_schedule__id);

-- Add failure tracking columns to subscription table
ALTER TABLE webhook.subscription
ADD COLUMN last_failure_at TIMESTAMPTZ,
ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0,
ADD COLUMN first_failure_at TIMESTAMPTZ,
ADD COLUMN auto_disabled_at TIMESTAMPTZ;

-- Create indexes for performance
CREATE INDEX subscription_retry_schedule_idx ON webhook.subscription(retry_schedule__id) 
WHERE retry_schedule__id IS NOT NULL;

CREATE INDEX subscription_failure_tracking_idx ON webhook.subscription(last_failure_at, consecutive_failures) 
WHERE is_enabled = true AND deleted_at IS NULL;

CREATE INDEX subscription_auto_disable_idx ON webhook.subscription(first_failure_at, consecutive_failures)
WHERE is_enabled = true AND deleted_at IS NULL AND first_failure_at IS NOT NULL;

-- Add comment for documentation
COMMENT ON TABLE webhook.retry_schedule IS 'Stores configurable retry policies for webhook delivery';
COMMENT ON COLUMN webhook.subscription.retry_schedule__id IS 'Optional custom retry schedule for this subscription';
COMMENT ON COLUMN webhook.subscription.last_failure_at IS 'Timestamp of the last failed delivery attempt';
COMMENT ON COLUMN webhook.subscription.consecutive_failures IS 'Number of consecutive failures since last success';
COMMENT ON COLUMN webhook.subscription.first_failure_at IS 'Timestamp of the first failure in the current failure sequence';
COMMENT ON COLUMN webhook.subscription.auto_disabled_at IS 'Timestamp when the subscription was automatically disabled due to continuous failures';

-- Create default retry schedules for organizations (optional, can be done in application logic)
-- These are examples that match the Svix/Stripe model
INSERT INTO webhook.retry_schedule (organization__id, name, strategy, intervals, max_attempts)
SELECT 
    o.organization__id,
    'Default Exponential Backoff' as name,
    'exponential' as strategy,
    ARRAY[5, 300, 1800, 7200, 18000, 36000, 36000, 36000] as intervals, -- 5s, 5m, 30m, 2h, 5h, 10h, 10h, 10h
    8 as max_attempts
FROM iam.organization o
WHERE NOT EXISTS (
    SELECT 1 FROM webhook.retry_schedule rs 
    WHERE rs.organization__id = o.organization__id 
    AND rs.name = 'Default Exponential Backoff'
);