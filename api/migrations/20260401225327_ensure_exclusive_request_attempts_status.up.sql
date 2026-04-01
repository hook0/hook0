alter table webhook.request_attempt add constraint request_attempt_status_exclusive check (not (succeeded_at is not null and failed_at is not null));
