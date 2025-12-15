---
sidebar_position: 10
---

# Glossary

Quick reference for Hook0 terminology.

## Core Concepts

### Event
An occurrence of something happening in your system. Events have a type, payload, and optional labels.

### Event Type
A schema defining a category of events. Format: `service.resource.verb` (e.g., `user.account.created`).

### Subscription
Configuration that defines where and when to deliver webhooks. Links event types to a target URL with optional label filtering.

### Webhook
The HTTP POST request Hook0 sends to your endpoint when an event matches a subscription.

### Request Attempt
A single delivery attempt of a webhook. Hook0 tracks success/failure and retries failed attempts.

### Labels
Key-value pairs attached to events for filtering. Used for multi-tenancy (e.g., `tenant_id: "customer_123"`).

## Authentication

### Biscuit Token
Hook0's authentication mechanism. A bearer token format that supports:
- Fine-grained permissions
- Offline verification
- Token attenuation (delegation)

Format: `EoQKCAoh...`

See [Security Model](../explanation/security-model.md) for details.

### API Key
Only used for `MASTER_API_KEY` - the admin credential for initial setup. Not for regular authentication.

## Infrastructure

### Output Worker
The background process that delivers webhooks. Handles retries, rate limiting, and concurrent delivery.

### PostgreSQL
Hook0's only database. Stores events, subscriptions, and delivery attempts.

:::warning Important
Hook0 does NOT use Redis. All state is stored in PostgreSQL.
:::

## Acronyms

| Acronym | Full Name |
|---------|-----------|
| HMAC | Hash-based Message Authentication Code |
| mTLS | Mutual Transport Layer Security |
| SSPL | Server Side Public License |
| WaaS | Webhooks as a Service |
| GDPR | General Data Protection Regulation |

## See Also

- [What is Hook0?](../explanation/what-is-hook0.md)
- [Hook0 Architecture](../explanation/hook0-architecture.md)
- [Configuration Reference](./configuration.md)
