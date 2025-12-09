create index if not exists request_attempt_application__id_idx on webhook.request_attempt (application__id);
alter table webhook.request_attempt alter column application__id set not null;
