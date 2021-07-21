create extension if not exists pgcrypto with schema public;

set search_path to pg_catalog, public; -- public is necessary so that sqlx can find its _sqlx_migrations table
set plpgsql.extra_warnings to 'all';






create schema event;

create table event.organization
(
    organization__id uuid not null default public.gen_random_uuid(),
    name text not null,
    created_at timestamptz not null default statement_timestamp(),
    constraint organization_pkey primary key (organization__id),
    constraint organization_name_chk check (length(name) > 1)
);

create table event.application
(
    application__id  uuid not null default public.gen_random_uuid(),
    organization__id uuid not null ,
    name text not null,
    created_at timestamptz not null default statement_timestamp(),
    constraint application_pkey primary key (application__id),
    constraint application_name_chk check (length(name) > 1)
);

alter table event.application add constraint application_organization__id_fkey
foreign key (organization__id)
references event.organization (organization__id)
match simple
on delete cascade
on update cascade;

create table event.service
(
    service__name text not null,
    application__id uuid not null,
    comment text,
    constraint service_pkey primary key (service__name, application__id)
);

alter table event.service add constraint service_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

create table event.resource_type
(
    resource_type__name text not null,
    application__id uuid not null,
    service__name text not null,
    constraint resource_type_pkey primary key (application__id, service__name, resource_type__name)
);

alter table event.resource_type add constraint resource_type_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

alter table event.resource_type add constraint resource_type_application__id_service__name_fkey
foreign key (application__id, service__name)
references event.service (application__id, service__name)
match simple
on delete restrict
on update restrict;

create table event.verb
(
    verb__name text not null,
    application__id uuid not null,
    constraint verb_pkey primary key (verb__name, application__id)
);

alter table event.verb add constraint verb_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

create table event.event_type
(
    application__id uuid not null,
    service__name text not null,
    resource_type__name text not null,
    verb__name text not null,
    created_at timestamptz not null default statement_timestamp(),
    is_enabled boolean not null default true,
    event_type__name text not null generated always as ((((service__name || '.'::text) || resource_type__name) || '.'::text) || verb__name) stored,
    constraint event_type_pkey primary key (event_type__name)
);

alter table event.event_type add constraint event_type_application__id_service__name_fkey
foreign key (application__id, service__name)
references event.service (application__id, service__name)
match simple
on delete restrict
on update restrict;

alter table event.event_type add constraint event_type_application__id_verb__name_fkey
foreign key (application__id, verb__name)
references event.verb (application__id, verb__name)
match simple
on delete restrict
on update restrict;

alter table event.event_type add constraint event_type_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

alter table event.event_type add constraint event_type_service__name_application__id_resource_type__name
foreign key (service__name, application__id, resource_type__name)
references event.resource_type (service__name, application__id, resource_type__name) match simple
on delete restrict on update restrict;

create table event.application_secret
(
    token uuid not null default public.gen_random_uuid(),
    application__id uuid not null,
    created_at timestamptz not null default statement_timestamp(),
    deleted_at timestamptz,
    name text,
    constraint application_secret_pkey primary key (token)
);

alter table event.application_secret add constraint application_secret_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

create table event.payload_content_type
(
    payload_content_type__name text not null,
    description text not null,
    created_at timestamptz not null default statement_timestamp(),
    constraint payload_content_type_pkey primary key (payload_content_type__name)
);

create table event.event
(
    event__id uuid not null default public.gen_random_uuid() primary key,
    application__id uuid not null,
    event_type__name text not null,
    payload bytea not null,
    payload_content_type__name text not null,
    ip inet not null,
    metadata jsonb,
    occurred_at timestamptz not null,
    received_at timestamptz not null default statement_timestamp(),
    dispatched_at timestamptz,
    application_secret__token  uuid not null,
    labels jsonb not null default jsonb_build_object(),
    constraint event_metadata_is_object check ((metadata is null) or (jsonb_typeof(metadata) = 'object')),
    constraint event_labels_is_object check (jsonb_typeof(labels) = 'object')
);

alter table event.event add constraint event_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

alter table event.event add constraint event_application_secret__token_fkey
foreign key (application_secret__token)
references event.application_secret (token)
match simple
on delete restrict
on update restrict;

alter table event.event add constraint event_payload_content_type__name_fkey
foreign key (payload_content_type__name)
references event.payload_content_type (payload_content_type__name)
match simple
on delete restrict
on update restrict;

alter table event.event add constraint event_event_type__name_fkey
foreign key (event_type__name)
references event.event_type (event_type__name)
match simple
on delete restrict
on update restrict;

