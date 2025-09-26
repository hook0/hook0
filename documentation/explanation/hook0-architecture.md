# Hook0 Architecture

This document explains Hook0's webhook architecture, design decisions, and how different components work together to provide reliable event delivery.

## System Architecture

Hook0 follows a modular architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Apps   │───▶│   API Server    │───▶│   PostgreSQL    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │  Worker Process │───▶│ Webhook Targets │
                       └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │ Web Dashboard   │
                       └─────────────────┘
```

## Component Responsibilities

### API Server
- **Event Ingestion**: Receives events via REST API
- **Authentication**: Validates Biscuit tokens for access control
- **Data Validation**: Ensures event payloads conform to schemas
- **Quota Management**: Enforces rate limits and usage quotas
- **Management Operations**: CRUD operations for organizations, applications, subscriptions

### Worker Process
- **Event Processing**: Retrieves pending events from database
- **Webhook Delivery**: Makes HTTP requests to configured endpoints
- **Retry Logic**: Implements exponential backoff for failed deliveries
- **Dead Letter Handling**: Manages permanently failed events
- **Monitoring**: Records delivery attempts and response data

### Web Dashboard
- **User Interface**: Vue.js-based dashboard for management
- **Real-time Monitoring**: Live updates on event processing
- **Configuration Management**: UI for setting up subscriptions and event types
- **Analytics**: Dashboards showing delivery metrics and health

## Event Flow

Hook0 processes events through a well-defined pipeline. For detailed information about the event lifecycle, retry mechanisms, and delivery logic, see the [Event Processing Model](./event-processing.md).

## Data Model

### Core Entities

```sql
organizations
├── users (many-to-many)
└── applications
    ├── event_types
    ├── events
    └── subscriptions
        └── request_attempts
```

### Event Storage
Events are stored with:
- Unique ID
- Event type reference
- JSON payload
- Metadata (labels, source IP, etc.)
- Timestamp

### Subscription Matching
Subscriptions define:
- Event type filters (exact match or patterns)
- Target HTTP endpoint
- Authentication headers
- Custom metadata
- Retry configuration

## Design Decisions

### Why Rust?
- Memory safety without garbage collection
- Excellent performance characteristics
- Strong type system prevents common bugs

### Why PostgreSQL?
- ACID compliance for reliability
- Rich query capabilities
- Excellent JSON support
- Mature ecosystem and tooling

### Why Biscuit Tokens?
- More flexible than JWT
- Built-in authorization capabilities
- Cryptographically secure
- Supports token delegation

## Next Steps

- [Event Processing Model](./event-processing.md)
- [Security Model](./security-model.md)
- [API Reference](../openapi/intro)
