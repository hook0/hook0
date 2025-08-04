# Error Codes Reference

Hook0 uses structured error responses with specific error codes to help you understand and handle different types of failures. This reference provides complete information about all error codes and their meanings.

## Error Response Format

All API errors follow this structure:

```json
{
  "error": {
    "type": "validation_error",
    "code": "INVALID_PARAMETER",
    "message": "The request contains invalid parameters",
    "details": [
      {
        "field": "event_type",
        "message": "Event type is required",
        "code": "REQUIRED_FIELD"
      }
    ],
    "request_id": "req_1234567890",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### Error Fields

- **type**: Category of error (e.g., `validation_error`, `authentication_error`)
- **code**: Specific error code for programmatic handling
- **message**: Human-readable error description
- **details**: Array of specific field-level errors (when applicable)
- **request_id**: Unique identifier for request tracing
- **timestamp**: ISO 8601 timestamp when the error occurred

## HTTP Status Codes

### 2xx Success Codes

| Status | Description |
|--------|-------------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 202 | Accepted - Request accepted for processing |
| 204 | No Content - Request successful, no content returned |

### 4xx Client Error Codes

| Status | Description |
|--------|-------------|
| 400 | Bad Request - Invalid request format or parameters |
| 401 | Unauthorized - Authentication required or invalid |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource does not exist |
| 405 | Method Not Allowed - HTTP method not supported |
| 409 | Conflict - Resource already exists or conflict |
| 413 | Payload Too Large - Request body exceeds limits |
| 422 | Unprocessable Entity - Valid format but semantic errors |
| 429 | Too Many Requests - Rate limit exceeded |

### 5xx Server Error Codes

| Status | Description |
|--------|-------------|
| 500 | Internal Server Error - Unexpected server error |
| 501 | Not Implemented - Feature not implemented |
| 502 | Bad Gateway - Upstream service error |
| 503 | Service Unavailable - Service temporarily unavailable |
| 504 | Gateway Timeout - Upstream service timeout |

## Authentication Errors (401)

### INVALID_TOKEN

**Description**: The provided authentication token is invalid or malformed.

**Response:**
```json
{
  "error": {
    "type": "authentication_error",
    "code": "INVALID_TOKEN",
    "message": "The provided authentication token is invalid"
  }
}
```

**Resolution:**
- Verify the token format: `Bearer biscuit:TOKEN`
- Check if the token has expired
- Ensure you're using the correct token for the environment

### TOKEN_EXPIRED

**Description**: The authentication token has expired.

**Response:**
```json
{
  "error": {
    "type": "authentication_error",
    "code": "TOKEN_EXPIRED",
    "message": "The authentication token has expired",
    "details": {
      "expired_at": "2024-01-15T10:30:00Z"
    }
  }
}
```

**Resolution:**
- Refresh the token using a refresh token
- Obtain a new authentication token
- Implement automatic token refresh in your client

### MISSING_TOKEN

**Description**: No authentication token provided in the request.

**Response:**
```json
{
  "error": {
    "type": "authentication_error",
    "code": "MISSING_TOKEN",
    "message": "Authentication token is required"
  }
}
```

**Resolution:**
- Include the Authorization header: `Authorization: Bearer biscuit:TOKEN`
- Verify your HTTP client is sending the header correctly

## Authorization Errors (403)

### INSUFFICIENT_PERMISSIONS

**Description**: The authenticated user lacks required permissions for this operation.

**Response:**
```json
{
  "error": {
    "type": "authorization_error",
    "code": "INSUFFICIENT_PERMISSIONS",
    "message": "Insufficient permissions to perform this action",
    "details": {
      "required_permissions": ["application:write"],
      "user_permissions": ["application:read"]
    }
  }
}
```

**Resolution:**
- Check the required permissions for the endpoint
- Request additional permissions from an administrator
- Use a token with appropriate permissions

### ORGANIZATION_ACCESS_DENIED

**Description**: Access denied to the specified organization.

**Response:**
```json
{
  "error": {
    "type": "authorization_error", 
    "code": "ORGANIZATION_ACCESS_DENIED",
    "message": "Access denied to organization",
    "details": {
      "organization_id": "org_1234567890"
    }
  }
}
```

**Resolution:**
- Verify you have access to the organization
- Check if the organization ID is correct
- Request access from an organization administrator

### APPLICATION_ACCESS_DENIED

**Description**: Access denied to the specified application.

**Response:**
```json
{
  "error": {
    "type": "authorization_error",
    "code": "APPLICATION_ACCESS_DENIED", 
    "message": "Access denied to application",
    "details": {
      "application_id": "app_1234567890"
    }
  }
}
```

## Validation Errors (400, 422)

### INVALID_PARAMETER

**Description**: One or more request parameters are invalid.

**Response:**
```json
{
  "error": {
    "type": "validation_error",
    "code": "INVALID_PARAMETER",
    "message": "Invalid request parameters",
    "details": [
      {
        "field": "event_type",
        "message": "Event type must be a valid string",
        "code": "INVALID_FORMAT",
        "value": 123
      },
      {
        "field": "payload",
        "message": "Payload must be a valid JSON object",
        "code": "INVALID_JSON"
      }
    ]
  }
}
```

**Resolution:**
- Check the API documentation for correct parameter formats
- Validate your request data before sending
- Ensure all required fields are included

### REQUIRED_FIELD

**Description**: A required field is missing from the request.

**Response:**
```json
{
  "error": {
    "type": "validation_error",
    "code": "REQUIRED_FIELD",
    "message": "Required field is missing",
    "details": [
      {
        "field": "event_type",
        "message": "Event type is required",
        "code": "REQUIRED_FIELD"
      }
    ]
  }
}
```

### INVALID_JSON

**Description**: The request body contains invalid JSON.

**Response:**
```json
{
  "error": {
    "type": "validation_error",
    "code": "INVALID_JSON",
    "message": "Request body contains invalid JSON",
    "details": {
      "parse_error": "Unexpected token } in JSON at position 15"
    }
  }
}
```

### PAYLOAD_TOO_LARGE

**Description**: The request payload exceeds size limits.

**Response:**
```json
{
  "error": {
    "type": "validation_error",
    "code": "PAYLOAD_TOO_LARGE",
    "message": "Request payload exceeds maximum size limit",
    "details": {
      "max_size": 1048576,
      "actual_size": 2097152
    }
  }
}
```

**Resolution:**
- Reduce the size of your request payload
- Split large requests into smaller batches
- Check the payload size limits in the documentation

### INVALID_EVENT_TYPE

**Description**: The specified event type is invalid or doesn't exist.

**Response:**
```json
{
  "error": {
    "type": "validation_error",
    "code": "INVALID_EVENT_TYPE",
    "message": "Invalid event type",
    "details": {
      "event_type": "user.invalid",
      "application_id": "app_1234567890"
    }
  }
}
```

**Resolution:**
- Create the event type before sending events
- Check the event type name for typos
- Verify the event type exists in the specified application

## Resource Errors (404, 409)

### RESOURCE_NOT_FOUND

**Description**: The requested resource does not exist.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "RESOURCE_NOT_FOUND",
    "message": "The requested resource was not found",
    "details": {
      "resource_type": "application",
      "resource_id": "app_1234567890"
    }
  }
}
```

