-- Restore the original single-column index
create index if not exists request_attempt_subscription__id_idx
    on webhook.request_attempt (subscription__id);

drop index if exists webhook.idx_request_attempt_sub_health;
drop index if exists webhook.idx_subscription_health_event_sub_id;
drop table if exists webhook.subscription_health_event;
