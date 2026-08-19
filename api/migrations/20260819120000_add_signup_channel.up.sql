-- Records WHERE a signup came from, as a bounded, non-identifying label, so a
-- move in registrations can be attributed to a source instead of guessed at.
--
-- Why this is needed: `iam.signup_attribution` only ever holds a Google Ads
-- `gclid`, only for the signups that clicked an ad, and it is purged after the
-- attribution window — it answers "which ad click", never "where do signups
-- come from". Web analytics only sees visitors who land on www.hook0.com,
-- while registrations happen on app.hook0.com; a signup that never loaded a
-- tracked www page is invisible to both. The origin of most registrations is
-- therefore simply unknown today.
--
-- What is stored is NOT a referrer URL. The browser maps the referrer and the
-- campaign parameters onto a closed vocabulary (`organic:google`,
-- `social:linkedin`, `referral:<host>`, `direct`, …) and the API re-validates
-- that label against the same grammar as the CHECK below, storing 'unknown'
-- for anything else. No URL, no query string, no identifier and no personal
-- data is persisted, which is what keeps this free of the minimisation and
-- retention machinery the gclid needs (see
-- documentation/hook0-cloud/legitimate-interest-balance-test-google-ads.md).
--
-- Existing rows keep the default: back-filling would need data nobody
-- captured. The default is non-volatile, so PostgreSQL adds the column without
-- rewriting the table.

ALTER TABLE iam.user
    ADD COLUMN signup_channel TEXT NOT NULL DEFAULT 'unknown';

-- Second barrier behind the API-side normalisation: whatever reaches this
-- column is one of a closed set of labels, each bounded in length. A future
-- caller that forgets to normalise gets a failed INSERT, not a free-text
-- column that silently becomes unqueryable.
ALTER TABLE iam.user
    ADD CONSTRAINT user_signup_channel_grammar CHECK (
        signup_channel ~ '^(unknown|direct|(ads|organic|ai|social|referral|campaign):[a-z0-9][a-z0-9.-]{0,63})$'
    );
