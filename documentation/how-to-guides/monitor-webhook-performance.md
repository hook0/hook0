---
title: "Webhook Monitoring: The 5 Delivery Metrics and the API Fields Behind Them"
sidebar_label: "Monitor webhook performance"
description: "Success rate, retry rate, error mix, endpoint response time and delivery latency: which Hook0 API field each one comes from, how to compute it, and when to alert."
keywords: [webhook monitoring, webhook observability, webhook alerting, webhook delivery metrics, webhook success rate, webhook error tracking, webhook error monitoring, webhook latency, webhook alerts, endpoint health]
faqItems:
  - question: "How do I monitor webhook delivery in Hook0?"
    answer: >-
      Every delivery try is stored as a request attempt with its status, retry
      count, HTTP status and timestamps, and links to the response your
      endpoint sent back. List them with GET /request_attempts/ (filter by
      subscription, event or time range), fetch the response with GET
      /responses/{response_id}, and compute success rate, retry rate, error
      mix and latency over a rolling window.
  - question: "Which metrics show that a webhook endpoint is unhealthy?"
    answer: >-
      Watch five numbers per subscription: first-attempt success rate, share
      of events that needed a retry, error names (E_TIMEOUT, E_CONNECTION,
      E_DNS, E_HTTP), endpoint response time from elapsed_time_ms, and
      delivery latency from created_at to succeeded_at. A falling success rate
      with a rising E_TIMEOUT count usually means a slow endpoint; a burst of
      E_HTTP means the handler is rejecting requests.
  - question: "What is a healthy webhook retry rate?"
    answer: >-
      Hook0 does not impose a threshold. A common starting point is to
      investigate when more than 5 percent of events need a retry, then tune
      that value to your own baseline. Read the error names before the rate:
      a spike in E_TIMEOUT or E_CONNECTION points at a slow or flaky endpoint
      rather than at the payload.
  - question: "How fast should a webhook be delivered?"
    answer: >-
      Measure it as succeeded_at minus created_at on the request attempt. For
      events sent without a delay, a first successful attempt within 30
      seconds is a reasonable target; alert when the p95 drifts above your
      baseline rather than on single slow deliveries.
  - question: "How do I track webhook errors?"
    answer: >-
      Group failed request attempts by the response_error_name of their
      response. E_CONNECTION, E_TIMEOUT and E_DNS are network problems,
      E_HTTP means your endpoint answered with a non-2xx status, and
      E_INVALID_TARGET means the subscription URL is wrong. Hook0 retries all
      of them on its schedule, so a growing count of the same error on one
      subscription is the signal to act on.
  - question: "What is webhook monitoring?"
    answer: >-
      Webhook monitoring is tracking whether the events you send reach your
      subscribers' endpoints: how many deliveries succeed on the first try,
      how many need a retry or are given up, how long endpoints take to
      answer, and which errors they return. On the sending side, it is done
      per subscription, from the delivery attempts the webhook server
      records, with alerts on success rate, retries and given-up deliveries.
  - question: "How do I get alerted when a webhook endpoint keeps failing?"
    answer: >-
      Poll GET /request_attempts/ per subscription and alert when the
      first-attempt success rate falls below your threshold over a 15-minute
      window, or when attempts pile up in the failed or paused status. On a
      self-hosted instance, export metrics over OTLP and alert on the
      webhook.delivery.outcomes and webhook.delivery.given_up counters in
      Prometheus, Grafana or Datadog.
  - question: "Can I send Hook0 webhook metrics to Prometheus, Grafana or Datadog?"
    answer: >-
      Yes, on a self-hosted instance. The API and the output worker export
      metrics, traces and logs over OTLP once you set OTLP_METRICS_ENDPOINT,
      OTLP_TRACES_ENDPOINT and OTLP_LOGS_ENDPOINT. Point them at an
      OpenTelemetry Collector, which forwards to Prometheus, Grafana Cloud,
      Datadog or any OTLP-compatible backend. On Hook0 Cloud, compute the
      same numbers from the request attempts API.
