# How-to Guides

Problem-solving guides for specific challenges you'll encounter when working with Hook0 in production. These guides assume you have basic Hook0 knowledge and focus on solving real-world problems.

## Production Operations

### [Debug Failed Webhooks](debug-failed-webhooks.md)
Troubleshoot webhook delivery failures and diagnose common issues.

**When to use:** When webhooks aren't being delivered or are failing consistently
**Covers:** Log analysis, retry mechanisms, endpoint debugging, common failure patterns

---

### [Monitor Webhook Performance](monitor-webhook-performance.md)
Track webhook delivery metrics, set up alerts, and optimize performance.

**When to use:** When setting up production monitoring or investigating performance issues
**Covers:** Dashboard usage, API metrics, Sentry integration, key performance indicators, debugging workflows

---


## Integration & Migration

### [Multi-Tenant Architecture](multi-tenant-architecture.md)
Implement multi-tenant webhook systems using Hook0's labels and subscriptions.

**When to use:** When building SaaS platforms that need tenant-isolated webhook delivery
**Covers:** Label-based filtering, subscription management, tenant isolation patterns

---

## Security & Reliability

### [Managing Service Tokens](manage-service-tokens.md)
Create, attenuate, and manage service tokens for API access.

**When to use:** When setting up API access for automation, CI/CD, or AI assistants
**Covers:** Token creation, token attenuation, least privilege, rotation, revocation

---

### [Secure Webhook Endpoints](secure-webhook-endpoints.md)
Implement robust security measures for webhook endpoints and payloads.

**When to use:** When setting up production webhook endpoints that need to be secure
**Covers:** Authentication methods, IP allowlisting, payload validation, rate limiting

---

### [Client-side Error Handling](client-error-handling.md)
Implement robust error handling for Hook0 API responses.

**When to use:** When building API integrations that need to handle errors gracefully
**Covers:** Error response parsing, retry logic, error logging, handling strategies

---


## Quick Reference

| Problem | Guide | Difficulty |
|---------|-------|------------|
| Webhooks not delivering | [Debug Failed Webhooks](debug-failed-webhooks.md) | Beginner |
| Need performance monitoring | [Monitor Webhook Performance](monitor-webhook-performance.md) | Intermediate |
| Setting up API authentication | [Managing Service Tokens](manage-service-tokens.md) | Beginner |
| Need security implementation | [Secure Webhook Endpoints](secure-webhook-endpoints.md) | Intermediate |
| Need error handling in client | [Client-side Error Handling](client-error-handling.md) | Beginner |
| Building multi-tenant SaaS | [Multi-Tenant Architecture](multi-tenant-architecture.md) | Intermediate |

## Before You Start

These guides assume you:

- Have Hook0 running (see [Getting Started Tutorial](../tutorials/getting-started.md))
- Understand basic webhook concepts
- Have access to Hook0 logs and configuration
- Are comfortable with command-line tools

## Related Resources

- **[Tutorials](../tutorials/index.md)** - If you need to learn Hook0 basics first
- **[API Reference](../openapi/intro)** - For technical implementation details
- **[Configuration Reference](../reference/configuration.md)** - For all configuration options
- **[Error Codes](../reference/error-codes.md)** - For troubleshooting specific errors

---

*Need help with a specific problem? Choose the most relevant guide above or ask in our [Discord community](https://www.hook0.com/community).*