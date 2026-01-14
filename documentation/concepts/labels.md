---
title: Labels
description: Key-value pairs for filtering and routing events to subscriptions
---

# Labels

Labels are key-value pairs attached to [events](events.md) that enable routing to specific [subscriptions](subscriptions.md). They provide a flexible mechanism for multi-tenancy, environment separation, and custom filtering logic.

## Key Points

- Every [event](events.md) must include at least one label
- [Subscriptions](subscriptions.md) filter [events](events.md) by matching label values
- Labels enable multi-tenant architectures
- Label matching is case-sensitive and requires exact matches

:::warning Minimum Requirement
Every [event](events.md) must include **at least one label**. The API rejects [events](events.md) with empty labels.
:::

## How Labels Work

```
Event with labels
     |
     v
+------------------------+
| tenant_id: "acme"      |
| environment: "prod"    |
+------------------------+
     |
     v
Hook0 finds subscriptions
with matching labels
     |
     +--------+--------+
     |                 |
     v                 v
+-----------+   +-----------+
| Sub A     |   | Sub B     |
| tenant_id:|   | tenant_id:|
| "acme"    |   | "beta"    |
+-----------+   +-----------+
     |                 |
     v                 x (no match)
  Delivered
```

## Matching Rules

- A [subscription](subscriptions.md) receives an [event](events.md) only if **all** its label filters match the event's labels
- If an [event](events.md) lacks a label that the [subscription](subscriptions.md) filters on, the event is not delivered
- [Subscriptions](subscriptions.md) can have fewer labels than [events](events.md) (partial matching)
- Label values are **case-sensitive** and must match exactly

## Common Patterns

| Pattern | Example Label | Purpose |
|---------|---------------|---------|
| **Multi-tenancy** | `tenant_id: "acme_corp"` | Isolate [events](events.md) per customer |
| **Environment** | `environment: "production"` | Separate prod/staging/dev |
| **Priority** | `priority: "critical"` | Route urgent [events](events.md) differently |
| **Geographic** | `region: "eu-west-1"` | Route by location |
| **Source** | `source: "mobile_app"` | Identify [event](events.md) origin |

## Naming Best Practices

- Use `snake_case` for keys: `tenant_id`, `event_source`
- Be descriptive: `payment_provider` not `pp`
- Avoid sensitive data: never use `password`, `ssn`, `credit_card`
- Use consistent naming across all [events](events.md)

## What's Next?

- [Events](events.md) - Attach labels to events
- [Subscriptions](subscriptions.md) - Filter events with labels
- [Event Types](event-types.md) - Categorize your events
- [Applications](applications.md) - Manage labels per application
