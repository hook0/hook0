-- Rollback FIFO mode support

DROP INDEX IF EXISTS webhook.idx_subscription_fifo_mode;

ALTER TABLE webhook.subscription
DROP COLUMN IF EXISTS fifo_mode;
