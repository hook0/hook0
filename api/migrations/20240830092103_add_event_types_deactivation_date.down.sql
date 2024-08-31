drop index event.event_type_application__id_idx;
alter table event.event_type drop column deactivated_at;
alter table event.event_type add column is_enabled boolean not null default true;
