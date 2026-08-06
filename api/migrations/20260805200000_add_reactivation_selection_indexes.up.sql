-- Back the reactivation drip's periodic selection query.
--
-- The job now runs by default on every API instance, so its scan must stay cheap
-- as iam.user grows. `select_candidates` narrows verified users to a sign-up-age
-- window and then looks up their organizations by `created_by`, so both columns
-- get an index and the user-side one is partial: only verified accounts are ever
-- candidates, which keeps the index small and skips the majority of rows.
CREATE INDEX reactivation_verified_signup_idx
    ON iam."user" (created_at)
    WHERE email_verified_at IS NOT NULL;

CREATE INDEX organization_created_by_idx
    ON iam.organization (created_by);
