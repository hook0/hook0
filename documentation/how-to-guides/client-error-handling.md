---
title: Client-side Error Handling Best Practices
description: Implement robust error handling for Hook0 API responses in your applications
---

# Client-side Error Handling Best Practices

This guide shows you how to implement robust error handling when integrating with Hook0's API.

## Prerequisites

- Basic understanding of Hook0 API
- Familiarity with your programming language's HTTP client
- Access to [Error Codes Reference](/reference/error-codes) for error details

## Understanding Hook0 Error Responses

Hook0 uses RFC 7807 Problem Details format. All errors include:
- `id`: Error identifier for programmatic handling
- `detail`: Human-readable explanation
- `status`: HTTP status code

## Client Implementation

Handle errors based on their `id` field for precise error handling:

```typescript
async function handleApiResponse(response: Response) {
  if (!response.ok) {
    const error = await response.json();

    switch (error.id) {
      case 'TooManyEventsToday':
        // Wait until tomorrow or upgrade plan
        throw new QuotaExceededError(error.detail);

      case 'AuthInvalidBiscuit':
        // Refresh or re-authenticate
        await refreshToken();
        return retryRequest();

      case 'EventTypeDoesNotExist':
        // Create event type first
        await createEventType(eventType);
        return retryRequest();

      case 'NotFound':
        throw new NotFoundError(error.detail);

      default:
        throw new ApiError(error.detail, error.id);
    }
  }

  return response.json();
}
```

## Implementing Retry Logic

Some errors are transient and can be retried:

```typescript
const retryableErrors = [
  'InternalServerError',
  'ServiceUnavailable',
  'AuthBiscuitLookupError',
  'AuthApplicationSecretLookupError'
];

async function requestWithRetry(requestFn: () => Promise<Response>, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await requestFn();
    } catch (error) {
      if (attempt === maxRetries || !retryableErrors.includes(error.id)) {
        throw error;
      }

      const delay = Math.min(1000 * Math.pow(2, attempt - 1), 30000);
      await sleep(delay);
    }
  }
}
```

## Error Logging

Log errors with full context for debugging:

```typescript
function logError(error: any) {
  console.error('Hook0 API Error', {
    id: error.id,
    type: error.type,
    title: error.title,
    detail: error.detail,
    status: error.status,
    timestamp: new Date().toISOString()
  });
}
```

## Error Categories and Handling Strategy

| Error Category | Strategy | Example Errors |
|---------------|----------|----------------|
| Authentication (401, 403) | Refresh token, re-authenticate | `AuthInvalidBiscuit`, `AuthFailedLogin` |
| Validation (400, 422) | Fix request data | `Validation`, `JsonPayload` |
| Quota (429) | Wait or upgrade | `TooManyEventsToday` |
| Server (500, 503) | Retry with backoff | `InternalServerError`, `ServiceUnavailable` |
| Not Found (404) | Check resource exists | `NotFound` |

## Next Steps

- See [Error Codes Reference](/reference/error-codes) for complete error list
- Learn about [Webhook Authentication](/tutorials/webhook-authentication)
- Set up [Monitoring](/how-to-guides/monitor-webhook-performance)
