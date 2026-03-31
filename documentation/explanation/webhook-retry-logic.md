---
title: "Webhook retry logic: delivery attempts and retry scheduling"
description: "How Hook0 retries failed webhook deliveries using a predefined retry schedule, and how to replay events after all retries are exhausted."
keywords: [webhook retry, retry schedule, webhook delivery, retry attempts, replay events]
---

# Webhook retry logic

When a webhook delivery fails (network timeout, 5xx response, DNS error, connection refused), Hook0 retries with increasing delays. Each failed attempt creates a new request attempt scheduled for later, until the retry limit is reached.

## Why retries matter

Most webhook delivery failures are transient. The receiving server was restarting, a load balancer was draining connections, or a brief network partition occurred. A retry a few seconds later usually succeeds.

Without retries, every transient failure becomes a lost event. With naive retries (fixed interval, no limit), you risk overwhelming a recovering server. Hook0 uses a predefined retry schedule that spaces out attempts over increasing intervals.

## Retry schedule

Hook0 uses a fixed retry schedule (not exponential backoff). Each retry attempt has a predefined delay:

```mermaid
flowchart LR
    A1["1: immediate"]:::hook0 --> A2["2: +3s"]:::external
    A2 --> A3["3: +10s"]:::external
    A3 --> A4["4: +3min"]:::external
    A4 --> A5["5: +30min"]:::processing
    A5 --> A6["6: +1h"]:::processing
    A6 --> A7["7: +3h"]:::processing
    A7 --> A8["8: +5h"]:::customer
    A8 --> A9["9+: +10h each"]:::customer

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click A1 "/concepts/request-attempts" "Request Attempts"
```

The delays are cumulative from the point of failure, not from the original event. For example, if the first delivery fails at T=0, the second attempt is scheduled at T+3s. If that also fails, the third attempt is at T+3s+10s, and so on.

### Retry limits

Retries are bounded by two configurable limits (whichever is reached first):

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MAX_RETRIES` | 25 | Maximum number of delivery attempts |
| `MAX_RETRY_WINDOW` | 8 days | Maximum total time window for retries |

At startup, the output worker evaluates the effective retry policy by computing how many retries fit within the configured window. For example, with the default settings, all 25 retries fit comfortably within 8 days.

## What happens on failure

When a delivery attempt fails, Hook0 follows this decision process:

```mermaid
flowchart TD
    FAIL[Delivery attempt fails]:::customer --> NR{Non-retryable error?}:::processing
    NR -->|Yes| GU1[Give up]:::customer
    NR -->|No| ACTIVE{Subscription still active?}:::processing
    ACTIVE -->|No| GU2[Give up]:::customer
    ACTIVE -->|Yes| MAX{Max retries reached?}:::processing
    MAX -->|Yes| GU3[Give up]:::customer
    MAX -->|No| SCHED[Schedule next retry]:::hook0

    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click FAIL "/concepts/request-attempts" "Request Attempts"
    click ACTIVE "/concepts/subscriptions" "Subscriptions"
```

### Non-retryable errors

Some errors are never retried because retrying would produce the same result:

- Invalid header: the webhook signature could not be constructed (e.g., event type contains characters that are invalid in HTTP headers).

### Subscription and application checks

Before scheduling a retry, Hook0 checks that the subscription is still enabled, has not been soft-deleted, and that the parent application still exists. If any of these fail, the retry is skipped.

## Delivery status flow

Each webhook delivery attempt goes through these states:

```mermaid
stateDiagram-v2
    [*] --> PENDING
    PENDING --> PICKED
    PICKED --> SUCCESSFUL
    PICKED --> FAILED
    FAILED --> PENDING : retry
    FAILED --> [*] : no retry (final FAILED)
```

More precisely, Hook0 tracks five statuses:

| Status | Meaning |
|--------|---------|
| Waiting | Scheduled for future delivery (`delay_until` has not elapsed yet) |
| Pending | Ready to be picked up by a worker |
| In Progress | Currently being delivered (picked by a worker) |
| Successful | Delivery succeeded (2xx HTTP response) |
| Failed | Delivery failed |

The `request_attempt` table stores every attempt with timestamps (`created_at`, `picked_at`, `succeeded_at`, `failed_at`, `delay_until`), so you can calculate:
- Time to first delivery: `picked_at - created_at`
- Delivery latency: `succeeded_at - picked_at`
- Total time to success: `succeeded_at - created_at` (including retries)

Each retry creates a new row in the `request_attempt` table with an incremented `retry_count` and a `delay_until` set to the scheduled retry time.

## When all retries are exhausted

When the maximum number of retries is reached (or the retry window expires), Hook0 does not create another attempt. The last attempt stays in `failed` status.

Failed deliveries are not lost. You can:

1. Inspect all delivery attempts and their responses via the API or dashboard
2. Replay the event via the API to re-trigger delivery to all matching subscriptions

Replaying an event resets its `dispatched_at` field. The dispatch trigger then creates new request attempts for all active subscriptions that match the event's type and labels.

## Idempotency

Every event in Hook0 has a unique `event_id`. Consumers should use this as an idempotency key to handle duplicate deliveries.

Duplicates happen when:
- The consumer processed the event but returned a non-2xx response (e.g., crashed after processing but before responding)
- Network issues caused the response to be lost
- Manual replay of an event

### Example implementation

```sql
-- PostgreSQL example
CREATE TABLE processed_webhooks (
    event_id UUID PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Before processing:
INSERT INTO processed_webhooks (event_id)
VALUES ($1)
ON CONFLICT (event_id) DO NOTHING
RETURNING event_id;

-- If no row returned, event was already processed -- skip it.
```

## Configuration

The output worker's retry and delivery behavior is configured via environment variables:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `MAX_RETRIES` | 25 | Maximum delivery attempts before giving up |
| `MAX_RETRY_WINDOW` | 8 days | Maximum time window for retries |
| `CONNECT_TIMEOUT` | 5 seconds | Timeout for establishing a TCP connection |
| `TIMEOUT` | 15 seconds | Total HTTP request timeout (including connect) |
| `CONCURRENT` | 1 | Number of request attempts handled concurrently |

## Error types

When a delivery fails, Hook0 records one of these error codes:

| Error code | Meaning |
|------------|---------|
| `E_TIMEOUT` | The HTTP request timed out |
| `E_CONNECTION` | Could not establish a connection to the target |
| `E_HTTP` | The server responded with a non-2xx status code |
| `E_INVALID_TARGET` | The target URL is invalid or resolves to a forbidden IP |
| `E_INVALID_HEADER` | A required header value could not be constructed (non-retryable) |
| `E_UNKNOWN` | An unexpected error occurred |

## SSRF protection

Hook0 blocks webhook deliveries to private/internal IP addresses by default (loopback, RFC 1918, link-local, etc.). This prevents Server-Side Request Forgery attacks. This check can be disabled with the `DISABLE_TARGET_IP_CHECK` flag for development environments.

## Further reading

- [Monitor webhook performance](/how-to-guides/monitor-webhook-performance) -- track delivery rates and latency
- [Debug failed webhooks](/how-to-guides/debug-failed-webhooks) -- investigate specific delivery failures
- [Webhook best practices](/how-to-guides/webhook-best-practices) -- patterns for producers and consumers
