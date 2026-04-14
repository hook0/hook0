-- Bounds how long this migration's ALTER/CREATE operations wait for locks
-- before giving up. Without it, a slow query holding a row lock could stall
-- our ACCESS EXCLUSIVE lock acquisition and block every writer on these
-- tables for the duration of the stall. Fail fast instead — the operator
-- can retry when the contending query is gone.
set lock_timeout = '5s';

-- Health buckets aggregate delivery attempt counts (total vs failed) over
-- bounded windows. Each bucket covers a time range [bucket_start, bucket_end)
-- for a single subscription. Buckets are bounded by EITHER duration
-- (subscription_health_monitor_bucket_duration) OR message count
-- (subscription_health_monitor_bucket_max_messages), whichever limit is reached
-- first. On every tick the monitor reads recent buckets, sums their
-- failed_count / total_count over the failure-rate window, and feeds the
-- resulting rates into the state machine.
create table webhook.subscription_health_bucket (
    subscription__id uuid not null
        references webhook.subscription(subscription__id) on delete cascade,
    bucket_start     timestamptz not null,
    bucket_end       timestamptz,
    total_count      integer not null default 0 check (total_count >= 0),
    failed_count     integer not null default 0 check (failed_count >= 0 and failed_count <= total_count),
    primary key (subscription__id, bucket_start)
);

-- Speeds up the daily cleanup pass in `queries::buckets::cleanup_old_buckets`,
-- which deletes buckets where bucket_start is older than bucket_retention.
-- Without this index the delete scans the full table every time.
create index idx_subscription_health_bucket_start
    on webhook.subscription_health_bucket(bucket_start);

-- Partial index limited to open buckets — `queries::buckets::upsert_buckets`
-- runs `where bucket_end is null` on every tick to find each subscription's
-- currently-open bucket. Partial keeps the index size tiny (one row per
-- active subscription) no matter how big the archived tail grows.
create index idx_subscription_health_bucket_open
    on webhook.subscription_health_bucket(subscription__id)
    where bucket_end is null;

-- Cursor singleton: a bookmark that tracks the last delivery timestamp we've
-- processed. On each tick the subscription health monitor only reads
-- deliveries newer than this cursor, avoiding a full table scan of
-- request_attempt every time.
create table webhook.subscription_health_monitor_cursor (
    cursor__id integer primary key default 1 check (cursor__id = 1),
    last_processed_at timestamptz not null default '-infinity'
);
insert into webhook.subscription_health_monitor_cursor default values
    on conflict do nothing;

-- Expression index for the cursor-based delta scan in
-- `queries::deltas::aggregate_recent_request_attempts`. Uses
-- COALESCE(succeeded_at, failed_at) because rows complete asynchronously — a
-- row created before the cursor may complete after it. This prevents losing
-- in-flight deliveries.
-- On large existing datasets, create this index manually first:
--   create index concurrently if not exists idx_request_attempt_completed_at on webhook.request_attempt (coalesce(succeeded_at, failed_at));
-- The `create index` below will be a no-op if the index already exists.
create index if not exists idx_request_attempt_completed_at
    on webhook.request_attempt (coalesce(succeeded_at, failed_at))
    where succeeded_at is not null or failed_at is not null;

-- Audit log of subscription health transitions. NOT mechanically derived from
-- subscription_health_bucket: the bucket table is where the monitor aggregates
-- raw request_attempt counts; on each tick the state machine reads those
-- aggregates, compares them against the configured thresholds, and appends a
-- row here whenever it decides a subscription's perceived health crossed a
-- boundary (warning triggered, auto-disabled, recovered). Manual API actions
-- on subscriptions also append rows here. The subscriptions UI reads this
-- table to reconstruct a subscription's health timeline.
create table webhook.subscription_health_event (
    health_event__id uuid not null default public.gen_random_uuid(),
    subscription__id uuid not null
        references webhook.subscription(subscription__id)
        on delete cascade,
    status text not null
        check (status in ('warning', 'disabled', 'resolved')),
    -- 'auto' = automatic (subscription health monitor), 'manual' = manual action (API).
    -- When cause = 'manual' and user__id IS NULL, the action was performed via a service
    -- token or application secret.
    cause text not null
        constraint subscription_health_event_cause_check
        check (cause in ('auto', 'manual')),
    user__id uuid
        references iam.user(user__id)
        on delete set null,
    created_at timestamptz not null default statement_timestamp(),
    constraint subscription_health_event_pkey primary key (health_event__id),
    -- cause = 'auto' must have user__id = NULL (automated actions have no user)
    constraint subscription_health_event_cause_user_check check (
        cause != 'auto' or user__id is null
    )
);

-- Primary lookup path for the health timeline endpoint: fetch one
-- subscription's events ordered by most recent first. Without this index the
-- timeline query would scan the full table for every page load. INCLUDE
-- makes it a covering index so Postgres serves status and cause from the
-- index heap without a table fetch.
create index if not exists idx_subscription_health_event_sub_id
    on webhook.subscription_health_event(subscription__id, created_at desc)
    include (status, cause);

-- Partial index for the daily cleanup pass in
-- `queries::events::cleanup_resolved_health_events`, which deletes resolved
-- events past the retention period. Partial keeps the index scoped to the
-- rows the cleanup query actually touches.
create index if not exists idx_subscription_health_event_cleanup
    on webhook.subscription_health_event(created_at)
    where status = 'resolved';

-- Denormalized failure_percent cache on the subscription row. Computed by the
-- subscription health monitor from the buckets table and read directly by the
-- subscriptions API, avoiding a bucket join on every subscription list/get
-- call.
alter table webhook.subscription
    add column failure_percent double precision;

reset lock_timeout;
