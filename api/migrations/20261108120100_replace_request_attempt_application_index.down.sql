create index if not exists request_attempt_application__id_idx on webhook.request_attempt (application__id);
drop index if exists webhook.request_attempt_application__id_created_at_idx;
