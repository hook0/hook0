-- Bounds how many verification emails the public "resend verification email"
-- endpoint may send to one account over a window, on top of the per-send
-- cooldown carried by email_verification_sent_at.
--
-- The cooldown alone only spaces sends out; it does not bound their total, so a
-- distributed caller could keep one mailbox busy indefinitely. These two columns
-- carry a counter and the start of the window it belongs to, both maintained by
-- the same atomic UPDATE that claims the right to send.

ALTER TABLE iam."user"
    ADD COLUMN email_verification_resend_window_started_at TIMESTAMPTZ,
    ADD COLUMN email_verification_resend_count INTEGER NOT NULL DEFAULT 0;
