# Event Processing Model

Hook0's event processing model is designed for reliability, scalability, and observability. This document explains how events flow through the system from ingestion to delivery.

## Event Lifecycle

### 1. Event Creation
Events begin their lifecycle when applications send them to Hook0:

```http
POST /api/v1/events
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

When an event is stored, Hook0 identifies matching subscriptions:

#### Matching Logic
```sql
SELECT * FROM subscriptions 
WHERE application_id = $1 
  AND is_enabled = true
  AND $2 = ANY(event_types)  -- event_type matching
  AND deleted_at IS NULL
```

#### Event Type Patterns
Subscriptions can match:
- **Exact types**: `user.created`
- **Wildcard patterns**: `user.*` (if supported)
- **Multiple types**: `["user.created", "user.updated"]`

### 3. Delivery Task Creation

For each matching subscription, Hook0 creates a delivery task:

```rust
struct DeliveryTask {
    id: Uuid,
    event_id: Uuid,
    subscription_id: Uuid,
    attempt_number: u32,
    scheduled_at: DateTime<Utc>,
    status: DeliveryStatus,
}
```

#### Initial Scheduling
- First delivery attempt: immediate
- Subsequent retries: exponential backoff
- Maximum retry limit: configurable per subscription

### 4. Worker Processing

The worker process continuously polls for pending delivery tasks:

#### Task Selection
```sql
SELECT * FROM delivery_tasks
WHERE scheduled_at <= NOW()
  AND status = 'pending'
ORDER BY scheduled_at ASC
LIMIT 100
```

#### Concurrency Control
- Configurable worker threads
- Task locking to prevent duplicates
- Graceful shutdown handling

### 5. Webhook Delivery

For each delivery task, the worker:

#### HTTP Request Construction
```rust
// Construct webhook payload
let webhook_payload = WebhookPayload {
    event_id: event.id,
    event_type: event.event_type,
    payload: event.payload,
    timestamp: event.created_at,
    labels: event.labels,
};

// Add signature header
let signature = hmac_sha256(&subscription.secret, &payload_json);
let headers = vec![
    ("Hook0-Signature", format!("sha256={}", signature)),
    ("Hook0-Event-Type", event.event_type.clone()),
    ("User-Agent", "Hook0/1.0"),
];
```

#### Request Execution
```rust
let response = http_client
    .request(subscription.target.method.clone(), &subscription.target.url)
    .headers(headers)
    .json(&webhook_payload)
    .timeout(Duration::from_secs(30))
    .send()
    .await;
```

### 6. Response Handling

Hook0 categorizes responses to determine next actions:

#### Success Responses (2xx)
- Mark delivery as successful
- Record response details
- No further action needed

#### Client Errors (4xx)
- Mark as permanently failed
- Do not retry (except 408, 429)
- Log for debugging

#### Server Errors (5xx) & Network Issues
- Schedule retry with exponential backoff
- Increment attempt counter
- Eventually move to dead letter queue

#### Retry Schedule
```rust
fn calculate_retry_delay(attempt: u32) -> Duration {
    let base_delay = Duration::from_secs(30);
    let max_delay = Duration::from_hours(24);
    
    let delay = base_delay * 2_u32.pow(attempt.saturating_sub(1));
    std::cmp::min(delay, max_delay)
}
```

### 7. Request Attempt Tracking

Every delivery attempt is recorded:

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

## Advanced Features

### Dead Letter Queues
Events that exceed maximum retry attempts are moved to dead letter queues:
- Preserved for manual inspection
- Can be manually re-queued
- Configurable retention period

### Event Ordering
Hook0 provides configurable ordering guarantees:
- **At-least-once**: Default delivery guarantee
- **Order preservation**: Optional per-subscription ordering
- **Idempotency**: Event IDs for deduplication

### Payload Transformation
Subscriptions can define payload transformations:
- Field filtering
- Format conversion
- Custom headers
- Template-based payloads

### Conditional Delivery
Advanced filtering based on:
- Payload content
- Event labels
- Custom predicates
- Time-based conditions

## Performance Characteristics

### Throughput
- **Event ingestion**: 10,000+ events/second
- **Webhook delivery**: 1,000+ concurrent requests
- **Database queries**: Optimized with proper indexing

### Latency
- **Ingestion to storage**: < 10ms
- **First delivery attempt**: < 100ms
- **End-to-end delivery**: Depends on target response time

### Resource Usage
- **Memory**: Scales with concurrent workers
- **CPU**: Efficient async processing
- **Network**: Connection pooling and reuse

## Monitoring & Observability

### Metrics Collected
- Event ingestion rate
- Delivery success/failure rates
- Response time percentiles
- Queue depths and processing delays

### Logging
- Structured logging with correlation IDs
- Error details and stack traces  
- Performance metrics
- Security audit logs

### Health Checks
- Database connectivity
- Worker process status
- Queue health monitoring
- External dependency checks

## Failure Scenarios

### Database Outages
- Events cached in memory temporarily
- Graceful degradation with backpressure
- Automatic recovery when database returns

### Target Endpoint Failures
- Exponential backoff prevents thundering herd
- Circuit breaker patterns for chronic failures
- Dead letter queues for permanent failures

### Worker Process Failures
- Tasks automatically recovered by other workers
- No event loss due to persistent storage
- Monitoring alerts on worker health

## Configuration Options

### Global Settings
- Maximum retry attempts
- Retry delay configuration
- Worker concurrency limits
- Timeout values

### Per-Subscription Settings
- Custom retry strategies
- Payload transformation rules
- Authentication headers
- Rate limiting

## Next Steps

- [Security Model](./security-model.md)
- [Scaling and Performance](./scaling-performance.md)
- [Debugging Failed Webhooks](../how-to-guides/debug-failed-webhooks.md)