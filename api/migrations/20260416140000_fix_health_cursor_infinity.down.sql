alter table webhook.subscription_health_monitor_cursor
    alter column last_processed_at set default '-infinity';

update webhook.subscription_health_monitor_cursor
set last_processed_at = '-infinity'
where last_processed_at = '1970-01-01T00:00:00Z';
