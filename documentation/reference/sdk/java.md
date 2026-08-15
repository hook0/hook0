---
title: "Java webhook SDK — com.hook0:hook0-client"
description: "Send Hook0 events and verify webhook signatures from Java 21. Blocking and CompletableFuture methods, no transitive dependencies. Not yet on Maven Central."
keywords: [Java webhook SDK, Hook0 Java client, verify webhook signature Java, Spring Boot webhook endpoint, CompletableFuture webhook, send webhook event Java]
---

# Java SDK

The Hook0 SDK for Java sends events and verifies webhook signatures. Every call that reaches the network comes in two forms: one that blocks, and one that answers a `CompletableFuture`.

The SDK brings nothing with it. It reaches the network, verifies signatures and reads what the API answers with the standard library alone, so adding it to an application never drags a JSON or HTTP library onto that application's classpath.

## Installation

Java 21 or later is required.

:::warning Not published to Maven Central yet
`com.hook0:hook0-client` does not resolve from Maven Central. The `com.hook0` namespace has not been claimed on the Central Portal, and the build does not yet carry the signing and javadoc plugins a Central release needs.

Until then, build the jar from a checkout and install it into your local repository.
:::

```bash
git clone https://gitlab.com/hook0/hook0.git
mvn -f hook0/clients/java/pom.xml install
```

That puts `com.hook0:hook0-client:1.1.0` in `~/.m2`, where your own build resolves it:

```xml
<dependency>
  <groupId>com.hook0</groupId>
  <artifactId>hook0-client</artifactId>
  <version>1.1.0</version>
</dependency>
```

For Gradle, the same coordinates: `implementation("com.hook0:hook0-client:1.1.0")`, with `mavenLocal()` in your repositories.

## Send an event

```java
import com.hook0.client.Event;
import com.hook0.client.Hook0Client;
import java.util.Map;
import java.util.UUID;

try (Hook0Client client = new Hook0Client("https://app.hook0.com/api/v1", applicationId, token)) {
  UUID sent = client.sendEvent(
      Event.of(
          "billing.invoice.paid",
          "{\"invoice\":\"in_123\"}",
          "application/json",
          Map.of("environment", "production")));
}
```

The same send, without waiting for it:

```java
CompletableFuture<UUID> sending = client.sendEventAsync(event);
```

`Event.of` covers the four fields most sends set. The rest arrive through withers:

```java
Event event = Event.of("billing.invoice.paid", payload, "application/json", Map.of())
    .withMetadata(Map.of("emitter", "billing-worker"))
    .withOccurredAt(OffsetDateTime.now(ZoneOffset.UTC))
    .withEventId(UUID.fromString(knownId));
```

`Hook0Client` implements `AutoCloseable`. Hold one for the life of the application rather than one per send.

## Sending an event is idempotent, and retried

`sendEvent` sends every event under an ID it knows: the one set with `withEventId`, or a UUIDv7 it mints when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `sendEvent` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and throws.

## Bounds, and how to change them

```java
import com.hook0.client.Options;
import com.hook0.client.RetryPolicy;
import java.time.Duration;

Options options = Options.defaults()
    .withRetryPolicy(new RetryPolicy(
        4,
        Duration.ofMillis(100),
        Duration.ofMillis(2000),
        Duration.ofMillis(5000)))
    .withRequestTimeout(Duration.ofSeconds(10))
    .withMaxPayloadBytes(1024 * 1024)
    .withMaxResponseBytes(8L * 1024 * 1024);

Hook0Client client = new Hook0Client("https://app.hook0.com/api/v1", applicationId, token, options);
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

```java
import com.hook0.client.Webhooks;
import java.time.Duration;

Webhooks.verify(
    request.getHeader("X-Hook0-Signature"),
    rawBody,
    headersAsTheyArrived,
    subscriptionSecret,
    Duration.ofMinutes(5));
```

`verify` returns for the one reason a delivery is accepted and throws `ClientException` for every reason it is refused. There is no default tolerance; pass one.

`rawBody` is the body as it arrived, as a `String`. A body that has been parsed and re-serialised no longer hashes to what was signed. Headers come either as a `List<Map.Entry<String, String>>`, in the order they arrived, or as a `Map<String, String>`.

The clock window is bilateral: a moment too far in the future is refused exactly like one too far in the past, since a window that only looked backwards is one a sender widens by dating its own delivery ahead. A header the signature covers but the request did not carry is refused before any code is computed.

`Webhooks.verifyAt` takes the same arguments followed by an `Instant`, for holding a delivery against a moment you name.

### Spring Boot

```java
@PostMapping(path = "/webhook", consumes = MediaType.ALL_VALUE)
public ResponseEntity<Void> webhook(
    @RequestBody String rawBody,
    @RequestHeader HttpHeaders headers) {

  List<Map.Entry<String, String>> delivered = headers.entrySet().stream()
      .map(header -> Map.entry(header.getKey(), String.join(",", header.getValue())))
      .toList();

  try {
    Webhooks.verify(
        headers.getFirst("X-Hook0-Signature"),
        rawBody,
        delivered,
        subscriptionSecret,
        Duration.ofMinutes(5));
  } catch (ClientException refused) {
    return ResponseEntity.badRequest().build();
  }

  handleDelivery(rawBody);
  return ResponseEntity.ok().build();
}
```

Binding the body as a `String` keeps the bytes Spring received. Binding it to a DTO does not.

## Upsert event types

An event whose type the application does not declare is refused. Only the missing ones are created, and those are what comes back:

```java
List<String> created = client.upsertEventTypes(
    List.of("billing.invoice.paid", "billing.invoice.voided"));
```

`upsertEventTypesAsync` is the same call, unwaited. An event type is written `service.resource_type.verb`; `EventType.parse` reads one and throws `ClientException` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is generated, grouped by entity, in both flavours:

```java
import com.hook0.client.generated.*;

ApplicationsApi applications = new ApplicationsApi(client.transport());
ApplicationInfo application = applications.get(applicationId);

ApplicationsAsyncApi waiting = new ApplicationsAsyncApi(client.transport());
CompletableFuture<ApplicationInfo> later = waiting.get(applicationId);
```

Schemas are records, closed lists of strings are enums, and each problem the API can report is an exception of its own under a `sealed ProblemException`. A `catch` may name one problem or the base, and a `switch` over the closed set is told by the compiler the day the API grows another.

## Errors

| Type | Thrown when |
|------|-------------|
| `ClientException` | A send failed, retries ran out, a payload was too large, an event type was invalid or could not be created, or a delivery was refused |
| `TransportException` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `causeName()` and `retryable()` |
| `JsonException` | A document could not be read as JSON |
| `DecodeException` | A response body could not be read as the shape it declared |
| `generated.ProblemException` and its subclasses | The API reported a problem |

All of them extend `Hook0Exception`, which extends `RuntimeException`, so none is checked.

## Links

- **Source**: [clients/java](https://gitlab.com/hook0/hook0/-/tree/master/clients/java)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
