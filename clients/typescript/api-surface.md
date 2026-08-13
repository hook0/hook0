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
constructor(retryPolicy?: RetryPolicy, requestTimeoutMs?: number, maxPayloadBytes?: number): Hook0ClientOptions
maxPayloadBytes: number
requestTimeoutMs: number
retryPolicy: RetryPolicy
```

## const MAX_ATTEMPTS_CAP

```ts
MAX_ATTEMPTS_CAP: number
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

## function verifyWebhookSignature

```ts
verifyWebhookSignature(signature: string, payload: Buffer, headers: Headers, subscriptionSecret: string, tolerance: number): boolean | Hook0ClientError
```

## function verifyWebhookSignatureWithCurrentTime

```ts
verifyWebhookSignatureWithCurrentTime(signature: string, payload: Buffer, headers: Headers, subscriptionSecret: string, tolerance: number, currentTime: Date): boolean | Hook0ClientError
```
