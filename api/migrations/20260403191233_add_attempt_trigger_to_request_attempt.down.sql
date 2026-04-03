ALTER TABLE webhook.request_attempt
    DROP COLUMN triggered_by,
    DROP COLUMN attempt_trigger;

DROP TYPE webhook.attempt_trigger_type;
