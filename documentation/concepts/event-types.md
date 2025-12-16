---
title: Event Types
description: Understanding event types in Hook0
---

# Event Types

Event types categorize events sent to Hook0. An event type consists of three dot-separated parts: service name, resource type, and verb.

## Naming Conventions

Event types follow a dot-separated format: `service.resource.verb`

- **Service**: The system or domain generating the event (e.g., `iam`, `billing`, `storage`)
- **Resource**: The entity being acted upon (e.g., `user`, `invoice`, `file`)
- **Verb**: The action in past tense (e.g., `created`, `updated`, `deleted`)

### Good Examples

- `iam.user.created` - IAM service, user resource, creation action
- `billing.invoice.paid` - Billing service, invoice resource, payment action
- `storage.file.removed` - Storage service, file resource, removal action
- `api.application.created` - API service, application resource, creation action

### Bad Examples

- `IAM_USER_CREATED` - Uses uppercase and underscores instead of lowercase dot-separated format
- `UserCreated` - Missing service context and uses PascalCase
- `user-created` - Missing service context and uses dashes instead of dots
- `iam_user_created` - Uses underscores instead of dots

## Key Functions

Event types serve two primary purposes:

1. **Payload Structure**: Events with the same event type are expected to have the same payload structure, making it simpler for webhook receivers to process data.

2. **Subscription Filtering**: Users creating subscriptions can choose which event types they want to hear about, allowing Hook0 to forward only matching events for specific subscriptions.

## What's Next?

- [Send your first event](/tutorials/getting-started)
- [Set up subscriptions](subscriptions.md)
- [Secure webhook endpoints](/how-to-guides/secure-webhook-endpoints)
