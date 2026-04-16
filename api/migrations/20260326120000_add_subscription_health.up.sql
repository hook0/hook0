set lock_timeout = '5s';

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

-- Default is epoch, not '-infinity': PostgreSQL accepts '-infinity' as timestamptz
-- but sqlx panics converting it to chrono::DateTime (NaiveDateTime overflow).
create table webhook.subscription_health_monitor_cursor (
    cursor__id integer primary key default 1 check (cursor__id = 1),
    last_processed_at timestamptz not null default '1970-01-01T00:00:00Z'
);
insert into webhook.subscription_health_monitor_cursor default values
    on conflict do nothing;

create index if not exists idx_request_attempt_completed_at
    on webhook.request_attempt (coalesce(succeeded_at, failed_at))
    where succeeded_at is not null or failed_at is not null;

create table webhook.subscription_health_event (
    health_event__id uuid not null default public.gen_random_uuid(),
    subscription__id uuid not null
        references webhook.subscription(subscription__id)
        on delete cascade,
    status text not null
        check (status in ('warning', 'disabled', 'resolved')),
    cause text not null
        constraint subscription_health_event_cause_check
        check (cause in ('auto', 'manual')),
    user__id uuid
        references iam.user(user__id)
        on delete set null,
    created_at timestamptz not null default statement_timestamp(),
    constraint subscription_health_event_pkey primary key (health_event__id),
    constraint subscription_health_event_cause_user_check check (
        cause != 'auto' or user__id is null
    )
);

create index if not exists idx_subscription_health_event_sub_id
    on webhook.subscription_health_event(subscription__id, created_at desc)
    include (status, cause);

create index if not exists idx_subscription_health_event_cleanup
    on webhook.subscription_health_event(created_at)
    where status = 'resolved';

alter table webhook.subscription
    add column failure_percent double precision;

reset lock_timeout;
