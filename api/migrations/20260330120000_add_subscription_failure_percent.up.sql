-- Denormalized failure_percent for frontend display. Computed by the health monitor from
-- subscription_health_bucket and synced here for efficient API reads (avoids joining
-- buckets on every subscription list/get).
ALTER TABLE webhook.subscription
    ADD COLUMN failure_percent double precision;
