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

-- Bucketed health aggregation
CREATE TABLE webhook.subscription_health_bucket (
    subscription__id UUID NOT NULL
        REFERENCES webhook.subscription(subscription__id) ON DELETE CASCADE,
    bucket_start     TIMESTAMPTZ NOT NULL,
    bucket_end       TIMESTAMPTZ,
    total_count      INTEGER NOT NULL DEFAULT 0 CHECK (total_count >= 0),
    failed_count     INTEGER NOT NULL DEFAULT 0 CHECK (failed_count >= 0 AND failed_count <= total_count),
    PRIMARY KEY (subscription__id, bucket_start)
);

CREATE INDEX idx_subscription_health_bucket_start
    ON webhook.subscription_health_bucket(bucket_start);

CREATE INDEX idx_subscription_health_bucket_open
    ON webhook.subscription_health_bucket(subscription__id)
    WHERE bucket_end IS NULL;

-- Watermark singleton for delta processing
CREATE TABLE webhook.health_monitor_watermark (
    id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    last_processed_at TIMESTAMPTZ NOT NULL DEFAULT '-infinity'
);
INSERT INTO webhook.health_monitor_watermark DEFAULT VALUES
    ON CONFLICT DO NOTHING;

-- Expression index for watermark-based delta scan (completed rows only)
CREATE INDEX IF NOT EXISTS idx_request_attempt_completed_at
    ON webhook.request_attempt (COALESCE(succeeded_at, failed_at))
    WHERE succeeded_at IS NOT NULL OR failed_at IS NOT NULL;

-- Partial index for daily cleanup of old resolved events
CREATE INDEX IF NOT EXISTS idx_subscription_health_event_cleanup
    ON webhook.subscription_health_event(created_at)
    WHERE status = 'resolved';
