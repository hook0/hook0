-- On large instances, run this statement by hand before deploying (the matching statement below then does
-- nothing), so that dropping the index does not block writes on webhook.request_attempt:
--
--   drop index concurrently if exists webhook.request_attempt_waiting_unpaused_idx;
--
-- This index held the pending request attempts that are not paused, sorted by creation date. Output workers
-- now read them in the order they become due, through request_attempt_due_idx, both to pick them (PostgreSQL
-- queue) and to load them into Pulsar at startup. Nothing reads them by creation date anymore.

drop index if exists webhook.request_attempt_waiting_unpaused_idx;