**Resolution:**
- Verify the resource ID is correct
- Check if the resource has been deleted
- Ensure you have access to the resource

### RESOURCE_ALREADY_EXISTS

**Description**: Attempted to create a resource that already exists.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "RESOURCE_ALREADY_EXISTS",
    "message": "Resource already exists",
    "details": {
      "resource_type": "event_type",
      "identifier": "user.created",
      "existing_id": "et_1234567890"
    }
  }
}
```

**Resolution:**
- Use a different identifier for the new resource
- Update the existing resource instead of creating a new one
- Check if you intended to update rather than create

### ORGANIZATION_NOT_FOUND

**Description**: The specified organization does not exist.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "ORGANIZATION_NOT_FOUND",
    "message": "Organization not found",
    "details": {
      "organization_id": "org_1234567890"
    }
  }
}
```

### APPLICATION_NOT_FOUND

**Description**: The specified application does not exist.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "APPLICATION_NOT_FOUND",
    "message": "Application not found", 
    "details": {
      "application_id": "app_1234567890"
    }
  }
}
```

### SUBSCRIPTION_NOT_FOUND

**Description**: The specified subscription does not exist.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "SUBSCRIPTION_NOT_FOUND",
    "message": "Subscription not found",
    "details": {
      "subscription_id": "sub_1234567890"
    }
  }
}
```

