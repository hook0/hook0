-- On large instances, run these statements by hand before deploying (the matching statements below
-- then do nothing), so that building the indexes does not block writes on webhook.request_attempt:
--
--   alter table webhook.request_attempt add column if not exists paused boolean not null default false;
--   create index concurrently if not exists request_attempt_waiting_unpaused_idx on webhook.request_attempt (created_at asc, retry_count asc nulls first) where (not paused and succeeded_at is null and failed_at is null);
--   create index concurrently if not exists request_attempt_paused_idx on webhook.request_attempt (subscription__id, created_at) where (paused and succeeded_at is null and failed_at is null);
--   create index concurrently if not exists request_attempt_subscription_waiting_idx on webhook.request_attempt (subscription__id, delay_until) where (not paused and succeeded_at is null and failed_at is null);
--
-- request_attempt_waiting_idx is kept on purpose: output workers that do not know about the paused
-- column yet still need it to pick request attempts. A later migration drops it and renames
-- request_attempt_waiting_unpaused_idx to request_attempt_waiting_idx.

create table webhook.subscription_health (
    subscription_health__id uuid not null default uuidv7(),
    subscription__id uuid not null,
    created_at timestamptz not null default statement_timestamp(),
    status text not null,
    constraint subscription_health_pkey primary key (subscription_health__id),
    constraint subscription_health_subscription__id_fkey foreign key (subscription__id)
        references webhook.subscription (subscription__id)
        on delete cascade,
    constraint subscription_health_status_chk check (status in ('healthy', 'degraded', 'recovering'))
);

create index subscription_health_subscription__id_created_at_idx on webhook.subscription_health (subscription__id, created_at desc);

alter table webhook.request_attempt add column if not exists paused boolean not null default false;

create index if not exists request_attempt_waiting_unpaused_idx on webhook.request_attempt (created_at asc, retry_count asc nulls first) where (not paused and succeeded_at is null and failed_at is null);
create index if not exists request_attempt_paused_idx on webhook.request_attempt (subscription__id, created_at) where (paused and succeeded_at is null and failed_at is null);
create index if not exists request_attempt_subscription_waiting_idx on webhook.request_attempt (subscription__id, delay_until) where (not paused and succeeded_at is null and failed_at is null);

-- Health of a subscription as it must affect deliveries: the latest known status (no status means
-- healthy) if subscription health is enforced for the subscription's application, 'healthy' otherwise.
-- The infrastructure.state keys are written by the API at startup from its configuration.
create function webhook.effective_subscription_health(p_subscription__id uuid, p_application__id uuid)
    returns text
    language sql
    stable
as
$$
    select case
        when coalesce((select value_boolean from infrastructure.state where key = 'subscription_health_enforced'), false)
            and coalesce((select cardinality(value_uuids) = 0 or p_application__id = any(value_uuids) from infrastructure.state where key = 'subscription_health_enforced_for'), true)
        then coalesce((select status from webhook.subscription_health where subscription__id = p_subscription__id order by created_at desc, subscription_health__id desc limit 1), 'healthy')
        else 'healthy'
    end;
$$;

create or replace function event.dispatch()
    returns trigger
    language plpgsql
as
$$
begin
    if new.dispatched_at is not null then
        return new;
    end if;

    insert into webhook.request_attempt (event__id, subscription__id, application__id, paused)
    select new.event__id, s.subscription__id, s.application__id, webhook.effective_subscription_health(s.subscription__id, s.application__id) <> 'healthy'
    from webhook.subscription as s
    inner join webhook.subscription__event_type as set on set.subscription__id = s.subscription__id
    where s.is_enabled
      and s.application__id = new.application__id
      and s.deleted_at is null
      and set.event_type__name = new.event_type__name
      and new.labels @> s.labels
    for share of s;

    update event.event set dispatched_at = statement_timestamp() where event__id = new.event__id;
    return new;
end;
$$;
