---
title: Concepts
description: Core concepts in Hook0
---

# Concepts

Understand the fundamental building blocks of Hook0.

## Architecture Overview

```
Application → Event → Hook0 → Subscription → Webhook Endpoint
                ↓
           Event Type
           (filtering)
```

Hook0 connects your applications to external systems through a simple flow:

1. Your **Application** sends an **Event** to Hook0
2. Each Event has an **Event Type** that categorizes it
3. **Subscriptions** filter Events by Event Type
4. Hook0 delivers matched Events to the configured **Webhook Endpoint**

This architecture allows you to decouple event producers from consumers, enabling flexible routing and reliable delivery.

## Core Concepts

- **[Events](events.md)** - Notifications sent from your applications to Hook0
- **[Subscriptions](subscriptions.md)** - Configure where and how to receive notifications
- **[Event Types](event-types.md)** - Categorize and structure your events
- **[Labels](labels.md)** - Filter and route events to subscriptions
- **[Metadata](metadata.md)** - Attach arbitrary key-value data to objects
