-- Drop triggers
drop trigger if exists subscription_operational_changes on webhook.subscription;

-- Drop functions
drop function if exists webhook.subscription_operational_trigger();
drop function if exists webhook.trigger_operational_event(uuid, text, jsonb);
drop function if exists webhook.update_message_stats();

-- Drop columns from subscription table
alter table webhook.subscription drop column if exists consecutive_failures;
alter table webhook.subscription drop column if exists last_failure_at;
alter table webhook.subscription drop column if exists auto_disabled_at;

-- Drop tables in reverse order of dependencies
drop table if exists webhook.message_stats;
drop table if exists webhook.operational_attempt;
drop table if exists webhook.operational_event;
drop table if exists webhook.operational_event_type;
drop table if exists webhook.operational_endpoint;