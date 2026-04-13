DROP INDEX IF EXISTS webhook.idx_request_attempt_completed_at;
DROP INDEX IF EXISTS webhook.idx_subscription_health_event_cleanup;
DROP INDEX IF EXISTS webhook.idx_subscription_health_bucket_start;
DROP INDEX IF EXISTS webhook.idx_subscription_health_bucket_open;
DROP INDEX IF EXISTS webhook.idx_subscription_health_event_sub_id;
DROP TABLE IF EXISTS webhook.health_monitor_cursor CASCADE;
DROP TABLE IF EXISTS webhook.subscription_health_bucket;
DROP TABLE IF EXISTS webhook.subscription_health_event;
