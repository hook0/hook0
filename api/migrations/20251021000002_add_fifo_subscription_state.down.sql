-- Rollback FIFO subscription state tracking table

DROP TABLE IF EXISTS webhook.fifo_subscription_state CASCADE;
