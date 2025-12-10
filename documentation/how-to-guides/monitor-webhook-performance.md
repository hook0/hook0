# Monitor Webhook Performance

Monitoring webhook delivery performance is crucial for ensuring reliability and diagnosing issues in your webhook infrastructure. Hook0 provides multiple mechanisms for tracking webhook performance, delivery status, and error patterns.

## Overview

Hook0 tracks every webhook delivery attempt through the `request_attempts` system. Each attempt records:
- HTTP request details (method, URL, headers)
- Response data (status code, headers, body)
- Timing information (created_at, picked_at, succeeded_at/failed_at)
- Worker information (worker_name, worker_version)
- Retry count and scheduling

## Using the Hook0 Dashboard

The Hook0 web interface provides visual monitoring of webhook delivery:

### Event Monitoring

Navigate to the Events page to see:
- Recent events ingested into Hook0
- Event types and payloads
- Associated subscriptions triggered
- Delivery status for each subscription

### Request Attempts View

For each event, you can view:
- All delivery attempts (initial + retries)
- HTTP response codes and bodies
- Elapsed time for each attempt
- Error categories (connection, timeout, HTTP error)
- Retry schedule and next attempt time

### Subscription Health

Monitor subscription-level metrics:
- Success rate over time
- Average response time
- Recent failures and error patterns
- Retry queue depth

## API Endpoints for Monitoring

### List Request Attempts

Query request attempts programmatically:

```bash
curl -X GET "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"

# Filter by specific event
curl -X GET "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&event_id={EVENT_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Query Parameters:**
- `application_id`: **(Required)** Filter by application
- `event_id`: Filter by specific event
- `subscription_id`: Filter by subscription
- `pagination_cursor`: For pagination
- `min_created_at` / `max_created_at`: Filter by time range

**Response Example:**

```json
[
  {
    "request_attempt_id": "uuid",
    "event_id": "uuid",
    "subscription_id": "uuid",
    "created_at": "2025-12-10T10:00:00Z",
    "picked_at": "2025-12-10T10:00:01Z",
    "succeeded_at": "2025-12-10T10:00:03Z",
    "failed_at": null,
    "retry_count": 0,
    "worker_name": "worker-01",
    "worker_version": "0.1.0",
    "response_id": "uuid",
    "delay_until": null
  }
]
```

### Get Response Details

Fetch HTTP response details for a specific attempt:

```bash
curl -X GET "http://localhost:8081/api/v1/responses/{RESPONSE_ID}?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Response Example:**

```json
{
  "response_id": "uuid",
  "http_code": 200,
  "headers": {
    "content-type": "application/json",
    "x-request-id": "abc123"
  },
  "body": "{\"status\": \"received\"}",
  "elapsed_time_ms": 145,
  "response_error_name": null
}
```

### List Events with Filtering

Query events to find patterns:

```bash
curl -X GET "http://localhost:8081/api/v1/events/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Query Parameters:**
- `application_id`: **(Required)** Filter by application

## Key Metrics to Monitor

### 1. Success Rate

Track the percentage of webhook deliveries that succeed on first attempt:

```sql
-- Example query pattern
SELECT
  COUNT(*) FILTER (WHERE succeeded_at IS NOT NULL AND retry_count = 0) * 100.0 / COUNT(*) as success_rate
FROM webhook.request_attempt
WHERE created_at > NOW() - INTERVAL '1 day';
```

**Healthy Range:** > 95% for first-attempt success

### 2. Average Response Time

Monitor how quickly target endpoints respond:

```sql
-- Example query pattern
SELECT AVG(r.elapsed_time_ms) as avg_response_time_ms
FROM webhook.response r
INNER JOIN webhook.request_attempt ra ON ra.response__id = r.response__id
WHERE ra.created_at > NOW() - INTERVAL '1 hour';
```

**Healthy Range:** < 1000ms (1 second) for most endpoints

### 3. Retry Rate

Percentage of deliveries requiring retries:

```sql
-- Example query pattern
SELECT
  COUNT(DISTINCT event__id) FILTER (WHERE retry_count > 0) * 100.0 /
  COUNT(DISTINCT event__id) as retry_rate
