---
title: Labels
description: Filter and route events to subscriptions using key-value labels
---

# Labels

Labels are key-value pairs attached to events that subscriptions use for filtering. They enable routing patterns like multi-tenancy, environment separation, and priority handling.

:::warning Minimum Requirement
Every event must include **at least one label**. The API rejects events with empty labels.
:::

## How Labels Work

When you send an event, you attach labels to categorize it:

```json
{
  "event_type": "order.created",
  "labels": {
    "tenant_id": "acme_corp",
    "environment": "production"
  },
  "payload": "..."
}
```

Subscriptions filter events by specifying which label values they want to receive:

```json
{
  "event_types": ["order.created"],
  "labels": { "tenant_id": "acme_corp" },
  "target": { "url": "https://acme.example.com/webhooks" }
}
```

**Filtering logic:**
- A subscription receives an event only if **all** its label filters match the event's labels
- If an event lacks a label that the subscription filters on, the event is not delivered
- Label values are **case-sensitive**

## Common Use Cases

| Pattern | Label Example | Purpose |
|---------|---------------|---------|
| **Multi-tenancy** | `tenant_id: "acme_corp"` | Isolate events per customer |
| **Environment** | `environment: "production"` | Separate prod/staging/dev |
| **Priority routing** | `priority: "critical"` | Route urgent events differently |
| **Geographic** | `region: "eu-west-1"` | Route by data center or region |
| **Source tracking** | `source: "mobile_app"` | Identify event origin |

## Validation Rules

| Constraint | Limit |
|------------|-------|
| Labels per event | 10 maximum |
| Key length | 1-50 characters |
| Value length | 1-50 characters |
| Value type | String only |

## Naming Conventions

- Use `snake_case` for keys: `tenant_id`, `event_source`
- Be descriptive: `payment_provider` not `pp`
- Avoid sensitive data: never use `password`, `ssn`, `credit_card`
- Use consistent naming across all events

## What's Next?

- [Events](events.md) - Learn about event structure
- [Subscriptions](subscriptions.md) - Configure webhook filtering
- [Event Types](event-types.md) - Categorize your events
