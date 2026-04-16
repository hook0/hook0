-- '-infinity' is valid in PostgreSQL but panics in sqlx (chrono overflow).
-- Replace with epoch so the cursor is representable in Rust.
update webhook.subscription_health_monitor_cursor
set last_processed_at = '1970-01-01T00:00:00Z'
where last_processed_at = '-infinity';

alter table webhook.subscription_health_monitor_cursor
    alter column last_processed_at set default '1970-01-01T00:00:00Z';
