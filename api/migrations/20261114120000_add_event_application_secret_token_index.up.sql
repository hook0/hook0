-- On large instances, run this statement by hand before deploying (the matching statement below then does
-- nothing), so that building the index does not block writes on event.event:
--
--   create index concurrently if not exists event_application_secret__token_idx on event.event (application_secret__token) where application_secret__token is not null;
--
-- If it fails or is cancelled, it leaves an invalid index under this name that the statement below would skip:
-- drop it (drop index concurrently if exists event.event_application_secret__token_idx) and run it again.
--
-- Deleting an application secret makes Postgres check event_application_secret__token_fkey, i.e. look for events
-- that still reference the secret. With no index on that column, each check read the whole event.event table, so
-- the soft-deleted applications cleanup read it once per secret it purged. Events no longer record the secret
-- they were sent with, so the column is null on almost every row: this partial index only holds the older events
-- that have one, and new events never enter it.

create index if not exists event_application_secret__token_idx on event.event (application_secret__token) where application_secret__token is not null;
