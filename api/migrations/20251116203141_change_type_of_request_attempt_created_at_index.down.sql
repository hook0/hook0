create index request_attempt_created_at_brin_idx on webhook.request_attempt using brin (created_at, subscription__id, event__id) with (autosummarize = on, pages_per_range = 50);
alter index webhook.request_attempt_created_at_idx rename to request_attempt_created_at_btree_idx;
alter index webhook.request_attempt_created_at_brin_idx rename to request_attempt_created_at_idx;
drop index webhook.request_attempt_created_at_btree_idx;
