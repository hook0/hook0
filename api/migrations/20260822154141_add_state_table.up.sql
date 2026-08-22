CREATE TABLE infrastructure.state (
    key TEXT NOT NULL PRIMARY KEY,
    value_uuid UUID,
    value_timestamptz TIMESTAMPTZ,
    CONSTRAINT state_key_grammar CHECK (key ~ '^[a-z][a-z0-9_]*$'),
    CONSTRAINT state_exactly_one_value CHECK (num_nonnulls(value_uuid, value_timestamptz) = 1)
);
