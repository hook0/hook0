---
title: Concepts
description: Core concepts in Hook0
---

# Concepts

Understand the fundamental building blocks of Hook0.

## Architecture Overview

```mermaid
flowchart TD
    ORG["Organization"]:::external
    APP["Application"]:::hook0
    ET["Event Type<br/>(defines structure)"]:::customer
    EVT["Event<br/>(sent by your app)"]:::customer
    LBL["Labels<br/>(routing metadata)"]:::processing
    SUB["Subscription<br/>(delivery target)"]:::hook0
    RA["Request Attempt<br/>(delivery tracking)"]:::processing
    WH["Webhook Endpoint"]:::customer

    ORG --> APP
    APP --> ET
    APP --> EVT
    APP --> SUB
    EVT --> LBL
    SUB --> RA
    RA --> WH

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click ORG "/concepts/organizations" "Organizations"
    click APP "/concepts/applications" "Applications"
    click ET "/concepts/event-types" "Event Types"
    click EVT "/concepts/events" "Events"
    click LBL "/concepts/labels" "Labels"
    click SUB "/concepts/subscriptions" "Subscriptions"
    click RA "/concepts/request-attempts" "Request Attempts"
```

Hook0 connects your applications to external systems through a hierarchical structure:

1. **Organizations** group your team and applications
2. **Applications** represent your services that emit events
3. **Events** are notifications sent when actions occur
4. **Event Types** categorize and validate your events
5. **Labels** route events to the correct subscriptions
6. **Subscriptions** define where to deliver webhooks
7. **Request Attempts** track each delivery attempt

## Core Concepts

### Structure & Organization

- **[Organizations](organizations.md)** - Multi-tenant containers for teams and applications
- **[Applications](applications.md)** - Logical containers grouping events and subscriptions

### Events & Routing

- **[Events](events.md)** - Notifications sent from your applications to Hook0
- **[Event Types](event-types.md)** - Categorize and structure your events
- **[Labels](labels.md)** - Filter and route events to subscriptions
- **[Metadata](metadata.md)** - Attach arbitrary key-value data to objects

### Delivery & Security

- **[Subscriptions](subscriptions.md)** - Configure where and how to receive notifications
- **[Request Attempts](request-attempts.md)** - Track webhook delivery status and retries
- **[Application Secrets](application-secrets.md)** - Sign webhooks for verification
- **[Service Tokens](service-tokens.md)** - API authentication for automated systems
