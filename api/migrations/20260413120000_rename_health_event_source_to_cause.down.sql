drop index if exists webhook.idx_subscription_health_event_sub_id;

alter table webhook.subscription_health_event
    drop constraint subscription_health_event_cause_check;
alter table webhook.subscription_health_event
    drop constraint subscription_health_event_cause_user_check;

update webhook.subscription_health_event
    set cause = case cause
        when 'auto' then 'system'
        when 'manual' then 'user'
    end;

alter table webhook.subscription_health_event
    rename column cause to source;

alter table webhook.subscription_health_event
    add constraint subscription_health_event_source_check
    check (source in ('system', 'user'));
alter table webhook.subscription_health_event
    add constraint subscription_health_event_source_user_check
    check (source != 'system' or user__id is null);

create index idx_subscription_health_event_sub_id
    on webhook.subscription_health_event (subscription__id, created_at desc)
    include (status, source);
