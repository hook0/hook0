---
title: Subscriptions
description: Webhook subscriptions in Hook0 — how to configure delivery endpoints, filter by event types and labels, and list subscriptions via the cursor-paginated API.
keywords:
  - subscriptions
  - webhook subscriptions
  - hook0 subscriptions
  - webhook endpoints
  - cursor pagination
---

# Subscriptions

A subscription tells Hook0 where to deliver [events](events.md) and which events to deliver. It pairs a webhook endpoint URL with filtering criteria so only relevant events get sent.

## Key points

- Subscriptions belong to an [Application](applications.md)
- Each subscription specifies a target URL and filtering criteria
- Filtering uses [Event Types](event-types.md) and [Labels](labels.md)
- Subscriptions can be enabled or disabled without deletion
- Each subscription has a [secret](application-secrets.md) for signature verification

## How subscriptions work

```mermaid
flowchart TD
    A["Event arrives at Hook0"]:::external
    B["Match Event Type"]:::processing
    C["Match Labels"]:::processing
    D["Create Request Attempt"]:::hook0
    E["Deliver to<br/>Subscription Target"]:::customer

    A --> B --> C --> D --> E

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click A "/concepts/events" "Events"
    click B "/concepts/event-types" "Event Types"
    click C "/concepts/labels" "Labels"
    click D "/concepts/request-attempts" "Request Attempts"
```

## Filtering

Subscriptions filter [events](events.md) in two ways:

### Event type filtering

Subscribe to specific [event types](event-types.md) (e.g., `order.created`, `user.updated`). Only [events](events.md) with matching types trigger deliveries.

### Label filtering

Narrow down further using [labels](labels.md). A subscription with label `tenant_id: "acme"` only receives [events](events.md) that have that exact label.

Both filters must match for an [event](events.md) to be delivered.

## Target types

Subscriptions support HTTP targets where webhooks are delivered via POST (or other methods) to your endpoint. The target configuration includes:

- URL where the webhook is sent
- HTTP method (typically POST)
- Custom headers

## Subscription secrets

Each subscription has an associated [secret](application-secrets.md) used to sign webhook payloads. Recipients use this [secret](application-secrets.md) to verify:

- The webhook came from Hook0
- The payload wasn't modified in transit
- The webhook is fresh (timestamp validation)

## Listing subscriptions

`GET /api/v1/subscriptions` is **cursor-paginated**. The response carries `Link: rel="next"` and `Link: rel="prev"` headers when more pages exist (see the [pagination contract](/openapi/intro#pagination) for full details).

Fetch the first page, then follow `Link: rel="next"` until it is absent:

```bash
curl -i -H "Authorization: Bearer $TOKEN" \
  "https://app.hook0.com/api/v1/subscriptions?application_id=$APPLICATION_ID&limit=100"

# The response's Link header points at the next page:
#   Link: <…&pagination_cursor=…&limit=100>; rel="next"
curl -i -H "Authorization: Bearer $TOKEN" "$NEXT_URL_FROM_LINK_HEADER"
```

The official [Hook0 SDKs](/reference/sdk/javascript) follow `Link` headers automatically.

## What's next?

- [Events](events.md) - Understanding event structure
- [Labels](labels.md) - Filtering events with labels
- [Request Attempts](request-attempts.md) - Track delivery status and retries
- [Application Secrets](application-secrets.md) - Understanding webhook signatures
- [Secure Webhook Endpoints](/how-to-guides/secure-webhook-endpoints) - Complete security guide
