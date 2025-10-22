-- Add FIFO mode support to subscriptions
-- When enabled, webhooks for this subscription are delivered in strict event order
-- The next webhook is only sent after the current one succeeds or exhausts all retries

ALTER TABLE webhook.subscription
ADD COLUMN fifo_mode BOOLEAN NOT NULL DEFAULT false;

-- Partial index for efficient querying of FIFO subscriptions
CREATE INDEX idx_subscription_fifo_mode
ON webhook.subscription(subscription__id)
WHERE fifo_mode = true;

COMMENT ON COLUMN webhook.subscription.fifo_mode IS
'When true, webhooks for this subscription are delivered in strict event order. The next webhook is only sent after the current one succeeds or exhausts all retries. This may significantly reduce throughput for this subscription.';
