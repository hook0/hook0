# What is Hook0?

Hook0 is an open-source webhook server (WaaS) that solves the fundamental challenge of reliable event delivery between distributed systems. 
It acts as a reliable intermediary that receives events from your applications and delivers them to configured webhook endpoints.

## The Problem Hook0 Solves

Modern applications need to communicate with external systems when events occur. Traditional approaches face several challenges:

- **Reliability**: What happens when the receiving system is down?
- **Scalability**: How do you handle thousands of webhook deliveries?
- **Observability**: How do you track and debug failed deliveries?
- **Security**: How do you ensure webhook authenticity?

## How Hook0 Works

Hook0 implements a producer-consumer pattern:

1. **Event Ingestion**: Your application sends events to Hook0 via REST API
2. **Event Processing**: Hook0 validates and stores events
3. **Webhook Delivery**: Hook0 delivers events to configured webhook endpoints
4. **Retry Logic**: Failed deliveries are automatically retried with exponential backoff
5. **Monitoring**: All attempts are logged and can be monitored

## Core Concepts

### Organizations
Organizations are the top-level entity that groups users and applications together. They provide isolation and access control.

### Applications
Applications represent your services or products within an organization. Each application can define event types and have multiple subscriptions.

### Event Types
Event types define the structure and metadata of events your application can emit. Examples:
- `user.created`
- `payment.completed`
- `order.shipped`

### Events
Events are specific occurrences of an event type, containing:
- Event type identifier
- Payload data
- Metadata
- Timestamp

### Subscriptions
Subscriptions define where and how webhook notifications should be delivered:
- Target webhook URL
- Which event types to receive
- Authentication headers
- Retry configuration

### Request Attempts
Hook0 tracks every delivery attempt, including:
- Response status codes
- Response bodies
- Timestamps
- Retry attempts

## Why Choose Hook0?

### Open Source & Self-Hostable
- Full control over your infrastructure
- No vendor lock-in
- SSPL v1 license

### Built for Scale
- Rust-based architecture for performance
- Horizontal scaling support
- Efficient retry mechanisms

### Developer-Friendly
- RESTful API
- Comprehensive SDKs (Rust, TypeScript)
- Rich observability features

### Production-Ready
- Automatic retries with exponential backoff
- Dead letter queues
- Rate limiting
- Security best practices

## Use Cases

### SaaS Integration
Enable customers to receive real-time notifications about events in your platform.

### Microservices Communication
Decouple services by using webhooks for async communication.

### Audit & Compliance
Track all events and their delivery status for compliance requirements.

### Third-Party Integrations
Connect your application to external services like Slack, Discord, or custom systems.


## Next Steps

- [Getting Started Tutorial](../tutorials/getting-started.md)
- [System Architecture](./hook0-architecture.md) - Detailed technical architecture
- [API Reference](../reference/api-reference.md)
