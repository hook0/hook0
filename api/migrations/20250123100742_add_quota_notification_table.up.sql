CREATE TABLE pricing.quota_notifications (
    quota_notification__id uuid not null default public.gen_random_uuid(),
    application__id uuid,
    organization__id uuid,
    name text not null,
    type text check (type in ('Reached', 'Warning')) not null,
    executed_at timestamptz not null default now(),
    constraint quota_notification_at_least_one_id_chk check (
        (application__id is not null and organization__id is null) or
        (organization__id is not null and application__id is null)
    ),
    constraint quota_notification_name_chk check (length(name) > 1),
    constraint quota_notification_application__id_fkey foreign key (application__id)
        references event.application (application__id)
        on delete cascade,
    constraint quota_notification_organization__id_fkey foreign key (organization__id)
        references iam.organization (organization__id)
        on delete cascade
);

CREATE INDEX idx_quota_notifications_app_id ON pricing.quota_notifications (application__id);
CREATE INDEX idx_quota_notifications_org_id ON pricing.quota_notifications (organization__id);
