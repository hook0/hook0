ALTER TABLE webhook.subscription
ADD COLUMN updated_at timestamptz NOT NULL DEFAULT statement_timestamp();

-- Backfill: set updated_at = created_at for existing rows
UPDATE webhook.subscription SET updated_at = created_at;
