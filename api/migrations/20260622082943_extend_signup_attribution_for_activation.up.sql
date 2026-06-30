-- Extends iam.signup_attribution to support a SECOND server-side conversion:
-- "activation" (fired when an organization creates its first API key), in
-- addition to the existing "signup" conversion (fired at email verification).
--
-- Because activation happens AFTER email verification, the gclid can no longer
-- be deleted at verification time. It is now retained until BOTH conversions
-- (signup + activation) have been uploaded to Google Ads, OR until the 30-day
-- cleanup fires (whichever comes first) — 30 days matches the Google Ads
-- attribution window. As soon as both conversions are uploaded, the gclid is
-- set to NULL (data minimisation). Still no PII leaves Hook0: only the
-- pseudonymous gclid (already issued by Google at ad-click) is sent back.
-- See documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md.

ALTER TABLE iam.signup_attribution
    ADD COLUMN organization__id UUID REFERENCES iam.organization(organization__id) ON DELETE CASCADE,
    ADD COLUMN signup_uploaded_at TIMESTAMPTZ,
    ADD COLUMN activation_uploaded_at TIMESTAMPTZ;

-- gclid becomes nullable so it can be cleared once both conversions are
-- uploaded. Replace the NOT NULL length check with a NULL-tolerant one.
ALTER TABLE iam.signup_attribution ALTER COLUMN gclid DROP NOT NULL;
ALTER TABLE iam.signup_attribution DROP CONSTRAINT signup_attribution_gclid_length;
ALTER TABLE iam.signup_attribution
    ADD CONSTRAINT signup_attribution_gclid_length
    CHECK (gclid IS NULL OR char_length(gclid) BETWEEN 1 AND 256);

-- Drives the activation lookup (claim by organization on first API key).
CREATE INDEX signup_attribution_organization_idx
    ON iam.signup_attribution (organization__id);
