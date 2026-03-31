---
title: "What is Hook0?"
description: "Hook0 is an open-source webhook server you can self-host or use as a cloud service for reliable event delivery between distributed systems."
keywords: [webhook, Hook0, open-source, webhooks as a service, event delivery]
---

# What is Hook0?

Hook0 is an open-source webhook server (Webhooks as a Service). It receives events from your applications and delivers them to configured webhook endpoints, handling retries, signatures, and monitoring for you.

## The problem

When your application needs to send webhooks, you end up building:

- Retry logic for when the receiving system is down
- Queue management for thousands of deliveries
- Monitoring to track and debug failures
- Signature generation for authenticity

Hook0 handles all of that so you don't have to.

## Before and after Hook0

### Without Hook0

```mermaid
flowchart TD
    subgraph YourApp["Your Application"]
        BL["Core Business Logic"]:::customer
        WL["Webhook Logic<br/>(you maintain)<br/>- Retry logic &amp; backoff<br/>- Failure handling<br/>- Queue management<br/>- Monitoring &amp; logging<br/>- Security signatures<br/>- Dead letter queues"]:::customer
        BL --> WL
    end
    WL --> CW["Customer Webhook<br/>Is it down?"]:::danger

    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef danger fill:#fee2e2,stroke:#f87171,color:#7f1d1d
```

Problems:
- Webhook code scattered across your application
- No centralized visibility on delivery status
- Retry and failure logic you have to maintain yourself
- Hard to debug failed deliveries

### With Hook0

```mermaid
flowchart TD
    subgraph YourApp["Your Application"]
        BL["Business Logic"]:::customer
    end

    subgraph H0["Hook0"]
        EP["Event Processing<br/>- Validation<br/>- Storage"]:::hook0
        WDE["Webhook Delivery Engine<br/>- Automatic retries<br/>- Fixed retry schedule<br/>- Signature generation<br/>- Rate limiting<br/>- Dead letter queues"]:::hook0
        OD["Observability Dashboard<br/>- All delivery attempts<br/>- Failure analytics<br/>- Debug tools"]:::hook0
        EP --> WDE --> OD
    end

    BL -- "Simple HTTP POST" --> EP
    OD --> CW["Customer Webhook<br/>Reliable"]:::external

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12

    click EP "/explanation/event-processing" "Event Processing"
    click WDE "/explanation/webhook-retry-logic" "Retry Logic"
    click CW "/concepts/subscriptions" "Subscriptions"
```

What changes:
- One API call to send events
- One dashboard for all deliveries
- Retry logic works out of the box
- Full visibility and debugging tools

## How Hook0 works

1. Your application sends events to Hook0 via REST API
2. Hook0 validates and stores events
3. Hook0 delivers events to configured webhook endpoints
4. Failed deliveries are retried on a fixed schedule (3s, 10s, 3min, 30min, 1h, 3h, 5h, 10h)
5. All attempts are logged and viewable in the dashboard

## Core concepts

### Organizations
The top-level entity. Organizations group users and applications together, providing isolation and access control.

### Applications
Applications represent your services or products within an organization. Each application can define event types and have multiple subscriptions.

### Event types
Event types define what events your application can emit. Examples:
- `user.account.created`
- `payment.transaction.completed`
- `order.shipment.shipped`

### Events
An event is a specific occurrence of an event type, containing:
- Event type identifier
- Payload data
- Metadata
- Timestamp

### Subscriptions
A subscription defines where and how webhook notifications get delivered:
- Target webhook URL
- Which event types to receive
- Authentication headers
- Retry configuration

### Request attempts
Hook0 tracks every delivery attempt:
- Response status codes
- Response bodies
- Timestamps
- Retry attempts

## Why Hook0?

- Open source (SSPL v1), self-hostable, or use the [cloud version](https://www.hook0.com/)
- Written in Rust, scales horizontally
- RESTful API with SDKs (Rust, TypeScript)
- Automatic retries, dead letter queues, rate limiting
- Full delivery visibility in the dashboard

## Use cases

- SaaS integration: let customers receive webhook notifications from your platform
- Microservices: decouple services with async webhook delivery
- Audit and compliance: track all events and their delivery status
- Third-party integrations: connect to Slack, Discord, or any HTTP endpoint

## Next steps

- [Getting Started Tutorial](../tutorials/getting-started.md)
- [System Architecture](./hook0-architecture.md) - Detailed technical architecture
- [API Reference](../openapi/intro)
- [Webhook best practices](../how-to-guides/webhook-best-practices.md) - Production patterns for sending and receiving webhooks
- [How Hook0 compares](../comparisons/) - Side-by-side comparison with Svix, Hookdeck, and other providers
