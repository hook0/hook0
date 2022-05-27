alter table event.event drop constraint event_payload_content_type__name_fkey;
drop table event.payload_content_type;
alter table event.event rename column payload_content_type__name to payload_content_type;
