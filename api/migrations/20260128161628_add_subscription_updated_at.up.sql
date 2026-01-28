-- Add updated_at column to subscription table
ALTER TABLE webhook.subscription
ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT statement_timestamp();

-- Update existing rows to have updated_at equal to created_at
UPDATE webhook.subscription SET updated_at = created_at;

-- Create trigger function to automatically update updated_at on changes
CREATE OR REPLACE FUNCTION webhook.update_subscription_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = statement_timestamp();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to call the function on UPDATE
CREATE TRIGGER subscription_updated_at_trigger
    BEFORE UPDATE ON webhook.subscription
    FOR EACH ROW
    EXECUTE FUNCTION webhook.update_subscription_updated_at();
