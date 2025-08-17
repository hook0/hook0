-- Rollback retry schedule system

-- Remove indexes
DROP INDEX IF EXISTS webhook.subscription_auto_disable_idx;
DROP INDEX IF EXISTS webhook.subscription_failure_tracking_idx;
DROP INDEX IF EXISTS webhook.subscription_retry_schedule_idx;

-- Remove columns from subscription table
ALTER TABLE webhook.subscription 
DROP COLUMN IF EXISTS auto_disabled_at,
DROP COLUMN IF EXISTS first_failure_at,
DROP COLUMN IF EXISTS consecutive_failures,
DROP COLUMN IF EXISTS last_failure_at,
DROP COLUMN IF EXISTS retry_schedule__id;

-- Drop retry schedule table
DROP TABLE IF EXISTS webhook.retry_schedule;