# Hook0 Kotlin SDK

Send events to [Hook0](https://www.hook0.com/), upsert the event types your application uses, verify the signature of an
incoming webhook, and call every operation the API declares through generated, documented types. Sending is idempotent
and retried under bounds you set.

Every call comes in two flavours: one that blocks, and one that suspends.

## Installing

`com.hook0:hook0-client-kotlin` does not resolve from Maven Central. No release job publishes it,
and the `com.hook0` namespace has not been claimed on the Central Portal;
[`ci/release-no-publish-job.toml`](https://gitlab.com/hook0/hook0/-/blob/master/ci/release-no-publish-job.toml) records what is missing.

Until then, build the jar from a checkout and install it into your local repository:

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

Kotlin 2.4 and Java 21 or later.

**It brings nothing with it.** The SDK reaches the network, verifies signatures, reads what the API answers and
suspends with the standard library and nothing else — no HTTP client, no JSON library, no coroutines artefact, no
logging façade. `kotlin-stdlib` is the only thing it declares, and that is the runtime every class file it emits links
against rather than a library it chose. Adding it to an application can therefore never drag a transitive dependency
onto that application's classpath, and never asks you to reconcile a version of `kotlinx.serialization` or of
`kotlinx-coroutines-core` with ours. `PackagingTest` is what keeps that sentence true rather than aspirational.

The suspending half is written against `kotlin.coroutines`, which is the language's own: it suspends inside whichever
coroutine runtime you brought — `kotlinx.coroutines`, Ktor's, Spring's, or none.

## Sending an event

```kotlin
import com.hook0.kotlin.Event
import com.hook0.kotlin.Hook0Client

Hook0Client("https://app.hook0.com/api/v1", applicationId, token).use { client ->
  val sent: UUID = client.sendEvent(
    Event(
      eventType = "auth.user.create",
      payload = """{"email":"someone@example.com"}""",
      payloadContentType = "application/json",
      labels = mapOf("environment" to "production")
    )
  )
}
```

…and the same send, suspending:

```kotlin
val sent: UUID = client.sendEventSuspending(event)
```

The identifier comes from here. Left unset, the client mints a UUIDv7, sends the event under it and answers it. That is
what makes a retry safe: Hook0 keys events on that identifier, so a request repeated after a network failure or a server
error ingests the event once rather than twice. Set `Event(eventId = …)` when you already have one to key the event on.

## Declaring event types

```kotlin
val created: List<String> = client.upsertEventTypes(listOf("auth.user.create", "billing.invoice.paid"))
```

Only the ones the application does not declare yet are created. `upsertEventTypesSuspending` is the same call, from a
coroutine.

## Verifying a webhook

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

It throws `ClientException` for every reason a delivery is refused, and returns for the one reason it is accepted. The
headers arrive either as a `Map<String, String>` or as a `List<Pair<String, String>>`, since a request may carry the
same header twice. The clock window is bilateral: a moment too far in the future is refused exactly like one too far in
the past, since a window that only looked backwards is one a sender widens by dating its own delivery ahead.

## Calling the rest of the API

Every operation the API declares is generated, grouped by entity, in both flavours:

```kotlin
import com.hook0.kotlin.generated.ApplicationsApi
import com.hook0.kotlin.generated.ApplicationsSuspendingApi

val application = ApplicationsApi(client.transport).get(applicationId)

val suspended = ApplicationsSuspendingApi(client.transport).get(applicationId)
```

Schemas are `data class`es, closed lists of strings are `enum class`es, and each problem the API can report is an
exception of its own under a `sealed class ProblemException` — so a `catch` may name one problem, or the base, or a
`when` may match over the closed set and be told by the compiler the day the API grows another. A member the API does
not declare required is a nullable type; everything else is not, so nothing in this SDK is ever read through `!!`.

## Bounds

Every one is yours to set, and every one has a default the shared conformance corpus committed at
`clients/conformance/bounds.json` writes down.

| Bound | Default | What it holds back |
|---|---|---|
| `maxAttempts` | 4 | requests one send issues, capped at 16 whatever a policy says |
| `initialBackoff` / `maxBackoff` | 100 ms / 2 s | how long one wait between attempts may be |
| `maxTotalDelay` | 5 s | how long every wait of one send may add up to |
| `requestTimeout` | 10 s | how long one attempt is given |
| `maxPayloadBytes` | 1 MiB | the payload, refused before a socket is opened |
| `maxResponseBytes` | 8 MiB | the body read off a socket |
| `maxHeadBytes` | 16 KiB | the head of an answer, every line taken together |
| `maxResponseHeaders` | 64 | header lines one answer may carry |
| `maxHeaderBytes` | 64 KiB | one header line |

```kotlin
val bounded = Options.defaults().copy(
  retryPolicy = RetryPolicy(6, Duration.ofMillis(50), Duration.ofSeconds(1), Duration.ofSeconds(10)),
  requestTimeout = Duration.ofSeconds(3)
)
```

`Options` and `RetryPolicy` are `data class`es carrying every default, so a caller names the one bound it means and
`copy` keeps the rest — there is no builder to learn and no partially-built value to hold.

The last three bound what a server on the other end may cost you: `java.net.http.HttpClient` bounds the head of an
answer at 393216 bytes and bounds the body not at all, so those ceilings are this client's own rather than the
runtime's.

## What is retried, and what is not

Only what could end differently: a request that got no answer, a server error, and an instance saying it is being
reached faster than it accepts. What the API refuses outright — a spent quota, a payload it will not read — is reported
as is. The verdict for every problem the API can report is written down once, in the corpus every Hook0 SDK reads, and
this client is driven against it over a real socket by its own suite.

## Two halves

`src/main/kotlin/com/hook0/kotlin/generated/` is written from the API's OpenAPI snapshot and rewritten wholesale on
every regeneration. Everything beside it — the transport, the retry loop, the signature verification, the JSON reader
and the suspending machinery — is hand-written and never regenerated.

```
UPDATE_SDK=kotlin cargo test -p hook0-sdkgen sdk_targets
```

The suite is at `test/kotlin`, outside `src` on purpose: a test file under a generated tree would be deleted without a
word at the next regeneration.

```
mvn ktlint:check verify
```

## Licence

MIT.
