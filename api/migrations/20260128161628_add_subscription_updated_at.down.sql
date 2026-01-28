-- Remove trigger
DROP TRIGGER IF EXISTS subscription_updated_at_trigger ON webhook.subscription;

-- Remove trigger function
DROP FUNCTION IF EXISTS webhook.update_subscription_updated_at();

-- Remove updated_at column
ALTER TABLE webhook.subscription DROP COLUMN IF EXISTS updated_at;
