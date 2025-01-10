alter table pricing.plan add column subscriptions_per_application_limit integer default null;
alter table pricing.plan add column event_types_per_application_limit integer default null;
