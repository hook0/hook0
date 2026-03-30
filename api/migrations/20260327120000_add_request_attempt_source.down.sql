-- Restore original covering index without source column
drop index if exists webhook.idx_request_attempt_sub_health;
create index if not exists idx_request_attempt_sub_health
    on webhook.request_attempt (subscription__id, created_at desc)
    include (succeeded_at, failed_at);

drop index if exists webhook.request_attempt_user__id_idx;
alter table webhook.request_attempt drop constraint if exists request_attempt_source_user_check;
alter table webhook.request_attempt drop column if exists user__id;
alter table webhook.request_attempt drop column if exists source;
