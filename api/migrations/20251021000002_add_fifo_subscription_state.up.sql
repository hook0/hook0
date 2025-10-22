-- Create table to track in-flight request attempts for FIFO subscriptions
-- This ensures only one webhook is processed at a time per FIFO subscription

CREATE TABLE webhook.fifo_subscription_state (
    subscription__id UUID NOT NULL PRIMARY KEY,
    current_request_attempt__id UUID,
    last_completed_event_created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),

    CONSTRAINT fk_fifo_subscription
        FOREIGN KEY (subscription__id)
        REFERENCES webhook.subscription(subscription__id)
        ON DELETE CASCADE,

    CONSTRAINT fk_fifo_current_attempt
        FOREIGN KEY (current_request_attempt__id)
        REFERENCES webhook.request_attempt(request_attempt__id)
        ON DELETE SET NULL
);

-- Index for efficient lookup of current in-flight attempts
CREATE INDEX idx_fifo_state_current_attempt
ON webhook.fifo_subscription_state(current_request_attempt__id)
WHERE current_request_attempt__id IS NOT NULL;

-- Index for monitoring and cleanup queries
CREATE INDEX idx_fifo_state_updated_at
ON webhook.fifo_subscription_state(updated_at);

COMMENT ON TABLE webhook.fifo_subscription_state IS
'Tracks the current in-flight request attempt for FIFO-enabled subscriptions to enforce strict ordering. Only one webhook can be in-flight at a time for each FIFO subscription.';

COMMENT ON COLUMN webhook.fifo_subscription_state.current_request_attempt__id IS
'The request attempt currently being processed or scheduled for retry. NULL means the subscription is ready to process the next event.';

COMMENT ON COLUMN webhook.fifo_subscription_state.last_completed_event_created_at IS
'Timestamp of the last event that was successfully processed or exhausted all retries. Used for monitoring and debugging.';
