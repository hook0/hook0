alter table event.event_type drop column is_enabled;
alter table event.event_type add column deactivated_at timestamptz default null;
create index if not exists event_type_application__id_idx on event.event_type (application__id);
