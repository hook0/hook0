-- Drop old check constraints referencing source
alter table webhook.subscription_health_event
    drop constraint subscription_health_event_source_check;
alter table webhook.subscription_health_event
    drop constraint subscription_health_event_source_user_check;

-- Rename column
alter table webhook.subscription_health_event
    rename column source to cause;

-- Migrate existing data
update webhook.subscription_health_event
    set cause = case cause
        when 'system' then 'auto'
        when 'user' then 'manual'
    end;

-- Add new check constraints with the new naming
alter table webhook.subscription_health_event
    add constraint subscription_health_event_cause_check
    check (cause in ('auto', 'manual'));
alter table webhook.subscription_health_event
    add constraint subscription_health_event_cause_user_check
    check (cause != 'auto' or user__id is null);

-- Recreate the index that included source as a covering column
drop index if exists webhook.idx_subscription_health_event_sub_id;
create index idx_subscription_health_event_sub_id
    on webhook.subscription_health_event (subscription__id, created_at desc)
    include (status, cause);
