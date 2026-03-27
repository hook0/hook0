create table webhook.subscription_health_event (
    health_event__id uuid not null default public.gen_random_uuid(),
    subscription__id uuid not null
        references webhook.subscription(subscription__id)
        on delete cascade,
    status text not null
        check (status in ('warning', 'disabled', 'resolved')),
    -- 'system' = automatic (health monitor), 'user' = manual action (API PUT)
    -- When source = 'user' and user__id IS NULL, the action was performed via a service token
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

-- Composite index for health evaluation query (replaces the single-column subscription__id index)
create index if not exists idx_request_attempt_sub_health
    on webhook.request_attempt (subscription__id, created_at desc)
    include (succeeded_at, failed_at);

-- Drop redundant single-column index (the new composite index is a strict superset)
drop index if exists webhook.request_attempt_subscription__id_idx;
