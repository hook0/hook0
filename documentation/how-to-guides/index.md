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

### [GitLab Webhook Migration](gitlab-webhook-migration.md)
Implement a GitLab-style webhook system using Hook0's labels and subscriptions.

**When to use:** When migrating from GitLab webhooks or implementing similar event-driven systems
**Covers:** Event mapping, label-based filtering, subscription management, signature verification

---

## Security & Reliability

### [Secure Webhook Endpoints](secure-webhook-endpoints.md)
Implement robust security measures for webhook endpoints and payloads.

**When to use:** When setting up production webhook endpoints that need to be secure
**Covers:** Authentication methods, IP allowlisting, payload validation, rate limiting

---


## Quick Reference

| Problem | Guide | Difficulty |
|---------|-------|------------|
| Webhooks not delivering | [Debug Failed Webhooks](debug-failed-webhooks.md) | Beginner |
| Need performance monitoring | [Monitor Webhook Performance](monitor-webhook-performance.md) | Intermediate |
| Need security implementation | [Secure Webhook Endpoints](secure-webhook-endpoints.md) | Intermediate |
| Migrating from GitLab | [GitLab Webhook Migration](gitlab-webhook-migration.md) | Intermediate |

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