-- Adds a THIRD server-side conversion signal to iam.signup_attribution:
-- "first event sent" (the organization has ingested at least one event),
-- uploaded by a background job as its own Google Ads conversion action.
--
-- Like activation, this happens after email verification, so the gclid must be
-- retained until this conversion has been uploaded too. The clear_gclid_* logic
-- in google_ads.rs therefore also waits on first_event_uploaded_at, but only
-- when first-event tracking is enabled (so data minimisation still nulls the
-- gclid right after signup + activation on instances that do not track the
-- first event). Still no PII leaves Hook0: only the pseudonymous gclid (already
-- issued by Google at ad-click) is sent back.
-- See documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md.

ALTER TABLE iam.signup_attribution
    ADD COLUMN first_event_uploaded_at TIMESTAMPTZ;

-- Drives the background scan for organizations whose first-event conversion is
-- still pending. Partial so the index stays tiny (only un-uploaded, still-
-- attributed rows) and the scan is a cheap, well-bounded lookup.
CREATE INDEX signup_attribution_first_event_pending_idx
    ON iam.signup_attribution (organization__id)
    WHERE first_event_uploaded_at IS NULL AND gclid IS NOT NULL;
