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

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#applicationnamemissing",
  "id": "ApplicationNameMissing",
  "title": "Application name cannot be empty",
  "detail": "Application name length must have more than 1 character.",
  "status": 400
}
```

### AuthInvalidAuthorizationHeader

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authinvalidauthorizationheader",
  "id": "AuthInvalidAuthorizationHeader",
  "title": "`Authorization` header is invalid",
  "detail": "`Authorization` header value could not be decoded as a valid UTF-8 string containing `Bearer {UUID}`.",
  "status": 400
}
```

### EventInvalidBase64Payload

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventinvalidbase64payload",
  "id": "EventInvalidBase64Payload",
  "title": "Invalid event base64 payload",
  "detail": "Event payload is not encoded in valid base64 format: ",
  "status": 400
}
```

### EventInvalidJsonPayload

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventinvalidjsonpayload",
  "id": "EventInvalidJsonPayload",
  "title": "Invalid event JSON payload",
  "detail": "Event payload is not encoded in valid JSON format: .",
  "status": 400
}
```

### EventInvalidPayloadContentType

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventinvalidpayloadcontenttype",
  "id": "EventInvalidPayloadContentType",
  "title": "Invalid event payload content type",
  "detail": "The specified event payload content type is not handled. Valid content types are: text/plain, application/json, application/octet-stream+base64",
  "status": 400
}
```

### EventTypeDoesNotExist

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventtypedoesnotexist",
  "id": "EventTypeDoesNotExist",
  "title": "Invalid event type",
  "detail": "Event type does not exist or was deactivated. You should (re)create it.",
  "status": 400
}
```

### InvalidDateRange

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#invaliddaterange",
  "id": "InvalidDateRange",
  "title": "Invalid date range",
  "detail": "'from' date must not be after 'to' date.",
  "status": 400
}
```

### InvalidRole

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#invalidrole",
  "id": "InvalidRole",
  "title": "Provided role does not exist",
  "detail": "Valid roles are: viewer, editor.",
  "status": 400
}
```

### JsonPayload

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#jsonpayload",
  "id": "JsonPayload",
  "title": "Provided body could not be decoded as JSON",
  "detail": "Unknown JSON payload error",
  "status": 400
}
```

### LabelsAmbiguity

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#labelsambiguity",
  "id": "LabelsAmbiguity",
  "title": "Ambiguous labels specification",
  "detail": "You must specify either the `labels` property as an object with a least one property (recommended) or separated `label_key` and `label_value` properties as strings (legacy), but not both.",
  "status": 400
}
```

### OrganizationNameMissing

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#organizationnamemissing",
  "id": "OrganizationNameMissing",
  "title": "Organization name cannot be empty",
  "detail": "Organization name length must have more than 1 character.",
  "status": 400
}
```

### PasswordNotDiverseEnough

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordnotdiverseenough",
  "id": "PasswordNotDiverseEnough",
  "title": "Provided password is not diverse enough",
  "detail": "Password is made of too few different characters, which makes it easy to guess despite its length. Please pick another one.",
  "status": 400
}
```

### PasswordSimilarToEmail

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordsimilartoemail",
  "id": "PasswordSimilarToEmail",
  "title": "Provided password is too close to the email address",
  "detail": "Password must not be built from the email address of the account: anyone who knows the address would guess it. Please pick something unrelated.",
  "status": 400
}
```

### PasswordSimilarToName

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordsimilartoname",
  "id": "PasswordSimilarToName",
  "title": "Provided password is too close to the user name",
  "detail": "Password must not be built from the first or last name of the account. Please pick something unrelated.",
  "status": 400
}
```

### PasswordTooCommon

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordtoocommon",
  "id": "PasswordTooCommon",
  "title": "Provided password is too common",
  "detail": "This password (or a lightly disguised version of it) is among the most frequently used ones, so it is one of the first an attacker tries. Please pick another one.",
  "status": 400
}
```

### PasswordTooLong

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordtoolong",
  "id": "PasswordTooLong",
  "title": "Provided password is too long",
  "detail": "Password must be at most 100 characters long.",
  "status": 400
}
```

