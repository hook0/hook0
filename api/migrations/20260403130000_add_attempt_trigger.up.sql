SET lock_timeout = '5s';

-- Step 1: add column with default (instant on PG 11+ — stored in catalog, no rewrite)
ALTER TABLE webhook.request_attempt
  ADD COLUMN attempt_trigger TEXT NOT NULL DEFAULT 'dispatch';

-- Step 2: CHECK as NOT VALID — no table scan, no ACCESS EXCLUSIVE hold
ALTER TABLE webhook.request_attempt
  ADD CONSTRAINT request_attempt_trigger_check
    CHECK (attempt_trigger IN ('dispatch', 'auto_retry', 'manual_retry'))
    NOT VALID;

-- Step 3: validate separately — SHARE UPDATE EXCLUSIVE lock, concurrent DML OK
VALIDATE CONSTRAINT request_attempt_trigger_check;

-- Step 4: nullable FK for user attribution (NULL for system/service-token callers)
ALTER TABLE webhook.request_attempt
  ADD COLUMN triggered_by UUID REFERENCES iam.user;

RESET lock_timeout;
