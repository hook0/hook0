DROP INDEX IF EXISTS iam.signup_attribution_first_event_pending_idx;

ALTER TABLE iam.signup_attribution
    DROP COLUMN first_event_uploaded_at;
