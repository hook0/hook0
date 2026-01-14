---
title: Event Types
description: Categories for webhook events in Hook0
---

# Event Types

Event types categorize [events](events.md) sent to Hook0. An event type consists of three dot-separated parts: service name, resource type, and verb.

## Naming Conventions

Event types follow a dot-separated format: `service.resource.verb`

- **Service**: The system or domain generating the [event](events.md) (e.g., `iam`, `billing`, `storage`)
- **Resource**: The entity being acted upon (e.g., `user`, `invoice`, `file`)
- **Verb**: The action in past tense (e.g., `created`, `updated`, `deleted`)

### Good Examples

- `iam.user.created` - IAM service, user resource, creation action
- `billing.invoice.paid` - Billing service, invoice resource, payment action
- `storage.file.removed` - Storage service, file resource, removal action
- `api.application.created` - API service, [application](applications.md) resource, creation action

### Bad Examples

- `IAM_USER_CREATED` - Uses uppercase and underscores instead of lowercase dot-separated format
- `UserCreated` - Missing service context and uses PascalCase
- `user-created` - Missing service context and uses dashes instead of dots
- `iam_user_created` - Uses underscores instead of dots

## Key Functions

Event types serve two primary purposes:

1. **Payload Structure**: [Events](events.md) with the same event type are expected to have the same payload structure, making it simpler for webhook receivers to process data.

2. **[Subscription](subscriptions.md) Filtering**: Users creating [subscriptions](subscriptions.md) can choose which event types they want to hear about, allowing Hook0 to forward only matching [events](events.md) for specific [subscriptions](subscriptions.md).

## What's Next?

- [Events](events.md) - Send notifications with event types
- [Applications](applications.md) - Manage event types per application
- [Subscriptions](subscriptions.md) - Filter subscriptions by event type
- [Labels](labels.md) - Route events to subscriptions
- [Send your first event](/tutorials/getting-started) - Quick start guide