create or replace function event.dispatch()
    returns trigger
    language plpgsql
as
$$
declare
    key text;
    value text;
    subscription_id uuid;
begin
    for key, value in select * from jsonb_each_text(new.labels) limit 50
        loop
            for subscription_id in
                select subscription__id
                from webhook.subscription
                where is_enabled
                  and label_key = key
                  and label_value = value
                loop
                    raise notice '[event %] matching subscription: %', event__id, subscription_id;
                    insert into webhook.request_attempt (event__id, subscription__id)
                    values (new.event__id, subscription_id);
                end loop;
        end loop;
    update event.event set dispatched_at = statement_timestamp() where event__id = new.event__id;
    return new;
end;
$$;





create schema webhook;

create table webhook.subscription
(
    subscription__id uuid not null default public.gen_random_uuid(),
    application__id uuid not null,
    is_enabled boolean not null default true,
    description text,
    secret uuid not null default public.gen_random_uuid(),
    metadata jsonb not null default jsonb_build_object(),
    label_key text not null,
    label_value text not null,
    target__id uuid not null,
    created_at timestamptz not null default statement_timestamp(),
    constraint subscription_pkey primary key (subscription__id),
    constraint subscription_target__id_key unique (target__id),
    constraint subscription_metadata_is_object check ((metadata is null) or (jsonb_typeof(metadata) = 'object'))
);

alter table webhook.subscription add constraint subscription_application__id_fkey
foreign key (application__id)
references event.application (application__id)
match simple
on delete cascade
on update cascade;

create table webhook.subscription__event_type
(
    subscription__id uuid not null,
    event_type__name text not null,
    constraint subscription__event_type_pkey primary key (subscription__id, event_type__name)
);

alter table webhook.subscription__event_type add constraint subscription__event_type_subscription__id_fkey
foreign key (subscription__id)
references webhook.subscription (subscription__id)
match simple
on delete cascade
on update cascade;

alter table webhook.subscription__event_type add constraint subscription__event_type_event_type__name_fkey
foreign key (event_type__name)
references event.event_type (event_type__name)
match simple
on delete cascade
on update cascade;

create table webhook.response_error
(
    response_error__name text not null,
    constraint response_error_pkey primary key (response_error__name)
);

create table webhook.response
(
    response__id uuid not null default public.gen_random_uuid(),
    response_error__name text,
    http_code smallint,
    headers jsonb,
    body text,
    elapsed_time_ms integer,
    constraint response_pkey primary key (response__id),
    constraint response_headers_is_object check ((headers is null) or (jsonb_typeof(headers) = 'object'))
);

alter table webhook.response add constraint response_response_error__name_fkey
foreign key (response_error__name)
references webhook.response_error (response_error__name)
match simple
on delete restrict
on update cascade;

create table webhook.request_attempt
(
    request_attempt__id uuid not null default public.gen_random_uuid(),
    event__id uuid not null,
    subscription__id uuid not null,
    created_at timestamptz not null default statement_timestamp(),
    picked_at timestamptz,
    worker_id text,
    worker_version text,
    failed_at timestamptz,
    succeeded_at timestamptz,
    delay_until timestamptz,
    response__id uuid,
    retry_count smallint not null default 0,
    constraint request_attempt_pkey primary key (request_attempt__id),
    constraint request_attempt_response__id_key unique (response__id)
);

alter table webhook.request_attempt add constraint request_attempt_subscription__id_fkey
foreign key (subscription__id)
references webhook.subscription (subscription__id)
match simple
on delete restrict
on update restrict;

alter table webhook.request_attempt add constraint request_attempt_event__id_fkey
foreign key (event__id)
references event.event (event__id)
match simple
on delete cascade
on update cascade;

alter table webhook.request_attempt add constraint request_attempt_response__id_fkey
foreign key (response__id)
references webhook.response (response__id)
match simple
on delete set null
on update cascade;

create table webhook.target
(
    target__id uuid not null default public.gen_random_uuid(),
    constraint target_pkey primary key (target__id)
);

alter table webhook.target add constraint target_target__id_fkey
foreign key (target__id)
references webhook.subscription (target__id)
match simple
on delete cascade
on update cascade;

create table webhook.target_http
(
    target__id uuid not null default public.gen_random_uuid(),
    method text not null,
    url text not null,
    headers jsonb not null default jsonb_build_object(),
    constraint target_http_headers_is_object check (jsonb_typeof(headers) = 'object')
)
inherits (webhook.target);

alter table webhook.target_http  add constraint target_http_target__id_fkey
foreign key (target__id)
references webhook.subscription (target__id)
match simple
on delete cascade
on update cascade;
