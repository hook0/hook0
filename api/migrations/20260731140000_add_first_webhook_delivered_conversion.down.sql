DROP INDEX IF EXISTS iam.signup_attribution_first_webhook_delivered_pending_idx;

ALTER TABLE iam.signup_attribution
    DROP COLUMN first_webhook_delivered_uploaded_at;
