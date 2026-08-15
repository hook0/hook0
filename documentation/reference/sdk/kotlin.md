---
title: "Kotlin webhook SDK — com.hook0:hook0-client-kotlin"
description: "Send Hook0 events and verify webhook signatures from Kotlin 2.4 on Java 21. Blocking and suspending calls, kotlin-stdlib only. Not yet on Maven Central."
keywords: [Kotlin webhook SDK, Hook0 Kotlin client, suspend function webhook, verify webhook signature Kotlin, Ktor webhook endpoint, send webhook event Kotlin]
---

# Kotlin SDK

The Hook0 SDK for Kotlin sends events and verifies webhook signatures. Every call that reaches the network comes in two forms: one that blocks, and one that suspends.

`kotlin-stdlib` is the only dependency it declares. The suspending half is written against `kotlin.coroutines`, the language's own, so it suspends inside whichever coroutine runtime you brought, whether that is `kotlinx.coroutines`, Ktor's, Spring's, or none.

## Installation

Kotlin 2.4 and Java 21 or later are required.

:::warning Not published to Maven Central yet
`com.hook0:hook0-client-kotlin` does not resolve from Maven Central. No release job publishes it, and the `com.hook0` namespace has not been claimed on the Central Portal.

Until then, build the jar from a checkout and install it into your local repository.
:::

```bash
git clone https://gitlab.com/hook0/hook0.git
mvn -f hook0/clients/kotlin/pom.xml install
```

That puts `com.hook0:hook0-client-kotlin:1.1.0` in `~/.m2`, where your own build resolves it:

```xml
<dependency>
  <groupId>com.hook0</groupId>
  <artifactId>hook0-client-kotlin</artifactId>
  <version>1.1.0</version>
</dependency>
```

For Gradle, the same coordinates: `implementation("com.hook0:hook0-client-kotlin:1.1.0")`, with `mavenLocal()` in your repositories.

## Send an event

```kotlin
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client
import java.util.UUID

Hook0Client("https://app.hook0.com/api/v1", applicationId, token).use { client ->
  val sent: UUID = client.sendEvent(
    Event(
      eventType = "billing.invoice.paid",
      payload = """{"invoice":"in_123"}""",
      payloadContentType = "application/json",
      labels = mapOf("environment" to "production")
    )
  )
}
```

The same send, suspending:

```kotlin
val sent: UUID = client.sendEventSuspending(event)
```

`Event` is a data class with three required parameters and four that default:

```kotlin
Event(
  eventType = "billing.invoice.paid",
  payload = """{"invoice":"in_123"}""",
  payloadContentType = "application/json",
  labels = mapOf("environment" to "production"),
  metadata = mapOf("emitter" to "billing-worker"),
  occurredAt = OffsetDateTime.now(ZoneOffset.UTC),
  eventId = null
)
```

`Hook0Client` implements `AutoCloseable`. Hold one for the life of the application rather than one per send.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set on `eventId`, or a UUIDv7 it mints when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `sendEvent` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and throws.

## Bounds, and how to change them

```kotlin
import com.hook0.kotlin.Options
import com.hook0.kotlin.RetryPolicy
import java.time.Duration

val options = Options.defaults().copy(
  retryPolicy = RetryPolicy(
    maxAttempts = 4,
    initialBackoff = Duration.ofMillis(100),
    maxBackoff = Duration.ofMillis(2000),
    maxTotalDelay = Duration.ofMillis(5000)
  ),
  requestTimeout = Duration.ofSeconds(10),
  maxPayloadBytes = 1024 * 1024,
  maxResponseBytes = 8L * 1024 * 1024
)

val client = Hook0Client("https://app.hook0.com/api/v1", applicationId, token, options)
```

Those are the defaults, and `Options.defaults()` and `RetryPolicy.defaults()` return them.

| Bound | Default |
|-------|---------|
| `maxAttempts` (the first attempt included) | `4`, capped at `RetryPolicy.MAX_ATTEMPTS_CAP` = 16 |
| `initialBackoff` | 100 ms |
| `maxBackoff` | 2 s |
| `maxTotalDelay`, the budget all delays of one send share | 5 s |
| `requestTimeout`, per attempt | 10 s |
| `maxPayloadBytes` | 1 MiB |
| `maxResponseBytes` | 8 MiB |

`RetryPolicy.disabled()` sends each event exactly once. A payload above the maximum throws before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```kotlin
import com.hook0.kotlin.Webhooks
import java.time.Duration

Webhooks.verify(
  request.getHeader("X-Hook0-Signature"),
  rawBody,
  headersAsTheyArrived,
  subscriptionSecret,
  Duration.ofMinutes(5)
)
```

`verify` returns for the one reason a delivery is accepted and throws `ClientException` for every reason it is refused. There is no default tolerance; pass one.

`rawBody` is the body as it arrived, as a `String`. A body that has been parsed and re-serialised no longer hashes to what was signed. Headers arrive either as a `Map<String, String>` or as a `List<Pair<String, String>>`, since a request may carry the same header twice.

The clock window is bilateral: a moment too far in the future is refused exactly like one too far in the past, since a window that only looked backwards is one a sender widens by dating its own delivery ahead. A header the signature covers but the request did not carry is refused before any code is computed.

`Webhooks.verifyAt` takes the same arguments followed by an `Instant`, for holding a delivery against a moment you name.

### Ktor

```kotlin
routing {
  post("/webhook") {
    val rawBody = call.receiveText()
    val delivered = call.request.headers.entries()
      .flatMap { header -> header.value.map { header.key to it } }

    try {
      Webhooks.verify(
        call.request.headers["X-Hook0-Signature"].orEmpty(),
        rawBody,
        delivered,
        subscriptionSecret,
        Duration.ofMinutes(5)
      )
    } catch (refused: ClientException) {
      call.respond(HttpStatusCode.BadRequest)
      return@post
    }

    handleDelivery(rawBody)
    call.respond(HttpStatusCode.OK)
  }
}
```

`receiveText()` keeps the bytes that arrived. Receiving into a data class does not.

## Upsert event types

An event whose type the application does not declare is refused. Only the missing ones are created, and those are what comes back:

```kotlin
val created: List<String> = client.upsertEventTypes(
  listOf("billing.invoice.paid", "billing.invoice.voided")
)
```

`upsertEventTypesSuspending` is the same call, from a coroutine. An event type is written `service.resource_type.verb`; `EventType.parse` reads one and throws `ClientException` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is generated, grouped by entity, in both flavours:

```kotlin
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ApplicationsSuspendingApi

val application = ApplicationsApi(client.transport).get(applicationId)

val suspended = ApplicationsSuspendingApi(client.transport).get(applicationId)
```

Schemas are data classes, closed lists of strings are enum classes, and each problem the API can report is an exception of its own under a `sealed class ProblemException`. A member the API does not declare required is a nullable type; everything else is not, so nothing in this SDK is ever read through `!!`.

## Errors

| Type | Thrown when |
|------|-------------|
| `ClientException` | A send failed, retries ran out, a payload was too large, an event type was invalid or could not be created, or a delivery was refused |
| `TransportException` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `causeName` and `retryable` |
| `JsonException` | A document could not be read as JSON |
| `DecodeException` | A response body could not be read as the shape it declared |
| `generated.ProblemException` and its subclasses | The API reported a problem |

All of them extend `Hook0Exception`, which extends `RuntimeException`.

## Links

- **Source**: [clients/kotlin](https://gitlab.com/hook0/hook0/-/tree/master/clients/kotlin)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
