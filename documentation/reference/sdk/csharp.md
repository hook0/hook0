---
title: "C# / .NET webhook SDK — Hook0.Client"
description: "Send Hook0 events and verify webhook signatures from .NET 8. Blocking and Task-returning methods, zero runtime dependencies, retries and payload bounds built in."
keywords: [C# webhook SDK, .NET webhook client, Hook0.Client NuGet, verify webhook signature C#, ASP.NET Core webhook endpoint, send webhook event dotnet]
---

# C# SDK

The Hook0 SDK for .NET sends events and verifies webhook signatures. Every operation comes in two forms: a blocking one, and a `Task`-returning one that takes a `CancellationToken`.

The package has zero runtime dependencies. It reaches the network, verifies signatures and reads what the API answers with nothing but the framework.

## Installation

```bash
dotnet add package Hook0.Client
```

The package targets `net8.0`. The root namespace is `Hook0`.

## Send an event

```csharp
using Hook0;

using Hook0Client client = new(
    apiUrl: "https://app.hook0.com/api/v1",
    applicationId: "your-application-id",
    token: "your-service-token");

string eventId = client.SendEvent(new Event
{
    EventType = "billing.invoice.paid",
    Payload = """{"invoice": "in_123", "amount": 4200}""",
    PayloadContentType = "application/json",
    Labels = new Dictionary<string, string> { ["environment"] = "production" },
});
```

The same send, awaited:

```csharp
string eventId = await client.SendEventAsync(anEvent, cancellationToken);
```

`Event` is a record with three `required` members and four optional ones:

```csharp
new Event
{
    EventType = "billing.invoice.paid",
    Payload = """{"invoice": "in_123"}""",
    PayloadContentType = "application/json",
    Labels = new Dictionary<string, string> { ["environment"] = "production" },
    Metadata = new Dictionary<string, string> { ["emitter"] = "billing-worker" },
    OccurredAt = DateTimeOffset.UtcNow,
    EventId = null,
}
```

`Hook0Client` implements `IDisposable`, so hold one for the life of the application rather than one per send.

## Sending an event is idempotent, and retried

`SendEvent` sends every event under an ID it knows: the one set on `EventId`, or a UUIDv7 it mints when the event carries none. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `SendEvent` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and throws.

`Hook0Client.NewEventId()` mints a UUIDv7 as a `Guid` if you want to record the ID before the send.

## Bounds, and how to change them

```csharp
using Hook0Client client = new(
    apiUrl: "https://app.hook0.com/api/v1",
    applicationId: applicationId,
    token: token,
    options: new ClientOptions
    {
        RetryPolicy = new RetryPolicy
        {
            MaxAttempts = 4,
            InitialBackoff = TimeSpan.FromMilliseconds(100),
            MaxBackoff = TimeSpan.FromSeconds(2),
            MaxTotalDelay = TimeSpan.FromSeconds(5),
        },
        RequestTimeout = TimeSpan.FromSeconds(10),
        MaxPayloadBytes = 1024 * 1024,
        MaxResponseBytes = 8 * 1024 * 1024,
    });
```

Those are the defaults.

| Bound | Default |
|-------|---------|
| `MaxAttempts` (the first attempt included) | `4`, capped at `RetryPolicy.MaxAttemptsCap` = 16 |
| `InitialBackoff` | 100 ms |
| `MaxBackoff` | 2 s |
| `MaxTotalDelay`, the budget all delays of one send share | 5 s |
| `RequestTimeout`, per attempt | 10 s |
| `MaxPayloadBytes` | 1 MiB |
| `MaxResponseBytes` | 8 MiB |

`RetryPolicy.Disabled` sends each event exactly once. A payload above the maximum throws before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```csharp
using Hook0;

try
{
    Webhooks.VerifyWebhookSignature(
        signature: request.Headers["X-Hook0-Signature"],
        payload: rawRequestBody,          // the bytes, before anything parsed them
        headers: request.Headers,
        subscriptionSecret: subscriptionSecret);
}
catch (SignatureException refused)
{
    // refused.Refusal says which of the five ways it was refused
}
```

The payload is `byte[]` rather than text on purpose: a signature covers what arrived, and re-encoding a string before hashing it is how a valid delivery comes to be refused.

The clock window is bilateral, so a delivery dated too far ahead is refused exactly like one dated too far behind. It defaults to `Webhooks.DefaultTolerance`, five minutes. Pass `tolerance` to widen or narrow it, or `VerifyWebhookSignatureWithCurrentTime` to hold a delivery against a moment you name.

`SignatureException.Refusal` is a `SignatureRefusal`: `Malformed`, `CodeNotHexadecimal`, `HeaderNotDelivered`, `CodeMismatch` or `OutsideTolerance`.

### ASP.NET Core

```csharp
app.MapPost("/webhook", async (HttpRequest request, CancellationToken cancellationToken) =>
{
    using MemoryStream buffered = new();
    await request.Body.CopyToAsync(buffered, cancellationToken);
    byte[] payload = buffered.ToArray();

    IEnumerable<KeyValuePair<string, string>> headers = request.Headers
        .Select(header => new KeyValuePair<string, string>(header.Key, header.Value.ToString()));

    try
    {
        Webhooks.VerifyWebhookSignature(
            signature: request.Headers["X-Hook0-Signature"]!,
            payload: payload,
            headers: headers,
            subscriptionSecret: subscriptionSecret);
    }
    catch (SignatureException)
    {
        return Results.BadRequest();
    }

    await HandleDeliveryAsync(payload, cancellationToken);
    return Results.Ok();
});
```

Read the body into bytes yourself. Model binding has already reshaped anything it parsed, and the signature covers the bytes that arrived.

## Upsert event types

An event whose type the application does not declare is refused. Only the missing ones are created, and those are what comes back:

```csharp
IReadOnlyList<string> created = client.UpsertEventTypes(
    ["billing.invoice.paid", "billing.invoice.voided"]);
```

`UpsertEventTypesAsync` is the same call, awaited. An event type is written `service.resource_type.verb`; `EventType.Parse` reads one and throws `EventTypeException` on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method on a generated group, in both idioms, over the transport the client already built:

```csharp
using Hook0;
using Hook0.Generated;

ApplicationsApi applications = new(client.Transport);
IReadOnlyList<Application> mine = applications.List("your-organization-id");

ApplicationsAsyncApi awaiting = new(client.Transport);
IReadOnlyList<Application> also = await awaiting.ListAsync("your-organization-id", cancellationToken);
```

Each problem the API can report is an exception of its own under `ProblemException`, which carries the HTTP `Status` and the parsed `Problem`.

## Errors

| Type | Thrown when |
|------|-------------|
| `SendException` | A send failed or ran out of attempts. Carries `EventId`, `Attempts` and `Waited` |
| `EventTypeException` | An event type was invalid, unavailable or could not be created |
| `SignatureException` | A webhook was refused. Carries `Refusal` |
| `TransportException` | The request never got an answer, or the answer crossed one of the transport's bounds. Carries `CauseName` and `Retryable` |
| `DecodeException` | A response body could not be read as the shape it declared |
| `Generated.ProblemException` and its subclasses | The API reported a problem |

`SendException`, `EventTypeException` and `SignatureException` all derive from `Hook0Exception`, so one `catch` covers the three.

```csharp
try
{
    string eventId = client.SendEvent(anEvent);
}
catch (SendException refused)
{
    logger.LogError("event {EventId} gave up after {Attempts} attempts", refused.EventId, refused.Attempts);
}
```

## Links

- **Package**: [Hook0.Client on NuGet](https://www.nuget.org/packages/Hook0.Client)
- **Source**: [clients/csharp](https://gitlab.com/hook0/hook0/-/tree/master/clients/csharp)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
