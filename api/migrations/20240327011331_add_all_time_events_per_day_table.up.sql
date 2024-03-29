create table event.all_time_events_per_day (
    application__id uuid not null,
    date date not null,
    amount integer not null,
    primary key (application__id, date)
);
