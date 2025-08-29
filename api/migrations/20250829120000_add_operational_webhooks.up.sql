-- Operational Webhooks: Webhooks that notify about the status of the webhook system itself
-- Similar to Svix's operational webhooks feature

-- Create table for operational webhook endpoints
create table webhook.operational_endpoint
(
    operational_endpoint__id uuid not null default public.gen_random_uuid(),
    application__id uuid not null,
    url text not null,
    description text,
    headers jsonb not null default jsonb_build_object(),
    secret uuid not null default public.gen_random_uuid(),
    is_enabled boolean not null default true,
    -- Filter by event types (e.g., 'endpoint.disabled', 'message.failed', etc.)
    filter_types jsonb not null default '[]'::jsonb,
    -- Rate limit configuration
    rate_limit integer,
    created_at timestamptz not null default statement_timestamp(),
    updated_at timestamptz not null default statement_timestamp(),
    deleted_at timestamptz,
    
    constraint operational_endpoint_pkey primary key (operational_endpoint__id),
    constraint operational_endpoint_headers_is_object check (jsonb_typeof(headers) = 'object'),
    constraint operational_endpoint_filter_types_is_array check (jsonb_typeof(filter_types) = 'array'),
    constraint operational_endpoint_url_valid check (url ~ '^https?://.*'),
    constraint operational_endpoint_rate_limit_positive check (rate_limit is null or rate_limit > 0)
);

alter table webhook.operational_endpoint add constraint operational_endpoint_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

create index idx_operational_endpoint_application on webhook.operational_endpoint(application__id) where deleted_at is null;
create index idx_operational_endpoint_enabled on webhook.operational_endpoint(is_enabled) where deleted_at is null;

-- Table for operational webhook event types
create table webhook.operational_event_type
(
    event_type__name text not null,
    description text not null,
    schema jsonb not null default '{}'::jsonb,
    
    constraint operational_event_type_pkey primary key (event_type__name)
);

-- Insert standard operational event types similar to Svix
insert into webhook.operational_event_type (event_type__name, description, schema) values
    ('endpoint.created', 'Fired when an endpoint is created', '{"type": "object", "properties": {"endpoint_id": {"type": "string"}, "url": {"type": "string"}}}'::jsonb),
    ('endpoint.updated', 'Fired when an endpoint is updated', '{"type": "object", "properties": {"endpoint_id": {"type": "string"}, "url": {"type": "string"}}}'::jsonb),
    ('endpoint.deleted', 'Fired when an endpoint is deleted', '{"type": "object", "properties": {"endpoint_id": {"type": "string"}}}'::jsonb),
    ('endpoint.disabled', 'Fired when an endpoint is automatically disabled', '{"type": "object", "properties": {"endpoint_id": {"type": "string"}, "reason": {"type": "string"}}}'::jsonb),
    ('message.attempt.exhausted', 'Fired when all retry attempts are exhausted', '{"type": "object", "properties": {"message_id": {"type": "string"}, "endpoint_id": {"type": "string"}, "attempts": {"type": "integer"}}}'::jsonb),
    ('message.attempt.failing', 'Fired when a message attempt is failing', '{"type": "object", "properties": {"message_id": {"type": "string"}, "endpoint_id": {"type": "string"}, "attempt": {"type": "integer"}}}'::jsonb),
    ('message.attempt.recovered', 'Fired when a message is successfully delivered after failures', '{"type": "object", "properties": {"message_id": {"type": "string"}, "endpoint_id": {"type": "string"}, "attempts": {"type": "integer"}}}'::jsonb);

-- Table for operational events (events about the webhook system itself)
create table webhook.operational_event
(
    operational_event__id uuid not null default public.gen_random_uuid(),
    application__id uuid not null,
    event_type__name text not null,
    payload jsonb not null,
    occurred_at timestamptz not null default statement_timestamp(),
    
    constraint operational_event_pkey primary key (operational_event__id)
);

alter table webhook.operational_event add constraint operational_event_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

alter table webhook.operational_event add constraint operational_event_event_type__name_fkey
foreign key (event_type__name)
references webhook.operational_event_type (event_type__name)
match simple
on delete restrict
on update cascade;

create index idx_operational_event_application on webhook.operational_event(application__id);
create index idx_operational_event_type on webhook.operational_event(event_type__name);
create index idx_operational_event_occurred_at on webhook.operational_event(occurred_at desc);

-- Table for operational webhook delivery attempts
create table webhook.operational_attempt
(
    operational_attempt__id uuid not null default public.gen_random_uuid(),
    operational_event__id uuid not null,
    operational_endpoint__id uuid not null,
    status text not null default 'pending',
    response_status_code integer,
    response_headers jsonb,
    response_body text,
    error_message text,
    attempted_at timestamptz,
    created_at timestamptz not null default statement_timestamp(),
    
    constraint operational_attempt_pkey primary key (operational_attempt__id),
    constraint operational_attempt_status_check check (status in ('pending', 'success', 'failed'))
);

alter table webhook.operational_attempt add constraint operational_attempt_operational_event__id_fkey
foreign key (operational_event__id)
references webhook.operational_event (operational_event__id)
match simple
on delete cascade
on update cascade;

