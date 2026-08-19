# hook0-client — public API surface

Everything the published npm package exports from `src/index.ts`, with the signature of every
public member. This file is the contract consumers depend on.

Generated — do not edit by hand. Regenerate with `npm run api-surface:update`.

`tests/apiSurface.test.ts` fails when the code and this file disagree. Renaming, removing
or reshaping anything below breaks consumers and requires a major version bump; adding to it
requires a minor one.

## const DEFAULT_MAX_PAYLOAD_BYTES

```ts
DEFAULT_MAX_PAYLOAD_BYTES: number
```

## const DEFAULT_MAX_RESPONSE_BYTES

```ts
DEFAULT_MAX_RESPONSE_BYTES: number
```

## const DEFAULT_REQUEST_TIMEOUT_MS

```ts
DEFAULT_REQUEST_TIMEOUT_MS: number
```

## class Event

```ts
constructor(eventType: string, payload: string, payloadContentType: string, labels: Record<string, string>, metadata?: Record<string, string>, occurredAt?: Date, eventId?: string): Event
eventId?: string
eventType: string
labels: Record<string, string>
metadata?: Record<string, string>
occurredAt?: Date
payload: string
payloadContentType: string
```

## class EventType

```ts
constructor(service: string, resourceType: string, verb: string): EventType
static fromString(s: string): EventType | Hook0ClientError
resourceType: string
service: string
verb: string
```

## class Hook0Client

```ts
constructor(apiUrl: string, applicationId: string, token: string, debug?: boolean, options?: Hook0ClientOptions): Hook0Client
sendEvent(event: Event): Promise<string>
upsertEventTypes(eventTypes: string[]): Promise<string[]>
```

## class Hook0ClientError extends Error

```ts
constructor(message?: string): Hook0ClientError
static EventSending(eventId: string | undefined, error: Error): Hook0ClientError
static ExpiredWebhook(signed_at: Date, tolerance: number, current_time: Date): Hook0ClientError
static GetAvailableEventTypes(error: Error): Hook0ClientError
static InvalidEventType(s: string): Hook0ClientError
static InvalidSignature(signature: string): Hook0ClientError
static MissingHeader(headerName: string): Hook0ClientError
static PayloadTooLarge(eventId: string, size: number, maximum: number): Hook0ClientError
static RetriesExhausted(eventId: string, attempts: number, waitedMs: number, lastFailure: string): Hook0ClientError
static SignatureParsing(signature: string): Hook0ClientError
static TimestampParsingInSignature(timestamp: string): Hook0ClientError
```

## class Hook0ClientOptions

```ts
constructor(retryPolicy?: RetryPolicy, requestTimeoutMs?: number, maxPayloadBytes?: number, maxResponseBytes?: number): Hook0ClientOptions
maxPayloadBytes: number
maxResponseBytes: number
requestTimeoutMs: number
retryPolicy: RetryPolicy
```

## const MAX_ATTEMPTS_CAP

```ts
MAX_ATTEMPTS_CAP: number
```

## const MAX_HEADER_BYTES

```ts
MAX_HEADER_BYTES: number
```

## const MAX_HEAD_BYTES

```ts
MAX_HEAD_BYTES: number
```

## const MAX_RESPONSE_HEADERS

```ts
MAX_RESPONSE_HEADERS: number
```

## class RetryPolicy

```ts
constructor(maxAttempts?: number, initialBackoffMs?: number, maxBackoffMs?: number, maxTotalDelayMs?: number): RetryPolicy
static disabled(): RetryPolicy
attempts(): number
backoffCeilingMs(retry: number): number
delaysMs(draws: number[]): number[]
initialBackoffMs: number
maxAttempts: number
maxBackoffMs: number
maxTotalDelayMs: number
```

## class Signature

```ts
constructor(timestamp: number, v0: Buffer | null, h: string[], v1: Buffer | null): Signature
static PAYLOAD_SEPARATOR: string
static PAYLOAD_SEPARATOR_BYTES: Buffer<ArrayBuffer>
static SIGNATURE_PART_ASSIGNATOR: string
static SIGNATURE_PART_HEADER_NAMES_SEPARATOR: string
static SIGNATURE_PART_SEPARATOR: string
static parse(signature: string): Signature
h: string[]
timestamp: number
v0: Buffer<ArrayBufferLike>
v1: Buffer<ArrayBufferLike>
verify(payload: Buffer, headers: Headers, secret: string): boolean
```

## interface generated.Application

```ts
application_id: string
name: string
organization_id: string
```

## interface generated.ApplicationInfo

