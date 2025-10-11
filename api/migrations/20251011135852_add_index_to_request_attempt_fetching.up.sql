create index if not exists request_attempt_created_at_idx on webhook.request_attempt using brin (created_at, subscription__id, event__id) with (autosummarize = on, pages_per_range = 50);