alter table webhook.operational_attempt add constraint operational_attempt_operational_endpoint__id_fkey
foreign key (operational_endpoint__id)
references webhook.operational_endpoint (operational_endpoint__id)
match simple
on delete cascade
on update cascade;

create index idx_operational_attempt_event on webhook.operational_attempt(operational_event__id);
create index idx_operational_attempt_endpoint on webhook.operational_attempt(operational_endpoint__id);
create index idx_operational_attempt_status on webhook.operational_attempt(status) where status = 'pending';

-- Message statistics table for tracking delivery health
create table webhook.message_stats
(
    message_stats__id uuid not null default public.gen_random_uuid(),
    application__id uuid not null,
    subscription__id uuid not null,
    period_start timestamptz not null,
    period_end timestamptz not null,
    total_messages integer not null default 0,
    successful_messages integer not null default 0,
    failed_messages integer not null default 0,
    pending_messages integer not null default 0,
    avg_delivery_time_ms integer,
    
    constraint message_stats_pkey primary key (message_stats__id),
    constraint message_stats_unique_period unique (application__id, subscription__id, period_start, period_end)
);

alter table webhook.message_stats add constraint message_stats_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

alter table webhook.message_stats add constraint message_stats_subscription__id_fkey
foreign key (subscription__id)
references webhook.subscription (subscription__id)
match simple
on delete cascade
on update cascade;

create index idx_message_stats_application on webhook.message_stats(application__id);
create index idx_message_stats_subscription on webhook.message_stats(subscription__id);
create index idx_message_stats_period on webhook.message_stats(period_start, period_end);

-- Function to trigger operational events
create or replace function webhook.trigger_operational_event(
    p_application_id uuid,
    p_event_type text,
    p_payload jsonb
)
returns uuid
language plpgsql
as $$
declare
    v_event_id uuid;
    v_endpoint record;
begin
    -- Insert the operational event
    insert into webhook.operational_event (application__id, event_type__name, payload)
    values (p_application_id, p_event_type, p_payload)
    returning operational_event__id into v_event_id;
    
    -- Create attempts for all enabled operational endpoints that match the filter
    for v_endpoint in
        select operational_endpoint__id
        from webhook.operational_endpoint
        where application__id = p_application_id
          and is_enabled = true
          and deleted_at is null
          and (filter_types = '[]'::jsonb or filter_types @> to_jsonb(p_event_type))
    loop
        insert into webhook.operational_attempt (operational_event__id, operational_endpoint__id)
        values (v_event_id, v_endpoint.operational_endpoint__id);
    end loop;
    
    return v_event_id;
end;
$$;

-- Trigger for subscription changes
create or replace function webhook.subscription_operational_trigger()
returns trigger
language plpgsql
as $$
declare
    v_event_type text;
    v_payload jsonb;
begin
    if TG_OP = 'INSERT' then
        v_event_type := 'endpoint.created';
        v_payload := jsonb_build_object(
            'endpoint_id', NEW.subscription__id,
            'url', (select url from webhook.target_http where target__id = NEW.target__id),
            'application_id', NEW.application__id
        );
    elsif TG_OP = 'UPDATE' then
        if OLD.deleted_at is null and NEW.deleted_at is not null then
            v_event_type := 'endpoint.deleted';
            v_payload := jsonb_build_object(
                'endpoint_id', NEW.subscription__id,
                'application_id', NEW.application__id
            );
        elsif OLD.is_enabled = true and NEW.is_enabled = false then
            v_event_type := 'endpoint.disabled';
            v_payload := jsonb_build_object(
                'endpoint_id', NEW.subscription__id,
                'reason', 'Manually disabled',
                'application_id', NEW.application__id
            );
        else
            v_event_type := 'endpoint.updated';
            v_payload := jsonb_build_object(
                'endpoint_id', NEW.subscription__id,
                'url', (select url from webhook.target_http where target__id = NEW.target__id),
                'application_id', NEW.application__id
            );
        end if;
    elsif TG_OP = 'DELETE' then
        v_event_type := 'endpoint.deleted';
        v_payload := jsonb_build_object(
            'endpoint_id', OLD.subscription__id,
            'application_id', OLD.application__id
        );
    end if;
    
    perform webhook.trigger_operational_event(
        coalesce(NEW.application__id, OLD.application__id),
        v_event_type,
        v_payload
    );
    
    return NEW;
end;
$$;

create trigger subscription_operational_changes
    after insert or update or delete
    on webhook.subscription
    for each row
execute function webhook.subscription_operational_trigger();

-- Add column to track failure count for automatic disabling
alter table webhook.subscription add column if not exists consecutive_failures integer not null default 0;
alter table webhook.subscription add column if not exists last_failure_at timestamptz;
alter table webhook.subscription add column if not exists auto_disabled_at timestamptz;

-- Function to update message statistics
create or replace function webhook.update_message_stats()
returns void
language plpgsql
as $$
begin
    -- This would typically be called periodically to update statistics
    -- Implementation would aggregate data from request_attempt table
    null;
end;
$$;