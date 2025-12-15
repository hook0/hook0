# Event Processing Model

Hook0's event processing model is designed for reliability, scalability, and observability. This document explains how events flow through the system from ingestion to delivery.

## Event Lifecycle

### 1. Event Creation
Events begin their lifecycle when applications send them to Hook0:

```http
POST /api/v1/event
Authorization: Bearer <biscuit-token>
Content-Type: application/json

{
  "event_type": "order.completed",
  "payload": {
    "order_id": "ord_123",
    "customer_id": "cust_456",
    "amount": 99.99,
    "currency": "USD"
  },
  "labels": {
    "environment": "production",
    "region": "us-east-1"
  }
}
```

#### Validation Steps
1. **Authentication**: Verify Biscuit token and permissions
2. **Event Type**: Ensure event type exists for the application
3. **Payload**: Validate against schema if defined
4. **Quotas**: Check organization limits
5. **Rate Limits**: Enforce ingestion rate limits

#### Storage
Upon successful validation, events are stored with:
- Unique UUID identifier
- Timestamp (UTC)
- Application and organization context
- Original payload and metadata
- Source IP address

### 2. Subscription Matching

When an event is stored, Hook0 identifies matching subscriptions. Subscriptions can match:
- **Exact types**: `user.account.created`
- **Multiple types**: `["user.account.created", "user.account.updated"]`

### 3. Delivery Task Creation

For each matching subscription, Hook0 creates a delivery task.

#### Initial Scheduling
- First delivery attempt: immediate
- Subsequent retries: exponential backoff
- Maximum retry limit: configurable per subscription

### 4. Webhook Delivery

For each delivery task, the worker send the HTTP request.

### 5. Response Handling

Hook0 categorizes HTTP responses to determine next actions:

#### Success Responses (2xx)
- Mark delivery as successful
- Record response details
- No further action needed

#### Non-Success Responses (4xx, 5xx) & Network Issues
- Schedule retry with exponential backoff
- Increment attempt counter
- See [HTTP Status Code Categories](../how-to-guides/debug-failed-webhooks.md#http-status-code-categories) for retry behavior details
- Eventually move to dead letter queue after max retries exhausted


### 6. Request Attempt Tracking

Every delivery attempt is recorded for logging.

```rust
struct RequestAttempt {
    id: Uuid,
    event_id: Uuid,
    subscription_id: Uuid,
    attempt_number: u32,
    status_code: Option<u16>,
    response_body: Option<String>,
    error_message: Option<String>,
    duration_ms: u32,
    created_at: DateTime<Utc>,
}
```

#### Status Categories
- **Pending**: Not yet attempted
- **Success**: 2xx response received
- **Failed**: Non-2xx response or network error
- **Timeout**: Request exceeded timeout limit
- **Cancelled**: Delivery cancelled by user


## Next Steps

- [Security Model](./security-model.md)
- [Debugging Failed Webhooks](../how-to-guides/debug-failed-webhooks.md)
