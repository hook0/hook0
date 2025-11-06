create index if not exists event_received_at_idx on event.event using brin (received_at) with (autosummarize = on);
create index if not exists application_organization__id_idx on event.application (organization__id);