```ts
application_id: string
consumption: ApplicationInfoConsumption
name: string
onboarding_steps: ApplicationInfoOnboardingSteps
organization_id: string
quotas: ApplicationInfoQuotas
```

## interface generated.ApplicationInfoConsumption

```ts
events_per_day?: number
```

## interface generated.ApplicationInfoOnboardingSteps

```ts
event: ApplicationInfoOnboardingStepsEvent
event_type: ApplicationInfoOnboardingStepsEventType
subscription: ApplicationInfoOnboardingStepsSubscription
```

## type generated.ApplicationInfoOnboardingStepsEvent

```ts
"ToDo" | "Done"
```

## type generated.ApplicationInfoOnboardingStepsEventType

```ts
"ToDo" | "Done"
```

## type generated.ApplicationInfoOnboardingStepsSubscription

```ts
"ToDo" | "Done"
```

## interface generated.ApplicationInfoQuotas

```ts
days_of_events_retention_limit: number
events_per_day_limit: number
```

## interface generated.ApplicationPost

```ts
name: string
organization_id: string
```

## interface generated.ApplicationSecret

```ts
created_at: string
deleted_at?: string
name?: string
token: string
```

## interface generated.ApplicationSecretPost

```ts
application_id: string
name?: string
```

## class generated.ApplicationSecretsApi

```ts
constructor(transport: Transport): ApplicationSecretsApi
create(body: ApplicationSecretPost): Promise<ApplicationSecret>
delete(applicationSecretToken: string, applicationId: string): Promise<void>
read(applicationId: string): Promise<ApplicationSecret[]>
update(applicationSecretToken: string, body: ApplicationSecretPost): Promise<ApplicationSecret>
```

## class generated.ApplicationsApi

```ts
constructor(transport: Transport): ApplicationsApi
create(body: ApplicationPost): Promise<Application>
delete(applicationId: string): Promise<void>
get(applicationId: string): Promise<ApplicationInfo>
list(organizationId: string): Promise<Application[]>
update(applicationId: string, body: ApplicationPost): Promise<Application>
```

## class generated.ErrorsApi

```ts
constructor(transport: Transport): ErrorsApi
list(): Promise<Problem[]>
```

## interface generated.Event

```ts
event_id: string
event_type_name: string
ip: string
labels: unknown
metadata?: unknown
occurred_at: string
payload_content_type: string
received_at: string
```

## interface generated.EventPost

```ts
application_id: string
event_id?: string
event_type: string
labels: Record<string, string>
metadata?: Record<string, string>
occurred_at: string
payload: string
payload_content_type: string
```

## interface generated.EventType

```ts
event_type_name: string
resource_type_name: string
service_name: string
verb_name: string
```

## interface generated.EventTypePost

```ts
application_id: string
resource_type: string
service: string
verb: string
```

## class generated.EventTypesApi

```ts
constructor(transport: Transport): EventTypesApi
create(body: EventTypePost): Promise<EventType>
delete(eventTypeName: string, applicationId: string): Promise<void>
get(eventTypeName: string, applicationId: string): Promise<EventType>
list(applicationId: string): Promise<EventType[]>
```

## interface generated.EventWithPayload

```ts
event_id: string
event_type_name: string
ip: string
labels: unknown
metadata?: unknown
occurred_at: string
payload: string
payload_content_type: string
received_at: string
```

## class generated.EventsApi

```ts
constructor(transport: Transport): EventsApi
get(eventId: string, applicationId: string): Promise<EventWithPayload>
ingest(body: EventPost): Promise<IngestedEvent>
list(applicationId: string): Promise<Event[]>
replay(eventId: string, body: ReplayEvent): Promise<void>
```

## class generated.EventsPerDayApi

```ts
constructor(transport: Transport): EventsPerDayApi
listForApplication(applicationId: string, from?: string, to?: string): Promise<EventsPerDayEntry[]>
listForOrganization(organizationId: string, from?: string, to?: string): Promise<EventsPerDayEntry[]>
```

## interface generated.EventsPerDayEntry

```ts
amount: number
application_id: string
application_name: string
date: string
is_provisional: boolean
```

## interface generated.IngestedEvent

```ts
application_id: string
event_id: string
received_at: string
```

## class generated.InstanceApi

```ts
constructor(transport: Transport): InstanceApi
get(): Promise<InstanceConfig>
```

## interface generated.InstanceConfig

