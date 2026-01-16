-- Add soft delete and deletion request columns to iam.user for GDPR compliance (Art. 17)
ALTER TABLE iam.user ADD COLUMN deleted_at TIMESTAMPTZ;
ALTER TABLE iam.user ADD COLUMN deletion_requested_at TIMESTAMPTZ;

-- Create index for efficient cleanup queries
CREATE INDEX user_deletion_requested_at_idx ON iam.user (deletion_requested_at) WHERE deletion_requested_at IS NOT NULL;
