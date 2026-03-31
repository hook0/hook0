---
title: "Hook0 architecture: API, queue backends, workers, and storage"
description: "How events flow through Hook0 — from API ingestion to queue processing, worker delivery, and storage. Supports PostgreSQL-only or Pulsar + S3 for high throughput."
keywords: [webhook, architecture, API, queue, workers, PostgreSQL, Pulsar, S3, Hook0]
---

# Hook0 Architecture

This document covers Hook0's architecture, the design decisions behind it, and how the components fit together.

## System architecture

### Default setup (PostgreSQL-only)

```mermaid
flowchart LR
    CA[Client Apps]:::external --> API[API Server]:::hook0 --> PG[PostgreSQL]:::hook0
    PG --> WK[Worker Process]:::processing
    WK --> WT[Webhook Targets]:::customer
    PG --> WD[Web Dashboard]:::hook0

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click API "/openapi/intro" "API Reference"
    click PG "/reference/configuration" "Configuration"
    click WK "/explanation/event-processing" "Event Processing"
```

PostgreSQL handles everything: event storage, queue, and delivery state. Workers poll for pending deliveries using `FOR UPDATE SKIP LOCKED` for concurrent processing. One database to operate.

### High-throughput setup (Pulsar + S3)

```mermaid
flowchart LR
    CA[Client Apps]:::external --> API[API Server]:::hook0
    API --> PG[PostgreSQL]:::hook0
    API --> PL[Apache Pulsar]:::processing
    PL --> WK[Worker Process]:::processing
    WK --> S3[S3-compatible Object Storage]:::hook0
    WK --> WT[Webhook Targets]:::customer

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click API "/openapi/intro" "API Reference"
    click PG "/reference/configuration" "Configuration"
    click WK "/explanation/event-processing" "Event Processing"
```

For high-throughput deployments, Hook0 can use Apache Pulsar for queuing and S3-compatible object storage for event payloads and response bodies. PostgreSQL remains the source of truth for metadata, subscriptions, and event types. The switch between backends is a configuration change (`QUEUE_TYPE`), not a code change.

## Component responsibilities

### API server
- Receives events via REST API
- Validates Biscuit tokens (user sessions) and service tokens (programmatic access)
- Validates event payloads against schemas
- Enforces rate limits and usage quotas
- CRUD for organizations, applications, subscriptions

### Worker process
- Retrieves pending events from the database
- Sends HTTP requests to configured endpoints
- Retries failed deliveries with increasing backoff
- Manages permanently failed events (dead letter)
- Records delivery attempts and response data

### Web dashboard
- Vue.js-based management UI
- Live updates on event processing
- Configuration for subscriptions and event types
- Delivery metrics and health dashboards

## Event flow

For details on the event lifecycle, retry logic, and delivery handling, see the [Event Processing Model](./event-processing.md).

## Data model

### Core entities

```mermaid
graph TD
    ORG[organizations]:::hook0 --> USR[users]:::customer
    ORG --> APP[applications]:::hook0
    APP --> ET[event_types]:::processing
    APP --> EV[events]:::processing
    APP --> SUB[subscriptions]:::processing
    SUB --> RA[request_attempts]:::external

    classDef external fill:#dbeafe,stroke:#60a5fa,color:#1e3a5f
    classDef hook0 fill:#dcfce7,stroke:#4ade80,color:#14532d
    classDef customer fill:#ffedd5,stroke:#fb923c,color:#7c2d12
    classDef processing fill:#ede9fe,stroke:#a78bfa,color:#3b0764

    click ORG "/concepts/organizations" "Organizations"
    click APP "/concepts/applications" "Applications"
    click ET "/concepts/event-types" "Event Types"
    click EV "/concepts/events" "Events"
    click SUB "/concepts/subscriptions" "Subscriptions"
    click RA "/concepts/request-attempts" "Request Attempts"
```

### Event storage
Events are stored with:
- Unique ID
- Event type reference
- JSON payload
- Metadata (labels, source IP, etc.)
- Timestamp

### Subscription matching
Subscriptions define:
- Event type filters (exact match or patterns)
- Target HTTP endpoint
- Authentication headers
- Custom metadata
- Retry configuration

## Design decisions

### Why Rust?
- Memory safety without garbage collection
- Good performance
- Strong type system catches bugs at compile time

### Why PostgreSQL as the default?
- ACID guarantees
- Good JSON support
- Doubles as a job queue via `FOR UPDATE SKIP LOCKED`, so most deployments don't need a separate queuing system

### Why Pulsar + S3 for high throughput?
- Pulsar handles message ordering, fan-out, and backpressure at scale
- S3-compatible storage offloads large payloads and response bodies from the database
- Better fit for deployments processing millions of events per day
- PostgreSQL stays the source of truth for metadata; Pulsar handles the delivery queue

### Why Biscuit tokens?
Hook0 uses [Biscuit tokens](https://www.biscuitsec.org/) for both user sessions and service tokens:
- More flexible than JWT
- Built-in authorization
- Supports token attenuation (restrict permissions without calling the server)

## Next steps

- [Event Processing Model](./event-processing.md)
- [Security Model](./security-model.md)
- [API Reference](../openapi/intro)