FROM webhook.request_attempt
WHERE created_at > NOW() - INTERVAL '1 day';
```

**Healthy Range:** < 5% of events requiring retries

### 4. Error Distribution

Categorize errors to identify patterns:

```sql
-- Example query pattern
SELECT
  r.response_error__name,
  COUNT(*) as error_count
FROM webhook.response r
WHERE r.response_error__name IS NOT NULL
GROUP BY r.response_error__name
ORDER BY error_count DESC;
```

**Common Error Types:**
- `E_CONNECTION`: Network connectivity issues
- `E_TIMEOUT`: Request exceeded timeout
- `E_HTTP`: Non-2xx HTTP response
- `E_INVALID_TARGET`: Invalid URL or configuration

### 5. Delivery Latency

Time from event creation to successful delivery:

```sql
-- Example query pattern
SELECT
  EXTRACT(EPOCH FROM (ra.succeeded_at - ra.created_at)) as delivery_latency_seconds
FROM webhook.request_attempt ra
WHERE ra.succeeded_at IS NOT NULL
  AND ra.created_at > NOW() - INTERVAL '1 hour';
```

**Healthy Range:** < 30 seconds for immediate deliveries

## Sentry Integration

Hook0 supports optional Sentry integration for advanced error tracking.

### API Server Configuration

```bash
SENTRY_DSN=https://your-key@sentry.io/project-id
SENTRY_TRACES_SAMPLE_RATE=0.1  # Sample 10% of transactions
```

### Output Worker Configuration

```bash
--sentry-dsn=https://your-key@sentry.io/project-id
```

### What Sentry Captures

**Errors:**
- Failed webhook deliveries (after retries exhausted)
- Database connection issues
- Invalid configuration problems
- Unexpected runtime errors

**Performance Traces:**
- API endpoint response times
- Database query performance
- Webhook delivery timing

**Context:**
- Event IDs, subscription IDs
- HTTP request/response details
- Worker information
- Stack traces for exceptions

### Sentry Alerts

Configure alerts in Sentry for:
- Spike in failed webhook deliveries
- Slow database queries
- High error rates
- Performance degradation

## Monitoring Best Practices

### 1. Set Up Baseline Metrics

Establish normal ranges for your webhook traffic:
- Average daily event volume
- Typical success rates
- Expected response times
- Common error patterns

### 2. Create Alerting Thresholds

Define when to alert based on deviation from baseline:
- Success rate drops below 90%
- Average response time exceeds 3 seconds
- Error rate spikes above 10%
- Retry queue grows beyond expected size

### 3. Monitor Endpoint Health

Track per-endpoint metrics:
- Which subscriptions have lowest success rates
- Which target URLs timeout most frequently
- Correlation between endpoint and error types

**Example API Query:**

```bash
# Get request attempts for specific subscription
curl -X GET "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

### 4. Track Retry Patterns

Monitor retry behavior:
- How many events require retries
- Average retry count before success
- Time spent in retry loops
- Events that exhaust all retries

### 5. Correlate with External Systems

Cross-reference Hook0 metrics with:
- Your application's event generation patterns
- Target endpoint uptime/availability
- Network infrastructure health
- Database performance metrics

## Debugging Failed Webhooks

When webhooks fail, follow this diagnosis flow:

### Step 1: Identify the Failure

Use the dashboard or API to find failed attempts:

```bash
# List recent failed attempts
curl -X GET "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}" \
  | jq '.[] | select(.failed_at != null)'
```

### Step 2: Examine Response Details

Get the HTTP response for the failed attempt:

```bash
curl -X GET "http://localhost:8081/api/v1/responses/{RESPONSE_ID}?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

Look for:
- HTTP status code (4xx client error, 5xx server error)
- Response body with error message
- Headers that might indicate rate limiting or auth issues

### Step 3: Check Error Category

The `response_error_name` field categorizes the failure:
- **`E_CONNECTION`**: Target unreachable, DNS failure, network issue
- **`E_TIMEOUT`**: Request exceeded configured timeout (default 15s)
- **`E_HTTP`**: Non-2xx response (check response body for details)
- **`E_INVALID_TARGET`**: Malformed URL or forbidden IP address

### Step 4: Review Event Payload

Ensure the event payload is valid:

```bash
curl -X GET "http://localhost:8081/api/v1/events/{EVENT_ID}?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

