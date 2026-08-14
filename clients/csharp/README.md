# Hook0.Client

The C# SDK for [Hook0](https://www.hook0.com/), open-source Webhooks as a service for SaaS.

Send events to Hook0, upsert the event types your application uses, verify the signature of an
incoming webhook, and call every operation the API declares through generated, documented types — in
both the blocking and the `Task`-returning idiom.

Sending is idempotent and retried under bounds you set. **Zero runtime dependencies:** the package
reaches the network, verifies signatures and reads what the API answers with nothing but the .NET
framework, so installing it cannot drag a transitive dependency into an application that only wanted
to send an event.

```
dotnet add package Hook0.Client
```

Targets `net8.0`.

## Sending an event

```csharp
using Hook0;

using Hook0Client client = new(
    apiUrl: "https://app.hook0.com/api/v1",
    applicationId: "your-application-id",
    token: "your-application-secret");

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

`SendEvent` answers the identifier the event was sent under. You did not pass one, so the client
minted a UUIDv7, sent it, and handed it back — and that is what makes retrying safe: Hook0 keys
events on that identifier, so a request repeated after a network failure or a server error ingests
the event once rather than twice. Pass `EventId` yourself when you already have something to key the
event on.

Only what could end differently is retried: a request that got no answer, a server error, and an
instance saying it is being reached faster than it accepts. What the API refuses outright — a quota
that is spent, a payload it will not read — is reported as is. A failure arrives as a
`SendException`, which says how many requests the send issued.

## Declaring the event types you use

```csharp
IReadOnlyList<string> created = client.UpsertEventTypes(
    ["billing.invoice.paid", "billing.invoice.voided"]);
```

Only the ones the application did not already declare are created, and those are what comes back.
`UpsertEventTypesAsync` is the same call, awaited.

## Verifying a webhook you received

```csharp
using Hook0;

try
{
    Webhooks.VerifyWebhookSignature(
        signature: request.Headers["X-Hook0-Signature"],
        payload: rawRequestBody,          // the bytes, before anything parsed them
        headers: request.Headers,
        subscriptionSecret: "your-subscription-secret");
}
catch (SignatureException refused)
{
    // refused.Refusal says which of the four ways it was refused.
}
```

The payload is bytes rather than text on purpose: a signature covers what arrived, and re-encoding a
string before hashing it is how a valid delivery comes to be refused.

The clock window is bilateral — a delivery dated too far ahead is refused exactly like one dated too
far behind — and defaults to five minutes. Pass `tolerance` to widen or narrow it, or
`VerifyWebhookSignatureWithCurrentTime` to hold a delivery against a moment you name.

## Calling the rest of the API

Every operation the API declares is a method on a generated group, in both idioms. Each takes the
transport the client already built:

```csharp
using Hook0;
using Hook0.Generated;

ApplicationsApi applications = new(client.Transport);
IReadOnlyList<Application> mine = applications.List("your-organization-id");

ApplicationsAsyncApi awaiting = new(client.Transport);
IReadOnlyList<Application> also = await awaiting.ListAsync("your-organization-id", cancellationToken);
```

A failure the API describes arrives as one exception per problem it can name —
`EventAlreadyIngestedException`, `TooManyEventsTodayException`, and so on — all of them
`ProblemException`, which carries the status and the problem document. A problem this client has
never heard of still arrives as a `ProblemException` rather than as nothing.

## Bounds

Every bound one send is held to is yours to set, and the defaults are the ones the shared
conformance corpus committed beside this package names:

```csharp
using Hook0Client client = new(apiUrl, applicationId, token, new ClientOptions
{
    RetryPolicy = new RetryPolicy
    {
        MaxAttempts = 4,                                // 1 disables retrying
        InitialBackoff = TimeSpan.FromMilliseconds(100),
        MaxBackoff = TimeSpan.FromSeconds(2),
        MaxTotalDelay = TimeSpan.FromSeconds(5),        // every delay of one send, together
    },
    RequestTimeout = TimeSpan.FromSeconds(10),
    MaxPayloadBytes = 1024 * 1024,                      // refused before a socket is opened
    MaxResponseBytes = 8 * 1024 * 1024,
    MaxHeadBytes = 16 * 1024,
    MaxResponseHeaders = 64,
    MaxHeaderBytes = 64 * 1024,
});
```

`RetryPolicy.Disabled` is one attempt and no waiting. Whatever `MaxAttempts` says, a single send
never issues more than `RetryPolicy.MaxAttemptsCap` requests.

## What is generated and what is not

Everything under `src/Hook0/Generated` is written from the API's OpenAPI snapshot by the generator in
`clients/sdkgen`, and is committed rather than built — the package is published with no copy of the
snapshot beside it. Do not edit those files: run

```
UPDATE_SDK=csharp cargo test -p hook0-sdkgen sdk_targets
```

and commit what it rewrites. A hand edit there is caught by the same test.

Everything beside it — how a request reaches the network, how a send is retried, how a webhook
signature is verified — is hand-written and never regenerated.

## Licence

MIT.