---

# Webhook monitoring: delivery metrics, alerts and dashboards

To monitor webhook delivery in Hook0, read the [request attempts](/concepts/request-attempts) of each subscription and track five numbers over a rolling window: first-attempt success rate, retry rate, error mix, endpoint response time and delivery latency. Each one comes from a field Hook0 already stores on every delivery, so you compute it from the API without adding instrumentation to your endpoint.

## Delivery metrics and where they come from

| Metric | Computed from | API source |
|--------|---------------|------------|
| First-attempt success rate | attempts with `retry_count = 0` and `status.type = successful`, divided by all attempts with `retry_count = 0` | `GET /request_attempts/` |
| Retry rate | events with at least one attempt where `retry_count > 0`, divided by all events | `GET /request_attempts/` |
| Error mix | count of `response_error_name` per value (`E_TIMEOUT`, `E_CONNECTION`, `E_DNS`, `E_HTTP`...) | `GET /responses/{response_id}` |
| Endpoint response time | `elapsed_time_ms`, p50 and p95 | `GET /responses/{response_id}` |
| Delivery latency | `succeeded_at - created_at` on the successful attempt | `GET /request_attempts/` |
| Pending backlog | attempts whose `status.type` is `pending`, `waiting` or `paused` | `GET /request_attempts/` |

Filter every query by `subscription_id` to get per-endpoint numbers, and by `min_created_at` / `max_created_at` to set the window.

