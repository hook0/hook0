alter table webhook.request_attempt alter column application__id drop not null;
drop index webhook.request_attempt_application__id_idx;
