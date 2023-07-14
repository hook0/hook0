create schema infrastructure;

create table infrastructure.worker (
    worker__id uuid not null primary key default public.gen_random_uuid(),
    name text not null unique,
    description text,
    created_at timestamptz not null default statement_timestamp(),
    public boolean not null default false
);

create table iam.organization__worker (
    organization__id uuid not null references iam.organization (organization__id) on update cascade on delete cascade,
    worker__id uuid not null references infrastructure.worker (worker__id) on update cascade on delete cascade,
    constraint worker__organization_pkey primary key (organization__id, worker__id)
);
comment on table iam.organization__worker is 'when a worker is associated to an organization it means that the organization can use this worker';

create table webhook.subscription__worker (
    subscription__id uuid not null references webhook.subscription (subscription__id) on update cascade on delete cascade,
    worker__id uuid not null references infrastructure.worker (worker__id) on update cascade on delete cascade,
    constraint subscription__worker_pkey primary key (subscription__id, worker__id)
);

alter table webhook.request_attempt rename column worker_id to worker_name;
