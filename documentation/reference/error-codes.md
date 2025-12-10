# Error Codes Reference

Hook0 uses RFC 7807 Problem Details for HTTP APIs format for structured error responses.

## Error Response Format

All API errors follow this structure (RFC 7807):

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidBiscuit",
  "id": "AuthInvalidBiscuit",
  "title": "Invalid Biscuit",
  "detail": "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired.",
  "status": 403
}
```

### Error Fields

- **type**: URL to error documentation
- **id**: Error identifier (enum variant name)
- **title**: Short human-readable summary
- **detail**: Explanation of the error
- **status**: HTTP status code
- **validation**: Additional validation errors (for validation failures)

## HTTP Status Codes

| Status | Description |
|--------|-------------|
| 400 | Bad Request - Invalid request format or parameters |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Insufficient permissions or invalid credentials |
| 404 | Not Found - Resource does not exist |
| 409 | Conflict - Resource already exists |
| 410 | Gone - Feature disabled |
| 422 | Unprocessable Entity - Validation errors |
| 429 | Too Many Requests - Rate limit or quota exceeded |
| 500 | Internal Server Error - Unexpected error |
| 503 | Service Unavailable - Service temporarily unavailable |

## Authentication Errors (401, 403)

### AuthNoAuthorizationHeader

**Status**: 401

No `Authorization` header found in the HTTP request.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthNoAuthorizationHeader",
  "id": "AuthNoAuthorizationHeader",
  "title": "No `Authorization` header was found in the HTTP request",
  "detail": "`Authorization` header must be provided and must contain a bearer token."
}
```

**Resolution**: Include `Authorization: Bearer {YOUR_TOKEN}` header.

### AuthInvalidAuthorizationHeader

**Status**: 400

`Authorization` header format is invalid.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidAuthorizationHeader",
  "id": "AuthInvalidAuthorizationHeader",
  "title": "`Authorization` header is invalid",
  "detail": "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`."
}
```

**Resolution**: Verify header format: `Authorization: Bearer TOKEN`.

### AuthInvalidBiscuit

**Status**: 403

The provided Biscuit token is invalid or expired.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidBiscuit",
  "id": "AuthInvalidBiscuit",
  "title": "Invalid Biscuit",
  "detail": "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired."
}
```

**Resolution**: Obtain a new token or verify your Biscuit private key configuration.

### AuthBiscuitLookupError

**Status**: 500

Database error while verifying Biscuit token revocation.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthBiscuitLookupError",
  "id": "AuthBiscuitLookupError",
  "title": "Could not check database to verify if the provided Biscuit was revoked",
  "detail": "This is likely to be caused by database unavailability."
}
```

### AuthInvalidApplicationSecret

**Status**: 403

The provided application secret does not exist.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidApplicationSecret",
  "id": "AuthInvalidApplicationSecret",
  "title": "Invalid application secret",
  "detail": "The provided application secret does not exist."
}
```

### AuthApplicationSecretLookupError

**Status**: 500

Database error while verifying application secret.

### AuthFailedLogin

**Status**: 401

