create index if not exists request_attempt_waiting_idx on webhook.request_attempt (created_at) where (succeeded_at is null and failed_at is null);
