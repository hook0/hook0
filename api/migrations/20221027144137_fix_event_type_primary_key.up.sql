alter table event.event_type rename constraint event_type_pkey to event_type_pkeyold;
create unique index event_type_pkey on event.event_type (application__id, event_type__name);
alter table event.event_type drop constraint event_type_pkeyold cascade;
alter table event.event_type add primary key using index event_type_pkey;

alter table event.event add constraint event_event_type__name_fkey
foreign key (application__id, event_type__name)
references event.event_type (application__id, event_type__name)
match simple
on delete restrict
on update restrict;

alter table webhook.subscription__event_type add column application__id uuid;
update webhook.subscription__event_type set application__id = event.event_type.application__id from event.event_type where webhook.subscription__event_type.event_type__name = event.event_type.event_type__name;
alter table webhook.subscription__event_type alter column application__id set not null;
alter table webhook.subscription__event_type add constraint subscription__event_type_event_type__name_fkey
foreign key (application__id, event_type__name)
references event.event_type (application__id, event_type__name)
match simple
on delete cascade
on update cascade;
