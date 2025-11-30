drop materialized view event.events_per_day;
create materialized view event.events_per_day as (
    select application__id, received_at::date as date, count(event__id)::integer as amount
    from event.event
    group by date, application__id
    order by date desc, amount desc
);

create unique index on event.events_per_day (application__id, date);
