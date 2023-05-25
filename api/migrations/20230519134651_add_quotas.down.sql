alter table event.application drop column events_per_day_limit;
alter table event.application drop column days_of_events_retention_limit;

alter table iam.organization drop column price__id;
drop table pricing.price;
drop table pricing.plan;
drop schema pricing;

alter table iam.organization set schema event;
drop schema iam;
