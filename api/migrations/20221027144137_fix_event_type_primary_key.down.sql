alter table event.event_type rename constraint event_type_pkey to event_type_pkeyold;
create unique index event_type_pkey on event.event_type (event_type__name);
alter table event.event_type drop constraint event_type_pkeyold cascade;
alter table event.event_type add primary key using index event_type_pkey;

alter table event.event add constraint event_event_type__name_fkey
foreign key (event_type__name)
references event.event_type (event_type__name)
match simple
on delete restrict
on update restrict;

alter table webhook.subscription__event_type add constraint subscription__event_type_event_type__name_fkey
foreign key (event_type__name)
references event.event_type (event_type__name)
match simple
on delete cascade
on update cascade;
alter table webhook.subscription__event_type drop column application__id;
