-- Records when the last email-verification message was (re)sent to a user.
-- The "resend verification email" endpoint uses it to enforce a per-account
-- cooldown (anti-abuse: at most one resend per account per cooldown window),
-- while always answering identically so it never leaks whether an account
-- exists (anti-enumeration).

ALTER TABLE iam."user"
    ADD COLUMN email_verification_sent_at TIMESTAMPTZ;
