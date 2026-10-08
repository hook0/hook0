---
title: "Multi-Tenant Webhook Service: Route Each Tenant With Labels"
sidebar_label: "Multi-tenant architecture"
description: "One Hook0 application for every customer: tag events with a tenant label, filter each tenant's subscriptions on it, and show per-tenant delivery logs."
keywords: [multi-tenant webhooks, multi-tenant webhook service, webhook per tenant, tenant isolation, label routing, SaaS webhooks, scalable webhook architecture]
faqItems:
  - question: "How do I isolate webhooks per tenant in Hook0?"
    answer: >-
      Put a tenant label on every event, for example tenant_id, and set the
      same label filter on each tenant's subscriptions. A subscription only
      receives an event when all of its label filters match the event's
      labels, so one tenant's endpoint never gets another tenant's events.
  - question: "Do I need one Hook0 application per tenant?"
    answer: >-
      No. One application can serve every tenant: event types are shared,
      and labels on events and subscriptions decide who receives what. Labels
      can also go finer than the tenant, such as project_id or environment,
      for project-level or group-level webhooks.
  - question: "How do tenants see their own webhook delivery logs?"
    answer: >-
      List request attempts filtered by subscription_id, then fetch each
      attempt's response for the status code, headers and body. Your backend
      calls the API for the tenant's own subscriptions and renders the result
      in your webhook settings page.
---

# Multi-tenant webhook architecture

To run a multi-tenant webhook service on Hook0, keep one application for your whole platform, add a tenant [label](/concepts/labels) such as `tenant_id` to every event you send, and create each customer's subscriptions with the same label filter. Hook0 then delivers each event only to the endpoints of the tenant it belongs to, and you read delivery logs per subscription to show each customer their own history.

The example below models a code-hosting platform in the style of GitLab.com, with namespace, project and pipeline events. The same architecture applies to any multi-tenant SaaS.

:::tip Try the pattern on a free account
[Create a free Hook0 Cloud account](https://app.hook0.com/register) (100 events a day, no credit card) and follow the steps below with two test tenants.
:::

## The problem

A multi-tenant platform needs:
- Webhook delivery with automatic retries
- Visibility into logs and delivery attempts
- Tenant isolation (data privacy)
- Infrastructure that scales to billions of events

### Set up environment variables

```bash
# Set your service token (from dashboard)
export HOOK0_TOKEN="YOUR_TOKEN_HERE"
export HOOK0_API="https://app.hook0.com/api/v1" # Replace by your domain (or http://localhost:8081 locally)

# Set your application ID (shown in dashboard URL or application details)
export APP_ID="YOUR_APPLICATION_ID_HERE"
```

Save these values:
```bash
# Save to .env file for later use
cat > .env <<EOF
HOOK0_TOKEN=$HOOK0_TOKEN
HOOK0_API=$HOOK0_API
APP_ID=$APP_ID
EOF
```

## Architecture overview

Hook0 gives multi-tenant platforms:
1. Event routing per tenant using labels
2. Request attempt tracking for user visibility
3. Automatic retries with a fixed retry schedule
4. Isolation between tenants

## Step 1: Create event types

First, define [event types](/concepts/event-types) for your platform activities. In this GitLab example:

```bash
# Create push event type
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "gitlab",
    "resource_type": "push",
    "verb": "created"
  }'

# Create merge request event type
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "gitlab",
    "resource_type": "merge_request",
    "verb": "opened"
  }'

# Create pipeline event type
curl -X POST "$HOOK0_API/event_types" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "service": "gitlab",
    "resource_type": "pipeline",
    "verb": "completed"
  }'
```

## Step 2: Multi-tenant event ingestion

When events occur in your platform, send them to Hook0 with tenant-specific labels:

```bash
# Example: User pushes code to a project
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_type": "gitlab.push.created",
    "payload": "{\"ref\":\"refs/heads/main\",\"commits\":3,\"author\":\"john@example.com\",\"message\":\"Fix bug in authentication\"}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T14:30:00Z",
    "labels": {
      "namespace_id": "12345",
      "project_id": "67890",
      "group_id": "54321",
      "user_id": "usr_abc123",
      "project_path": "acme-corp/api-gateway",
      "visibility": "private",
      "plan": "premium",
      "ref": "main"
    }
  }'

# Example: Pipeline completion event
curl -X POST "$HOOK0_API/event" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "event_type": "gitlab.pipeline.completed",
    "payload": "{\"pipeline_id\":\"9876\",\"status\":\"success\",\"duration\":240,\"stages\":[\"test\",\"build\",\"deploy\"]}",
    "payload_content_type": "application/json",
    "occurred_at": "2024-01-15T14:45:00Z",
    "labels": {
      "namespace_id": "12345",
      "project_id": "67890",
      "pipeline_id": "9876",
      "environment": "production",
      "triggered_by": "usr_abc123"
    }
  }'
```

## Step 3: User webhook management

Users create webhooks that are registered as Hook0 [subscriptions](/concepts/subscriptions) with label-based filtering:

```bash
# User creates a project webhook via GitLab UI
# GitLab backend registers it with Hook0:
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["gitlab.push.created", "gitlab.merge_request.opened"],
    "description": "ACME Corp API Gateway project webhook",
    "labels": {
      "project_id": "67890"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://ci.acme-corp.com/webhooks/gitlab",
      "headers": {
        "X-Custom-Header": "acme-api-gateway"
      }
    },
    "metadata": {
      "gitlab_webhook_id": "webhook_123",
      "created_by_user": "usr_abc123",
      "project_path": "acme-corp/api-gateway"
    }
  }'

# Group-level webhook (triggers for all projects in group)
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["gitlab.pipeline.completed"],
    "description": "ACME Corp group-wide pipeline notifications",
    "labels": {
      "group_id": "54321"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://monitoring.acme-corp.com/pipelines",
      "headers": {}
    },
    "metadata": {
      "gitlab_group_webhook_id": "group_webhook_456",
      "group_path": "acme-corp"
    }
  }'
```

## Step 4: Exposing logs and request attempts to users

Expose webhook delivery information through your UI by querying Hook0's API:

```bash
# Get request attempts for a subscription (webhook)
# GitLab UI calls this when user views webhook logs
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Response includes delivery attempts with status, response codes, and timing
# GitLab formats this data for display in their webhook settings UI

# Filter by time range for recent attempts
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={SUBSCRIPTION_ID}&min_created_at=2024-01-15T00:00:00Z" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Get response body for debugging failed deliveries
# First get the response_id from request_attempt, then fetch response details
curl -X GET "$HOOK0_API/responses/{RESPONSE_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

## Step 5: User-facing webhook management

Users can manage their webhooks through your UI, which internally calls Hook0:

:::warning PUT Requires ALL Fields
When updating subscriptions, the PUT endpoint requires ALL fields (`application_id`, `is_enabled`, `event_types`, `labels`, `target`), not just the ones you want to change. First GET the current subscription, then send the complete object with your modifications.
:::

```bash
# List all webhooks for an application (GitLab queries subscriptions)
curl -X GET "$HOOK0_API/subscriptions/?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Update webhook (e.g., change URL or events) - include ALL fields
curl -X PUT "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["gitlab.push.created", "gitlab.merge_request.opened", "gitlab.pipeline.completed"],
    "description": "ACME Corp API Gateway project webhook",
    "labels": {
      "project_id": "67890"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://new-ci.acme-corp.com/webhooks",
      "headers": {"Content-Type": "application/json"}
    }
  }'

