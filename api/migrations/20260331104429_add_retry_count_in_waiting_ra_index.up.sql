create index if not exists request_attempt_waiting_temp_idx on webhook.request_attempt (created_at asc, retry_count asc nulls first) where (succeeded_at is null and failed_at is null);
drop index webhook.request_attempt_waiting_idx;
alter index webhook.request_attempt_waiting_temp_idx rename to request_attempt_waiting_idx;
