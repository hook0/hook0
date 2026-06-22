DROP INDEX IF EXISTS iam.signup_attribution_organization_idx;

-- Rows whose gclid was minimised to NULL after upload cannot satisfy the
-- original NOT NULL constraint; drop them before restoring it.
DELETE FROM iam.signup_attribution WHERE gclid IS NULL;

ALTER TABLE iam.signup_attribution DROP CONSTRAINT signup_attribution_gclid_length;
ALTER TABLE iam.signup_attribution
    ADD CONSTRAINT signup_attribution_gclid_length
    CHECK (char_length(gclid) BETWEEN 1 AND 256);
ALTER TABLE iam.signup_attribution ALTER COLUMN gclid SET NOT NULL;

ALTER TABLE iam.signup_attribution
    DROP COLUMN activation_uploaded_at,
    DROP COLUMN signup_uploaded_at,
    DROP COLUMN organization__id;
