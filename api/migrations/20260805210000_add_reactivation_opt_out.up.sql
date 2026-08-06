-- Records that a user asked to stop receiving the "0 event sent" onboarding
-- reminders, so the opt-out offered in every one of those emails is something
-- the job actually enforces rather than a promise handled off-system.
--
-- Scoped to the reactivation drip on purpose: it must never suppress
-- transactional mail (email verification, password reset), which is not
-- marketing and which the user still needs.
ALTER TABLE iam."user"
    ADD COLUMN reactivation_opted_out_at TIMESTAMPTZ;
