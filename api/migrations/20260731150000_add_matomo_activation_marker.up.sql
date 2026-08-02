-- Adds a per-organization one-shot marker for the server-side Matomo
-- "activation" event (category = activation, action = first-webhook-delivered).
--
-- Unlike the Google Ads conversions (which live on iam.signup_attribution and
-- only concern gclid-attributed organizations), this is a product-activation
-- Goal that must fire once for EVERY organization on its genuine first
-- successful webhook delivery. The marker therefore lives on iam.organization,
-- which is the only table that covers all organizations.
--
-- A background job claims eligible organizations by flipping this timestamp
-- from NULL under `WHERE matomo_activation_emitted_at IS NULL` (so the claim is
-- exclusive across instances and passes), emits the event server-side through
-- Matomo's HTTP Tracking API, and releases the claim (sets the marker back to
-- NULL) when the send fails so the organization stays eligible on the next
-- pass. No PII is sent: the Matomo visitor id is a fresh random value generated
-- per emission, never derived from nor stored against the organization or user.
--
-- The claim is exactly-once, but the emission is at-most-once: the marker is
-- committed before the send, so a process crash between claim and send leaves
-- the marker stamped and that organization's activation event is lost for good
-- (no reconciliation re-claims it). This is a deliberate trade-off — the
-- inverse of the at-least-once Google Ads conversion job — to avoid
-- over-counting a Goal that already de-dupes within a visit.
ALTER TABLE iam.organization
    ADD COLUMN matomo_activation_emitted_at TIMESTAMPTZ;

-- Keeps the pending scan tiny and cheap: only organizations whose activation
-- event has not been emitted yet. iam.organization is not a hot table
-- (organizations are created rarely). No index is added on the hot
-- webhook.request_attempt table (its writes are on the delivery hot path); the
-- eligibility EXISTS probe reuses the existing request_attempt (application__id)
-- index.
CREATE INDEX organization_matomo_activation_pending_idx
    ON iam.organization (organization__id)
    WHERE matomo_activation_emitted_at IS NULL;
