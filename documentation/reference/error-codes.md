# Error Codes Reference

<!--
  ⚠️  AUTO-GENERATED FILE - DO NOT EDIT MANUALLY

  This file is generated from the Hook0 API /errors endpoint.
  To regenerate, run: npm run generate:errors
-->

Hook0 uses RFC 7807 Problem Details for HTTP APIs format for structured error responses.

## Error Response Format

All API errors follow this structure (RFC 7807):

- **type**: URL to error documentation
- **id**: Error identifier (enum variant name)
- **title**: Short human-readable summary
- **detail**: Explanation of the error
- **status**: HTTP status code

## 400 Bad Request

### ApplicationNameMissing

```json
{
  "type": "https://hook0.com/documentation/errors/ApplicationNameMissing",
  "id": "ApplicationNameMissing",
  "title": "Application name cannot be empty",
  "detail": "Application name length must have more than 1 character.",
  "status": 400
}
```

### AuthInvalidAuthorizationHeader

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidAuthorizationHeader",
  "id": "AuthInvalidAuthorizationHeader",
  "title": "`Authorization` header is invalid",
  "detail": "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`.",
  "status": 400
}
```

### EventInvalidBase64Payload

```json
{
  "type": "https://hook0.com/documentation/errors/EventInvalidBase64Payload",
  "id": "EventInvalidBase64Payload",
  "title": "Invalid event base64 payload",
  "detail": "Event payload is not encoded in valid base64 format: ",
  "status": 400
}
```

### EventInvalidJsonPayload

```json
{
  "type": "https://hook0.com/documentation/errors/EventInvalidJsonPayload",
  "id": "EventInvalidJsonPayload",
  "title": "Invalid event JSON payload",
  "detail": "Event payload is not encoded in valid JSON format: .",
  "status": 400
}
```

### EventInvalidPayloadContentType

```json
{
  "type": "https://hook0.com/documentation/errors/EventInvalidPayloadContentType",
  "id": "EventInvalidPayloadContentType",
  "title": "Invalid event payload content type",
  "detail": "The specified event payload content type is not handled. Valid content types are: text/plain, application/json, application/octet-stream+base64",
  "status": 400
}
```

### EventTypeDoesNotExist

```json
{
  "type": "https://hook0.com/documentation/errors/EventTypeDoesNotExist",
  "id": "EventTypeDoesNotExist",
  "title": "Invalid event type",
  "detail": "Event type does not exist or was deactivated. You should (re)create it.",
  "status": 400
}
```

### InvalidRole

```json
{
  "type": "https://hook0.com/documentation/errors/InvalidRole",
  "id": "InvalidRole",
  "title": "Provided role does not exist",
  "detail": "Valid roles are: viewer, editor.",
  "status": 400
}
```

### JsonPayload

```json
{
  "type": "https://hook0.com/documentation/errors/JsonPayload",
  "id": "JsonPayload",
  "title": "Provided body could not be decoded as JSON",
  "detail": "",
  "status": 400
}
```

### LabelsAmbiguity

```json
{
  "type": "https://hook0.com/documentation/errors/LabelsAmbiguity",
  "id": "LabelsAmbiguity",
  "title": "Ambiguous labels specification",
  "detail": "You must specify either the `labels` property as an object with a least one property (recommended) or separated `label_key` and `label_value` properties as strings (legacy), but not both.",
  "status": 400
}
```

### OrganizationNameMissing

```json
{
  "type": "https://hook0.com/documentation/errors/OrganizationNameMissing",
  "id": "OrganizationNameMissing",
  "title": "Organization name cannot be empty",
  "detail": "Organization name length must have more than 1 character.",
  "status": 400
}
```

### PasswordTooShort

```json
{
  "type": "https://hook0.com/documentation/errors/PasswordTooShort",
  "id": "PasswordTooShort",
  "title": "Provided password is too short",
  "detail": "Password must be at least 0 characters long.",
  "status": 400
}
```

### UnauthorizedWorkers

```json
{
  "type": "https://hook0.com/documentation/errors/UnauthorizedWorkers",
  "id": "UnauthorizedWorkers",
  "title": "Some of the provided dedicated workers are not authorized for your organization",
  "detail": "You do not have access to the following workers: ",
  "status": 400
}
```

## 401 Unauthorized

### AuthEmailExpired

```json
{
  "type": "https://hook0.com/documentation/errors/AuthEmailExpired",
  "id": "AuthEmailExpired",
  "title": "Could not verify your link",
  "detail": "The link you clicked might be expired. Please retry the whole process or contact support.",
  "status": 401
}
```

### AuthEmailNotVerified

```json
{
  "type": "https://hook0.com/documentation/errors/AuthEmailNotVerified",
  "id": "AuthEmailNotVerified",
  "title": "Email not verified",
  "detail": "Your email has not been verified yet. Please check your inbox.",
  "status": 401
}
```

### AuthFailedLogin

```json
{
  "type": "https://hook0.com/documentation/errors/AuthFailedLogin",
  "id": "AuthFailedLogin",
  "title": "Login failed",
  "detail": "The provided credentials do not match ones of a valid user.",
  "status": 401
}
```

### AuthFailedRefresh

```json
{
  "type": "https://hook0.com/documentation/errors/AuthFailedRefresh",
  "id": "AuthFailedRefresh",
  "title": "Refreshing access token failed",
  "detail": "The provided refresh token is probably invalid or expired.",
  "status": 401
}
```

### AuthNoAuthorizationHeader

```json
{
  "type": "https://hook0.com/documentation/errors/AuthNoAuthorizationHeader",
  "id": "AuthNoAuthorizationHeader",
  "title": "No `Authorization` header was found in the HTTP request",
  "detail": "`Authorization` header must be provided and must contain a bearer token.",
  "status": 401
}
```

## 403 Forbidden

### AuthInvalidApplicationSecret

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidApplicationSecret",
  "id": "AuthInvalidApplicationSecret",
  "title": "Invalid application secret",
  "detail": "The provided application secret does not exist.",
  "status": 403
}
```