Login credentials do not match a valid user.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthFailedLogin",
  "id": "AuthFailedLogin",
  "title": "Login failed",
  "detail": "The provided credentials do not match ones of a valid user."
}
```

### AuthEmailNotVerified

**Status**: 401

User's email address has not been verified yet.

```json
{
  "type": "https://hook0.com/documentation/errors/AuthEmailNotVerified",
  "id": "AuthEmailNotVerified",
  "title": "Email not verified",
  "detail": "Your email has not been verified yet. Please check your inbox."
}
```

### AuthFailedRefresh

**Status**: 401

Refresh token is invalid or expired.

### AuthEmailExpired

**Status**: 401

Email verification link has expired.

## Validation Errors (400, 422)

### Validation

**Status**: 422

Request contains validation errors.

```json
{
  "type": "https://hook0.com/documentation/errors/Validation",
  "id": "Validation",
  "title": "Provided input is malformed",
  "detail": "event_type: required field",
  "validation": {
    "event_type": [
      {
        "code": "required",
        "message": "This field is required"
      }
    ]
  }
}
```

**Resolution**: Fix the validation errors listed in the `validation` field.

### JsonPayload

**Status**: 400

Request body contains invalid JSON or exceeds size limits.

```json
{
  "type": "https://hook0.com/documentation/errors/JsonPayload",
  "id": "JsonPayload",
  "title": "Provided body could not be decoded as JSON",
  "detail": "Body is too big (maximum is 1048576 bytes)"
}
```

Possible detail messages:
- "Body is too big (maximum is X bytes)"
- "Content-Type header should be set to 'application/json'"
- "JSON deserialization error: ..."
- "JSON serialization error: ..."

## Organization Errors

### OrganizationNameMissing

**Status**: 400

Organization name cannot be empty.

### OrganizationIsNotEmpty

**Status**: 409

Cannot delete organization that contains applications.

```json
{
  "type": "https://hook0.com/documentation/errors/OrganizationIsNotEmpty",
  "id": "OrganizationIsNotEmpty",
  "title": "Organization is not empty",
  "detail": "Organizations that contain at least an application cannot be deleted; applications must be deleted first. If you believe this is a mistake, please contact the Hook0 support team."
}
```

## User & Registration Errors

### UserAlreadyExist

**Status**: 409

Email address is already registered.

### RegistrationDisabled

**Status**: 410

User registration has been disabled by an administrator.

### PasswordTooShort

**Status**: 400

Password does not meet minimum length requirement.

```json
{
  "type": "https://hook0.com/documentation/errors/PasswordTooShort",
  "id": "PasswordTooShort",
  "title": "Provided password is too short",
  "detail": "Password must be at least 12 characters long."
}
```

### InvitedUserDoesNotExist

**Status**: 404

User to invite does not exist.

### InvitedUserAlreadyInOrganization

**Status**: 409

User is already a member of the organization.

## Application Errors

### ApplicationNameMissing

**Status**: 400

Application name cannot be empty.

## Event Type Errors

### EventTypeAlreadyExist

**Status**: 409

Event type with this name already exists.

### EventTypeDoesNotExist

**Status**: 400

Event type does not exist or was deactivated.

```json
{
  "type": "https://hook0.com/documentation/errors/EventTypeDoesNotExist",
  "id": "EventTypeDoesNotExist",
  "title": "Invalid event type",
  "detail": "Event type does not exist or was deactivated. You should (re)create it."
}
```

## Event Errors

### EventAlreadyIngested

**Status**: 409

Event with this ID was already ingested (idempotency check).

```json
{
  "type": "https://hook0.com/documentation/errors/EventAlreadyIngested",
  "id": "EventAlreadyIngested",
  "title": "Event already Ingested",
  "detail": "This event was previously ingested and recorded inside Hook0 service."
}
```

### EventInvalidPayloadContentType

**Status**: 400

Specified payload content type is not supported.

```json
{
  "type": "https://hook0.com/documentation/errors/EventInvalidPayloadContentType",
  "id": "EventInvalidPayloadContentType",
  "title": "Invalid event payload content type",
  "detail": "The specified event payload content type is not handled. Valid content types are: application/json, text/plain, application/octet-stream"
}
```

### EventInvalidBase64Payload

**Status**: 400

Event payload is not valid base64.

### EventInvalidJsonPayload

**Status**: 400

Event payload is not valid JSON.

## Role Errors

### InvalidRole

**Status**: 400

Provided role does not exist.

```json
{
  "type": "https://hook0.com/documentation/errors/InvalidRole",
  "id": "InvalidRole",
  "title": "Provided role does not exist",
  "detail": "Valid roles are: admin, member."
}
```

## Worker Errors

### UnauthorizedWorkers

**Status**: 400

Attempted to use dedicated workers not authorized for your organization.

## Quota Errors (429)

All quota errors return status 429 (Too Many Requests).

### TooManyMembersPerOrganization

Organization has reached maximum number of members.

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyMembersPerOrganization",
  "id": "TooManyMembersPerOrganization",
  "title": "Exceeded number of users that can be invited in this organization",
  "detail": "This organization cannot have more than 10 users. You might want to upgrade to a better plan."
}
```

### TooManyApplicationsPerOrganization

Organization has reached maximum number of applications.

### TooManyEventsToday