# Disable webhook temporarily (all fields required)
curl -X PUT "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": false,
    "event_types": ["gitlab.push.created", "gitlab.merge_request.opened"],
    "description": "ACME Corp API Gateway project webhook",
    "labels": {
      "project_id": "67890"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://ci.acme-corp.com/webhooks/gitlab",
      "headers": {"Content-Type": "application/json"}
    }
  }'

# Delete webhook
curl -X DELETE "$HOOK0_API/subscriptions/{SUBSCRIPTION_ID}?application_id=$APP_ID" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

## Step 6: Retry failed webhook deliveries

Users can manually retry failed webhook deliveries through your UI:

```bash
# Replay a specific event (user clicks "Retry" in GitLab UI)
curl -X POST "$HOOK0_API/events/{EVENT_ID}/replay" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'"
  }'
```

## Step 7: System hooks for platform administrators

Platform administrators can set up system-wide hooks:

```bash
# Create system hook for user creation events
curl -X POST "$HOOK0_API/subscriptions" \
  -H "Authorization: Bearer $HOOK0_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "'"$APP_ID"'",
    "is_enabled": true,
    "event_types": ["gitlab.user.created", "gitlab.user.blocked", "gitlab.project.created"],
    "description": "GitLab.com system hooks for compliance",
    "labels": {
      "hook_type": "system"
    },
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://compliance.gitlab.com/system-events",
      "headers": {
        "X-System-Hook": "true"
      }
    }
  }'
```

## Step 8: Analytics and monitoring

Monitor webhook health across your platform:

```bash
# Query delivery metrics by time range
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&min_created_at=2024-01-15T00:00:00Z&max_created_at=2024-01-15T23:59:59Z" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Monitor webhook performance for a specific subscription
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&subscription_id={SUBSCRIPTION_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"

# Filter by event to track specific webhook deliveries
curl -X GET "$HOOK0_API/request_attempts/?application_id=$APP_ID&event_id={EVENT_ID}" \
  -H "Authorization: Bearer $HOOK0_TOKEN"
```

## Benefits for end users

1. Users see delivery attempts, response codes, and error messages in your UI
2. Automatic retries with predefined delays handle transient failures
3. Request/response bodies are available for debugging
4. Full isolation between organizations and projects
5. Hook0 handles the infrastructure for millions of webhooks

## Security considerations

- Each tenant (namespace/project/organization) has isolated webhook data
- Biscuit tokens (used by both user sessions and Service tokens) include facts limiting access to specific tenants
- Request logs are retained according to user's plan (e.g., 7 days for free, 30 days for premium)
- Sensitive headers are redacted in logs
- Webhook signatures prevent tampering and replay attacks

## Summary

With Hook0, multi-tenant platforms can:
- Offload webhook delivery to a specialized service
- Give users delivery logs and retry controls
- Keep strict tenant isolation
- Scale to billions of events across millions of projects
- Stop building webhook infrastructure and focus on the product

The label-based routing (`labels`) handles flexible filtering without a query language. It fits hierarchical multi-tenant structures (users, groups, organizations, projects) well.
