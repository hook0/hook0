# Hook0 Go Client

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../LICENSE.txt)
[![Go Reference](https://pkg.go.dev/badge/github.com/hook0/hook0/clients/go.svg)](https://pkg.go.dev/github.com/hook0/hook0/clients/go)

This is the Go SDK for [Hook0](https://www.hook0.com), an open source Webhooks-as-a-Service platform designed for SaaS applications.

## Features

- **Send Events**: Send events to Hook0, retried and bounded.
- **Upsert Event Types**: Make sure event types you use in your application's events are created in Hook0.
- **Verifying Webhook Signatures**: Ensure the authenticity and integrity of incoming webhooks.
- **The whole API, typed**: one type per schema Hook0 declares, one error value per problem it reports, one method per operation — generated from the OpenAPI snapshot the API commits.

## No dependencies

`go.mod` carries no `require` at all. The SDK reaches the network with `net/http`, verifies signatures with `crypto/hmac` and `crypto/sha256`, and reads what the API answers with `encoding/json`. Adding it to a project drags nothing else in, and the CI job fails if that ever stops being true.

The two halves never import each other either. The generated package declares the `Transport` interface it issues requests through, and the hand-written transport satisfies it by shape, which is what keeps the generated half inside the standard library.

## Getting Started

```
go get github.com/hook0/hook0/clients/go
```

The package is `hook0`, and the module path ends in `go` because that is where it sits in the repository; import it under its own name if your tooling does not fill that in:

```go
import hook0 "github.com/hook0/hook0/clients/go"
```

Go 1.24 or later is required.

## Sending an event is idempotent, and retried

`SendEvent` sends every event under an ID it knows: the one set on the `Event`, or a UUIDv7 it generates when the event carries none. **Passing none does not mean the ID comes from Hook0** — the value comes from the client, is sent with the request, and is what `SendEvent` answers.

That is what makes retrying safe. Hook0 keys events on their ID, so a request repeated after a network failure or a server error ingests the event once rather than twice; without a client-chosen ID, a repeated request would create a second event and deliver it to every subscriber.

Only a network failure or a server error is retried. A retried request Hook0 answers with `EventAlreadyIngested` reports success — an earlier attempt of that same send reached the API. The same answer to a *first* attempt is a genuine conflict and is answered as an error.

Every send is bounded, and every bound is configurable:

```go
options := hook0.Options{
	RetryPolicy: hook0.RetryPolicy{
		MaxAttempts:    4,
		InitialBackoff: 100 * time.Millisecond,
		MaxBackoff:     2 * time.Second,
		MaxTotalDelay:  5 * time.Second,
	},
	RequestTimeout:   10 * time.Second,
	MaxPayloadBytes:  1024 * 1024,
	MaxResponseBytes: 8 * 1024 * 1024,
}

client := hook0.NewClient("https://app.hook0.com/api/v1", applicationID, token, options)

eventID, err := client.SendEvent(ctx, hook0.Event{
	EventType:          "billing.invoice.paid",
	Payload:            `{"invoice": "in_123"}`,
	PayloadContentType: "application/json",
	Labels:             map[string]string{"environment": "production"},
})
```

Those are the defaults, and `hook0.DefaultOptions()` is them. `hook0.DisabledRetryPolicy()` sends each event exactly once. A payload above the maximum is refused before any request is issued, with `errors.Is(err, hook0.ErrPayloadTooLarge)`.

The context bounds the whole send, retries and waits included; the request timeout bounds one attempt.

## Verifying webhook signatures

```go
err := hook0.VerifyWebhookSignature(
	request.Header.Get("X-Hook0-Signature"),
	body,
	request.Header,
	subscriptionSecret,
	5*time.Minute,
)
if err != nil {
	// answer 400, and do not act on the delivery
}
```

Every reason a delivery is refused is a value `errors.Is` names: `ErrSignatureUnreadable`, `ErrHeaderNotDelivered`, `ErrSignatureMismatch`, `ErrSignatureOutsideTolerance`.

The clock window is bilateral: a webhook signed too far in the future is refused exactly like one signed too long ago. A header the signature covers but the request did not carry is refused before any code is computed.

## Calling the rest of the API

Every operation Hook0 declares is a method of a generated group, built on the transport the client already carries:

```go
import "github.com/hook0/hook0/clients/go/generated"

applications := generated.NewApplicationsAPI(client.Transport())

application, err := applications.Get(ctx, applicationID)
if errors.Is(err, generated.ErrNotFound) {
	// every problem the API names is a value of its own
}

var reported *generated.ProblemError
if errors.As(err, &reported) {
	// and every one of them carries the status and the document the API answered
}
```

A group can also be built on a transport of its own:

```go
applications := generated.NewApplicationsAPI(hook0.NewTransport("https://app.hook0.com", token, 0, 0))
```

## What is Hook0?

**Hook0** is an open source product that helps any software system (such as Software-as-a-Service applications) to expose webhooks to their end users.

Want to know more? Check out our [detailed documentation](https://documentation.hook0.com/docs/what-is-hook0) or visit our [website](https://hook0.com).

## Contributing

`generated/` is written by `hook0-sdkgen` from the OpenAPI snapshot the API crate commits, and is rewritten wholesale on every regeneration — a hand edit there is reverted the next time anyone regenerates. It is emitted already formatted, so `gofmt -l` printing a file under it is a defect in the generator rather than something to fix in place. Change the generator instead, then run:

```
UPDATE_SDK=go cargo test -p hook0-sdkgen sdk_targets
```

Everything beside `generated/` is hand-written and never regenerated, the `_test.go` files included. Go keeps a package's tests next to the package, so there is no `src` here and nothing under `generated/` is ever a test file.

## Authors

- David Sferruzza - [david@hook0.com](mailto:david@hook0.com)
- François-Guillaume Ribreau - [fg@hook0.com](mailto:fg@hook0.com)

For more information, visit our [homepage](https://www.hook0.com/), join our [Discord community](https://www.hook0.com/community) or contact us at [support@hook0.com](mailto:support@hook0.com)

### LICENSE

Hook0 Go SDK is free and open-source. It is released under the [MIT License](../../LICENSE.txt).

This license grants you the freedom to use, modify, distribute, and sublicense the SDK with minimal restrictions. You may use it in both open-source and commercial projects, as long as you include the original copyright notice.
