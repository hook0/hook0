-- Adds a FOURTH server-side conversion signal to iam.signup_attribution:
-- "first webhook delivered" (the organization delivered at least one webhook
-- successfully — a webhook.request_attempt whose succeeded_at is set), uploaded
-- by a background job as its own Google Ads conversion action. This is the
-- north-star activation event: the moment a customer's integration actually
-- receives a webhook end to end (not merely the event being ingested).
--
-- Like the first-event conversion, this happens after email verification and
-- after activation, so the gclid must be retained until this conversion has
-- been uploaded too. The clear_gclid_*_by_org logic in google_ads.rs therefore
-- also waits on first_webhook_delivered_uploaded_at, but only when
-- first-webhook-delivered tracking is enabled (so data minimisation still nulls
-- the gclid on instances that do not track it). Still no PII leaves Hook0: only
-- the pseudonymous gclid (already issued by Google at ad-click) is sent back.
-- See documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md.

ALTER TABLE iam.signup_attribution
    ADD COLUMN first_webhook_delivered_uploaded_at TIMESTAMPTZ;

-- Drives the background scan for organizations whose first-webhook-delivered
-- conversion is still pending. Partial so the index stays tiny (only
-- un-uploaded, still-attributed rows) and the scan is a cheap, well-bounded
-- lookup. No index is added on the hot webhook.request_attempt table (its
-- writes are on the delivery hot path); the pending scan is bounded per pass
-- and probes the existing request_attempt (application__id) index.
CREATE INDEX signup_attribution_first_webhook_delivered_pending_idx
    ON iam.signup_attribution (organization__id)
    WHERE first_webhook_delivered_uploaded_at IS NULL AND gclid IS NOT NULL;
