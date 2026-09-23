delete from infrastructure.state where value_boolean is not null or value_uuids is not null;

alter table infrastructure.state drop constraint state_exactly_one_value;
alter table infrastructure.state add constraint state_exactly_one_value
    check (num_nonnulls(value_uuid, value_timestamptz) = 1);

alter table infrastructure.state
    drop column value_boolean,
    drop column value_uuids;
