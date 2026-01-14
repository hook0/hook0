---
title: Metadata
description: Attach arbitrary key-value data to Hook0 objects
---

# Metadata

Metadata is a mechanism to attach arbitrary key-value data to Hook0 objects like [events](events.md) and [subscriptions](subscriptions.md). It allows you to store custom information alongside your webhook data without affecting Hook0's core functionality.

## Key Points

- Metadata can be attached to [events](events.md) and [subscriptions](subscriptions.md)
- Hook0 stores but does not process metadata
- Metadata is searchable via the Search API
- Not visible to webhook consumers unless explicitly exposed

## Metadata vs Labels

A common question is when to use metadata versus [labels](labels.md). They serve different purposes:

```
+-------------------+     +-------------------+
|     Labels        |     |    Metadata       |
+-------------------+     +-------------------+
| Used for routing  |     | Not used for      |
| events to subs    |     | routing           |
+-------------------+     +-------------------+
| Affects delivery  |     | No effect on      |
|                   |     | delivery          |
+-------------------+     +-------------------+
| Required (min 1)  |     | Optional          |
+-------------------+     +-------------------+
```

Use **[labels](labels.md)** when you need to route [events](events.md) to specific [subscriptions](subscriptions.md).
Use **metadata** when you need to store additional context for your own systems.

## Common Use Cases

- **Correlation IDs** - Link [events](events.md) to external systems
- **User context** - Store the user ID who triggered the [event](events.md)
- **Debug info** - Include request IDs or trace identifiers
- **Business context** - Store domain-specific identifiers

## Description Field

In addition to metadata, a separate `description` field exists for human-readable annotations. For example: "Customer onboarding webhook for Acme Corp".

:::warning Security
Don't store sensitive information (bank accounts, card details, passwords) in metadata or the description field.
:::

## What's Next?

- [Events](events.md) - Attach metadata to events
- [Subscriptions](subscriptions.md) - Attach metadata to subscriptions
- [Labels](labels.md) - Use labels for event routing
