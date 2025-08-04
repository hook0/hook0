# Webhook Architecture

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

### 1. Event Ingestion
```rust
POST /events
{
  "event_type": "user.created",
  "payload": { "user_id": 123, "email": "user@example.com" }
}
```

The API server:
- Validates the event type exists
- Checks payload against schema (if defined)
- Enforces quota limits
- Stores event in database
- Returns event ID

### 2. Event Processing
The worker process:
- Polls database for pending events
- Finds matching subscriptions based on event type
- Creates delivery tasks for each subscription
- Queues tasks for processing

### 3. Webhook Delivery
For each delivery task:
- Constructs HTTP request with proper headers
- Includes webhook signature for authentication
- Makes HTTP request to target URL
- Records response and status
- Schedules retry if needed

### 4. Retry Logic
Hook0 implements exponential backoff:
- First retry: 30 seconds
- Second retry: 1 minute
- Third retry: 2 minutes
- And so on, up to maximum retry attempts

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

## Security Architecture

### Authentication
Hook0 uses [Biscuit tokens](https://github.com/biscuit-auth/biscuit) for authentication:
- Cryptographically secure
- Supports fine-grained permissions
- Stateless token validation
- Token attenuation for limited access

### Webhook Signatures
All webhook deliveries include HMAC-SHA256 signatures:
- Computed using subscription secret
- Includes full request body
- Allows recipients to verify authenticity

### Access Control
- Organization-level isolation
- Application-scoped permissions
- User role management
- Service token support for API access

## Scalability Considerations

### Horizontal Scaling
- Multiple API server instances behind load balancer
- Multiple worker processes for increased throughput
- Database connection pooling
- Stateless architecture enables easy scaling

### Performance Optimizations
- Efficient database queries with proper indexing
- Batch processing of webhook deliveries
- Connection reuse for HTTP requests
- Configurable worker concurrency

### Resource Management
- Configurable quotas per organization
- Rate limiting at multiple levels
- Memory-efficient event processing
- Automatic cleanup of old events

## Reliability Features

### Durability
- All events persisted to PostgreSQL
- Atomic transactions for consistency
- Database migrations for schema evolution

### Observability
- Comprehensive logging throughout system
- Metrics collection for monitoring
- Request/response tracking
- Performance metrics

### Error Handling
- Graceful degradation on failures
- Circuit breaker patterns
- Dead letter queues
- Detailed error reporting

## Design Decisions

### Why Rust?
- Memory safety without garbage collection
- Excellent performance characteristics
- Strong type system prevents common bugs
- Great ecosystem for web services

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
- [API Reference](../reference/api-reference.md)