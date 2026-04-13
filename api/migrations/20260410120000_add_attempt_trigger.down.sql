drop index if exists webhook.triggered_by_idx;
alter table webhook.request_attempt drop column if exists triggered_by;
alter table webhook.request_attempt drop constraint if exists trigger_chk;
alter table webhook.request_attempt drop column if exists attempt_trigger;
