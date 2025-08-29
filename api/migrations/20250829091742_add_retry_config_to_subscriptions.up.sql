-- Add retry configuration columns to subscription table
ALTER TABLE webhook.subscription
ADD COLUMN retry_config jsonb NOT NULL DEFAULT jsonb_build_object(
    'max_fast_retries', 30,
    'max_slow_retries', 30,
    'fast_retry_delay_seconds', 5,
    'max_fast_retry_delay_seconds', 300,
    'slow_retry_delay_seconds', 3600
);

-- Add a check constraint to ensure retry_config is a valid JSON object
ALTER TABLE webhook.subscription
ADD CONSTRAINT subscription_retry_config_is_object 
CHECK ((retry_config IS NULL) OR (jsonb_typeof(retry_config) = 'object'));

-- Add comment to explain the retry configuration structure
COMMENT ON COLUMN webhook.subscription.retry_config IS 'Retry configuration for this subscription. Contains max_fast_retries, max_slow_retries, fast_retry_delay_seconds, max_fast_retry_delay_seconds, and slow_retry_delay_seconds';