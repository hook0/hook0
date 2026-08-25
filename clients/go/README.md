<div align="center">

# Hook0 Go SDK

**A webhook SDK whose go.mod has no require at all**

<br/>

<img src="assets/go-flow.svg" alt="How the Hook0 Go SDK sits between your application and your users" width="850"/>

<br/>
<br/>

[![Go Reference](https://pkg.go.dev/badge/github.com/hook0/hook0-go/v2.svg)](https://pkg.go.dev/github.com/hook0/hook0-go/v2)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

</div>

---

## What is this?

The Go SDK for [Hook0](https://www.hook0.com/), the open source Webhooks-as-a-Service platform
for SaaS applications. It sends events, declares the event types your application uses, verifies the
signature of a webhook you receive, and calls every operation the API declares through generated,
documented types.

`go.mod` carries no `require`. The SDK reaches the network with `net/http`, verifies signatures with
`crypto/hmac`, and reads what the API answers with `encoding/json`. Adding it to a project drags
nothing else in, and the CI job fails the day that stops being true.

## Features

- **Send events** - under an ID the client mints, so a retry cannot duplicate one
- **Declare event types** - upsert the ones your application emits, in one call
- **Verify signatures** - HMAC-SHA256 over a bilateral clock window
- **The whole API, typed** - one type per schema, one error value per problem, one method per operation
- **Bounded everywhere** - attempts, backoff, timeouts, payload and answer, all yours to set
- **Zero dependencies** - the standard library and nothing else, enforced in CI

---

## Quick Start

### 1. Install

```bash
go get github.com/hook0/hook0-go/v2
```

The package is `hook0` while the module path ends in `go`, so import it under its own name if your
tooling does not fill that in: `import hook0 "github.com/hook0/hook0-go/v2"`. Go 1.25.13 or later,
which is the floor `go.mod` declares.

### 2. Send an event

```go
package main

import (
	"context"
	"log"

	hook0 "github.com/hook0/hook0-go/v2"
)

func main() {
	client := hook0.NewClient(
		"https://app.hook0.com/api/v1",
		applicationId,
		token,
		hook0.DefaultOptions(),
	)

	eventId, err := client.SendEvent(context.Background(), hook0.Event{
		EventType:          "billing.invoice.paid",
		Payload:            `{"invoice": "in_123"}`,
		PayloadContentType: "application/json",
		Labels:             map[string]string{"environment": "production"},
	})
	if err != nil {
		log.Fatalf("event not sent: %v", err)
	}

	log.Printf("ingested as %s", eventId)
}
```

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

```go
options := hook0.DefaultOptions()
options.RetryPolicy = hook0.RetryPolicy{
	MaxAttempts:    4,
	InitialBackoff: 100 * time.Millisecond,
	MaxBackoff:     2 * time.Second,
	MaxTotalDelay:  5 * time.Second,
}
options.RequestTimeout = 10 * time.Second
options.MaxPayloadBytes = 1024 * 1024
options.MaxResponseBytes = 8 * 1024 * 1024

client := hook0.NewClient(apiURL, applicationId, token, options)
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

```go
created, err := client.UpsertEventTypes(ctx, []string{
	"billing.invoice.paid",
	"billing.invoice.voided",
})
```

Only the ones your application does not declare yet are created, and those are what comes back.

### Every failure is a value you can match

Every reason a delivery or a send is refused is a value `errors.Is` names: `ErrSignatureUnreadable`,
`ErrHeaderNotDelivered`, `ErrSignatureMismatch`, `ErrSignatureOutsideTolerance`,
`ErrPayloadTooLarge`. A problem the API reported also arrives as a `*generated.ProblemError`,
carrying the status and the document it answered.

### Two clocks, two bounds

The context bounds the whole send, retries and waits included. The request timeout bounds one
attempt.

---

## Development

`clients/go/generated/` is written by [`hook0-sdkgen`](https://gitlab.com/hook0/hook0/-/tree/master/clients/sdkgen)
from the OpenAPI snapshot the API commits, and is rewritten whole on every regeneration. A hand edit
there is reverted the next time anyone regenerates, and the drift guard says so before that. Change
the generator, then run:

```
UPDATE_SDK=go cargo test -p hook0-sdkgen sdk_targets
```

Everything beside it, the transport, the retry loop and the signature verification, is hand-written
and never regenerated, and so is every `_test.go` file.

What a send retries, the bounds it is held to and how a signature is verified are dictated by the
shared corpus at [`clients/conformance`](https://gitlab.com/hook0/hook0/-/tree/master/clients/conformance),
which every SDK's suite reads, so a verdict changed there fails this client until it agrees again.

Every case runs against a real Hook0 over a loopback socket. Nothing here stands in for a part of
the client.

```
gofmt -l .
go vet ./...
go test ./...
```

---

## License

The Hook0 Go SDK is free and open source, released under the [MIT License](./LICENSE). Use it,
change it, ship it, in open source and in commercial work alike, as long as the copyright notice
travels with it.

Hook0 itself is open source too. Read [what Hook0 is](https://documentation.hook0.com/docs/what-is-hook0),
visit [hook0.com](https://www.hook0.com/), join the [community](https://www.hook0.com/community), or
write to [support@hook0.com](mailto:support@hook0.com).

Maintained by [David Sferruzza](mailto:david@hook0.com) and [François-Guillaume Ribreau](mailto:fg@hook0.com).