### PasswordTooShort

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#passwordtooshort",
  "id": "PasswordTooShort",
  "title": "Provided password is too short",
  "detail": "Password must be at least 0 characters long.",
  "status": 400
}
```

### UnauthorizedWorkers

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#unauthorizedworkers",
  "id": "UnauthorizedWorkers",
  "title": "Some of the provided dedicated workers are not authorized for your organization",
  "detail": "You do not have access to the following workers: ",
  "status": 400
}
```

## 401 Unauthorized

### AuthEmailExpired

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authemailexpired",
  "id": "AuthEmailExpired",
  "title": "Could not verify your link",
  "detail": "This link no longer works. Links expire after a while and can only be used once. Asking for a new link retires the previous one straight away, and so does changing the password. If you asked more than once, the most recent email is the one that still works. Otherwise you can start again from the beginning, or contact support.",
  "status": 401
}
```

### AuthEmailNotVerified

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authemailnotverified",
  "id": "AuthEmailNotVerified",
  "title": "Email not verified",
  "detail": "Your email has not been verified yet. Please check your inbox.",
  "status": 401
}
```

### AuthFailedLogin

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authfailedlogin",
  "id": "AuthFailedLogin",
  "title": "Login failed",
  "detail": "The provided credentials do not match ones of a valid user.",
  "status": 401
}
```

### AuthFailedRefresh

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authfailedrefresh",
  "id": "AuthFailedRefresh",
  "title": "Refreshing access token failed",
  "detail": "The provided refresh token is probably invalid or expired.",
  "status": 401
}
```

### AuthNoAuthorizationHeader

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authnoauthorizationheader",
  "id": "AuthNoAuthorizationHeader",
  "title": "No `Authorization` header was found in the HTTP request",
  "detail": "`Authorization` header must be provided and must contain a bearer token.",
  "status": 401
}
```

## 403 Forbidden

### AuthInvalidApplicationSecret

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authinvalidapplicationsecret",
  "id": "AuthInvalidApplicationSecret",
  "title": "Invalid application secret",
  "detail": "The provided application secret does not exist.",
  "status": 403
}
```

### AuthInvalidBiscuit

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authinvalidbiscuit",
  "id": "AuthInvalidBiscuit",
  "title": "Invalid Biscuit",
  "detail": "The provided authentication token (Biscuit) is not valid, was not created using the current private key or is expired.",
  "status": 403
}
```

### Forbidden

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#forbidden",
  "id": "Forbidden",
  "title": "Insufficient rights",
  "detail": "You don't have the right to access or edit this resource.",
  "status": 403
}
```

## 404 Not Found

### InvitedUserDoesNotExist

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#inviteduserdoesnotexist",
  "id": "InvitedUserDoesNotExist",
  "title": "Invited user does not exist",
  "detail": "The user you are trying to invite does not exist. Please make sure the user is already register in Hook0.",
  "status": 404
}
```

### NotFound

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#notfound",
  "id": "NotFound",
  "title": "Item not found",
  "detail": "Could not find the item. Check the identifier or that you have the right to access it.",
  "status": 404
}
```

## 409 Conflict

### AuthEmailAlreadyVerified

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authemailalreadyverified",
  "id": "AuthEmailAlreadyVerified",
  "title": "Email already verified",
  "detail": "This address has already been verified, so this link has nothing left to do. Sign in to continue.",
  "status": 409
}
```

### EventAlreadyIngested

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventalreadyingested",
  "id": "EventAlreadyIngested",
  "title": "Event already Ingested",
  "detail": "This event was previously ingested and recorded inside Hook0 service.",
  "status": 409
}
```

### EventTypeAlreadyExist

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#eventtypealreadyexist",
  "id": "EventTypeAlreadyExist",
  "title": "This event type already exist",
  "detail": "An event type with this name is already present.",
  "status": 409
}
```

### InvitedUserAlreadyInOrganization

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#inviteduseralreadyinorganization",
  "id": "InvitedUserAlreadyInOrganization",
  "title": "Invited user is already in the organization",
  "detail": "The user you are trying to invite has already access to the organization.",
  "status": 409
}
```

