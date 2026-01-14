---
title: Subscriptions
description: Webhook endpoints that receive events from Hook0
---

# Subscriptions

A subscription is a configuration that tells Hook0 where to deliver [events](events.md) and which [events](events.md) to deliver. Subscriptions connect your webhook endpoints to the [events](events.md) they care about.

## Key Points

- Subscriptions belong to an [Application](applications.md)
- Each subscription specifies a target URL and filtering criteria
- Filtering is based on [Event Types](event-types.md) and [Labels](labels.md)
- Subscriptions can be enabled or disabled without deletion
- Each subscription has a [secret](application-secrets.md) for signature verification

## How Subscriptions Work

```
Event arrives at Hook0
         |
         v
+-------------------+
|  Match Event Type |
+-------------------+
         |
         v
+-------------------+
|   Match Labels    |
+-------------------+
         |
         v
+-------------------+
| Create Request    |
| Attempt           |
+-------------------+
         |
         v
    Deliver to
   Subscription
      Target
```

## Filtering Criteria

Subscriptions filter incoming [events](events.md) using two mechanisms:

### Event Type Filtering

Subscribe to specific [event types](event-types.md) (e.g., `order.created`, `user.updated`). Only [events](events.md) with matching types trigger deliveries.

### Label Filtering

Further refine which [events](events.md) to receive using [labels](labels.md). For example, a subscription with label `tenant_id: "acme"` only receives [events](events.md) that have that exact label.

Both filters must match for an [event](events.md) to be delivered.

## Target Types

Subscriptions support HTTP targets where webhooks are delivered via POST (or other methods) to your endpoint. The target configuration includes:

- **URL** - Where to send the webhook
- **HTTP Method** - Typically POST
- **Headers** - Custom headers to include

## Subscription Secrets

Each subscription has an associated [secret](application-secrets.md) used to sign webhook payloads. Recipients use this [secret](application-secrets.md) to verify:

- The webhook came from Hook0
- The payload wasn't modified in transit
- The webhook is fresh (timestamp validation)

## Benefits

- **Real-Time Notifications** - Immediate updates when [events](events.md) occur
- **Decoupled Architecture** - Separate concerns between [event](events.md) producers and consumers
- **Selective Delivery** - Only receive [events](events.md) you care about
- **Automated Actions** - Trigger workflows, emails, or database updates

## Use Cases

Webhook subscriptions enable automation across many domains:

- **E-commerce** - Order notifications, inventory updates, shipping alerts
- **Payments** - Transaction confirmations, refund notifications
- **CRM** - New customer alerts, profile updates
- **DevOps** - Deployment notifications, monitoring alerts
- **Healthcare** - Appointment reminders, record updates

## What's Next?

- [Events](events.md) - Understanding event structure
- [Labels](labels.md) - Filtering events with labels
- [Request Attempts](request-attempts.md) - Track delivery status and retries
- [Application Secrets](application-secrets.md) - Understanding webhook signatures
- [Secure Webhook Endpoints](/how-to-guides/secure-webhook-endpoints) - Complete security guide
