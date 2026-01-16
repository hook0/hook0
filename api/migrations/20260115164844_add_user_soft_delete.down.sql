-- Remove soft delete columns from iam.user
DROP INDEX IF EXISTS iam.user_deletion_requested_at_idx;
ALTER TABLE iam.user DROP COLUMN IF EXISTS deletion_requested_at;
ALTER TABLE iam.user DROP COLUMN IF EXISTS deleted_at;
