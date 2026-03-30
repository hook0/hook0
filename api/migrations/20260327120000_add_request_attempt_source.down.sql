alter table webhook.request_attempt drop constraint if exists request_attempt_source_user_check;
alter table webhook.request_attempt drop column if exists user__id;
alter table webhook.request_attempt drop column if exists source;
