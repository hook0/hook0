create table iam.user (
    user__id uuid not null primary key default public.gen_random_uuid(),
    email text not null unique,
    password text not null,
    first_name text not null,
    last_name text not null,
    created_at timestamptz not null default statement_timestamp(),
    email_verified_at timestamptz,
    last_login timestamptz
);

create table iam.user__organization (
    user__id uuid not null,
    organization__id uuid not null,
    role text not null,
    created_at timestamptz not null default statement_timestamp(),
    primary key (user__id, organization__id),
    constraint user__organization_user__id_fk foreign key (user__id) references iam.user (user__id) on delete cascade on update cascade,
    constraint user__organization_organization__id_fk foreign key (organization__id) references iam.organization (organization__id) on delete cascade on update cascade,
    constraint user__organization_role_chk check (role in ('editor', 'viewer'))
);

create table iam.token (
    token__id uuid not null primary key default public.gen_random_uuid(),
    created_at timestamptz not null default statement_timestamp(),
    type text not null,
    revocation_id bytea not null,
    expired_at timestamptz,
    organization__id uuid,
    name text,
    biscuit text,
    user__id uuid ,
    session_id uuid,
    constraint token_type_chk check (type in ('service_access', 'user_access', 'refresh')),
    constraint token_expired_at_chk check (type = 'service_access' or expired_at is not null),
    constraint token_organization__id_fk foreign key (organization__id) references iam.organization (organization__id) on delete cascade on update cascade,
    constraint token_organization__id_chk check (type != 'service_access' or organization__id is not null),
    constraint token_name_chk check (type != 'service_access' or name is not null),
    constraint token_biscuit_chk check (type != 'service_access' or biscuit is not null),
    constraint token_user__id_fk foreign key (user__id) references iam.user (user__id) on delete cascade on update cascade,
    constraint token_user__id_chk check (type not in ('user_access', 'refresh') or user__id is not null),
    constraint token_session_id_chk check (type not in ('user_access', 'refresh') or session_id is not null)
);

create index token_revocation_id_idx on iam.token (revocation_id);
create index token_organization__id_idx on iam.token (organization__id);
create index token_user__id_idx on iam.token (user__id);

alter table event.event alter column application_secret__token set default null;
