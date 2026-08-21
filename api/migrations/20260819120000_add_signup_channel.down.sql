-- Dropping the column drops its CHECK constraint with it.
ALTER TABLE iam.user
    DROP COLUMN signup_channel;
