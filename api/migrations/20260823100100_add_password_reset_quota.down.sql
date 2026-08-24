ALTER TABLE iam."user"
    DROP COLUMN password_reset_sent_at,
    DROP COLUMN password_reset_window_started_at,
    DROP COLUMN password_reset_count;