Check for:
- Correct event type
- Valid JSON structure
- Expected payload fields
- Appropriate content type

### Step 5: Test Target Endpoint

Manually verify the target endpoint:

```bash
# Simulate Hook0 webhook request
curl -X POST "https://your-endpoint.com/webhook" \
  -H "Content-Type: application/json" \
  -H "X-Event-Type: users.account.created" \
  -H "X-Event-Id: test-event-id" \
  -H "X-Hook0-Signature: t=1234567890,v1=..." \
  -d '{"user_id": "123", "email": "test@example.com"}'
```

### Step 6: Verify Signature (if endpoint validates)

If the target endpoint validates webhook signatures, ensure:
- Subscription secret is correctly configured
- Signature header name matches (default: `X-Hook0-Signature`)
- Endpoint signature validation logic is correct

See [Secure Webhook Endpoints](secure-webhook-endpoints.md) for signature verification details.

## Creating Custom Dashboards

### Using the API for Custom Metrics

Build your own monitoring dashboard by polling Hook0's API:

```javascript
// Example: Fetch recent request attempts and calculate metrics
async function fetchWebhookMetrics(applicationId, token) {
  const response = await fetch(
    `http://localhost:8081/api/v1/request_attempts?application_id=${applicationId}&limit=1000`,
    {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    }
  );

  const attempts = await response.json();

  const metrics = {
    total: attempts.length,
    succeeded: attempts.filter(a => a.succeeded_at).length,
    failed: attempts.filter(a => a.failed_at).length,
    pending: attempts.filter(a => !a.succeeded_at && !a.failed_at).length,
    avgRetries: attempts.reduce((sum, a) => sum + a.retry_count, 0) / attempts.length,
  };

  return metrics;
}
```

### Integration with Monitoring Tools

Export Hook0 metrics to your monitoring stack:

**Prometheus/Grafana:**
- Create a metrics exporter that queries Hook0 API
- Expose metrics in Prometheus format
- Build Grafana dashboards with alerts

**Datadog/New Relic:**
- Use custom metric submission APIs
- Schedule periodic API queries to collect metrics
- Set up anomaly detection and alerting

**CloudWatch/Azure Monitor:**
- Use serverless functions to poll Hook0 API
- Push metrics to cloud monitoring services
- Configure alarms based on thresholds

## Performance Optimization Based on Monitoring

### Identifying Slow Endpoints

Query for endpoints with high response times:

```bash
# Find request attempts for your application
curl -X GET "http://localhost:8081/api/v1/request_attempts/?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

Then fetch response details to check elapsed time:

```bash
# Get response details including elapsed_time_ms
curl -X GET "http://localhost:8081/api/v1/responses/{RESPONSE_ID}?application_id={APP_ID}" \
  -H "Authorization: Bearer {YOUR_TOKEN}"
```

**Actions:**
- Contact target endpoint operators about performance
- Consider adjusting timeouts if endpoint is legitimately slow
- Implement caching or async processing on target side

### Reducing Retry Overhead

If many events require retries:
- Investigate why first attempts fail
- Fix common issues (authentication, validation errors)
- Improve target endpoint reliability
- Consider adjusting retry strategy if appropriate

### Scaling Workers

Monitor worker performance:
- Check if workers are keeping up with event volume
- Look for growing `delay_until` queues
- Scale worker `--concurrent` setting or add workers

See [Scaling and Performance](../explanation/scaling-performance.md) for scaling guidance.

## Summary

Effective webhook monitoring involves:
1. **Real-time Visibility**: Use dashboard and API for current status
2. **Metric Tracking**: Monitor success rates, response times, errors
3. **Error Analysis**: Categorize and diagnose failure patterns
4. **Alerting**: Set thresholds for critical metrics
5. **Integration**: Connect to external monitoring tools (Sentry, etc.)
6. **Optimization**: Use insights to improve reliability and performance

Regular monitoring helps maintain high webhook delivery reliability and quickly resolve issues when they occur.

---

*For debugging specific webhook failures, see [Debug Failed Webhooks](debug-failed-webhooks.md). For performance tuning, see [Scaling and Performance](../explanation/scaling-performance.md).*