```ts
application_secret_compatibility: boolean
auto_db_migration: boolean
biscuit_public_key: string
cloudflare_turnstile_site_key?: string
formbricks?: InstanceConfigFormbricks
matomo?: InstanceConfigMatomo
password_minimum_length: number
quota_enforcement: boolean
registration_disabled: boolean
support_email_address: string
```

## interface generated.InstanceConfigFormbricks

```ts
api_host: string
environment_id: string
```

## interface generated.InstanceConfigMatomo

```ts
site_id: number
url: string
```

## interface generated.Organization

```ts
name: string
organization_id: string
plan?: OrganizationPlan
role: string
```

## interface generated.OrganizationInfo

```ts
consumption: OrganizationInfoConsumption
name: string
onboarding_steps: OrganizationInfoOnboardingSteps
organization_id: string
plan?: OrganizationInfoPlan
quotas: OrganizationInfoQuotas
users: OrganizationInfoUsers[]
```

## interface generated.OrganizationInfoConsumption

```ts
applications?: number
events_per_day?: number
members?: number
```

## interface generated.OrganizationInfoOnboardingSteps

```ts
application: OrganizationInfoOnboardingStepsApplication
event: OrganizationInfoOnboardingStepsEvent
event_type: OrganizationInfoOnboardingStepsEventType
subscription: OrganizationInfoOnboardingStepsSubscription
```

## type generated.OrganizationInfoOnboardingStepsApplication

```ts
"ToDo" | "Done"
```

## type generated.OrganizationInfoOnboardingStepsEvent

```ts
"ToDo" | "Done"
```

## type generated.OrganizationInfoOnboardingStepsEventType

```ts
"ToDo" | "Done"
```

## type generated.OrganizationInfoOnboardingStepsSubscription

```ts
"ToDo" | "Done"
```

## interface generated.OrganizationInfoPlan

```ts
label: string
name: string
```

## interface generated.OrganizationInfoQuotas

```ts
applications_per_organization_limit: number
days_of_events_retention_limit: number
events_per_day_limit: number
members_per_organization_limit: number
```

## interface generated.OrganizationInfoUsers

```ts
email: string
first_name: string
last_name: string
role: string
user_id: string
```

## interface generated.OrganizationPlan

```ts
label: string
name: string
```

## class generated.PayloadContentTypesApi

```ts
constructor(transport: Transport): PayloadContentTypesApi
list(): Promise<string[]>
```

## interface generated.Problem

```ts
detail: string
id: ProblemId
status: number
title: string
type: string
validation?: unknown
```

## class generated.ProblemError extends Error

```ts
constructor(status: number, problem: Problem | undefined, detail: string): ProblemError
kind?: ProblemId
problem?: Problem
status: number
```

## type generated.ProblemId

```ts
"OrganizationNameMissing" | "UserAlreadyExist" | "RegistrationDisabled" | "PasswordTooShort" | "PasswordTooLong" | "PasswordSimilarToEmail" | "PasswordSimilarToName" | "PasswordTooCommon" | "PasswordNotDiverseEnough" | "OrganizationIsNotEmpty" | "InvitedUserDoesNotExist" | "InvitedUserAlreadyInOrganization" | "ApplicationNameMissing" | "InvalidRole" | "EventTypeAlreadyExist" | "EventTypeDoesNotExist" | "UnauthorizedWorkers" | "EventAlreadyIngested" | "EventInvalidPayloadContentType" | "EventInvalidBase64Payload" | "EventInvalidJsonPayload" | "LabelsAmbiguity" | "InvalidDateRange" | "AuthNoAuthorizationHeader" | "AuthInvalidAuthorizationHeader" | "AuthApplicationSecretLookupError" | "AuthInvalidApplicationSecret" | "AuthBiscuitLookupError" | "AuthInvalidBiscuit" | "AuthFailedLogin" | "AuthEmailNotVerified" | "AuthEmailAlreadyVerified" | "AuthFailedRefresh" | "AuthEmailExpired" | "TooManyMembersPerOrganization" | "TooManyApplicationsPerOrganization" | "TooManyEventsToday" | "TooManySubscriptionsPerApplication" | "TooManyEventTypesPerApplication" | "JsonPayload" | "Validation" | "NotFound" | "InternalServerError" | "Forbidden" | "RateLimited" | "ServiceUnavailable"
```

## class generated.QuotasApi

```ts
constructor(transport: Transport): QuotasApi
get(): Promise<QuotasResponse>
```

## interface generated.QuotasResponse

```ts
enabled: boolean
limits: QuotasResponseLimits
```

## interface generated.QuotasResponseLimits

