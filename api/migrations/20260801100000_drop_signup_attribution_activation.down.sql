-- Restore the activation timestamp column (nullable, matching its original type).
ALTER TABLE iam.signup_attribution
    ADD COLUMN activation_uploaded_at TIMESTAMPTZ;
