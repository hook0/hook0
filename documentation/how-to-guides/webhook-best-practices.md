---
title: "Webhook Best Practices: 8 Rules That Prevent Lost and Duplicate Events"
sidebar_label: "Webhook best practices"
description: "Sign payloads, deduplicate on event_id, answer 200 fast and process async, version event types, use HTTPS: the producer and consumer rules, with Hook0's signature format."
keywords: [webhook best practices, webhook security, webhook signing hmac verification, webhook idempotency, webhook retry, webhook design, webhook versioning]
faqItems:
  - question: "How fast should a webhook endpoint respond?"
    answer: >-
      Return a 200 OK within 5 seconds and process the event asynchronously. If
      the endpoint takes too long, the producer may time out and retry, which
      leads to duplicate processing. Validate the signature, store the raw
      event, return 200, then do the real work in the background.
  - question: "How do you handle duplicate webhook deliveries?"
    answer: >-
      Deduplicate on the event_id. Store the ID of every event you have already
      processed and skip any repeat. Webhooks can be delivered more than once
      because failed deliveries are retried, so treating the event_id as a
      dedup key keeps side effects from running twice.
  - question: "How should you version webhook payloads?"
    answer: >-
      Version the event type rather than mutating an existing payload shape, for
      example order.shipped.v1 then order.shipped.v2. Keep the old version
      running for at least 6 months after you announce deprecation so consumers
      have time to migrate.
  - question: "What are the most important webhook best practices?"
    answer: >-
      As a producer, sign every payload with HMAC-SHA256, give each event a
      unique ID, send self-contained payloads and version event types instead
      of changing them. As a consumer, verify the signature on the raw body
      before anything else, return a 2xx quickly and process asynchronously,
      deduplicate on the event ID, and only expose HTTPS endpoints.
---

# Webhook best practices

The short version: the producer signs every payload, gives each event a unique ID and versions its event types; the consumer verifies the signature on the raw body, returns a 2xx within a few seconds, processes the event in the background and skips event IDs it has already handled. The sections below cover each rule for both sides of the integration, the producer (sender) and the consumer (receiver), with Hook0's own formats where they apply.