```ts
global_applications_per_organization_limit: number
global_days_of_events_retention_limit: number
global_event_types_per_application_limit: number
global_events_per_day_limit: number
global_members_per_organization_limit: number
global_subscriptions_per_application_limit: number
```

## interface generated.ReplayEvent

```ts
application_id: string
```

## interface generated.RequestAttempt

```ts
created_at: string
delay_until?: string
event: RequestAttemptEvent
event_id: string
failed_at?: string
http_response_status?: number
picked_at?: string
request_attempt_id: string
response_id?: string
retry_count: number
status: RequestAttemptStatus
subscription: RequestAttemptSubscription
succeeded_at?: string
```

## interface generated.RequestAttemptEvent

```ts
event_id: string
event_type_name: string
```

## interface generated.RequestAttemptStatus

```ts
at?: string
full_processing_ms?: number
since?: string
type: RequestAttemptStatusType
until?: string
```

## type generated.RequestAttemptStatusType

```ts
"waiting" | "pending" | "in_progress" | "successful" | "failed"
```

## interface generated.RequestAttemptSubscription

```ts
description?: string
subscription_id: string
```

## class generated.RequestAttemptsApi

```ts
constructor(transport: Transport): RequestAttemptsApi
get(requestAttemptId: string, applicationId: string): Promise<RequestAttempt>
read(applicationId: string, eventEventTypeNames?: string, eventId?: string, maxCreatedAt?: string, minCreatedAt?: string, paginationCursor?: string, subscriptionId?: string): Promise<RequestAttempt[]>
```

## interface generated.Response

```ts
body?: string
elapsed_time_ms?: number
headers?: Record<string, string>
http_code?: number
response_error_name?: string
response_id: string
```

## class generated.ResponseApi

```ts
constructor(transport: Transport): ResponseApi
get(responseId: string, applicationId: string): Promise<Response>
```

## interface generated.ServiceToken

```ts
biscuit: string
created_at: string
name: string
token_id: string
```

## class generated.ServiceTokenApi

```ts
constructor(transport: Transport): ServiceTokenApi
create(body: ServiceTokenPost): Promise<ServiceToken>
delete(serviceTokenId: string, organizationId: string): Promise<void>
edit(serviceTokenId: string, body: ServiceTokenPost): Promise<ServiceToken>
get(serviceTokenId: string, organizationId: string): Promise<ServiceToken>
list(organizationId: string): Promise<ServiceToken[]>
```

## interface generated.ServiceTokenPost

```ts
name: string
organization_id: string
```

## interface generated.Subscription

```ts
application_id: string
created_at: string
dedicated_workers: string[]
description?: string
event_types: string[]
is_enabled: boolean
label_key: string
label_value: string
labels: Record<string, string>
metadata: Record<string, string>
secret: string
subscription_id: string
target: SubscriptionTarget
updated_at: string
```

## interface generated.SubscriptionPost

```ts
application_id: string
dedicated_workers?: string[]
description?: string
event_types: string[]
is_enabled: boolean
label_key?: string
label_value?: string
labels?: Record<string, string>
metadata?: Record<string, string>
target: SubscriptionPostTarget
```

## interface generated.SubscriptionPostTarget

```ts
headers: unknown
method: string
type: string
url: string
```

## interface generated.SubscriptionTarget

```ts
headers: unknown
method: string
type: string
url: string
```

## class generated.SubscriptionsApi

```ts
constructor(transport: Transport): SubscriptionsApi
create(body: SubscriptionPost): Promise<Subscription>
delete(subscriptionId: string, applicationId: string): Promise<void>
get(subscriptionId: string): Promise<Subscription>
list(applicationId: string): Promise<Subscription[]>
update(subscriptionId: string, body: SubscriptionPost): Promise<Subscription>
```

## interface generated.Transport

```ts
request(request: TransportRequest): Promise<TransportResponse>
```

## interface generated.TransportRequest

```ts
body?: string
method: string
path: string
query: readonly [string, string][]
```

## interface generated.TransportResponse

```ts
payload: string
status: number
```

## function generated.raiseForStatus

```ts
raiseForStatus(status: number, payload: string): void
```

## function generated.readPayload

```ts
readPayload<T>(status: number, payload: string): T
```

## function verifyWebhookSignature

```ts
verifyWebhookSignature(signature: string, payload: Buffer, headers: Headers, subscriptionSecret: string, tolerance: number): true
```

## function verifyWebhookSignatureWithCurrentTime

```ts
verifyWebhookSignatureWithCurrentTime(signature: string, payload: Buffer, headers: Headers, subscriptionSecret: string, tolerance: number, currentTime: Date): true
```
