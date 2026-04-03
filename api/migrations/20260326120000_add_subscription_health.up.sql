create table webhook.subscription_health_event (
    health_event__id uuid not null default public.gen_random_uuid(),
    subscription__id uuid not null
        references webhook.subscription(subscription__id)
        on delete cascade,
    status text not null
        check (status in ('warning', 'disabled', 'resolved')),
    -- 'system' = automatic (health monitor), 'user' = manual action (API).
    -- When source = 'user' and user__id IS NULL, the action was performed via a service token or application secret.
    source text not null
        check (source in ('system', 'user')),
    user__id uuid
        references iam.user(user__id)
        on delete set null,
    created_at timestamptz not null default statement_timestamp(),
    constraint subscription_health_event_pkey primary key (health_event__id),
    -- source = 'system' must have user__id = NULL (automated actions have no user)
    constraint subscription_health_event_source_user_check check (
        source != 'system' or user__id is null
    )
);

create index if not exists idx_subscription_health_event_sub_id
    on webhook.subscription_health_event(subscription__id, created_at desc)
    include (status, source);

-- Health buckets aggregate delivery attempt counts (total vs failed) over bounded windows.
-- Each bucket covers a time range [bucket_start, bucket_end) for a single subscription.
-- Buckets are bounded by EITHER duration (health_monitor_bucket_duration) OR message count
-- (health_monitor_bucket_max_messages), whichever limit is reached first.
-- The health monitor evaluates subscription health by summing recent buckets within the
-- configured time_window, computing failure_percent = failed_count / total_count.
create table webhook.subscription_health_bucket (
    subscription__id uuid not null
        references webhook.subscription(subscription__id) on delete cascade,
    bucket_start     timestamptz not null,
    bucket_end       timestamptz,
    total_count      integer not null default 0 check (total_count >= 0),
    failed_count     integer not null default 0 check (failed_count >= 0 and failed_count <= total_count),
    primary key (subscription__id, bucket_start)
);

create index idx_subscription_health_bucket_start
    on webhook.subscription_health_bucket(bucket_start);

create index idx_subscription_health_bucket_open
    on webhook.subscription_health_bucket(subscription__id)
    where bucket_end is null;

-- Cursor singleton: a bookmark that tracks the last delivery timestamp we've processed.
-- On each tick the health monitor only reads deliveries newer than this cursor, avoiding
-- a full table scan of request_attempt every time.
create table webhook.health_monitor_cursor (
    cursor__id integer primary key default 1 check (cursor__id = 1),
    last_processed_at timestamptz not null default '-infinity'
);
insert into webhook.health_monitor_cursor default values
    on conflict do nothing;

-- Expression index for cursor-based delta scan.
-- Uses COALESCE(succeeded_at, failed_at) because rows complete asynchronously — a row created
-- before the cursor may complete after it. This prevents losing in-flight deliveries.
create index if not exists idx_request_attempt_completed_at
    on webhook.request_attempt (coalesce(succeeded_at, failed_at))
    where succeeded_at is not null or failed_at is not null;

-- Partial index for daily cleanup of old resolved events
create index if not exists idx_subscription_health_event_cleanup
    on webhook.subscription_health_event(created_at)
    where status = 'resolved';
