create schema iam;
alter table event.organization set schema iam;

create table iam.plan
(
    plan__id uuid not null primary key default public.gen_random_uuid(),
    name text not null unique,
    created_at timestamptz not null default statement_timestamp(),
    members_per_organization_limit integer,
    applications_per_organization_limit integer,
    events_per_day_limit integer,
    days_of_events_retention_limit integer
);

alter table iam.organization add column plan__id uuid default null references iam.plan (plan__id);

alter table event.application add column events_per_day_limit integer default null;
alter table event.application add column days_of_events_retention_limit integer default null;