:::tip Try it on your own endpoint
The free Hook0 Cloud plan signs and retries 100 events a day, with no credit card. [Create an account](https://app.hook0.com/register), or send a signed request to [Hook0 Play](https://play.hook0.com) to inspect the headers your endpoint will receive.
:::

## For producers (sending webhooks)

### Sign every payload

Every outgoing webhook should include an HMAC-SHA256 signature in the headers. This lets consumers verify the payload hasn't been tampered with. Include a timestamp in the signed string so that a captured request cannot be replayed later. Hook0's header carries the timestamp (`t`), the names of the signed headers (`h`) and the signature (`v1`):

```
X-Hook0-Signature: t=1765443663,h=content-type x-event-id,v1=85da0586ae0b711d...
```

Generate the signature from the raw request body using the shared secret. Never sign a re-serialized version of the payload; byte differences will break verification.

For a step-by-step implementation with code examples, see [Webhook authentication tutorial](/tutorials/webhook-authentication). Hook0 generates HMAC signatures automatically for every delivery -- see [Subscription secrets](/concepts/subscriptions#subscription-secrets) for where the signing key comes from.

### Include an idempotency key

Add a unique identifier to each event so consumers can deduplicate:

```json
{
  "event_id": "evt_01H8...",
  "event_type": "invoice.paid",
  "created_at": "2025-01-15T10:30:00Z",
  "data": { }
}
```

Use UUIDs or ULIDs. The consumer stores seen event IDs and skips duplicates.

### Design payloads for stability

Keep payloads self-contained. Include the data the consumer needs rather than forcing them to make API calls back to you:

Good (self-contained):
```json
{
  "event_type": "order.shipped",
  "data": {
    "order_id": "ord_123",
    "tracking_number": "1Z999AA10123456784",
    "carrier": "ups",
    "shipped_at": "2025-01-15T14:00:00Z"
  }
}
```

Avoid (requires follow-up API call):
```json
{
  "event_type": "order.shipped",
  "data": {
    "order_id": "ord_123"
  }
}
```

### Version your webhooks

When you change payload shapes, use event type versioning:

```
order.shipped.v1
order.shipped.v2
```

Give consumers time to migrate. Support old versions for at least 6 months after announcing deprecation.

## For consumers (receiving webhooks)

### Verify signatures first

Before processing any webhook, verify the HMAC signature. Reject requests with missing or invalid signatures immediately. For a Hook0 `v1` signature, the signed string is the timestamp, the header names, the header values and the raw body, joined by dots:

```python
import hmac
import hashlib

def verify_hook0_signature(raw_body: bytes, headers, secret: str, signature_header: str) -> bool:
    parts = dict(p.split("=", 1) for p in signature_header.split(","))
    names = parts.get("h", "")
    if names:
        values = ".".join(headers.get(n, "") for n in names.split(" "))
        signed = f"{parts['t']}.{names}.{values}.".encode() + raw_body
    else:
        signed = f"{parts['t']}.".encode() + raw_body
    expected = hmac.new(secret.encode(), signed, hashlib.sha256).hexdigest()
    return hmac.compare_digest(expected, parts.get("v1", ""))
```

Also reject requests whose `t` is too old (a few minutes) to block replays.

Use constant-time comparison (`hmac.compare_digest`) to prevent timing attacks. See [Secure webhook endpoints](/how-to-guides/secure-webhook-endpoints) for a more detailed walkthrough, and [Hook0's security model](/explanation/security-model) for how signatures fit into the broader security architecture.

### Respond fast, process later

Return a `200 OK` within 5 seconds. Do the actual processing asynchronously:

```mermaid
flowchart LR
    R["POST /webhooks"]:::external --> V[Validate signature]:::processing
    V --> S[Store raw event]:::processing
    S --> OK[Return 200 OK]:::hook0
    OK --> P[Process event async]:::customer

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click V "/tutorials/webhook-authentication" "Webhook Authentication"
```

If your endpoint takes too long, the producer may time out and retry, leading to duplicate processing. See [Client error handling](/how-to-guides/client-error-handling) for patterns on handling these edge cases.

### Handle duplicates (idempotency)

Webhooks can be delivered more than once. Track processed event IDs:

```sql
CREATE TABLE processed_events (
  event_id TEXT PRIMARY KEY,
  processed_at TIMESTAMPTZ DEFAULT NOW()
);
```

Before processing, check if the event ID exists. If it does, skip it.

### Use HTTPS endpoints only

Always expose webhook endpoints over HTTPS. This prevents payload interception and man-in-the-middle attacks. Hook0 accepts both `http://` and `https://` targets, so a plain-HTTP URL works for local testing; use HTTPS for every production subscription. See [Subscriptions](/concepts/subscriptions) for how to configure your endpoints in Hook0.

## Payload design guidelines

### Use a consistent envelope structure

Every event should follow the same top-level structure:

```json
{
  "event_id": "evt_...",
  "event_type": "resource.action",
  "api_version": "2025-01-15",
  "created_at": "2025-01-15T10:30:00Z",
  "data": { }
}
```

### Event type naming

Use `resource.action` format with dot separators:

- `user.created`
- `invoice.payment_succeeded`
- `subscription.trial_ending`

Be specific. `user.updated` is better than `user.changed`. `invoice.payment_failed` is better than `invoice.error`.

Hook0 uses a `service.resource_type.verb` convention for event types that we consider even better ! See [Event types](/concepts/event-types) for how this works, and [Event types & subscriptions tutorial](/tutorials/event-types-subscriptions) for a hands-on setup guide.

### Timestamp format

Always use ISO 8601 with timezone: `2025-01-15T10:30:00Z`. Never use Unix timestamps in the payload; they are harder to read when debugging.

## Monitoring and alerting

### Metrics to track

As a producer:
- Delivery success rate (target: >99.5%)
- P95 delivery latency
- Retry rate per endpoint
- Deliveries that exhausted their retries

As a consumer:
- Processing success rate
- Duplicate detection rate
- Average processing time
- Queue depth

### Alerting thresholds

Alert on:
- Delivery success rate drops below 99%
- Any endpoint has >5 consecutive failures
- Events whose retries were exhausted without a success
- Processing latency exceeds 30 seconds

See [Monitor webhook performance](/how-to-guides/monitor-webhook-performance) for Hook0-specific monitoring setup.

## Further reading

- [Webhook authentication tutorial](/tutorials/webhook-authentication) -- HMAC implementation walkthrough
- [Webhook retry logic](/explanation/webhook-retry-logic) -- the retry schedule and when Hook0 gives up
- [Debug failed webhooks](/how-to-guides/debug-failed-webhooks) -- troubleshooting delivery failures
- [Secure webhook endpoints](/how-to-guides/secure-webhook-endpoints) -- endpoint security in depth
- [Event types & subscriptions](/tutorials/event-types-subscriptions) -- setting up routing and filtering
- [Getting started with Hook0](/tutorials/getting-started) -- send your first webhook
- [Hook0 Play](https://play.hook0.com) -- test webhook deliveries and inspect payloads in real-time, no signup needed
