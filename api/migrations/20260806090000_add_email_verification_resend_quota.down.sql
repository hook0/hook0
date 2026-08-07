ALTER TABLE iam."user"
    DROP COLUMN email_verification_resend_window_started_at,
    DROP COLUMN email_verification_resend_count;
