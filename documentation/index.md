---
title: Hook0 Documentation
description: Complete documentation for Hook0 - The open-source webhook server for reliable event delivery
keywords: [hook0, webhooks, event delivery, api, documentation]
---

# Hook0 Documentation

Welcome to Hook0, the open-source webhook server for reliable event delivery at scale.

## What is Hook0?

Hook0 is a production-ready webhook server that ensures your events reach their destinations with enterprise-grade reliability, security, and observability. Available as a cloud service or self-hosted.

<div className="row">
<div className="col col--6">

### New to Hook0?

1. **[What is Hook0?](explanation/what-is-hook0.md)** - Understand core concepts
2. **[Quick Start](tutorials/getting-started.md)** - Get running in 5 minutes
3. **[First Webhook](tutorials/first-webhook-integration.md)** - Build your first integration

</div>
<div className="col col--6">

### Ready to Build?

1. **[Event Types & Subscriptions](tutorials/event-types-subscriptions.md)** - Configure events
2. **[API Reference](/api)** - Complete REST API docs
3. **[JavaScript SDK](reference/sdk/javascript.md)** - Official client library

</div>
</div>

---

## Documentation Sections

This documentation follows the [Diataxis methodology](https://diataxis.fr/):

| Section | Purpose | Start Here |
|---------|---------|------------|
| **[Tutorials](tutorials/index.md)** | Step-by-step learning | [Getting Started](tutorials/getting-started.md) |
| **[How-to Guides](how-to-guides/index.md)** | Solve specific problems | [Debug Webhooks](how-to-guides/debug-failed-webhooks.md) |
| **[Reference](reference/index.md)** | Technical specifications | [API Reference](/api) |
| **[Explanation](explanation/index.md)** | Deep understanding | [Architecture](explanation/hook0-architecture.md) |

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Reliable Delivery** | Automatic retries with exponential backoff |
| **Security** | HMAC-SHA-256 signatures, TLS encryption |
| **Observable** | Built-in metrics, logs, and delivery tracking |
| **High Performance** | Rust-based, handles thousands of events/second |
| **Flexible** | Event filtering, label-based routing |
| **Open Source** | Self-host or use our cloud service |

---

## Quick Links

### By Role

**Developers**
- [API Reference](/api) - REST API documentation
- [JavaScript SDK](reference/sdk/javascript.md) - TypeScript/JS client
- [Rust SDK](reference/sdk/rust.md) - Rust client library
- [Error Codes](reference/error-codes.md) - Troubleshooting

**DevOps**
- [Configuration](reference/configuration.md) - All options
- [Monitor Performance](how-to-guides/monitor-webhook-performance.md) - Observability

**Architects**
- [Architecture](explanation/hook0-architecture.md) - System design
- [Security Model](explanation/security-model.md) - Security architecture
- [Event Processing](explanation/event-processing.md) - Delivery pipeline

---

## Resources

- **[GitHub](https://github.com/hook0/hook0)** - Source code
- **[Discord](https://www.hook0.com/community)** - Community chat
- **[Blog](https://www.hook0.com/blog)** - Articles & tutorials
- **[Status](https://status.hook0.com)** - Service status

---

*Hook0 - Enterprise-grade webhook delivery for modern applications.*
