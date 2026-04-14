alter table webhook.subscription drop column if exists failure_percent;
drop index if exists webhook.idx_request_attempt_completed_at;
drop index if exists webhook.idx_subscription_health_event_cleanup;
drop index if exists webhook.idx_subscription_health_bucket_start;
drop index if exists webhook.idx_subscription_health_bucket_open;
drop index if exists webhook.idx_subscription_health_event_sub_id;
drop table if exists webhook.subscription_health_monitor_cursor cascade;
drop table if exists webhook.subscription_health_bucket;
drop table if exists webhook.subscription_health_event;
