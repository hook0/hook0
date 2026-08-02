DROP INDEX IF EXISTS iam.organization_matomo_activation_pending_idx;

ALTER TABLE iam.organization
    DROP COLUMN matomo_activation_emitted_at;
