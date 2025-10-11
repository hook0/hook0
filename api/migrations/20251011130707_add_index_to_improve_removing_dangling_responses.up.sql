create index if not exists request_attempt_no_response on webhook.request_attempt (response__id) where response__id is null;