:::tip Try it on your own events
The free Hook0 Cloud plan records every delivery attempt for 100 events a day, with no credit card. [Create an account](https://app.hook0.com/register), or send a first request to [Hook0 Play](https://play.hook0.com) to see the attempt and response a delivery produces.
:::

## Alert rules to start from

A webhook monitoring setup is only useful if it tells someone when to act. These thresholds are starting points, not limits Hook0 enforces: run them for a week, compare them with your own baseline, then tighten or loosen them. Evaluate each one per subscription over a 15-minute window, so one slow customer endpoint does not hide behind a healthy average.

| Signal | Warning | Page someone | First thing to check |
|--------|---------|--------------|----------------------|
| First-attempt success rate | below 95% | below 90% | Error mix of the failed attempts (`response_error_name`) |
| Events needing a retry | above 5% | above 10% | Whether `E_TIMEOUT` grows with the endpoint's response time |
| Endpoint response time, p95 (`elapsed_time_ms`) | above 1,000 ms | above 3,000 ms | The endpoint's own latency; Hook0 times out after 15 s by default |
| Delivery latency, p95 (`succeeded_at - created_at`) | above 30 s | above 5 min | Pending backlog and worker capacity (self-hosted) |
| Deliveries given up after the last retry | n/a | any | The last response of the event, then [replay it](../explanation/webhook-retry-logic.md#when-all-retries-are-exhausted) once the endpoint is fixed |
| Attempts in `paused` status | any | growing for 1 hour | The subscription's health (see below) |

Alert on rates over a window, not on single failures: one `E_TIMEOUT` is noise, the same error on 30% of a subscription's attempts for 15 minutes is an outage.

## Subscription health: degraded and recovering endpoints

A self-hosted Hook0 instance can also classify each subscription for you. Every 5 minutes, it looks at the attempts that completed in the last 15 minutes: a subscription with at least 10 completed attempts and more than 30% of failures becomes **degraded**, then **recovering** once its endpoint answers again, then **healthy**. Failures caused by Hook0 itself (`E_INTERNAL`, `E_INVALID_HEADER`) do not count.

Health tracking is off by default. Set `SUBSCRIPTION_HEALTH_PROBE_MODE` on the API to `shadow` to record it without changing deliveries, or to `enforce` to also slow down deliveries to unhealthy subscriptions: their new attempts are created in the `paused` status and released at a capped rate, so an endpoint that just came back is not flooded. A paused attempt is never dropped.

For monitoring, this means a growing number of `paused` attempts on a subscription is the signal that its endpoint is failing. The thresholds, states and settings are described in [Subscription health](../explanation/webhook-retry-logic.md#subscription-health).

## Overview

Hook0 tracks every webhook delivery attempt through the [request attempts](/concepts/request-attempts) system. Each attempt records:
- HTTP request details (method, URL, headers)
- Response data (status code, headers, body)
- Timing information (created_at, picked_at, succeeded_at/failed_at)
- Delivery status and the reference to the endpoint's response (response_id)
- Retry count and scheduling

### Set Up Environment Variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081 locally)

# Set your application ID (shown in dashboard URL or application details)
export APP_ID="YOUR_APPLICATION_ID_HERE"
```

Save these values:
```bash
# Save to .env file for later use
cat > .env <<EOF
HOOK0_TOKEN=$HOOK0_TOKEN
HOOK0_API=$HOOK0_API
APP_ID=$APP_ID
EOF
```

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

### Application dashboard

The application dashboard charts the number of events received per day against your plan's quota. The dashboard does not chart per-subscription success rate or response time: compute those from the API as shown in [Delivery metrics and where they come from](#delivery-metrics-and-where-they-come-from).

## API Endpoints for Monitoring

### List Request Attempts

Query request attempts programmatically:

```bash
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Filter by specific event
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&event_id={EVENT_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Query Parameters:**
- `application_id`: **(Required)** Filter by application
- `event_id`: Filter by specific event
- `subscription_id`: Filter by subscription
- `pagination_cursor`: For pagination
- `min_created_at` / `max_created_at`: Filter by time range

**Response Example:**

<!-- openapi: RequestAttempt -->
```json
[
  {
    "request_attempt_id": "uuid",
    "event_id": "uuid",
    "event": {
      "event_id": "uuid",
      "event_type_name": "billing.invoice.paid"
    },
    "subscription": {
      "subscription_id": "uuid",
      "description": "Billing service webhook"
    },
    "created_at": "2025-12-10T10:00:00Z",
    "picked_at": "2025-12-10T10:00:01Z",
    "succeeded_at": "2025-12-10T10:00:03Z",
    "failed_at": null,
    "delay_until": null,
    "response_id": "uuid",
    "retry_count": 0,
    "http_response_status": 200,
    "status": {
      "type": "successful",
      "at": "2025-12-10T10:00:03Z",
      "full_processing_ms": 2000
    }
  }
]
```

### Get Response Details

Fetch HTTP response details for a specific attempt:

```bash
curl -X GET "$HOOK0_API/responses/{RESPONSE_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Response Example:**

<!-- openapi: Response -->
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
curl -X GET "$HOOK0_API/events/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

**Query Parameters:**
- `application_id`: **(Required)** Filter by application

## Key Metrics to Monitor

The SQL queries below read the Hook0 database directly, so they only work on a self-hosted instance. On Hook0 Cloud, compute the same numbers from `GET /request_attempts/` and `GET /responses/{response_id}`.

### 1. Success Rate

Track the percentage of webhook deliveries that succeed on first attempt:

```sql
-- Example query pattern
SELECT
  COUNT(*) FILTER (WHERE succeeded_at IS NOT NULL) * 100.0 / COUNT(*) as first_attempt_success_rate
FROM webhook.request_attempt
WHERE retry_count = 0
  AND created_at > NOW() - INTERVAL '1 day';
```

**Starting target:** above 95% first-attempt success. Hook0 does not enforce this value; set yours from your own baseline.

### 2. Average Response Time

Monitor how quickly target endpoints respond:

```sql
-- Example query pattern
SELECT AVG(r.elapsed_time_ms) as avg_response_time_ms
FROM webhook.response r
INNER JOIN webhook.request_attempt ra ON ra.response__id = r.response__id
WHERE ra.created_at > NOW() - INTERVAL '1 hour';
```

**Starting target:** under 1,000 ms for most endpoints.

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

**Starting target:** under 5% of events requiring a retry.

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
- `E_DNS`: The target's hostname could not be resolved
- `E_HTTP`: Non-2xx HTTP response
- `E_INVALID_TARGET`: Invalid URL or configuration
- `E_INVALID_HEADER`: A required header value could not be constructed
- `E_INTERNAL`: A failure on Hook0's side (for example, the event's payload could not be found); it does not count against the subscription's health
- `E_UNKNOWN`: An unexpected error occurred

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

**Starting target:** under 30 seconds for events sent without a delay.

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

Start from the table in [Alert rules to start from](#alert-rules-to-start-from), then adjust each threshold to the baseline you measured in step 1.

### 3. Monitor Endpoint Health

Track per-endpoint metrics:
- Which subscriptions have lowest success rates
- Which target URLs timeout most frequently
- Correlation between endpoint and error types

**Example API Query:**

```bash
# Get request attempts for specific subscription
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
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
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  | jq '.[] | select(.failed_at != null)'
```

### Step 2: Examine Response Details

Get the HTTP response for the failed attempt:

```bash
curl -X GET "$HOOK0_API/responses/{RESPONSE_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

Look for:
- HTTP status code (4xx client error, 5xx server error)
- Response body with error message
- Headers that might indicate rate limiting or auth issues

### Step 3: Check Error Category

The `response_error_name` field categorizes the failure:
- **`E_CONNECTION`**: Target unreachable once resolved, network issue
- **`E_TIMEOUT`**: Request exceeded configured timeout (default 15s)
- **`E_DNS`**: The target's hostname could not be resolved — no name server could be reached, or one answered with an error such as SERVFAIL or REFUSED. The URL itself may be perfectly valid, so this is not necessarily something the subscription owner can fix
- **`E_HTTP`**: Non-2xx response (check response body for details)
- **`E_INVALID_TARGET`**: Malformed URL, a hostname that does not exist (NXDOMAIN), or a forbidden IP address
- **`E_INVALID_HEADER`**: A required header value could not be constructed (not retried)
- **`E_INTERNAL`**: A failure on Hook0's side that says nothing about the target (for example, the event's payload could not be found); it does not count against the subscription's health
- **`E_UNKNOWN`**: An unexpected error occurred

### Step 4: Review Event Payload

Ensure the event payload is valid:

```bash
curl -X GET "$HOOK0_API/events/{EVENT_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

Check for:
- Correct event type
- Valid JSON structure
- Expected payload fields
- Appropriate content type

### Step 5: Test Target Endpoint

Alternatively, use [Hook0 Play](https://play.hook0.com) as a temporary endpoint to capture and inspect the exact request Hook0 sends.

Manually verify the target endpoint:

```bash
# Simulate Hook0 webhook request
curl -X POST "https://your-endpoint.com/webhook" \
  -H "Content-Type: application/json" \
  -H "X-Event-Type: user.account.created" \
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
    `${process.env.HOOK0_API}/request_attempts/?application_id=${applicationId}`,
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
    avgRetries: attempts.length > 0
      ? attempts.reduce((sum, a) => sum + a.retry_count, 0) / attempts.length
      : 0,
  };

  return metrics;
}
```

### Export metrics to Prometheus and Grafana over OpenTelemetry (self-hosted)

The Hook0 API and the output worker export metrics, traces and logs over OTLP (HTTP, protobuf, gzip). Nothing is sent until you set the endpoints:

```bash
OTLP_METRICS_ENDPOINT=http://otel-collector:4318/v1/metrics
OTLP_TRACES_ENDPOINT=http://otel-collector:4318/v1/traces
OTLP_LOGS_ENDPOINT=http://otel-collector:4318/v1/logs
# OTLP_AUTHORIZATION="Bearer mytoken"  # only if your collector requires it
```

Point them at an OpenTelemetry Collector, which forwards to Prometheus, Grafana Cloud, Datadog or any OTLP backend. See [Observability with OpenTelemetry](../self-hosting/index.md#observability-with-opentelemetry) for the full setup.

The output worker exports these delivery metrics:

| Metric | Type | Attributes | Use it for |
|--------|------|------------|------------|
| `webhook.delivery.outcomes` | counter | `outcome`: `success`, `timeout`, `dns`, `http_4xx`, `http_5xx`, `connection_error`, `invalid_target`, `internal`, `other` | Success rate and error mix across all subscriptions |
| `webhook.delivery.given_up` | counter | `reason`: `retries_exhausted`, `signature_failed`, `subscription_gone` | Events lost for a subscriber; `subscription_gone` is the expected end of a deleted subscription |
| `worker.delivery_lag_seconds` | histogram | none | Time between an attempt's scheduled delivery and its pickup by a worker, the first sign that workers fall behind |
| `paused_request_attempts.subscriptions` | gauge | none | Subscriptions holding paused attempts because of their health |
| `paused_request_attempts.released` | counter | `health` | Paused attempts released back into delivery |

These counters cover the whole instance; per-subscription numbers still come from the request attempts API. With the Collector's Prometheus exporter, dots become underscores and counters get a `_total` suffix, which gives queries like these:

```promql
# Delivery success rate over 15 minutes, Hook0-side failures excluded
sum(rate(webhook_delivery_outcomes_total{outcome="success"}[15m]))
  /
sum(rate(webhook_delivery_outcomes_total{outcome!="internal"}[15m]))

# Events given up after the last retry, per hour
sum(increase(webhook_delivery_given_up_total{reason="retries_exhausted"}[1h]))

# p95 pickup lag of the output workers
histogram_quantile(0.95, sum by (le) (rate(worker_delivery_lag_seconds_bucket[5m])))
```

A matching Prometheus alert rule:

```yaml
groups:
  - name: hook0-webhooks
    rules:
      - alert: WebhookSuccessRateLow
        expr: |
          sum(rate(webhook_delivery_outcomes_total{outcome="success"}[15m]))
            / sum(rate(webhook_delivery_outcomes_total{outcome!="internal"}[15m])) < 0.90
        for: 15m
        labels:
          severity: page
      - alert: WebhookDeliveriesGivenUp
        expr: sum(increase(webhook_delivery_given_up_total{reason=~"retries_exhausted|signature_failed"}[1h])) > 0
        labels:
          severity: page
```

Metric names can differ slightly with other exporters (Datadog, Grafana Cloud): check how your backend translates OpenTelemetry names before you copy a query.

**Prometheus/Grafana on Hook0 Cloud:**
- Poll the request attempts API on a schedule
- Expose the computed rates in Prometheus format
- Build Grafana dashboards and alerts on them

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
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

Then fetch response details to check elapsed time:

```bash
# Get response details including elapsed_time_ms
curl -X GET "$HOOK0_API/responses/{RESPONSE_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
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

## Webhook monitoring checklist

- [ ] First-attempt success rate tracked per subscription, with a warning and a paging threshold
- [ ] Retry rate and error mix (`response_error_name`) on the same dashboard
- [ ] Endpoint response time p95 from `elapsed_time_ms`
- [ ] Delivery latency p95 from `succeeded_at - created_at`
- [ ] An alert on any delivery given up after its last retry, with the replay procedure linked in the alert
- [ ] Self-hosted: OTLP endpoints set, subscription health in `shadow` or `enforce`, and an alert on paused attempts
- [ ] A baseline written down, and thresholds reviewed after the first week

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

*For debugging specific webhook failures, see [Debug Failed Webhooks](debug-failed-webhooks.md).*

*For production patterns on payload design, signatures, and idempotency, see [Webhook best practices](webhook-best-practices.md).*

*For details on retry behavior, circuit breakers, and dead letter queues, see [Webhook retry logic](../explanation/webhook-retry-logic.md).*
