-- Gives a password reset link something to be invalidated against.
--
-- The link itself is a signed token with no server-side row, so nothing used to
-- tell a link that had already reset a password from one that had not, and
-- issuing a new link did not retire the previous ones. The token now carries
-- this value, and every write that sets a password rotates it: a link is
-- accepted only while the nonce it carries is still the one on the row, so
-- using a link, asking for another one, or changing the password from the
-- account settings all retire whatever links were outstanding.
--
-- Defaulted rather than nullable so existing rows get a value they never had to
-- match, and so no code path has to reason about an account without one.

ALTER TABLE iam."user"
    ADD COLUMN password_reset_nonce UUID NOT NULL DEFAULT public.gen_random_uuid();
