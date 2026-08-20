<div align="center">

# Hook0 C# SDK

**Send and verify webhooks from .NET, blocking or awaited**

<br/>

<img src="assets/csharp-flow.svg" alt="How the Hook0 C# SDK sits between your application and your users" width="850"/>

<br/>
<br/>

[![NuGet](https://img.shields.io/nuget/v/Hook0.Client)](https://www.nuget.org/packages/Hook0.Client)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

</div>

---

## What is this?

The C# SDK for [Hook0](https://www.hook0.com/), the open source Webhooks-as-a-Service platform
for SaaS applications. It sends events, declares the event types your application uses, verifies the
signature of a webhook you receive, and calls every operation the API declares through generated,
documented types.

Every call comes twice: one that blocks and one that answers a `Task`. The package reaches the
network, verifies signatures and reads what the API answers with nothing but the .NET framework, so
installing it cannot drag a transitive dependency into an application that only wanted to send an
event.

## Features

- **Send events** - under an ID the client mints, so a retry cannot duplicate one
- **Declare event types** - upsert the ones your application emits, in one call
- **Verify signatures** - HMAC-SHA256 over the bytes that arrived, on a bilateral clock window
- **The whole API, typed** - one type per schema, one exception per problem, one method per operation
- **Bounded everywhere** - attempts, backoff, timeouts, payload and answer, all yours to set
- **Zero dependencies** - the framework and nothing else

---

## Quick Start

### 1. Install

```bash
dotnet add package Hook0.Client
```

Targets `net8.0`.

### 2. Send an event

```csharp
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

### 3. Verify a webhook you receive

```csharp
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

The clock window is bilateral, so a delivery dated too far ahead is refused exactly like one dated
too far behind, because a window that only looked backwards is one a sender widens by dating its own
delivery in the future. A header the signature covers but the request did not carry is refused
before any code is computed.

---

## Configuration

Every bound one send is held to is yours to set, and every one has a default.

| Bound | Default | What it holds back |
|-|-|-|
| `MaxAttempts` | 4 | requests one send issues, capped at 16 whatever a policy says |
| `InitialBackoff` | 100 ms | the ceiling of the wait before the first retry |
| `MaxBackoff` | 2 s | the ceiling no single wait between attempts crosses |
| `MaxTotalDelay` | 5 s | the budget every wait of one send shares |
| `RequestTimeout` | 10 s | how long one attempt is given |
| `MaxPayloadBytes` | 1 MiB | the payload, refused before a socket is opened |
| `MaxResponseBytes` | 8 MiB | the body read off a socket |
| `MaxHeadBytes` | 16 KiB | the head of an answer, every line taken together |
| `MaxResponseHeaders` | 64 | header lines one answer may carry |
| `MaxHeaderBytes` | 64 KiB | one header line |

Every default comes from [`clients/conformance/bounds.json`](https://gitlab.com/hook0/hook0/-/blob/master/clients/conformance/bounds.json),
the corpus every Hook0 SDK reads. A number changed there fails every SDK still carrying the old one,
so no two of them can bound different things.

The last three bound what the other end may cost you. A server that is broken or hostile can
otherwise stream a head, a header or a body of any length into your process.

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

---

## Usage

### Sending is idempotent, and retried

`SendEvent` sends every event under an ID it knows, either the one set on the event or a UUIDv7 it
mints when the event carries none. **Passing no ID does not mean the ID comes from Hook0.** The value comes
from the client, travels with the request, and is what `SendEvent` answers.

That is what makes a retry safe. Hook0 keys events on their ID, so a request repeated after a network
failure or a server error ingests the event once rather than twice. Without a client-chosen ID, the
repeated request would create a second event and deliver it to every subscriber.

Retrying is limited to what could end differently. A request that got no answer, a server error and
an instance saying it is being reached faster than it accepts are all retried. A `429` naming a
spent quota is not, because a quota clears when a plan changes or a day turns, and no send can wait
for that. A
`Retry-After` the answer carries is honoured, clamped to what is left of the delay budget. A retried
request Hook0 answers with `EventAlreadyIngested` reports success, since an earlier attempt of that
same send reached the API. The same answer to a *first* attempt is a genuine conflict, and is
reported as an error.

### Declaring the event types you use

```csharp
IReadOnlyList<string> created = client.UpsertEventTypes(
    ["billing.invoice.paid", "billing.invoice.voided"]);
```

Only the ones your application does not declare yet are created, and those are what comes back.

### Calling the rest of the API

Every operation the API declares is a method of a generated group, one group per entity.

```csharp
ApplicationsApi applications = new(client.Transport);
IReadOnlyList<Application> mine = applications.List("your-organization-id");

ApplicationsAsyncApi awaiting = new(client.Transport);
IReadOnlyList<Application> also = await awaiting.ListAsync("your-organization-id", cancellationToken);
```

### The payload is bytes, on purpose

A signature covers what arrived. Re-encoding a string before hashing it is how a valid delivery
comes to be refused, so verification takes the raw body.

### One exception per problem

A failure the API describes arrives as `EventAlreadyIngestedException`,
`TooManyEventsTodayException`, and so on, all of them `ProblemException`, which carries the status
and the problem document. A problem this client has never heard of still arrives as a
`ProblemException` rather than as nothing.

---

## Development

`clients/csharp/src/Hook0/Generated/` is written by [`hook0-sdkgen`](https://gitlab.com/hook0/hook0/-/tree/master/clients/sdkgen)
from the OpenAPI snapshot the API commits, and is rewritten whole on every regeneration. A hand edit
there is reverted the next time anyone regenerates, and the drift guard says so before that. Change
the generator, then run:

```
UPDATE_SDK=csharp cargo test -p hook0-sdkgen sdk_targets
```

Everything beside it, the transport, the retry loop and the signature verification, is hand-written
and never regenerated, and so is `tests/`.

What a send retries, the bounds it is held to and how a signature is verified are dictated by the
shared corpus at [`clients/conformance`](https://gitlab.com/hook0/hook0/-/tree/master/clients/conformance),
which every SDK's suite reads, so a verdict changed there fails this client until it agrees again.

Every case runs against a real Hook0 over a loopback socket. Nothing here stands in for a part of
the client.

```
dotnet format --verify-no-changes
dotnet build --configuration Release -warnaserror
dotnet test --configuration Release --no-build
```

---

## License

The Hook0 C# SDK is free and open source, released under the [MIT License](./LICENSE). Use it,
change it, ship it, in open source and in commercial work alike, as long as the copyright notice
travels with it.

Hook0 itself is open source too. Read [what Hook0 is](https://documentation.hook0.com/docs/what-is-hook0),
visit [hook0.com](https://www.hook0.com/), join the [community](https://www.hook0.com/community), or
write to [support@hook0.com](mailto:support@hook0.com).

Maintained by [David Sferruzza](mailto:david@hook0.com) and [François-Guillaume Ribreau](mailto:fg@hook0.com).
