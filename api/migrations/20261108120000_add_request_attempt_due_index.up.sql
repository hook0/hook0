-- On large instances, run this statement by hand before deploying (the matching statement below then does
-- nothing), so that building the index does not block writes on webhook.request_attempt:
--
--   create index concurrently if not exists request_attempt_due_idx on webhook.request_attempt ((coalesce(delay_until, created_at)), retry_count nulls first) where (not paused and succeeded_at is null and failed_at is null);
--
-- Output workers pick request attempts in the order they become due. Ordering by creation date made each pick
-- walk every pending request attempt that is not due yet (mostly retries scheduled for later) before reaching
-- the due ones; this index only holds pending request attempts, sorted by the time they become due.
-- The indexed expression must stay identical to the one used by the pick query in output-worker/src/pg.rs.

create index if not exists request_attempt_due_idx on webhook.request_attempt ((coalesce(delay_until, created_at)), retry_count nulls first) where (not paused and succeeded_at is null and failed_at is null);
