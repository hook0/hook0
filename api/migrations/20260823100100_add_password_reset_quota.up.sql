-- Bounds how many password reset emails the public "begin reset password"
-- endpoint may send to one address, in mirror of the verification email quota.
--
-- The endpoint answers the same way for every address, so nothing a caller can
-- read bounds it; without these columns a script can keep one mailbox busy
-- indefinitely, and the per-IP limiter in front of it does not follow a caller
-- that rotates source addresses. The cooldown spaces sends out, the counter and
-- its window anchor bound their total, and all three are maintained by the same
-- atomic UPDATE that claims the right to send.
--
-- Deliberately left NULL at signup, unlike email_verification_sent_at: an
-- account that has never asked for a reset link must get the first one it asks
-- for without waiting out a cooldown it did not spend.

ALTER TABLE iam."user"
    ADD COLUMN password_reset_sent_at TIMESTAMPTZ,
    ADD COLUMN password_reset_window_started_at TIMESTAMPTZ,
    ADD COLUMN password_reset_count INTEGER NOT NULL DEFAULT 0;
