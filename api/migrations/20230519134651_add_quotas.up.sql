create schema iam;
alter table event.organization set schema iam;

create schema pricing;
create table pricing.plan
(
    plan__id uuid not null primary key default public.gen_random_uuid(),
    name text not null unique,
    label text not null,
    created_at timestamptz not null default statement_timestamp(),
    members_per_organization_limit integer,
    applications_per_organization_limit integer,
    events_per_day_limit integer,
    days_of_events_retention_limit integer
);
create table pricing.price
(
    price__id uuid not null primary key default public.gen_random_uuid(),
    plan__id uuid not null references pricing.plan (plan__id),
    amount numeric(7, 2) not null,
    time_basis text not null,
    created_at timestamptz not null default statement_timestamp(),
    description text
);

alter table iam.organization add column price__id uuid default null references pricing.price (price__id);

alter table event.application add column events_per_day_limit integer default null;
alter table event.application add column days_of_events_retention_limit integer default null;
