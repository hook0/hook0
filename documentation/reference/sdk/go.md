---
title: "Go webhook SDK: hook0 client"
description: "Send Hook0 events and verify webhook signatures from Go. Standard library only, context on every call, retries and payload bounds built in. Go 1.24 or later."
keywords: [Go webhook SDK, Hook0 Go client, verify webhook signature Go, send webhook event Go, golang webhook library, HMAC signature Go]
sdkTarget: go
---

# Go SDK

The Hook0 SDK for Go sends events and verifies webhook signatures. It is blocking, and every call that reaches the network takes a `context.Context` as its first argument, so cancellation and deadlines are yours.

The module declares no requirements at all. Everything it does it does with the standard library, and the test suite fails if a dependency ever appears.

## Installation

```bash
go get github.com/hook0/hook0-go@v1.1.0
```

Go 1.24 or later is required. The package is named `hook0` while the module path ends in `go`, so import it under an alias:

```go example=import
import hook0 "github.com/hook0/hook0-go"
```

The module sits at the root of `github.com/hook0/hook0-go`, a read-only mirror of `clients/go` that the release pipeline pushes to and tags. Every `sdk-vX.Y.Z` release of the Hook0 SDKs puts the matching `vX.Y.Z` on that mirror, which is the version the Go module proxy answers. Issues and merge requests belong on [the monorepo](https://gitlab.com/hook0/hook0); nothing merged into the mirror survives the next release.

## Send an event

```go example=program
package main

import (
	"context"
	"log"

	hook0 "github.com/hook0/hook0-go"
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

`Event` has three fields you must set and four you may:

```go example=event
hook0.Event{
	EventType:          "billing.invoice.paid",
	Payload:            `{"invoice": "in_123"}`,
	PayloadContentType: "application/json",
	Labels:             map[string]string{"environment": "production"},
	Metadata:           map[string]string{"emitter": "billing-worker"},
	OccurredAt:         time.Now().UTC(), // the zero moment means now
	EventId:            "",               // empty means the client chooses
}
```

The token goes in without a `Bearer` prefix; the client adds it.

:::caution Always start from `DefaultOptions()`
`NewClient` takes `Options` by value and does not fill in what a zero value left out. `hook0.Options{}` carries `MaxPayloadBytes: 0`, which refuses every payload. Call `hook0.DefaultOptions()` and change the fields you care about.
:::

## Sending an event is idempotent, and retried

`SendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when `EventId` is empty. Passing no ID does not mean the ID comes from Hook0. The value comes from the client, is sent with the request, and is what `SendEvent` returns.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice. Without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

A network failure, a server error, and a `429` whose body names the `RateLimited` problem are retried. A `429` that names a spent daily quota is not, because a quota clears when a plan changes or a day turns and no send can wait for that. A `Retry-After` header is honoured and clamped to what is left of the delay budget.

A retried request that Hook0 answers with `EventAlreadyIngested` reports success, because an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is returned as an error.

## Bounds, and how to change them

```go example=options
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

Those are the defaults, exported as `DefaultRequestTimeout`, `DefaultMaxPayloadBytes` and `DefaultMaxResponseBytes`.

| Bound | Default |
|-------|---------|
| `MaxAttempts` (the first attempt included) | `4`, capped at `MaxAttemptsCap` = 16 |
| `InitialBackoff` | 100 ms |
| `MaxBackoff` | 2 s |
| `MaxTotalDelay`, the budget all delays of one send share | 5 s |
| `RequestTimeout`, per attempt | 10 s |
| `MaxPayloadBytes` | 1 MiB |
| `MaxResponseBytes` | 8 MiB |

`hook0.DisabledRetryPolicy()` sends each event exactly once. A payload above the maximum is refused before any request is issued, so neither the round trip nor the retries after it are spent on a request the API would refuse.

## Verify a webhook signature

```go example=handler
import (
	"io"
	"net/http"
	"time"

	hook0 "github.com/hook0/hook0-go"
)

func handleWebhook(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 1024*1024))
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	err = hook0.VerifyWebhookSignature(
		r.Header.Get("X-Hook0-Signature"),
		body,
		r.Header,
		subscriptionSecret,
		5*time.Minute,
	)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		return
	}

	// act on the delivery
	w.WriteHeader(http.StatusOK)
}
```

`VerifyWebhookSignature` returns `nil` when the webhook is genuine and an error for every reason it is not. Read the raw body before anything decodes it: a body that has been parsed and re-serialised no longer hashes to what was signed.

`tolerance` is a `time.Duration`. The clock window is bilateral, so a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

Each refusal wraps a sentinel you can match with `errors.Is`:

```go example=matching
switch {
case errors.Is(err, hook0.ErrSignatureMismatch):
	// the body or a covered header was changed in flight, or the secret is wrong
case errors.Is(err, hook0.ErrSignatureOutsideTolerance):
	// too old, or a clock is off
case errors.Is(err, hook0.ErrHeaderNotDelivered):
	// a header the signature covers did not arrive
case errors.Is(err, hook0.ErrSignatureUnreadable):
	// the header is not a Hook0 signature
}
```

`VerifyWebhookSignatureAt` takes the same arguments followed by a `time.Time`, for holding a signature against a moment you choose.

## Upsert event types

An event whose type the application does not declare is refused. `UpsertEventTypes` creates the ones that are missing and returns only those it created:

```go example=upsert
created, err := client.UpsertEventTypes(ctx, []string{
	"billing.invoice.paid",
	"billing.invoice.voided",
})
```

An event type is written `service.resource_type.verb`. `ParseEventType` reads one and returns an error on anything else.

## Calling the rest of the API

Sending events is two methods out of the whole API. Every operation Hook0 declares is a method of a generated group:

```go example=file
import (
	"context"
	"errors"

	hook0 "github.com/hook0/hook0-go"
	"github.com/hook0/hook0-go/generated"
)