### EVENT_NOT_FOUND

**Description**: The specified event does not exist.

**Response:**
```json
{
  "error": {
    "type": "resource_error",
    "code": "EVENT_NOT_FOUND",
    "message": "Event not found",
    "details": {
      "event_id": "evt_1234567890"
    }
  }
}
```

## Rate Limiting Errors (429)

### RATE_LIMIT_EXCEEDED

**Description**: Too many requests made within the rate limit window.

**Response:**
```json
{
  "error": {
    "type": "rate_limit_error",
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": {
      "limit": 1000,
      "window_seconds": 3600,
      "reset_at": "2024-01-15T11:30:00Z",
      "retry_after": 1800
    }
  }
}
```

**Headers:**
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1642167000
Retry-After: 1800
```

**Resolution:**
- Wait for the rate limit window to reset
- Reduce the frequency of your requests
- Implement exponential backoff in your client
- Consider upgrading to a higher tier for increased limits

### QUOTA_EXCEEDED

**Description**: Organization quota has been exceeded.

**Response:**
```json
{
  "error": {
    "type": "quota_error",
    "code": "QUOTA_EXCEEDED",
    "message": "Organization quota exceeded",
    "details": {
      "quota_type": "events_per_month",
      "limit": 100000,
      "current": 100000,
      "reset_date": "2024-02-01T00:00:00Z"
    }
  }
}
```

**Resolution:**
- Wait for the quota to reset
- Upgrade to a higher plan
- Optimize your event sending patterns
- Contact support for quota increases

## Business Logic Errors (422)

### INVALID_SUBSCRIPTION_TARGET

**Description**: The subscription target URL or configuration is invalid.

**Response:**
```json
{
  "error": {
    "type": "business_logic_error",
    "code": "INVALID_SUBSCRIPTION_TARGET",
    "message": "Invalid subscription target configuration",
    "details": {
      "field": "target.url",
      "message": "URL must use HTTPS protocol",
      "value": "http://example.com/webhook"
    }
  }
}
```

**Resolution:**
- Use HTTPS URLs for webhook endpoints
- Verify the URL format is correct
- Ensure the target endpoint is accessible

### EVENT_TYPE_DEACTIVATED

**Description**: Attempted to use a deactivated event type.

**Response:**
```json
{
  "error": {
    "type": "business_logic_error",
    "code": "EVENT_TYPE_DEACTIVATED",
    "message": "Event type is deactivated",
    "details": {
      "event_type": "user.deprecated",
      "deactivated_at": "2024-01-01T00:00:00Z"
    }
  }
}
```

**Resolution:**
- Use an active event type
- Reactivate the event type if needed
- Migrate to a replacement event type

### SUBSCRIPTION_DISABLED

**Description**: Attempted to process events for a disabled subscription.

**Response:**
```json
{
  "error": {
    "type": "business_logic_error",
    "code": "SUBSCRIPTION_DISABLED",
    "message": "Subscription is disabled",
    "details": {
      "subscription_id": "sub_1234567890",
      "disabled_at": "2024-01-10T15:00:00Z"
    }
  }
}
```

## Server Errors (5xx)

### INTERNAL_SERVER_ERROR

**Description**: An unexpected internal error occurred.

**Response:**
```json
{
  "error": {
    "type": "server_error",
    "code": "INTERNAL_SERVER_ERROR",
    "message": "An internal server error occurred",
    "details": {
      "incident_id": "inc_1234567890"
    }
  }
}
```

**Resolution:**
- Retry the request after a delay
- Check Hook0 status page for ongoing issues
- Contact support with the incident ID if the issue persists

### DATABASE_ERROR

**Description**: Database operation failed.

**Response:**
```json
{
  "error": {
    "type": "server_error",
    "code": "DATABASE_ERROR",
    "message": "Database operation failed",
    "details": {
      "operation": "insert",
      "table": "events"
    }
  }
}
```

### SERVICE_UNAVAILABLE

**Description**: The service is temporarily unavailable.

**Response:**
```json
{
  "error": {
    "type": "server_error",
    "code": "SERVICE_UNAVAILABLE",
    "message": "Service temporarily unavailable",
    "details": {
      "retry_after": 300,
      "maintenance": false
    }
  }
}
```

**Headers:**
```http
Retry-After: 300
```

## Webhook-Specific Errors

### WEBHOOK_DELIVERY_FAILED

**Description**: Webhook delivery to target endpoint failed.

**Response:**
```json
{
  "error": {
    "type": "webhook_error",
    "code": "WEBHOOK_DELIVERY_FAILED",
    "message": "Failed to deliver webhook",
    "details": {
      "event_id": "evt_1234567890",
      "subscription_id": "sub_1234567890",
      "target_url": "https://api.example.com/webhook",
      "status_code": 500,
      "response_body": "Internal Server Error",
      "attempt_number": 3,
      "next_retry_at": "2024-01-15T11:00:00Z"
    }
  }
}
```

### WEBHOOK_TIMEOUT

**Description**: Webhook delivery timed out.

**Response:**
```json
{
  "error": {
    "type": "webhook_error",
    "code": "WEBHOOK_TIMEOUT",
    "message": "Webhook delivery timed out",
    "details": {
      "event_id": "evt_1234567890",
      "subscription_id": "sub_1234567890",
      "target_url": "https://api.example.com/webhook",
      "timeout_seconds": 30,
      "attempt_number": 2
    }
  }
}
```

### WEBHOOK_SSL_ERROR

**Description**: SSL/TLS error when connecting to webhook endpoint.

**Response:**
```json
{
  "error": {
    "type": "webhook_error",
    "code": "WEBHOOK_SSL_ERROR",
    "message": "SSL certificate verification failed",
    "details": {
      "target_url": "https://api.example.com/webhook",
      "ssl_error": "certificate verify failed: self signed certificate"
    }
  }
}
```

## Error Handling Best Practices

### Client Implementation

```javascript
async function handleApiResponse(response) {
  if (!response.ok) {
    const error = await response.json();
    
    switch (error.error.code) {
      case 'RATE_LIMIT_EXCEEDED':
        const retryAfter = error.details.retry_after;
        await sleep(retryAfter * 1000);
        return retryRequest();
        
      case 'TOKEN_EXPIRED':
        await refreshToken();
        return retryRequest();
        
      case 'QUOTA_EXCEEDED':
        throw new QuotaExceededError(error.error.message);
        
      case 'RESOURCE_NOT_FOUND':
        throw new NotFoundError(error.error.message);
        
      default:
        throw new ApiError(error.error.message, error.error.code);
    }
  }
  
  return response.json();
}
```

### Retry Logic

```javascript
const retryableErrors = [
  'INTERNAL_SERVER_ERROR',
  'DATABASE_ERROR', 
  'SERVICE_UNAVAILABLE',
  'WEBHOOK_TIMEOUT'
];