### OrganizationIsNotEmpty

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#organizationisnotempty",
  "id": "OrganizationIsNotEmpty",
  "title": "Organization is not empty",
  "detail": "Organizations that contain at least an application cannot be deleted; applications must be deleted first. If you believe this is a mistake, please contact the Hook0 support team.",
  "status": 409
}
```

### UserAlreadyExist

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#useralreadyexist",
  "id": "UserAlreadyExist",
  "title": "This user already exist",
  "detail": "This email is already registered.",
  "status": 409
}
```

## 410 Gone

### RegistrationDisabled

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#registrationdisabled",
  "id": "RegistrationDisabled",
  "title": "Registrations are disabled",
  "detail": "Registration was disabled by an administrator.",
  "status": 410
}
```

## 422 Unprocessable Entity

### Validation

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#validation",
  "id": "Validation",
  "title": "Provided input is malformed",
  "detail": "Provided input did not pass validation.",
  "status": 422
}
```

## 429 Too Many Requests

### RateLimited

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#ratelimited",
  "id": "RateLimited",
  "title": "Too many requests",
  "detail": "Requests are coming in faster than this Hook0 instance accepts them, so this one was not processed. This is a temporary, client-side pacing condition, not a rights issue: the request is safe to send again once the delay given by the `Retry-After` response header has elapsed.",
  "status": 429
}
```

### TooManyApplicationsPerOrganization

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#toomanyapplicationsperorganization",
  "id": "TooManyApplicationsPerOrganization",
  "title": "Exceeded number of applications that can be created in this organization",
  "detail": "This organization cannot have more than 0 applications. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyEventsToday

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#toomanyeventstoday",
  "id": "TooManyEventsToday",
  "title": "Exceeded number of events that can be ingested in this organization today",
  "detail": "This organization cannot ingest more than 0 events per day. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyEventTypesPerApplication

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#toomanyeventtypesperapplication",
  "id": "TooManyEventTypesPerApplication",
  "title": "Exceeded number of event types that can be created in this application",
  "detail": "This application cannot have more than 0 event types. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManyMembersPerOrganization

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#toomanymembersperorganization",
  "id": "TooManyMembersPerOrganization",
  "title": "Exceeded number of users that can be invited in this organization",
  "detail": "This organization cannot have more than 0 users. You might want to upgrade to a better plan.",
  "status": 429
}
```

### TooManySubscriptionsPerApplication

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#toomanysubscriptionsperapplication",
  "id": "TooManySubscriptionsPerApplication",
  "title": "Exceeded number of subscriptions that can be created in this application",
  "detail": "This application cannot have more than 0 subscriptions. You might want to upgrade to a better plan.",
  "status": 429
}
```

## 500 Internal Server Error

### AuthApplicationSecretLookupError

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authapplicationsecretlookuperror",
  "id": "AuthApplicationSecretLookupError",
  "title": "Could not check database to verify the provided application secret",
  "detail": "This is likely to be caused by database unavailability.",
  "status": 500
}
```

### AuthBiscuitLookupError

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#authbiscuitlookuperror",
  "id": "AuthBiscuitLookupError",
  "title": "Could not check database to verify if the provided Biscuit was revoked",
  "detail": "This is likely to be caused by database unavailability.",
  "status": 500
}
```

### InternalServerError

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#internalservererror",
  "id": "InternalServerError",
  "title": "Something wrong happened",
  "detail": "Hook0 server had issue handling your request. Our team was notified.",
  "status": 500
}
```

## 503 Service Unavailable

### ServiceUnavailable

<!-- openapi: Problem -->
```json
{
  "type": "https://documentation.hook0.com/reference/error-codes#serviceunavailable",
  "id": "ServiceUnavailable",
  "title": "Service temporarily unavailable",
  "detail": "Hook0 is under heavy load and could not authorize your request in time. This is a temporary, server-side condition, not a rights issue: the request is safe to retry. Wait a moment and resubmit, honoring the Retry-After response header.",
  "status": 503
}
```

## Handling Errors

For implementation guidance on error handling in your client code, see [Client-side Error Handling Best Practices](/how-to-guides/client-error-handling).
