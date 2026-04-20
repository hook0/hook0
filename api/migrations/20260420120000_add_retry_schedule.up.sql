-- Fail fast if a long-running transaction holds table lock.
set lock_timeout = '5s';

create table webhook.retry_schedule (
    retry_schedule__id uuid not null default public.gen_random_uuid(),
    organization__id uuid not null
        constraint retry_schedule_organization__id_fkey
        references iam.organization(organization__id)
        on update cascade on delete cascade,
    name text not null,
    strategy text not null,
    max_retries integer not null,
    custom_intervals integer[],
    linear_delay integer,
    increasing_base_delay integer,
    increasing_wait_factor double precision,
    created_at timestamptz not null default statement_timestamp(),
    updated_at timestamptz not null default statement_timestamp(),
    constraint retry_schedule_pkey primary key (retry_schedule__id),
    constraint retry_schedule_organization__id_name_key unique (organization__id, name),
    -- DB checks are anti-corruption bounds: wider than API business limits on purpose.
    -- API enforces stricter values (see api/src/handlers/retry_schedules.rs consts).
    constraint retry_schedule_name_chk check (length(name) >= 1 and length(name) <= 200),
    constraint retry_schedule_strategy_chk check (strategy in ('exponential_increasing', 'linear', 'custom')),
    constraint retry_schedule_max_retries_chk check (max_retries > 0 and max_retries <= 25),
    -- Case-switch enforces strategy-specific nullability.
    constraint retry_schedule_strategy_fields_chk check (
        case strategy
            when 'exponential_increasing' then
                custom_intervals is null
                and linear_delay is null
                and increasing_base_delay is not null
                and increasing_base_delay >= 1 and increasing_base_delay <= 604800
                and increasing_wait_factor is not null
                and increasing_wait_factor >= 1.0 and increasing_wait_factor <= 20.0
            when 'linear' then
                custom_intervals is null
                and increasing_base_delay is null
                and increasing_wait_factor is null
                and linear_delay is not null
                and linear_delay >= 1 and linear_delay <= 604800
            when 'custom' then
                linear_delay is null
                and increasing_base_delay is null
                and increasing_wait_factor is null
                and custom_intervals is not null
                and array_length(custom_intervals, 1) = max_retries
                and 1 <= all(custom_intervals)
                and 604800 >= all(custom_intervals)
            else false
        end
    )
);

comment on table webhook.retry_schedule is
    'Per-organization retry policy attached to subscriptions. NULL subscription.retry_schedule__id means the built-in exponential backoff.';
comment on column webhook.retry_schedule.strategy is
    'Pacing family: exponential_increasing | linear | custom. Controls which strategy-specific columns must be populated.';
comment on column webhook.retry_schedule.max_retries is
    'Upper bound on auto-retries. API enforces a tighter business limit.';
comment on column webhook.retry_schedule.custom_intervals is
    'Populated only when strategy=custom. Array length must equal max_retries; each element is a per-retry delay in seconds.';
comment on column webhook.retry_schedule.linear_delay is
    'Populated only when strategy=linear. Constant delay between every retry, in seconds.';
comment on column webhook.retry_schedule.increasing_base_delay is
    'Populated only when strategy=exponential_increasing. First-retry delay in seconds; subsequent retries multiply by increasing_wait_factor.';
comment on column webhook.retry_schedule.increasing_wait_factor is
    'Populated only when strategy=exponential_increasing. Multiplier applied to the previous delay to compute the next one.';

create index retry_schedule_organization__id_idx
    on webhook.retry_schedule (organization__id);

alter table webhook.subscription add column retry_schedule__id uuid;

-- NOT VALID skips scan; VALIDATE takes weaker lock.
alter table webhook.subscription
    add constraint subscription_retry_schedule__id_fkey
    foreign key (retry_schedule__id)
    references webhook.retry_schedule(retry_schedule__id)
    on update cascade on delete set null
    not valid;

alter table webhook.subscription
    validate constraint subscription_retry_schedule__id_fkey;

create index subscription_retry_schedule__id_idx
    on webhook.subscription (retry_schedule__id);

reset lock_timeout;
