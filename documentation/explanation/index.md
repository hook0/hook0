# Explanation

Concepts, design decisions, and architecture behind Hook0. Read these to understand how and why things work the way they do.

## Introduction

### [What is Hook0?](what-is-hook0.md)
Core concepts and what Hook0 does.

**Topics covered:**
- The webhook reliability problem
- Hook0's approach to solving it
- Core concepts (Organizations, Applications, Events, Subscriptions)
- Key features and benefits
- When to use Hook0

---

## Suggested reading order

**Beginner path:**
1. [What is Hook0?](what-is-hook0.md) - Core concepts
2. [Hook0 Architecture](hook0-architecture.md) - System overview
3. [Event Processing](event-processing.md) - How events flow

**Advanced path:**
4. [Security Model](security-model.md) - Security architecture

---

## Architecture and design

### [System architecture](hook0-architecture.md)
How Hook0's components fit together.

**Topics covered:**
- System architecture overview
- Component responsibilities (API server, workers, database)
- Event flow and lifecycle
- Scaling considerations
- Deployment patterns

---

### [Event processing](event-processing.md)
How events flow from creation to delivery.

**Topics covered:**
- Event lifecycle stages
- Queue management and priority handling
- Retry mechanisms and backoff strategies
- Dead letter queues and failure handling
- Performance characteristics

---

## Security and reliability

### [Security model](security-model.md)
Hook0's approach to security, authentication, and data protection.

**Topics covered:**
- Authentication system (Biscuit tokens for user sessions, Service tokens for programmatic access)
- Payload signing and verification
- Transport security (TLS)
- Authorization and access control
- Data privacy and compliance

---

### [Webhook delivery guarantees](webhook-delivery-guarantees.md)
At-most-once, at-least-once, effectively-once -- and the idempotency pattern that makes duplicates harmless.

**Topics covered:**
- The three delivery guarantees and their tradeoffs
- Why at-least-once is the right default
- Idempotency pattern with code examples (Python, Node.js, Rust)
- Common mistakes that break idempotency

---

### [Webhook vs Polling](webhook-vs-polling.md)
When to use webhooks, when to poll, and the hybrid pattern that covers both.

**Topics covered:**
- Latency and cost comparison with concrete numbers
- When polling is the right choice
- The webhook-first + polling-fallback hybrid pattern
- How Hook0 eliminates the need for polling fallback

---

## Design philosophy

### Reliability first
Every design decision prioritizes reliable delivery over raw performance.
- Events are persisted before acknowledgment
- Retries on a fixed schedule
- Circuit breakers protect downstream systems
- Delivery status tracked for every attempt

### Operational simplicity
Easy to deploy, monitor, and maintain.
- Single binary, minimal dependencies
- Observability built in
- Clear error messages
- Sensible defaults with escape hatches

### Developer experience
Webhooks that just work.
- Straightforward API
- Documentation with working examples
- Multiple SDKs
- Local development support

## Mental models

### Hook0 as a message broker
Hook0 makes sure your events reach their destinations, the same way a message broker delivers messages between systems.

### Events vs. webhooks
- **Events** are things that happened in your system
- **Webhooks** are HTTP requests that deliver event notifications
- Hook0 turns events into reliable webhook deliveries

### Subscriptions as routing rules
Subscriptions define which events go to which endpoints, with what delivery guarantees.

---

For implementation details, see [Tutorials](../tutorials/index.md) and [How-to Guides](../how-to-guides/index.md). For specifications, see [Reference](../reference/index.md).
