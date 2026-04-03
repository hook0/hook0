-- Add attempt_trigger (what caused this attempt) and triggered_by (which user triggered it)
-- to support manual retry tracking.
-- Default 'scheduled' so all existing rows are correctly labeled as system-scheduled attempts.
CREATE TYPE webhook.attempt_trigger_type AS ENUM ('scheduled', 'manual_retry');

ALTER TABLE webhook.request_attempt
    ADD COLUMN attempt_trigger webhook.attempt_trigger_type NOT NULL DEFAULT 'scheduled',
    ADD COLUMN triggered_by UUID REFERENCES iam.user (user__id) ON DELETE SET NULL;
