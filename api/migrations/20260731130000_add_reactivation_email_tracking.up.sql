-- Tracks which "0 event sent" reactivation email (drip step) has already been
-- sent to an organization, so the background job never sends the same step
-- twice. One row per (organization, step); the primary key makes each step an
-- exclusive claim (idempotent across passes AND across API instances).
--
-- Steps: 1 = J+1, 2 = J+3, 3 = J+7. The series stops as soon as the org sends
-- its first event (the selection query filters those out) or after the last
-- step. The row is removed only via ON DELETE CASCADE when its organization is
-- deleted, or by the job itself when a send fails (so a later pass can retry).
CREATE TABLE iam.reactivation_email (
    organization__id UUID NOT NULL REFERENCES iam.organization(organization__id) ON DELETE CASCADE,
    step SMALLINT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    PRIMARY KEY (organization__id, step),
    CONSTRAINT reactivation_email_step_chk CHECK (step BETWEEN 1 AND 3)
);
