-- Extends the request_attempt table to record *why* an attempt was created
-- (initial dispatch, automatic retry, or manual retry) and *who* triggered
-- manual retries (user audit trail).
--
-- Five steps, each designed to avoid long locks on the hot request_attempt table:
--   1. ADD COLUMN with DEFAULT (catalog-only on PG 11+)
--   2. ADD CHECK NOT VALID (no scan)
--   3. VALIDATE separately (weaker lock — DML continues)
--   4. ADD triggered_by FK column
--   5. Partial index on triggered_by (only manual retries populate it)

-- Fail fast if a long-running transaction is holding a lock on the table.
-- Without this, our ALTER would queue behind it AND block all subsequent
-- writes until it completes — potentially cascading into a full outage.
set lock_timeout = '5s';

-- ADD COLUMN with a DEFAULT value. On PG 11+ this is catalog-only (no table
-- rewrite), so it completes instantly even on a table with billions of rows.
-- If we omitted the DEFAULT, existing rows would need backfilling separately.
alter table webhook.request_attempt
  add column attempt_trigger text not null default 'dispatch';

-- Add the CHECK constraint as NOT VALID. This skips scanning existing rows,
-- which would hold an ACCESS EXCLUSIVE lock for the duration of the scan —
-- potentially minutes on a large table, blocking all reads and writes.
alter table webhook.request_attempt
  add constraint trigger_chk
    check (attempt_trigger in ('dispatch', 'auto_retry', 'manual_retry'))
    not valid;

-- VALIDATE the constraint in a separate statement. This takes a weaker
-- SHARE UPDATE EXCLUSIVE lock that allows concurrent INSERTs, UPDATEs, and
-- DELETEs to continue. If steps 2 and 3 were combined, the validation scan
-- would hold the stronger lock from step 2.
alter table webhook.request_attempt
  validate constraint trigger_chk;

-- Nullable FK for user attribution. NULL when the caller is a service token
-- or the system itself (no user to attribute).
-- ON DELETE SET NULL: if the user who triggered a retry is later deleted from
-- iam.user, we keep the attempt row (for audit/billing) but lose the
-- attribution. CASCADE would destroy the attempt row, losing delivery history.
alter table webhook.request_attempt
  add column triggered_by uuid references iam.user on delete set null on update no action;

-- Partial index: only manual retries populate triggered_by (<0.01% of rows),
-- so this index stays tiny Without it, a DELETE on iam.user would require a
-- full table scan to find FK references.
--
-- Ideally this would use CREATE INDEX CONCURRENTLY to avoid blocking writes,
-- but sqlx::migrate!() runs each file in a transaction, which is incompatible
-- with CONCURRENTLY. Since the partial index covers 0 rows at migration time
-- (no manual retries exist yet), the build is instantaneous regardless.
create index triggered_by_idx
  on webhook.request_attempt (triggered_by)
  where triggered_by is not null;

comment on column webhook.request_attempt.attempt_trigger
  is 'Why this attempt was created: dispatch (initial delivery), auto_retry (worker successor after failure), manual_retry (user-initiated one-shot via API or UI).';

comment on column webhook.request_attempt.triggered_by
  is 'User who initiated a manual retry. null for system-created attempts (dispatch, auto_retry) and service-token callers with no user identity.';

reset lock_timeout;