### AuthInvalidBiscuit

```json
{
  "type": "https://hook0.com/documentation/errors/AuthInvalidBiscuit",
  "id": "AuthInvalidBiscuit",
  "title": "Invalid Biscuit",
  "detail": "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired.",
  "status": 403
}
```

### Forbidden

```json
{
  "type": "https://hook0.com/documentation/errors/Forbidden",
  "id": "Forbidden",
  "title": "Insufficient rights",
  "detail": "You don't have the right to access or edit this resource.",
  "status": 403
}
```

## 404 Not Found

### InvitedUserDoesNotExist

```json
{
  "type": "https://hook0.com/documentation/errors/InvitedUserDoesNotExist",
  "id": "InvitedUserDoesNotExist",
  "title": "Invited user does not exist",
  "detail": "The user you are trying to invite does not exist. Please make sure the user is already register in Hook0.",
  "status": 404
}
```

### NotFound

```json
{
  "type": "https://hook0.com/documentation/errors/NotFound",
  "id": "NotFound",
  "title": "Item not found",
  "detail": "Could not find the item. Check the identifier or that you have the right to access it.",
  "status": 404
}
```

## 409 Conflict

### EventAlreadyIngested

```json
{
  "type": "https://hook0.com/documentation/errors/EventAlreadyIngested",
  "id": "EventAlreadyIngested",
  "title": "Event already Ingested",
  "detail": "This event was previously ingested and recorded inside Hook0 service.",
  "status": 409
}
```

### EventTypeAlreadyExist

```json
{
  "type": "https://hook0.com/documentation/errors/EventTypeAlreadyExist",
  "id": "EventTypeAlreadyExist",
  "title": "This event type already exist",
  "detail": "An event type with this name is already present.",
  "status": 409
}
```

### InvitedUserAlreadyInOrganization

