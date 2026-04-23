-- Drop subscription FK + index before table (dependency order).
drop index if exists webhook.subscription_retry_schedule__id_idx;
alter table webhook.subscription drop constraint if exists subscription_retry_schedule__id_fkey;
alter table webhook.subscription drop column if exists retry_schedule__id;

drop index if exists webhook.retry_schedule_organization__id_idx;
drop table if exists webhook.retry_schedule;
