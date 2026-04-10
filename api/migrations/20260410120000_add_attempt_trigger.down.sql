SET lock_timeout = '5s';

DROP INDEX IF EXISTS webhook.idx_request_attempt_triggered_by;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS triggered_by;
ALTER TABLE webhook.request_attempt DROP CONSTRAINT IF EXISTS request_attempt_trigger_check;
ALTER TABLE webhook.request_attempt DROP COLUMN IF EXISTS attempt_trigger;

RESET lock_timeout;