```json
{
  "type": "https://hook0.com/documentation/errors/InvitedUserAlreadyInOrganization",
  "id": "InvitedUserAlreadyInOrganization",
  "title": "Invited user is already in the organization",
  "detail": "The user you are trying to invite has already access to the organization.",
  "status": 409
}
```

### OrganizationIsNotEmpty

```json
{
  "type": "https://hook0.com/documentation/errors/OrganizationIsNotEmpty",
  "id": "OrganizationIsNotEmpty",
  "title": "Organization is not empty",
  "detail": "Organizations that contain at least an application cannot be deleted; applications must be deleted first. If you believe this is a mistake, please contact the Hook0 support team.",
  "status": 409
}
```

### UserAlreadyExist

```json
{
  "type": "https://hook0.com/documentation/errors/UserAlreadyExist",
  "id": "UserAlreadyExist",
  "title": "This user already exist",
  "detail": "This email is already registered.",
  "status": 409
}
```

## 410 Gone

### RegistrationDisabled

```json
{
  "type": "https://hook0.com/documentation/errors/RegistrationDisabled",
  "id": "RegistrationDisabled",
  "title": "Registrations are disabled",
  "detail": "Registration was disabled by an administrator.",
  "status": 410
}
```

## 422 Unprocessable Entity

### Validation

```json
{
  "type": "https://hook0.com/documentation/errors/Validation",
  "id": "Validation",
  "title": "Provided input is malformed",
  "detail": "",
  "status": 422
}
```

## 429 Too Many Requests

### TooManyApplicationsPerOrganization

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyApplicationsPerOrganization",
  "id": "TooManyApplicationsPerOrganization",
  "title": "Exceeded number of applications that can be created in this organization",
  "detail": "This organization cannot have more than 0 applications. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyEventsToday

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyEventsToday",
  "id": "TooManyEventsToday",
  "title": "Exceeded number of events that can be ingested in this organization today",
  "detail": "This organization cannot ingest more than 0 events per day. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyEventTypesPerApplication

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyEventTypesPerApplication",
  "id": "TooManyEventTypesPerApplication",
  "title": "Exceeded number of event types that can be created in this application",
  "detail": "This application cannot have more than 0 event types. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyMembersPerOrganization

```json
{
  "type": "https://hook0.com/documentation/errors/TooManyMembersPerOrganization",
  "id": "TooManyMembersPerOrganization",
  "title": "Exceeded number of users that can be invited in this organization",
  "detail": "This organization cannot have more than 0 users. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManySubscriptionsPerApplication

```json
{
  "type": "https://hook0.com/documentation/errors/TooManySubscriptionsPerApplication",
  "id": "TooManySubscriptionsPerApplication",
  "title": "Exceeded number of subscriptions that can be created in this application",
  "detail": "This application cannot have more than 0 subscriptions. You might want to upgrade to a better plan.",
  "status": 429
}
```

## 500 Internal Server Error

### AuthApplicationSecretLookupError

```json
{
  "type": "https://hook0.com/documentation/errors/AuthApplicationSecretLookupError",
  "id": "AuthApplicationSecretLookupError",
  "title": "Could not check database to verify the provided application secret",
  "detail": "This is likely to be caused by database unavailability.",
  "status": 500
}
```

### AuthBiscuitLookupError

```json
{
  "type": "https://hook0.com/documentation/errors/AuthBiscuitLookupError",
  "id": "AuthBiscuitLookupError",
  "title": "Could not check database to verify if the provided Biscuit was revoked",
  "detail": "This is likely to be caused by database unavailability.",
  "status": 500
}
```

### InternalServerError

```json
{
  "type": "https://hook0.com/documentation/errors/InternalServerError",
  "id": "InternalServerError",
  "title": "Something wrong happened",
  "detail": "Hook0 server had issue handling your request. Our team was notified.",
  "status": 500
}
```

## Handling Errors

For implementation guidance on error handling in your client code, see [Client-side Error Handling Best Practices](/how-to-guides/client-error-handling).
