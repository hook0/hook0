# Explanation

Understanding-oriented documentation that explains the concepts, design decisions, and architectural choices behind Hook0. These materials help you develop a deeper understanding of how and why Hook0 works the way it does.

## Introduction

### [What is Hook0?](what-is-hook0.md)
A comprehensive introduction to Hook0, including core concepts and fundamental understanding.

**Topics covered:**
- The webhook reliability problem
- Hook0's approach to solving it
- Core concepts (Organizations, Applications, Events, Subscriptions)
- Key features and benefits
- When to use Hook0

---

## Suggested Reading Order

**Beginner Path:**
1. [What is Hook0?](what-is-hook0.md) - Core concepts
2. [Hook0 Architecture](hook0-architecture.md) - System overview
3. [Event Processing](event-processing.md) - How events flow
4. [Event Labels](labels.md) - Filtering and routing

**Advanced Path:**
5. [Security Model](security-model.md) - Security architecture

---

## Architecture & Design

### [System Architecture](hook0-architecture.md)
Deep dive into Hook0's system architecture and component interactions.

**Topics covered:**
- System architecture overview
- Component responsibilities (API server, workers, database)
- Event flow and lifecycle
- Scaling considerations
- Deployment patterns

---

### [Event Processing](event-processing.md)
Detailed explanation of how Hook0 processes events from creation to delivery.

**Topics covered:**
- Event lifecycle stages
- Queue management and priority handling
- Retry mechanisms and backoff strategies
- Dead letter queues and failure handling
- Performance characteristics

---

### [Event Labels](labels.md)
Understanding label-based filtering and routing for events.

**Topics covered:**
- What labels are and how they work
- Multi-tenancy patterns
- Environment-based routing
- Geographic and priority routing
- Best practices

---

## Security & Reliability

### [Security Model](security-model.md)
Hook0's approach to security, authentication, and data protection.

**Topics covered:**
- Biscuit token authentication system
- Payload signing and verification
- Transport security (TLS)
- Authorization and access control
- Data privacy and compliance

---

## Design Philosophy

Hook0 is built on several key principles:

### Reliability First
Every design decision prioritizes reliable event delivery over raw performance. This means:
- Events are persisted before acknowledgment
- Comprehensive retry mechanisms with exponential backoff
- Circuit breaker patterns to protect downstream systems
- Detailed delivery status tracking and reporting

### Operational Simplicity
Hook0 aims to be easy to deploy, monitor, and maintain:
- Single binary deployment with minimal dependencies
- Rich observability and monitoring capabilities
- Clear error messages and debugging information
- Sensible defaults with escape hatches for customization


### Developer Experience
Built for developers who need webhooks to just work:
- Clear, comprehensive API design
- Extensive documentation with practical examples
- Multiple SDK options
- Local development support


## Mental Models

### Think of Hook0 as a Reliable Message Broker
Hook0 takes the responsibility of ensuring your application events reach their destinations, just like a message broker ensures messages are delivered between systems.

### Events vs Webhooks
- **Events** are things that happened in your system
- **Webhooks** are HTTP requests that deliver event notifications
- Hook0 transforms events into webhook deliveries reliably

### Subscriptions as Event Routing Rules
Subscriptions define which events should trigger webhooks to which endpoints, with what payload format and delivery guarantees.

---

*For practical implementation details, see [Tutorials](../tutorials/index.md) and [How-to Guides](../how-to-guides/index.md). For technical specifications, see [Reference](../reference/index.md).*
