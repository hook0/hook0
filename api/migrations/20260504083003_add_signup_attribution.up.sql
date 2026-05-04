-- Stores the Google Ads click identifier (gclid) captured at /register so
-- the actual conversion can be uploaded server-side once the user verifies
-- their email (which filters out throwaway / bot signups).
--
-- The gclid is a pseudonymous identifier issued by Google Ads. It is held
-- only between /register and verify_email (typically minutes, sometimes
-- hours), then deleted as soon as the conversion has been uploaded. A
-- safety-net 30-day cleanup avoids accumulating rows for users who never
-- verify their email — this matches the conversion attribution window
-- configured in Google Ads and the legitimate-interest balance test
-- documented under documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md.
CREATE TABLE iam.signup_attribution (
    user__id UUID PRIMARY KEY REFERENCES iam.user(user__id) ON DELETE CASCADE,
    gclid TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    CONSTRAINT signup_attribution_gclid_length CHECK (char_length(gclid) BETWEEN 1 AND 256)
);

-- Drives the periodic 30-day cleanup that runs lazily on each signup.
CREATE INDEX signup_attribution_created_at_idx
    ON iam.signup_attribution (created_at);
