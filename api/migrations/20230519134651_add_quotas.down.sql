alter table event.application drop column events_per_day_limit;
alter table event.application drop column days_of_events_retention_limit;

alter table iam.organization drop column plan__id;
drop table iam.plan;

alter table iam.organization set schema event;
drop schema iam;
