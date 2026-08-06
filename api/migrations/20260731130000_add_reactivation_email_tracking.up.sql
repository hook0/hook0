-- Tracks which "0 event sent" reactivation email (drip step) has already been
-- sent to a user, so the background job never sends the same step twice. One
-- row per (user, step); the primary key makes each step an exclusive claim,
-- idempotent across passes AND across API instances.
--
-- Keyed on the user rather than the organization because the mail lands in a
-- person's inbox and never names an organization: someone who registered two
-- dormant organizations must receive one J+1, not one per organization. Keying
-- it per organization would also let two API instances each claim a different
-- organization of the same reader and both send.
--
-- Steps: 1 = J+1, 2 = J+3, 3 = J+7. The series stops as soon as the user sends
-- a first event from any of their organizations (the selection query filters
-- those out) or after the last step. The row is removed only via ON DELETE
-- CASCADE when the user is deleted, or by the job itself when a send fails (so
-- a later pass can retry).
CREATE TABLE iam.reactivation_email (
    user__id UUID NOT NULL REFERENCES iam."user"(user__id) ON DELETE CASCADE,
    step SMALLINT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (user__id, step),
    CONSTRAINT reactivation_email_step_chk CHECK (step BETWEEN 1 AND 3)
);