Organization has reached daily event limit.

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyEventsToday",
  "id": "TooManyEventsToday",
  "title": "Exceeded number of events that can be ingested in this organization today",
  "detail": "This organization cannot ingest more than 1000 events per day. You might want to upgrade to a better plan."
}
```

### TooManySubscriptionsPerApplication

Application has reached maximum number of subscriptions.

### TooManyEventTypesPerApplication

Application has reached maximum number of event types.

## Generic Errors

### NotFound

**Status**: 404

Resource does not exist.

```json
{
  "type": "https://hook0.com/documentation/errors/NotFound",
  "id": "NotFound",
  "title": "Item not found",
  "detail": "Could not find the item. Check the identifier or that you have the right to access it."
}
```

### Forbidden

**Status**: 403

Insufficient rights to access or modify resource.

### InternalServerError

**Status**: 500

Unexpected server error occurred.

```json
{
  "type": "https://hook0.com/documentation/errors/InternalServerError",
  "id": "InternalServerError",
  "title": "Something wrong happened",
  "detail": "Hook0 server had issue handling your request. Our team was notified."
}
```

### ServiceUnavailable

**Status**: 503

Service is temporarily unavailable.

```json
{
  "type": "https://hook0.com/documentation/errors/ServiceUnavailable",
  "id": "ServiceUnavailable",
  "title": "Service unavailable",
  "detail": "Database is unavailable."
}
```

## Error Handling Best Practices

### Client Implementation

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

### Retry Logic

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

### Error Logging

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

## Complete Error List

| Error ID | Status | Description |
|----------|--------|-------------|
| AuthNoAuthorizationHeader | 401 | No Authorization header |
| AuthInvalidAuthorizationHeader | 400 | Invalid Authorization header format |
| AuthApplicationSecretLookupError | 500 | Database error checking application secret |
| AuthInvalidApplicationSecret | 403 | Invalid application secret |
| AuthBiscuitLookupError | 500 | Database error checking Biscuit token |
| AuthInvalidBiscuit | 403 | Invalid or expired Biscuit token |
| AuthFailedLogin | 401 | Invalid login credentials |
| AuthEmailNotVerified | 401 | Email not verified |
| AuthFailedRefresh | 401 | Invalid refresh token |
| AuthEmailExpired | 401 | Email verification link expired |
| OrganizationNameMissing | 400 | Empty organization name |
| OrganizationIsNotEmpty | 409 | Cannot delete non-empty organization |
| UserAlreadyExist | 409 | Email already registered |
| RegistrationDisabled | 410 | Registration disabled |
| PasswordTooShort | 400 | Password below minimum length |
| InvitedUserDoesNotExist | 404 | User to invite not found |
| InvitedUserAlreadyInOrganization | 409 | User already in organization |
| ApplicationNameMissing | 400 | Empty application name |
| InvalidRole | 400 | Invalid role specified |
| EventTypeAlreadyExist | 409 | Event type already exists |
| EventTypeDoesNotExist | 400 | Event type not found |
| UnauthorizedWorkers | 400 | Dedicated workers not authorized |
| EventAlreadyIngested | 409 | Duplicate event ID |
| EventInvalidPayloadContentType | 400 | Unsupported content type |
| EventInvalidBase64Payload | 400 | Invalid base64 payload |
| EventInvalidJsonPayload | 400 | Invalid JSON payload |
| TooManyMembersPerOrganization | 429 | Member quota exceeded |
| TooManyApplicationsPerOrganization | 429 | Application quota exceeded |
| TooManyEventsToday | 429 | Daily event quota exceeded |
| TooManySubscriptionsPerApplication | 429 | Subscription quota exceeded |
| TooManyEventTypesPerApplication | 429 | Event type quota exceeded |
| JsonPayload | 400 | Invalid JSON request body |
| Validation | 422 | Validation errors |
| NotFound | 404 | Resource not found |
| InternalServerError | 500 | Unexpected server error |
| Forbidden | 403 | Insufficient permissions |
| ServiceUnavailable | 503 | Service unavailable |

## Support

For issues not covered by this reference:
- Check the API documentation
- Review server logs if self-hosting
- Contact support with the error details
