---
title: Request Attempts
description: Webhook delivery tracking and retry mechanism in Hook0
---

# Request Attempts

A request attempt represents a single delivery attempt for a webhook. When an [event](events.md) matches a [subscription](subscriptions.md), Hook0 creates a request attempt and tracks its lifecycle from creation to successful delivery or final failure.

## Key Points

- Each [event](events.md)/[subscription](subscriptions.md) pair generates one or more request attempts
- Hook0 automatically retries failed deliveries with exponential backoff
- Request attempts transition through distinct statuses
- Full delivery history is available for debugging

## Delivery Lifecycle

```
Event Created
    |
    v
+-------------------+
|     PENDING       | <- Queued for delivery
+-------------------+
    |
    v
+-------------------+
|   IN_PROGRESS     | <- Being delivered
+-------------------+
    |
    +-------+-------+
    |               |
    v               v
+----------+  +----------+
|SUCCESSFUL|  |  FAILED  |
+----------+  +----------+
                   |
                   v (if retries remaining)
              +----------+
              | WAITING  | <- Backoff delay
              +----------+
                   |
                   v
              (retry cycle)
```

## Status States

Request attempts transition through these states:

- **Pending** - Queued and waiting to be picked up for delivery
- **In Progress** - Currently being delivered to the endpoint
- **Waiting** - Delivery failed, waiting for retry (backoff delay)
- **Successful** - Webhook delivered and endpoint returned 2xx
- **Failed** - All retry attempts exhausted or permanently failed

## Retry Behavior

When a delivery fails, Hook0 automatically schedules retries with exponential backoff:

1. First failure → Short delay
2. Subsequent failures → Progressively longer delays
3. After max retries → Marked as permanently failed

This ensures transient failures don't cause data loss while avoiding overwhelming struggling endpoints.

## Failure Scenarios

Request attempts fail when:

- Endpoint returns 4xx or 5xx status codes
- Connection times out
- DNS resolution fails
- SSL/TLS handshake fails
- [Application](applications.md) is deleted (all pending attempts cancelled)

## Debugging Failed Webhooks

When webhooks fail, request attempts provide:

- **Timestamps** - When each phase occurred
- **Retry count** - How many attempts were made
- **Response reference** - Link to the endpoint's response

This information helps identify:
- Endpoint availability issues
- Authentication problems
- Payload processing errors

## What's Next?

- [Debug Failed Webhooks](/how-to-guides/debug-failed-webhooks) - Troubleshooting guide
- [Monitor Webhook Performance](/how-to-guides/monitor-webhook-performance) - Tracking metrics
- [Subscriptions](subscriptions.md) - Configuring webhook endpoints
- [Events](events.md) - Understanding event structure
