# GitLab.com Multi-Tenant Webhook Management with Hook0

This guide demonstrates how GitLab.com could leverage Hook0 to manage webhooks for millions of users, projects, and organizations while providing visibility into delivery attempts and logs.

## The GitLab.com Challenge

GitLab.com hosts millions of projects across thousands of organizations. Each project can have multiple webhooks, and organizations need:
- Reliable webhook delivery with automatic retries
- Visibility into webhook logs and delivery attempts
- Multi-tenant isolation ensuring data privacy
- Scalable infrastructure handling billions of events

## Architecture Overview

Hook0 provides GitLab.com with:
1. **Multi-tenant event routing** using labels
2. **Request attempt tracking** for user visibility
3. **Automatic retry logic** with exponential backoff
4. **Secure isolation** between tenants

## Step 1: Create Event Types for GitLab Events

First, GitLab.com needs to define event types for all GitLab activities:

```bash
# Create push event type
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:GITLAB_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "service": "gitlab",
    "resource_type": "push",
    "verb": "created",
    "description": "Code pushed to repository"
  }'

# Create merge request event type
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:GITLAB_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "service": "gitlab",
    "resource_type": "merge_request",
    "verb": "opened",
    "description": "Merge request opened"
  }'

# Create pipeline event type
curl -X POST "https://app.hook0.com/api/v1/event_types" \
  -H "Authorization: Bearer biscuit:GITLAB_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "service": "gitlab",
    "resource_type": "pipeline",
    "verb": "completed",
    "description": "CI/CD pipeline completed"
  }'
```

## Step 2: Multi-Tenant Event Ingestion

When events occur in GitLab, they're sent to Hook0 with tenant-specific labels:

```bash
# Example: User pushes code to a project
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "event_id": "evt-push-789abc",
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
curl -X POST "https://app.hook0.com/api/v1/event" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "event_id": "evt-pipeline-def456",
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

## Step 3: User Webhook Management

GitLab users create webhooks that are registered as Hook0 subscriptions with label-based filtering:

```bash
# User creates a project webhook via GitLab UI
# GitLab backend registers it with Hook0:
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "is_enabled": true,
    "event_types": ["gitlab.push.created", "gitlab.merge_request.opened"],
    "description": "ACME Corp API Gateway project webhook",
    "label_key": "project_id",
    "label_value": "67890",
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
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "is_enabled": true,
    "event_types": ["gitlab.pipeline.completed"],
    "description": "ACME Corp group-wide pipeline notifications",
    "label_key": "group_id",
    "label_value": "54321",
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

## Step 4: Exposing Logs and Request Attempts to Users

GitLab.com exposes webhook delivery information through their UI by querying Hook0's API:

```bash
# Get request attempts for a specific project's webhooks
# GitLab UI calls this when user views webhook logs
curl -X GET "https://app.hook0.com/api/v1/request_attempts?application_id=gitlab-main-app&label_key=project_id&label_value=67890&limit=50" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"

# Response includes delivery attempts with status, response codes, and timing
# GitLab formats this data for display in their webhook settings UI

# Get specific request attempt details
curl -X GET "https://app.hook0.com/api/v1/request_attempts/req_xyz789" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"

# Get response body for debugging failed deliveries
curl -X GET "https://app.hook0.com/api/v1/responses/resp_abc123" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"
```

## Step 5: User-Facing Webhook Management

GitLab users can manage their webhooks through GitLab's UI, which internally calls Hook0:

```bash
# List all webhooks for a project (GitLab queries subscriptions)
curl -X GET "https://app.hook0.com/api/v1/subscriptions?application_id=gitlab-main-app" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"

# Update webhook (e.g., change URL or events)
curl -X PUT "https://app.hook0.com/api/v1/subscriptions/sub_123abc" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "is_enabled": true,
    "event_types": ["gitlab.push.created", "gitlab.merge_request.opened", "gitlab.pipeline.completed"],
    "target": {
      "type": "http",
      "method": "POST",
      "url": "https://new-ci.acme-corp.com/webhooks",
      "headers": {}
    }
  }'

# Disable webhook temporarily
curl -X PUT "https://app.hook0.com/api/v1/subscriptions/sub_123abc" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "is_enabled": false
  }'

# Delete webhook
curl -X DELETE "https://app.hook0.com/api/v1/subscriptions/sub_123abc" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"
```

## Step 6: Retry Failed Webhook Deliveries

Users can manually retry failed webhook deliveries through GitLab's UI:

```bash
# Replay a specific event (user clicks "Retry" in GitLab UI)
curl -X POST "https://app.hook0.com/api/v1/events/evt-push-789abc/replay" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "subscription_id": "sub_123abc"
  }'
```

## Step 7: System Hooks for GitLab Administrators

GitLab.com administrators can set up system-wide hooks:

```bash
# Create system hook for user creation events
curl -X POST "https://app.hook0.com/api/v1/subscriptions" \
  -H "Authorization: Bearer biscuit:GITLAB_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "application_id": "gitlab-main-app",
    "is_enabled": true,
    "event_types": ["gitlab.user.created", "gitlab.user.blocked", "gitlab.project.created"],
    "description": "GitLab.com system hooks for compliance",
    "label_key": "hook_type",
    "label_value": "system",
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

## Step 8: Analytics and Monitoring

GitLab.com monitors webhook health across the platform:

```bash
# Query delivery metrics for a namespace
curl -X GET "https://app.hook0.com/api/v1/request_attempts?application_id=gitlab-main-app&label_key=namespace_id&label_value=12345&start_time=2024-01-15T00:00:00Z&end_time=2024-01-15T23:59:59Z" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"

# Monitor webhook performance for premium customers
curl -X GET "https://app.hook0.com/api/v1/request_attempts?application_id=gitlab-main-app&label_key=plan&label_value=premium&status=failed" \
  -H "Authorization: Bearer biscuit:GITLAB_SERVICE_TOKEN"
```

## Benefits for GitLab.com Users

1. **Visibility**: Users see all webhook delivery attempts, response codes, and error messages directly in GitLab's UI
2. **Reliability**: Automatic retries with exponential backoff ensure events are delivered
3. **Debugging**: Access to request/response bodies helps users debug integration issues
4. **Multi-tenancy**: Complete isolation between different organizations and projects
5. **Scalability**: Hook0 handles the infrastructure complexity of managing millions of webhooks

## Security Considerations

- Each GitLab.com namespace/project has isolated webhook data
- Biscuit tokens include facts limiting access to specific tenants
- Request logs are retained according to user's plan (e.g., 7 days for free, 30 days for premium)
- Sensitive headers are redacted in logs
- Webhook signatures prevent tampering and replay attacks

## Summary

By using Hook0, GitLab.com can:
- Offload webhook delivery infrastructure to a specialized service
- Provide users with detailed delivery logs and retry capabilities
- Maintain strict multi-tenant isolation
- Scale to handle billions of events across millions of projects
- Focus on core Git and CI/CD functionality rather than webhook infrastructure

The label-based routing system (`label_key` and `label_value`) enables flexible filtering without requiring complex query languages, making it perfect for GitLab's hierarchical structure of users, groups, and projects.