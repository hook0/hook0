alter table event.event rename column payload_content_type to payload_content_type__name;
create table event.payload_content_type
(
    payload_content_type__name text not null,
    description text not null,
    created_at timestamptz not null default statement_timestamp(),
    constraint payload_content_type_pkey primary key (payload_content_type__name)
);
insert into event.payload_content_type (payload_content_type__name, description) select payload_content_type__name, payload_content_type__name as description from event.event group by payload_content_type__name;
alter table only event.event add constraint event_payload_content_type__name_fkey foreign key (payload_content_type__name) references event.payload_content_type(payload_content_type__name) on update restrict on delete restrict;