func readApplication(ctx context.Context, applicationId string) (*generated.ApplicationInfo, error) {
	transport := hook0.NewTransport("https://app.hook0.com", token, 0, 0)
	applications := generated.NewApplicationsAPI(transport)

	application, err := applications.Get(ctx, applicationId)
	if errors.Is(err, generated.ErrNotFound) {
		// every problem the API names has a sentinel of its own
	}
	return application, err
}
```

Passing `0` for the timeout or the response ceiling gives `NewTransport` its defaults. The generated groups depend on a one-method `generated.Transport` interface, so a fake in a test satisfies it without importing the client.

## Errors

| Type | Returned when |
|------|---------------|
| `*SendError` | A send failed or ran out of attempts. Carries `EventId`, `Attempts`, `Waited` and `Detail` |
| `*EventTypeError` | An event type was invalid or could not be created |
| `*TransportError` | The request never got an answer, or the answer crossed one of the transport's bounds |
| `*generated.ProblemError` | The API reported a problem. Carries `Status`, `Kind` and the parsed `Problem` |

Every one of them implements `Unwrap`, and the sentinels below match through it with `errors.Is`: `ErrPayloadTooLarge`, `ErrInvalidEventType`, `ErrUnreachable`, `ErrAnswerAboveABound`, `ErrUnusableAPIURL`, `ErrSignatureUnreadable`, `ErrHeaderNotDelivered`, `ErrSignatureMismatch`, `ErrSignatureOutsideTolerance`.

```go example=matching
var sent *hook0.SendError
if errors.As(err, &sent) {
	log.Printf("event %s gave up after %d attempts, %s waited", sent.EventId, sent.Attempts, sent.Waited)
}
```

## Links

- **Package**: [pkg.go.dev](https://pkg.go.dev/github.com/hook0/hook0-go)
- **Source**: [clients/go](https://gitlab.com/hook0/hook0/-/tree/master/clients/go)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
