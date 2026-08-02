-- Removes the "activation" server-side conversion (fired when an organization
-- created its first API key or service token). That signal was superseded by
-- the "first event sent" conversion as the mid-funnel activation signal, and its
-- own trigger became a hollow ~100% signal once a default API key is
-- auto-created at organization setup. The funnel is now: signup → first event
-- sent → first webhook delivered.
--
-- The gclid is no longer gated on activation before being minimised: it is
-- cleared once signup (always) plus the enabled first-event / first-webhook-
-- delivered conversions have been uploaded. See google_ads.rs
-- (clear_gclid_if_fully_uploaded_by_org / _by_user) and
-- documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md.
--
-- Only the activation timestamp column is dropped. organization__id,
-- signup_uploaded_at, first_event_uploaded_at and
-- first_webhook_delivered_uploaded_at are still used by the surviving
-- conversions and are left untouched.

ALTER TABLE iam.signup_attribution
    DROP COLUMN activation_uploaded_at;
