-- On large instances, run these statements by hand before deploying (the matching statements below then do
-- nothing), so that building and dropping the indexes does not block writes on webhook.request_attempt:
--
--   create index concurrently if not exists request_attempt_application__id_created_at_idx on webhook.request_attempt (application__id, created_at);
--   drop index concurrently if exists webhook.request_attempt_application__id_idx;
--
-- Listing an application's request attempts filters on application__id and orders by created_at. With one
-- index on each column, Postgres either read all of the application's request attempts, or walked the
-- created_at index through every application's request attempts. The composite index serves that listing
-- directly, and its leading column serves everything the application__id index did (foreign key cascade,
-- per-organization lookups), which is why that index is dropped.

create index if not exists request_attempt_application__id_created_at_idx on webhook.request_attempt (application__id, created_at);
drop index if exists webhook.request_attempt_application__id_idx;
