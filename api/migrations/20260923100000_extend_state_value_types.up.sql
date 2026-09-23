alter table infrastructure.state
    add column value_boolean boolean,
    add column value_uuids uuid[];

alter table infrastructure.state drop constraint state_exactly_one_value;
alter table infrastructure.state add constraint state_exactly_one_value
    check (num_nonnulls(value_uuid, value_timestamptz, value_boolean, value_uuids) = 1);