async function requestWithRetry(requestFn, maxRetries = 3) {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await requestFn();
    } catch (error) {
      if (attempt === maxRetries || !retryableErrors.includes(error.code)) {
        throw error;
      }
      
      const delay = Math.min(1000 * Math.pow(2, attempt - 1), 30000);
      await sleep(delay);
    }
  }
}
```

### Error Logging

```javascript
function logError(error, context = {}) {
  console.error('Hook0 API Error', {
    code: error.error?.code,
    type: error.error?.type,
    message: error.error?.message,
    request_id: error.error?.request_id,
    context,
    timestamp: new Date().toISOString()
  });
}
```

## Support and Troubleshooting

### Getting Help

1. **Check Status Page**: Visit [status.hook0.com](https://status.hook0.com)
2. **Review Documentation**: Ensure you're following API guidelines
3. **Contact Support**: Include the `request_id` from error responses
4. **Community**: Join our Discord for community support

### Error Debugging

1. **Request ID**: Always include the `request_id` when reporting issues
2. **Context**: Provide information about what you were trying to do
3. **Reproduction**: Include steps to reproduce the error
4. **Environment**: Specify if using production or staging

### Monitoring

Set up monitoring for these error patterns:
- High rates of 4xx errors (client issues)
- 5xx errors (server issues)  
- Rate limit errors
- Quota exceeded errors
- Webhook delivery failures

For more information on error handling and recovery strategies, see our [Debugging Guide](../how-to-guides/debug-failed-webhooks.md).