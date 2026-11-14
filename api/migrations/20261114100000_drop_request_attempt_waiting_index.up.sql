-- On large instances, run this statement by hand before deploying (the matching statement below then does
-- nothing), so that dropping the index does not block writes on webhook.request_attempt:
--
--   drop index concurrently if exists webhook.request_attempt_waiting_idx;
--
-- This index held every pending request attempt, paused or not, sorted by creation date. It was kept for the
-- output workers that did not know about the paused column yet. Every query on pending request attempts now
-- says whether it wants paused ones, and is served by a narrower index: request_attempt_due_idx,
-- request_attempt_subscription_waiting_idx or request_attempt_paused_idx.

drop index if exists webhook.request_attempt_waiting_idx;
