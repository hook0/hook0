---
title: Event Types
description: Categories for webhook events in Hook0
keywords:
  - event types
  - webhook event types
  - hook0 event types
  - event naming conventions
  - cursor pagination
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

## Listing event types

`GET /api/v1/event_types` is **cursor-paginated**. The response carries `Link: rel="next"` and `Link: rel="prev"` headers when more pages exist (see the [pagination contract](/openapi/intro#pagination) for full details).

Fetch the first page, then follow `Link: rel="next"` until it is absent:

```bash
curl -i -H "Authorization: Bearer $TOKEN" \
  "https://app.hook0.com/api/v1/event_types?application_id=$APPLICATION_ID&limit=100"

# The response's Link header points at the next page:
#   Link: <…&pagination_cursor=…&limit=100>; rel="next"
curl -i -H "Authorization: Bearer $TOKEN" "$NEXT_URL_FROM_LINK_HEADER"
```

The official [Hook0 SDKs](/reference/sdk/javascript) follow `Link` headers automatically.

## What's Next?

- [Events](events.md) - Send notifications with event types
- [Applications](applications.md) - Manage event types per application
- [Subscriptions](subscriptions.md) - Filter subscriptions by event type
- [Labels](labels.md) - Route events to subscriptions
- [Send your first event](/tutorials/getting-started) - Quick start guide
